use std::error::Error;
use log::{info, error};
use moka::future::Cache;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone)]
pub struct Email {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
    pub attachments: Vec<(String, Vec<u8>)>,
}

#[derive(Clone)]
pub struct MailServer {
    emails: Cache<String, Email>,
}

impl MailServer {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let emails = Cache::builder()
            .time_to_live(Duration::from_secs(20 * 60)) // 20分钟过期
            .build();
        Ok(Self { emails })
    }

    pub async fn start(&self, addr: &str) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;
        info!("Mail server starting on {}...", addr);

        loop {
            let (socket, addr) = listener.accept().await?;
            info!("New connection from {}", addr);
            let emails = self.emails.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_smtp_connection(socket, emails).await {
                    error!("Error handling SMTP connection: {}", e);
                }
            });
        }
    }

    pub async fn get_email(&self, to: &str) -> Option<Email> {
        self.emails.get(to).await
    }
}

async fn handle_smtp_connection(mut socket: TcpStream, emails: Cache<String, Email>) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];
    let mut current_email = Email {
        from: String::new(),
        to: Vec::new(),
        subject: String::new(),
        body: String::new(),
        attachments: Vec::new(),
    };
    let mut data_mode = false;
    let mut data_buffer = Vec::new();
    info!("Mail server connection established");
    // 发送欢迎消息
    socket.write_all(b"220 Simple SMTP Server\r\n").await?;

    loop {
        let n = socket.read(&mut buffer).await?;
        if n == 0 {
            info!("Client connection closed");
            break;
        }

        let command = String::from_utf8_lossy(&buffer[..n]);
        let command = command.trim();
        info!("Received command: {}", command);

        if data_mode {
            data_buffer.extend_from_slice(&buffer[..n]);
            
            // 检查是否包含结束序列 "\r\n.\r\n"
            if data_buffer.len() >= 5 {
                let end_sequence = b"\r\n.\r\n";
                if let Some(end_pos) = data_buffer.windows(5).position(|window| window == end_sequence) {
                    data_mode = false;
                    // 提取实际数据（不包含结束序列）
                    let actual_data = &data_buffer[..end_pos];
                    
                    // 解析邮件数据
                    let data_str = String::from_utf8_lossy(actual_data);
                    let mut lines = data_str.lines();
                    let mut is_header = true;
                    let mut body_lines = Vec::new();

                    while let Some(line) = lines.next() {
                        if line.is_empty() && is_header {
                            is_header = false;
                            continue;
                        }

                        if is_header {
                            if let Some(idx) = line.find(":") {
                                let (key, value) = line.split_at(idx);
                                let value = value[1..].trim();
                                match key.trim().to_lowercase().as_str() {
                                    "subject" => current_email.subject = value.to_string(),
                                    _ => {}
                                }
                            }
                        } else {
                            body_lines.push(line);
                        }
                    }

                    current_email.body = body_lines.join("\n");

                    // 存储邮件
                    for to in &current_email.to {
                        info!("Storing email for {}", to);
                        emails.insert(to.clone(), current_email.clone()).await;
                    }

                    socket.write_all(b"250 Ok: message accepted\r\n").await?;
                    data_buffer.clear();
                }
            }
            continue;
        }

        let response = match command.split_whitespace().next().unwrap_or("").to_uppercase().as_str() {
            "HELO" | "EHLO" => {
                let mut response = Vec::new();
                response.extend_from_slice(b"250-Simple SMTP Server\r\n");
                response.extend_from_slice(b"250-SIZE 10240000\r\n"); // 10MB max size
                response.extend_from_slice(b"250-8BITMIME\r\n");
                response.extend_from_slice(b"250 HELP\r\n");
                response
            },
            "MAIL" => {
                if let Some(from_idx) = command.find("FROM:") {
                    let from = command[from_idx + 5..].trim();
                    current_email.from = from.to_string();
                    info!("Mail from: {}", from);
                    b"250 Ok\r\n".to_vec()
                } else {
                    b"501 Syntax error in parameters or arguments\r\n".to_vec()
                }
            },
            "RCPT" => {
                if let Some(to_idx) = command.find("TO:") {
                    let to = command[to_idx + 3..].trim();
                    current_email.to.push(to.to_string());
                    info!("Rcpt to: {}", to);
                    b"250 Ok\r\n".to_vec()
                } else {
                    b"501 Syntax error in parameters or arguments\r\n".to_vec()
                }
            },
            "DATA" => {
                if current_email.from.is_empty() || current_email.to.is_empty() {
                    b"503 Bad sequence of commands\r\n".to_vec()
                } else {
                    data_mode = true;
                    //read data
                    b"354 Start mail input; end with <CRLF>.<CRLF>\r\n".to_vec()
                }
            },
            "QUIT" => {
                socket.write_all(b"221 Bye\r\n").await?;
                break;
            },
            "RSET" => {
                current_email = Email {
                    from: String::new(),
                    to: Vec::new(),
                    subject: String::new(),
                    body: String::new(),
                    attachments: Vec::new(),
                };
                data_buffer.clear();
                data_mode = false;
                b"250 Ok\r\n".to_vec()
            },
            _ => b"500 Unknown command\r\n".to_vec(),
        };

        info!("Sending response: {}", String::from_utf8_lossy(&response));
        socket.write_all(&response).await?;
    }

    Ok(())
}