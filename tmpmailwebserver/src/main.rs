use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
mod static_files;
use static_files::Asset;

use actix_cors::Cors;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{Serialize, Deserialize};
use log::info;
use clap::Parser;
mod mail;
mod db;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'p', long = "port", default_value = "8080")]
    port: u16,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    code: i32,
    msg: String,
    data: T,
}

#[derive(Serialize)]
struct EmailResponse {
    email: String,
}

#[derive(Deserialize)]
struct InboxRequest {
    email: String,
}

#[derive(Serialize)]
struct InboxResponse {
    id: u32,
    from_name: String,
    from_email: String,
    subject: String,
    created_at: String,
}

#[post("/api/mail/inbox")]
async fn get_inbox(req: web::Json<InboxRequest>) -> impl Responder {
    match db::find_by_email(&req.email).await {
        Ok(emails) => {
            let email_list: Vec<InboxResponse> = emails
                .into_iter()
                .map(|email| InboxResponse {
                    id: email.id,
                    from_name: email.from_name.unwrap_or_default(),
                    from_email: email.from_email.unwrap_or_default(),
                    subject: email.subject.unwrap_or_default(),
                    created_at: email.created_at
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default(),
                })
                .collect();

            let response = ApiResponse {
                code: 200,
                msg: String::from("success"),
                data: email_list,
            };
            HttpResponse::Ok().json(response)
        },
        Err(_) => {
            let response = ApiResponse {
                code: 500,
                msg: String::from("服务器内部错误"),
                data: Vec::<InboxResponse>::new(),
            };
            HttpResponse::InternalServerError().json(response)
        }
    }
}

#[get("/api/mail/refresh")]
async fn refresh_email() -> impl Responder {
    let email: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let email = email+"@mm.swiftr.top";
    let response = ApiResponse {
        code: 200,
        msg: String::from("success"),
        data: EmailResponse { email },
    };
    HttpResponse::Ok().json(response)
}


#[derive(Deserialize)]
struct DetailRequest {
    id: u32,
}

#[derive(Serialize)]
struct DetailResponse {
    id: u32,
    email: String,
    from_name: String,
    from_email: String,
    subject: String,
    content: String,
    created_at: String,
}

#[post("/api/mail/detail")]
async fn get_detail(req: web::Json<DetailRequest>) -> impl Responder {
    match db::find_by_id(req.id).await {
        Ok(Some(email)) => {
            let response = ApiResponse {
                code: 200,
                msg: String::from("success"),
                data: DetailResponse {
                    id: email.id,
                    email: email.email.unwrap_or_default(),
                    from_name: email.from_name.unwrap_or_default(),
                    from_email: email.from_email.unwrap_or_default(),
                    subject: email.subject.unwrap_or_default(),
                    content: email.content.unwrap_or_default(),
                    created_at: email.created_at
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default(),
                },
            };
            HttpResponse::Ok().json(response)
        },
        Ok(None) => {
            let response = ApiResponse {
                code: 404,
                msg: String::from("邮件不存在"),
                data: Option::<DetailResponse>::None,
            };
            HttpResponse::NotFound().json(response)
        },
        Err(_) => {
            let response = ApiResponse {
                code: 500,
                msg: String::from("服务器内部错误"),
                data: Option::<DetailResponse>::None,
            };
            HttpResponse::InternalServerError().json(response)
        }
    }
}

async fn serve_static_files(path: web::Path<String>) -> impl Responder {
    let path = path.into_inner();
    let path = if path.is_empty() { "index.html".to_string() } else { path };
    
    match Asset::get(path.as_str()) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(mime.as_ref())
                .body(content.data.into_owned())
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志系统
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("Starting server...");

    // 使用clap解析命令行参数
    let args = Args::parse();
    let port = args.port;
    info!("Server will start on port {}", port);

    info!("init db ...");
    db::init_db().await.unwrap();

    // 启动Web服务器
    let web_handle = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .service(refresh_email)
            .service(get_inbox)
            .service(get_detail)
            .service(web::resource("/{filename:.*}").to(serve_static_files))  // 添加静态文件处理
            .service(web::resource("/{filename:.*}").to(serve_static_files))  // 添加静态文件处理
    })
    .bind(("0.0.0.0", port))?
    .run();

    // 等待两个服务器
    tokio::select! {
        _ = web_handle => Ok(()),
    }
}