document.addEventListener('DOMContentLoaded', () => {
  document.querySelectorAll('.feature-card').forEach(card => {
    card.addEventListener('mouseenter', () => {
      card.style.boxShadow = '0 8px 25px rgba(0, 0, 0, 0.1)';
    });
    
    card.addEventListener('mouseleave', () => {
      card.style.boxShadow = '0 5px 15px rgba(0, 0, 0, 0.05)';
    });
  });
});