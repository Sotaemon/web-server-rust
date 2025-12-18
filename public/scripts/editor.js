    // 模拟文档编辑功能
    document.addEventListener('DOMContentLoaded', function() {
      const textarea = document.getElementById('note-textarea');
      const wordCount = document.querySelector('.note-word-count');
      const charCount = document.querySelector('.note-char-count');
      
      // 更新字数统计
      function updateWordCount() {
        const text = textarea.value;
        const words = text.trim() === '' ? 0 : text.trim().split(/\s+/).length;
        const characters = text.length;
        
        wordCount.textContent = `<i class="fas fa-file-word"></i> ${words} words`;
        charCount.textContent = `<i class="fas fa-font"></i> ${characters} characters`;
      }
      
      // 初始更新
      updateWordCount();
      
      // 实时更新字数
      textarea.addEventListener('input', updateWordCount);
      
      // 保存功能
      document.querySelector('.btn-primary').addEventListener('click', function() {
        const title = document.querySelector('.note-title').value;
        const content = textarea.value;
        
        // 模拟保存到数据库
        console.log('Saving note:', {
          title: title,
          content: content,
          date: new Date().toISOString()
        });
        
        // 更新最后编辑时间
        document.querySelector('.note-date').textContent = `<i class="far fa-calendar-alt"></i> Last edited: ${new Date().toLocaleDateString('en-US', { 
          year: 'numeric', 
          month: 'long', 
          day: 'numeric' 
        })}`;
        
        // 显示保存成功提示
        alert('Note saved successfully!');
      });
      
      // 历史记录按钮
      document.querySelector('.btn-outline').addEventListener('click', function() {
        alert('Viewing note history... (In a real app, this would show version history)');
      });
      
      // 模拟标题输入
      document.querySelector('.note-title').addEventListener('input', function() {
        document.title = this.value + ' - NoteFlow';
      });
    });