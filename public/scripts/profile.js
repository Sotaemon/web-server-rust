// 模拟用户数据（实际应用中会从API获取）
const currentUser = {
    id: "user123",
    name: "John Doe",
    username: "john_doe",
    email: "john.doe@example.com",
    joined: "January 15, 2023",
    lastActivity: "Today at 10:30 AM",
    notesCount: 24,
    projectsCount: 12,
    teamsCount: 3
};

// 更新页面内容
document.addEventListener('DOMContentLoaded', function() {
    // 更新用户信息
    document.querySelector('.profile-name').textContent = currentUser.name;
    document.querySelector('.profile-email').textContent = currentUser.email;
    document.querySelector('.stat-value:nth-child(1)').textContent = currentUser.notesCount;
    document.querySelector('.stat-value:nth-child(2)').textContent = currentUser.projectsCount;
    document.querySelector('.stat-value:nth-child(3)').textContent = currentUser.teamsCount;      
    // 更新用户名    
    document.querySelector('.info-value:nth-child(1)').textContent = currentUser.username;      
    // 模拟编辑功能
    const editButtons = document.querySelectorAll('.edit-btn');
      
    editButtons.forEach(button => {
        button.addEventListener('click', function() {
            const infoValue = this.closest('.info-row').querySelector('.info-value');
            const currentValue = infoValue.textContent;          
        // 如果是密码，显示密码输入框
            if (this.closest('.info-row').querySelector('.info-label').textContent === 'Password') {
                infoValue.innerHTML = '<input type="password" class="password-input" value="' + currentValue + '">';
                this.style.display = 'none';
                return;
            }
          
          // 其他字段使用输入框
            infoValue.innerHTML = '<input type="text" class="edit-input" value="' + currentValue + '">';
            this.style.display = 'none';
          
          // 添加保存按钮
            const saveBtn = document.createElement('button');
            saveBtn.className = 'btn-save';
            saveBtn.innerHTML = '<i class="fas fa-check"></i>';
            saveBtn.style.marginLeft = '5px';
          
            this.closest('.info-row').appendChild(saveBtn);
          
          // 保存编辑
            saveBtn.addEventListener('click', function() {
                const newValue = this.previousSibling.value;
                infoValue.textContent = newValue;
                this.remove();
                this.previousSibling.remove();
                this.closest('.info-row').querySelector('.edit-btn').style.display = 'flex';
            });
        });
    });
      
      // 退出登录
    document.getElementById('logoutBtn').addEventListener('click', function() {
        localStorage.removeItem('authToken');
        window.location.href = 'login.html';
    });
      
      // 保存更改
    document.querySelector('.btn-save').addEventListener('click', function() {
        alert('Profile updated successfully!');
    });
      
      // 取消编辑
    document.querySelector('.btn-cancel').addEventListener('click', function() {
        alert('Changes discarded');
    });
});