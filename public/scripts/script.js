// 平滑滚动到锚点
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
        e.preventDefault();
        
        const targetId = this.getAttribute('href');
        if(targetId === '#') return;
        
        const targetElement = document.querySelector(targetId);
        if(targetElement) {
            window.scrollTo({
                top: targetElement.offsetTop - 80,
                behavior: 'smooth'
            });
        }
    });
});

// 按钮悬停效果
document.querySelectorAll('.btn').forEach(button => {
    button.addEventListener('mouseenter', function() {
        this.style.transform = 'translateY(-2px)';
    });
    
    button.addEventListener('mouseleave', function() {
        this.style.transform = 'translateY(0)';
    });
});

// 特性卡片悬停动画
document.querySelectorAll('.feature-card').forEach(card => {
    card.addEventListener('mouseenter', function() {
        this.style.transform = 'translateY(-5px)';
    });
    
    card.addEventListener('mouseleave', function() {
        this.style.transform = 'translateY(0)';
    });
});

// 窗口滚动时的导航栏效果
window.addEventListener('scroll', function() {
    const nav = document.querySelector('nav');
    if(window.scrollY > 50) {
        nav.style.boxShadow = '0 4px 20px rgba(0, 0, 0, 0.1)';
        nav.style.background = 'rgba(230, 227, 232, 0.95)';
    } else {
        nav.style.boxShadow = '0 2px 10px rgba(0, 0, 0, 0.05)';
        nav.style.background = '';
    }
});

// 页面加载完成后添加动画效果
document.addEventListener('DOMContentLoaded', function() {
    // 为所有功能卡片添加延迟动画
    const featureCards = document.querySelectorAll('.feature-card');
    featureCards.forEach((card, index) => {
        setTimeout(() => {
            card.style.opacity = '1';
            card.style.transform = 'translateY(0)';
        }, 200 * index);
    });
    
    // 为所有元素添加初始样式
    featureCards.forEach(card => {
        card.style.opacity = '0';
        card.style.transform = 'translateY(20px)';
        card.style.transition = 'opacity 0.5s ease, transform 0.5s ease';
    });
});

// 模拟代码高亮
function highlightCode() {
    const codeBlocks = document.querySelectorAll('pre code');
    codeBlocks.forEach(block => {
        const text = block.textContent;
        // 简单的语法高亮示例
        let highlighted = text
            .replace(/(fn|let|struct|impl|pub|mod|use|match|if|else|for|while|loop)\b/g, '<span style="color:var(--syntax-purple);">$1</span>')
            .replace(/(println!|print!)\b/g, '<span style="color:var(--syntax-green);">$1</span>')
            .replace(/(".*?")/g, '<span style="color:var(--syntax-yellow);">$1</span>')
            .replace(/(\d+)/g, '<span style="color:var(--syntax-cyan);">$1</span>');
        
        block.innerHTML = highlighted;
    });
}

// 页面加载后执行高亮
document.addEventListener('DOMContentLoaded', highlightCode);
