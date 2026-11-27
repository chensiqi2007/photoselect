/**
 * PhotoSelect Documentation Website Interactive Features
 * Search filter, Scrollspy, Code Copy, and Mobile Navigation
 */

document.addEventListener('DOMContentLoaded', () => {
  // 1. Code block copy functionality
  const copyButtons = document.querySelectorAll('.code-copy-btn');
  copyButtons.forEach(btn => {
    btn.addEventListener('click', () => {
      const codeWrapper = btn.closest('.code-block-wrapper');
      const codeEl = codeWrapper ? codeWrapper.querySelector('pre.code-content') : null;
      if (!codeEl) return;

      const text = codeEl.innerText;
      navigator.clipboard.writeText(text).then(() => {
        const originalHtml = btn.innerHTML;
        const isEnglish = document.documentElement.lang.startsWith('en');
        btn.innerHTML = `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg> <span style="color:#10b981;">${isEnglish ? 'Copied!' : '已复制!'}</span>`;
        setTimeout(() => {
          btn.innerHTML = originalHtml;
        }, 2000);
      }).catch(err => {
        console.error('Copy failed:', err);
      });
    });
  });

  // 2. Interactive search filter
  const searchInput = document.getElementById('docsSearchInput');
  const navItems = document.querySelectorAll('.docs-nav-item');
  if (searchInput) {
    searchInput.addEventListener('input', (e) => {
      const query = e.target.value.toLowerCase().trim();
      navItems.forEach(item => {
        const text = item.textContent.toLowerCase();
        if (!query || text.includes(query)) {
          item.style.display = 'block';
        } else {
          item.style.display = 'none';
        }
      });
    });

    // Keyboard shortcut: Ctrl + K or / to focus search
    window.addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        searchInput.focus();
      }
    });
  }

  // 3. Mobile sidebar toggle
  const mobileToggle = document.getElementById('docsMobileToggle');
  const sidebarLeft = document.getElementById('docsSidebarLeft');
  if (mobileToggle && sidebarLeft) {
    mobileToggle.addEventListener('click', () => {
      sidebarLeft.classList.toggle('open');
    });

    // Close when clicking an anchor link inside sidebar on mobile
    sidebarLeft.querySelectorAll('a').forEach(a => {
      a.addEventListener('click', () => {
        if (window.innerWidth < 900) {
          sidebarLeft.classList.remove('open');
        }
      });
    });
  }

  // 4. Scrollspy for Right Table of Contents
  const sections = document.querySelectorAll('.docs-section');
  const tocLinks = document.querySelectorAll('.toc-item a');
  const navLinks = document.querySelectorAll('.docs-nav-item a');

  function updateActiveToc() {
    let currentId = '';
    const scrollPos = window.scrollY + 120;

    sections.forEach(sec => {
      const top = sec.offsetTop;
      const height = sec.offsetHeight;
      if (scrollPos >= top && scrollPos < top + height) {
        currentId = sec.id;
      }
    });

    if (currentId) {
      // Update Right TOC
      tocLinks.forEach(link => {
        const href = link.getAttribute('href');
        if (href === `#${currentId}`) {
          link.parentElement.classList.add('active');
        } else {
          link.parentElement.classList.remove('active');
        }
      });

      // Update Left Nav
      navLinks.forEach(link => {
        const href = link.getAttribute('href');
        if (href === `#${currentId}`) {
          link.parentElement.classList.add('active');
        } else {
          link.parentElement.classList.remove('active');
        }
      });
    }
  }

  window.addEventListener('scroll', updateActiveToc, { passive: true });
  updateActiveToc();
});

