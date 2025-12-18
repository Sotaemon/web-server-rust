document.addEventListener('DOMContentLoaded', () => {
  const registerForm = document.querySelector('.register-form');
  registerForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    
    // 获取表单数据
    const formData = {
      username: document.getElementById('username').value.trim(),
      email: document.getElementById('email').value.trim(),
      password: document.getElementById('password').value
    };
    
    // 验证必填字段
    if (!formData.username || !formData.email || !formData.password) {
      alert('所有字段都是必填的！请检查用户名、邮箱和密码。');
      return;
    }

    // 显示加载状态
    const submitBtn = document.querySelector('.register-btn');
    const originalBtnText = submitBtn.textContent;
    submitBtn.textContent = '注册中...';
    submitBtn.disabled = true;

    try {
      const response = await fetch('/api/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(formData)
      });

      const result = await response.json();
      
      if (response.ok) {
        // 注册成功处理
        alert('注册成功！欢迎使用 NoteFlow！\n请前往登录页面使用新账号。');
        window.location.href = 'login.html';
      } else {
        // 错误处理（更友好的提示）
        let errorMessage = '注册失败';
        if (result.error) {
          errorMessage = result.error;
        } else if (result.message) {
          errorMessage = result.message;
        }
        
        // 根据错误类型提供特定建议
        if (errorMessage.includes('email')) {
          errorMessage += '\n请检查邮箱格式是否正确';
        } else if (errorMessage.includes('username')) {
          errorMessage += '\n用户名已被使用';
        } else if (errorMessage.includes('password')) {
          errorMessage += '\n密码需至少8个字符';
        }
        
        alert(`注册失败: ${errorMessage}`);
      }
    } catch (error) {
      console.error('注册请求失败:', error);
      alert('网络连接问题。请检查您的互联网连接并重试。');
    } finally {
      // 恢复按钮状态
      submitBtn.textContent = originalBtnText;
      submitBtn.disabled = false;
    }
  });
});