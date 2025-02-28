// API配置
import { API_BASE_URL } from './config.js';
let currentEmail = '';

// 更新邮箱地址显示
function updateEmailAddress(email) {
    currentEmail = email;
    const emailAddressElement = document.getElementById('email-address');
    emailAddressElement.textContent = email || 'Generating email address...';
}

// 格式化日期时间
function formatDateTime(date) {
    const options = {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false
    };
    return new Date(date).toLocaleString('zh-CN', options);
}

// 创建邮件列表项
function createEmailItem(email) {
    const item = document.createElement('div');
    item.className = 'inbox-item';
    item.innerHTML = `
        <div class="subject">${email.subject}</div>
        <div class="sender">${email.from_email}${email.from_name ? ` (${email.from_name})` : ''}</div>
        <div class="time">${formatDateTime(email.created_at)}</div>
    `;
    item.addEventListener('click', () => {
        window.location.href = `detail.html?id=${email.id}`;
    });
    return item;
}

// 更新收件箱显示
function updateInbox(emails) {
    const inboxContent = document.getElementById('inbox-content');
    if (emails.length === 0) {
        inboxContent.innerHTML = '<div class="empty-inbox">暂无邮件</div>';
        return;
    }

    inboxContent.innerHTML = '';
    emails.forEach(email => {
        inboxContent.appendChild(createEmailItem(email));
    });
}

// 获取邮件列表
function fetchEmails() {
    return fetch(`${API_BASE_URL}/api/mail/refresh`)
        .then(response => {
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            return response.json();
        })
        .then(result => {
            if (result.code === 200) {
                updateEmailAddress(result.data.email);
            } else {
                throw new Error('Failed to get email address');
            }
        })
        .catch(error => {
            console.error('Failed to fetch emails:', error);
        });
}

// 获取收件箱内容
function fetchInbox() {
    if (!currentEmail) return;

    return fetch(`${API_BASE_URL}/api/mail/inbox`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            email: currentEmail
        })
    })
        .then(response => {
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            return response.json();
        })
        .then(result => {
            if (result.code === 200) {
                updateInbox(result.data);
            } else {
                throw new Error('Failed to get inbox');
            }
        })
        .catch(error => {
            console.error('Failed to get inbox:', error);
        });
}

// 初始化邮箱
async function initializeEmail() {
    await fetchEmails();
    // 启动自动刷新
    setInterval(fetchInbox, 5000);
}

// 刷新按钮点击事件
document.getElementById('refresh-btn').addEventListener('click', async () => {
    document.getElementById('refresh-btn').disabled = true;
    document.getElementById('refresh-btn').textContent = 'Refreshing...';
    
    try {
        await fetchEmails();
        await fetchInbox();
    } catch (error) {
        console.error('Failed to refresh emails:', error);
    } finally {
        document.getElementById('refresh-btn').disabled = false;
        document.getElementById('refresh-btn').textContent = 'Refresh Email';
    }
});

// 页面加载完成后初始化
document.addEventListener('DOMContentLoaded', initializeEmail);