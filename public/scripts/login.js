// 为登录表单添加提交事件监听器
document.getElementById('loginForm').addEventListener('submit', async (e) => {
    // 阻止表单默认提交行为，避免页面刷新
    e.preventDefault();
    
    // 收集表单数据
    const formData = {
        username: document.getElementById('username').value,  // 获取用户名输入值
        password: document.getElementById('password').value   // 获取密码输入值
    };
    
    try {
        // 发送POST请求到/api/login端点进行用户认证
        const response = await fetch('/api/login', {
          method: 'POST',                     // 使用POST方法
          headers: { 'Content-Type': 'application/json' },  // 设置请求头为JSON格式
          body: JSON.stringify(formData)       // 将表单数据转换为JSON字符串发送
        });
        
        // 解析服务器返回的JSON响应
        const result = await response.json();
        
        // 检查HTTP响应状态是否成功
        if (response.ok) {
          // 登录成功：显示成功提示
          alert('Success!');
          // 重定向到仪表板页面
          window.location.href = './dashboard.html';
        } else {
          // 登录失败：显示错误信息
          alert('Error:' + result.message);
        }
      } catch (err) {
        // 网络错误处理：显示连接错误提示
        alert('Connect Error, Retry Later Please');
      }
    });