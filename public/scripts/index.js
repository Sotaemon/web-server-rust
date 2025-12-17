document.addEventListener('DOMContentLoaded', () => {
  const registerBtn = document.querySelector('.register-btn');
  const loginBtn = document.querySelector('.login-btn');

  // 按钮点击动画 + 提示（真实项目替换为路由）
  registerBtn.addEventListener('click', () => {
    alert('✨ 正在跳转注册页！（Rust后端已就绪，速度超快~）');
    // 实际项目：window.location.href = '/register';
  });

  loginBtn.addEventListener('click', () => {
    alert('🚀 正在登录！AI模糊搜索已启动，找笔记像呼吸一样简单');
    // 实际项目：window.location.href = '/login';
  });

  // 顶部标题加个微动效（提升沉浸感）
  const title = document.querySelector('h1');
  title.style.opacity = '0';
  title.style.transition = 'opacity 0.8s ease';
  setTimeout(() => {
    title.style.opacity = '1';
  }, 300);
});