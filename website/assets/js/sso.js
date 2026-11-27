/**
 * URL Time SSO Client Integration
 * Platform: https://login.urltime.cn
 * Client ID: app-t1i11ft3
 * Primary Redirect URI: https://www.urltime.org/app/index.html
 *
 * Flow:
 * 1. Client initiates login, records returnUrl in sessionStorage, redirects to:
 *    https://login.urltime.cn/auth/login?client_id=app-t1i11ft3&redirect_uri=https://www.urltime.org/app/index.html&state={state}
 * 2. login.urltime.cn matches client_id and redirect_uri against its whitelist, authenticates user,
 *    and redirects back to: https://www.urltime.org/app/index.html?code={code}&state={state}
 * 3. index.html captures code/state, saves login session in localStorage,
 *    and redirects back to returnUrl (e.g. docs.html or docs-en.html).
 */

(function (window) {
  'use strict';

  const URLTIME_SSO = {
    config: {
      serverUrl: 'https://login.urltime.cn',
      authPath: '/auth/login',
      clientId: 'app-t1i11ft3',
      appName: 'PhotoSelect Desktop',
      // Primary whitelist callback registered on login.urltime.cn
      primaryRedirectUri: 'https://www.urltime.org/app/index.html',
      storageKey: 'urltime_sso_session',
      stateKey: 'urltime_sso_state',
      returnUrlKey: 'urltime_sso_return_url'
    },

    /**
     * Get authorized Redirect URI.
     * login.urltime.cn strictly verifies that the redirect_uri query parameter matches
     * the authorized list configured in the SSO App console.
     * Defaulting to the primary authorized callback: https://www.urltime.org/app/index.html
     */
    getRedirectUri: function () {
      return this.config.primaryRedirectUri;
    },

    /**
     * Generate secure random string for OAuth state (32 to 64 chars)
     * URL Time SSO validator: /^[A-Za-z0-9_-]{32,128}$/
     */
    randomString: function (len) {
      len = len || 36;
      const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-';
      let res = '';
      if (window.crypto && window.crypto.getRandomValues) {
        const bytes = new Uint8Array(len);
        window.crypto.getRandomValues(bytes);
        for (let i = 0; i < len; i++) {
          res += chars[bytes[i] % chars.length];
        }
      } else {
        for (let i = 0; i < len; i++) {
          res += chars.charAt(Math.floor(Math.random() * chars.length));
        }
      }
      return res;
    },

    /**
     * Decode JWT Payload safely with UTF-8 support (if token is returned)
     */
    decodeJwt: function (token) {
      try {
        if (!token || typeof token !== 'string') return null;
        const parts = token.split('.');
        if (parts.length < 2) return null;
        const base64Url = parts[1];
        const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
        const jsonPayload = decodeURIComponent(
          atob(base64)
            .split('')
            .map(function (c) {
              return '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2);
            })
            .join('')
        );
        return JSON.parse(jsonPayload);
      } catch (err) {
        return null;
      }
    },

    /**
     * Initiate SSO Login redirect to https://login.urltime.cn
     */
    login: function (customRedirectUri) {
      // Must be at least 32 characters matching /^[A-Za-z0-9_-]{32,128}$/
      const state = this.randomString(36);
      const redirectUri = customRedirectUri || this.getRedirectUri();

      try {
        sessionStorage.setItem(this.config.stateKey, state);
        sessionStorage.setItem(this.config.returnUrlKey, window.location.href);
      } catch (e) {
        console.warn('[URL Time SSO] sessionStorage unavailable:', e);
      }

      // Format: https://login.urltime.cn/auth/login?client_id=app-t1i11ft3&redirect_uri=...&state=...
      const params = new URLSearchParams({
        client_id: this.config.clientId,
        redirect_uri: redirectUri,
        state: state
      });

      const authUrl = `${this.config.serverUrl}${this.config.authPath}?${params.toString()}`;
      window.location.href = authUrl;
    },

    /**
     * Sign out current user and clear local session
     */
    logout: function () {
      try {
        localStorage.removeItem(this.config.storageKey);
      } catch (e) {
        console.warn(e);
      }
      this.renderAllWidgets();
      const event = new CustomEvent('urltime:sso:logout');
      window.dispatchEvent(event);
    },

    /**
     * Get stored user session
     */
    getCurrentUser: function () {
      try {
        const raw = localStorage.getItem(this.config.storageKey);
        if (!raw) return null;
        const session = JSON.parse(raw);
        // Check session expiration (default 7 days)
        if (session.exp && Date.now() >= session.exp * 1000) {
          localStorage.removeItem(this.config.storageKey);
          return null;
        }
        return session.user;
      } catch (e) {
        return null;
      }
    },

    /**
     * Parse code or token from URL redirect callback
     */
    handleCallback: function () {
      let code = null;
      let token = null;
      let state = null;

      // 1. Check Query Parameters for authorization code: ?code=...&state=...
      if (window.location.search) {
        const searchParams = new URLSearchParams(window.location.search);
        if (searchParams.has('code')) {
          code = searchParams.get('code');
          state = searchParams.get('state');
        } else if (searchParams.has('access_token') || searchParams.has('token')) {
          token = searchParams.get('access_token') || searchParams.get('token');
          state = searchParams.get('state');
        }
      }

      // 2. Check Hash Fragment (#access_token=...&state=...)
      if (!code && !token && window.location.hash && window.location.hash.length > 1) {
        const hashParams = new URLSearchParams(window.location.hash.substring(1));
        if (hashParams.has('access_token')) {
          token = hashParams.get('access_token');
          state = hashParams.get('state');
        } else if (hashParams.has('code')) {
          code = hashParams.get('code');
          state = hashParams.get('state');
        }
      }

      if (code || token) {
        let claims = token ? this.decodeJwt(token) : null;
        let displayName = claims ? (claims.displayName || claims.name || claims.preferred_username) : 'URL Time 用户';
        let userEmail = claims ? (claims.email || '') : '';
        let userAvatar = claims ? (claims.avatar || claims.picture || '') : '';

        // If returned authorization code
        if (code && !claims) {
          displayName = 'URL Time 认证用户';
          userEmail = 'SSO 凭证: ' + code.substring(0, 10) + '...';
        }

        const user = {
          id: (claims && (claims.sub || claims.id)) || (code ? 'code-' + code.substring(0, 8) : 'urltime-user'),
          name: displayName,
          displayName: displayName,
          avatar: userAvatar,
          email: userEmail,
          clientId: this.config.clientId,
          authCode: code || null,
          token: token || null
        };

        const session = {
          code: code,
          token: token,
          exp: Math.floor(Date.now() / 1000) + 86400 * 7,
          user: user
        };

        try {
          localStorage.setItem(this.config.storageKey, JSON.stringify(session));
        } catch (e) {
          console.error('[URL Time SSO] Failed to store session:', e);
        }

        // Clean up URL without triggering full page reload
        if (window.history && window.history.replaceState) {
          try {
            let cleanUrl = window.location.pathname;
            if (window.location.search) {
              const sp = new URLSearchParams(window.location.search);
              sp.delete('code');
              sp.delete('state');
              sp.delete('access_token');
              sp.delete('token');
              const qs = sp.toString();
              if (qs) cleanUrl += '?' + qs;
            }
            window.history.replaceState({}, document.title, cleanUrl);
          } catch (err) {
            // file:/// scheme fallback
          }
        }

        // Notify application
        const event = new CustomEvent('urltime:sso:login', { detail: { user: user } });
        window.dispatchEvent(event);

        // Auto redirect back to original page if returning via index.html callback
        try {
          const returnUrl = sessionStorage.getItem(this.config.returnUrlKey);
          if (returnUrl) {
            sessionStorage.removeItem(this.config.returnUrlKey);
            // Only redirect if returnUrl differs from current page URL
            if (returnUrl !== window.location.href && !returnUrl.endsWith(window.location.pathname)) {
              let target = returnUrl;
              // If returnUrl was local file://, navigate to matching online URL
              if (target.startsWith('file:')) {
                const fname = target.split('/').pop() || 'docs.html';
                target = 'https://www.urltime.org/app/' + fname;
              }
              window.location.replace(target);
              return user;
            }
          }
        } catch (e) {
          console.warn('[URL Time SSO] Return URL redirection skipped:', e);
        }

        return user;
      }

      return this.getCurrentUser();
    },

    /**
     * Render the SSO button or user avatar dropdown in target container
     */
    renderWidget: function (container, isEnglish) {
      if (!container) return;
      const user = this.getCurrentUser();
      const lang = isEnglish ? 'en' : 'zh';

      if (!user) {
        // Unauthenticated State
        const label = lang === 'en' ? 'URL Time SSO' : 'URL Time SSO 登录';
        container.innerHTML = `
          <button class="urltime-sso-btn" type="button" title="${lang === 'en' ? 'Sign in with URL Time Login' : '使用 URL Time 统一身份中心安全登录'}">
            <svg class="sso-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
              <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
            </svg>
            <span>${label}</span>
          </button>
        `;
        const btn = container.querySelector('.urltime-sso-btn');
        btn.addEventListener('click', () => {
          this.login();
        });
      } else {
        // Authenticated State
        const displayName = user.displayName || user.name || 'User';
        const initial = displayName.charAt(0).toUpperCase();
        const avatarHtml = user.avatar
          ? `<img src="${user.avatar}" alt="${displayName}" class="sso-user-avatar-img" onerror="this.style.display='none';this.nextElementSibling.style.display='flex';" /><span class="sso-user-avatar-fallback" style="display:none;">${initial}</span>`
          : `<span class="sso-user-avatar-fallback">${initial}</span>`;

        container.innerHTML = `
          <div class="urltime-sso-user-dropdown">
            <button class="urltime-sso-profile-btn" type="button" aria-expanded="false" title="${displayName} (${lang === 'en' ? 'Click for menu' : '点击展开个人菜单'})">
              <div class="sso-avatar-wrap">
                ${avatarHtml}
              </div>
              <span class="sso-user-name">${displayName}</span>
              <svg class="sso-caret" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="6 9 12 15 18 9"/></svg>
            </button>
            <div class="sso-dropdown-menu">
              <div class="sso-dropdown-header">
                <div class="sso-dropdown-name">${displayName}</div>
                <div class="sso-dropdown-sub">${user.email || user.id || 'Client: app-t1i11ft3'}</div>
                <div class="sso-badge-verified">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg>
                  <span>${lang === 'en' ? 'SSO Verified' : 'URL Time 认证用户'}</span>
                </div>
              </div>
              <div class="sso-dropdown-divider"></div>
              <a href="${this.config.serverUrl}/account/profile" target="_blank" rel="noopener" class="sso-dropdown-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
                <span>${lang === 'en' ? 'Account Center' : 'URL Time 账号中心'}</span>
              </a>
              <div class="sso-dropdown-divider"></div>
              <button type="button" class="sso-dropdown-item sso-logout-btn">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
                <span>${lang === 'en' ? 'Sign Out' : '退出登录'}</span>
              </button>
            </div>
          </div>
        `;

        const profileBtn = container.querySelector('.urltime-sso-profile-btn');
        const dropdownMenu = container.querySelector('.sso-dropdown-menu');
        const logoutBtn = container.querySelector('.sso-logout-btn');

        profileBtn.addEventListener('click', (e) => {
          e.stopPropagation();
          const isOpen = dropdownMenu.classList.contains('show');
          document.querySelectorAll('.sso-dropdown-menu.show').forEach((m) => m.classList.remove('show'));
          if (!isOpen) {
            dropdownMenu.classList.add('show');
            profileBtn.setAttribute('aria-expanded', 'true');
          } else {
            dropdownMenu.classList.remove('show');
            profileBtn.setAttribute('aria-expanded', 'false');
          }
        });

        logoutBtn.addEventListener('click', (e) => {
          e.stopPropagation();
          this.logout();
        });
      }
    },

    /**
     * Render widgets across all containers on the current page
     */
    renderAllWidgets: function () {
      const isEnglish =
        document.documentElement.lang.startsWith('en') ||
        window.location.pathname.includes('-en') ||
        window.location.pathname.endsWith('en.html');

      const containers = document.querySelectorAll(
        '#urltime-sso-widget, .urltime-sso-widget, [data-urltime-sso]'
      );
      containers.forEach((container) => {
        this.renderWidget(container, isEnglish);
      });
    },

    /**
     * Initialize SSO
     */
    init: function () {
      // 1. Process possible OAuth redirect callback in URL
      this.handleCallback();

      // 2. Render UI widgets
      this.renderAllWidgets();

      // 3. Global click listener to close dropdown when clicking outside
      document.addEventListener('click', (e) => {
        if (!e.target.closest('.urltime-sso-user-dropdown')) {
          document.querySelectorAll('.sso-dropdown-menu.show').forEach((menu) => {
            menu.classList.remove('show');
            const btn = menu.previousElementSibling;
            if (btn) btn.setAttribute('aria-expanded', 'false');
          });
        }
      });
    }
  };

  // Expose to window
  window.URLTIME_SSO = URLTIME_SSO;

  // Auto initialize on DOM ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', function () {
      URLTIME_SSO.init();
    });
  } else {
    URLTIME_SSO.init();
  }
})(window);
