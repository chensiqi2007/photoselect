/**
 * PhotoSelect Desktop Official Website Scripts
 * Interactive Darkroom Simulator, Hero Showcase, FAQ & Event Handlers
 */

document.addEventListener('DOMContentLoaded', () => {
  // 1. Header scroll effect
  const header = document.querySelector('.site-header');
  window.addEventListener('scroll', () => {
    if (window.scrollY > 30) {
      header?.classList.add('scrolled');
    } else {
      header?.classList.remove('scrolled');
    }
  });

  // 2. Mobile nav toggle
  const mobileToggle = document.querySelector('.mobile-toggle');
  const navLinks = document.querySelector('.nav-links');
  if (mobileToggle && navLinks) {
    mobileToggle.addEventListener('click', () => {
      navLinks.classList.toggle('open');
    });
  }

  // 3. Hero Mockup Switcher
  const mockupTabs = document.querySelectorAll('.mockup-tab-btn');
  const heroMockupImg = document.getElementById('heroMockupImage');
  
  const mockupViews = {
    'burst': {
      src: 'assets/images/feature-burst.svg',
      alt: 'SubSecTime 连拍聚合与 Best Shot 推荐选片'
    },
    'peaking': {
      src: 'assets/images/feature-peaking.svg',
      alt: 'WebGL 2.0 原生 Shader 硬件级峰值对焦'
    },
    'eye': {
      src: 'assets/images/feature-eye.svg',
      alt: 'Narrative Select 风格 100% 人眼特写焦点检视'
    },
    'compare': {
      src: 'assets/images/feature-compare.svg',
      alt: '基准图与候选图双图同屏联动比对与极速裁决'
    },
    'darkroom': {
      src: 'assets/images/hero-darkroom.svg',
      alt: 'PhotoSelect Studio Darkroom 全景工作台'
    }
  };

  mockupTabs.forEach(tab => {
    tab.addEventListener('click', () => {
      mockupTabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');

      const viewKey = tab.dataset.view;
      if (heroMockupImg && mockupViews[viewKey]) {
        heroMockupImg.style.opacity = '0';
        heroMockupImg.style.transform = 'scale(0.98)';
        setTimeout(() => {
          heroMockupImg.src = mockupViews[viewKey].src;
          heroMockupImg.alt = mockupViews[viewKey].alt;
          heroMockupImg.style.opacity = '1';
          heroMockupImg.style.transform = 'scale(1)';
        }, 160);
      }
    });
  });

  // 4. Interactive Keyboard Darkroom Simulator
  const simCard = document.getElementById('simCard');
  const simViewport = document.getElementById('simViewport');
  const simStarsEl = document.getElementById('simStars');
  const simTagPick = document.getElementById('simTagPick');
  const simTagReject = document.getElementById('simTagReject');
  const simStatusNotice = document.getElementById('simStatusNotice');
  const keyButtons = document.querySelectorAll('.key-btn');

  let simState = {
    stars: 5,
    flag: 'pick', // 'pick', 'reject', 'unflag'
    peaking: false,
    zoomed: false,
    colorTag: 'none'
  };

  function updateSimulatorUI() {
    if (!simCard) return;

    // Stars
    if (simStarsEl) {
      let starsStr = '';
      for (let i = 1; i <= 5; i++) {
        starsStr += i <= simState.stars ? '★' : '☆';
      }
      simStarsEl.textContent = starsStr;
    }

    // Flags
    if (simTagPick && simTagReject) {
      if (simState.flag === 'pick') {
        simTagPick.style.display = 'inline-block';
        simTagReject.style.display = 'none';
        simCard.style.borderColor = '#10b981';
      } else if (simState.flag === 'reject') {
        simTagPick.style.display = 'none';
        simTagReject.style.display = 'inline-block';
        simCard.style.borderColor = '#ef4444';
      } else {
        simTagPick.style.display = 'none';
        simTagReject.style.display = 'none';
        simCard.style.borderColor = 'rgba(255, 255, 255, 0.1)';
      }
    }

    // Peaking
    if (simViewport) {
      if (simState.peaking) {
        simViewport.classList.add('sim-peaking-active');
      } else {
        simViewport.classList.remove('sim-peaking-active');
      }

      // Zoom
      if (simState.zoomed) {
        simViewport.classList.add('sim-zoom-active');
      } else {
        simViewport.classList.remove('sim-zoom-active');
      }
    }
  }

  function handleSimAction(action, key) {
    if (!action) return;

    // Flash key button visually
    keyButtons.forEach(btn => {
      if (btn.dataset.key === key || btn.dataset.action === action) {
        btn.classList.add('active');
        setTimeout(() => btn.classList.remove('active'), 200);
      }
    });

    switch (action) {
      case 'star-0': simState.stars = 0; break;
      case 'star-1': simState.stars = 1; break;
      case 'star-2': simState.stars = 2; break;
      case 'star-3': simState.stars = 3; break;
      case 'star-4': simState.stars = 4; break;
      case 'star-5': simState.stars = 5; break;
      case 'pick':
        simState.flag = simState.flag === 'pick' ? 'unflag' : 'pick';
        break;
      case 'reject':
        simState.flag = simState.flag === 'reject' ? 'unflag' : 'reject';
        break;
      case 'unflag':
        simState.flag = 'unflag';
        break;
      case 'peaking':
        simState.peaking = !simState.peaking;
        break;
      case 'zoom':
        simState.zoomed = !simState.zoomed;
        break;
      case 'swap':
        // simulate swap
        if (simCard) {
          simCard.style.transform = 'scale(0.97)';
          setTimeout(() => simCard.style.transform = 'scale(1)', 150);
        }
        break;
    }

    updateSimulatorUI();
  }

  // Click on on-screen keys
  keyButtons.forEach(btn => {
    btn.addEventListener('click', () => {
      const action = btn.dataset.action;
      const key = btn.dataset.key;
      handleSimAction(action, key);
    });
  });

  // Physical keyboard support when simulator area is active
  const simulatorArea = document.querySelector('.simulator-wrapper');
  if (simulatorArea) {
    window.addEventListener('keydown', (e) => {
      // Check if mouse is hovering over simulator or in view
      const rect = simulatorArea.getBoundingClientRect();
      const inView = rect.top < window.innerHeight && rect.bottom > 0;
      if (!inView) return;

      const key = e.key.toUpperCase();
      if (key >= '0' && key <= '5') {
        handleSimAction(`star-${key}`, key);
      } else if (key === 'P') {
        handleSimAction('pick', 'P');
      } else if (key === 'X') {
        handleSimAction('reject', 'X');
      } else if (key === 'U') {
        handleSimAction('unflag', 'U');
      } else if (key === 'F') {
        handleSimAction('peaking', 'F');
      } else if (key === ' ' || e.code === 'Space') {
        e.preventDefault();
        handleSimAction('zoom', 'SPACE');
      } else if (key === 'S') {
        handleSimAction('swap', 'S');
      }
    });
  }

  // 5. FAQ Accordion
  const faqItems = document.querySelectorAll('.faq-item');
  faqItems.forEach(item => {
    const question = item.querySelector('.faq-question');
    question?.addEventListener('click', () => {
      const isActive = item.classList.contains('active');
      faqItems.forEach(i => i.classList.remove('active'));
      if (!isActive) {
        item.classList.add('active');
      }
    });
  });

  // 6. Camera Model Live Filter
  const cameraSearchInput = document.getElementById('cameraSearchInput');
  const cameraCards = document.querySelectorAll('.camera-card');
  if (cameraSearchInput && cameraCards.length > 0) {
    cameraSearchInput.addEventListener('input', (e) => {
      const term = e.target.value.toLowerCase().trim();
      cameraCards.forEach(card => {
        const text = card.textContent.toLowerCase();
        if (text.includes(term)) {
          card.style.display = 'block';
        } else {
          card.style.display = 'none';
        }
      });
    });
  }

  // 7. Initial state
  updateSimulatorUI();
});
