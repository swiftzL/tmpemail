// API配置
import { API_BASE_URL } from './config.js';

// 获取URL参数
function getUrlParam(param) {
    const urlParams = new URLSearchParams(window.location.search);
    return urlParams.get(param);
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

// 更新邮件详情显示
function updateEmailDetail(email) {
    document.getElementById('email-subject').textContent = email.subject;
    document.getElementById('email-from').textContent = `发件人：${email.from_email}${email.from_name ? ` (${email.from_name})` : ''}`;
    document.getElementById('email-time').textContent = formatDateTime(email.created_at);
    
    const contentElement = document.getElementById('email-content');
    contentElement.innerHTML = '';
    
    // 创建一个安全的容器来显示HTML内容
    const contentContainer = document.createElement('div');
    contentContainer.innerHTML = email.content;
    contentElement.appendChild(contentContainer);
}

// 获取邮件详情
async function fetchEmailDetail() {
    const emailId = getUrlParam('id');
    if (!emailId) {
        alert('邮件ID不能为空');
        window.location.href = 'index.html';
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/api/mail/detail`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                id: Number.parseInt(emailId)
            })
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const result = await response.json();
        if (result.code === 200) {
            updateEmailDetail(result.data);
        } else {
            throw new Error('获取邮件详情失败');
        }
    } catch (error) {
        console.error('获取邮件详情失败:', error);
        alert('获取邮件详情失败');
        window.location.href = 'index.html';
    }
}

// 页面加载完成后初始化
document.addEventListener('DOMContentLoaded', fetchEmailDetail);