<script lang="ts">
  import { onMount } from 'svelte';
  import { push } from 'svelte-spa-router';
  import { t, getLocale, setLocale } from '../i18n';

  let closeToTray = $state(true);
  let notificationsEnabled = $state(true);
  let language = $state('en');
  let isTauri = $state(false);

  onMount(async () => {
    isTauri = '__TAURI_INTERNALS__' in window;
    closeToTray = localStorage.getItem('vrcpulse-close-to-tray') !== 'false';
    notificationsEnabled = localStorage.getItem('vrcpulse-notifications') !== 'false';
    language = getLocale();
    // Sync close-to-tray preference with Rust backend
    if (isTauri) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('set_close_to_tray', { enabled: closeToTray });
      } catch {}
    }
  });

  async function toggleCloseToTray() {
    closeToTray = !closeToTray;
    localStorage.setItem('vrcpulse-close-to-tray', String(closeToTray));
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_close_to_tray', { enabled: closeToTray });
    } catch {}
  }

  function toggleNotifications() {
    notificationsEnabled = !notificationsEnabled;
    localStorage.setItem('vrcpulse-notifications', String(notificationsEnabled));
  }

  function changeLang(lang: string) {
    language = lang;
    setLocale(lang);
  }
</script>

<div class="page">
  <div class="page-header">
    <button class="back-btn" onclick={() => push('/')}>{t('nav.dashboard')}</button>
    <h1>{t('settings.title')}</h1>
  </div>

  <div class="settings-list">
    <div class="setting-group">
      <h2>{t('settings.language')}</h2>
      <div class="setting-row">
        <span class="setting-label">{t('settings.languageDesc')}</span>
        <div class="lang-select">
          <button class="lang-opt" class:active={language === 'en'} onclick={() => changeLang('en')}>English</button>
          <button class="lang-opt" class:active={language === 'ko'} onclick={() => changeLang('ko')}>한국어</button>
          <button class="lang-opt" class:active={language === 'jp'} onclick={() => changeLang('jp')}>日本語</button>
        </div>
      </div>
    </div>

    {#if isTauri}
      <div class="setting-group">
        <h2>{t('settings.app')}</h2>
        <div class="setting-row">
          <div class="setting-text">
            <span class="setting-label">{t('settings.closeToTray')}</span>
            <span class="setting-desc">{t('settings.closeToTrayDesc')}</span>
          </div>
          <button class="toggle" class:on={closeToTray} onclick={toggleCloseToTray} aria-label={t('settings.closeToTray')}>
            <span class="toggle-thumb"></span>
          </button>
        </div>
        <div class="setting-row">
          <div class="setting-text">
            <span class="setting-label">{t('settings.notifications')}</span>
            <span class="setting-desc">{t('settings.notificationsDesc')}</span>
          </div>
          <button class="toggle" class:on={notificationsEnabled} onclick={toggleNotifications} aria-label={t('settings.notifications')}>
            <span class="toggle-thumb"></span>
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .page {
    padding: 24px;
    width: 100%;
    max-width: 600px;
    margin: 0 auto;
    min-height: calc(100vh - 56px);
  }

  .page-header {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 32px;
  }

  .back-btn {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    color: #60a5fa;
    background: none;
    border: 1px solid #2a2d37;
    padding: 6px 12px;
    cursor: pointer;
  }

  .back-btn:hover { background: #22252f; }

  h1 {
    font-size: 20px;
    font-weight: 600;
    color: #e4e4e7;
    margin: 0;
  }

  .settings-list {
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .setting-group h2 {
    font-size: 12px;
    font-weight: 500;
    color: #71717a;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0 0 12px 0;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    background: #1a1d27;
    border: 1px solid #2a2d37;
    margin-bottom: -1px;
  }

  .setting-row:first-of-type { border-top-left-radius: 0; border-top-right-radius: 0; }
  .setting-row:last-of-type { border-bottom-left-radius: 0; border-bottom-right-radius: 0; }

  .setting-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .setting-label {
    font-size: 14px;
    color: #e4e4e7;
  }

  .setting-desc {
    font-size: 12px;
    color: #71717a;
  }

  .lang-select {
    display: flex;
    gap: 0;
    border: 1px solid #2a2d37;
  }

  .lang-opt {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    padding: 6px 16px;
    background: transparent;
    color: #71717a;
    border: none;
    border-right: 1px solid #2a2d37;
    cursor: pointer;
  }

  .lang-opt:last-child { border-right: none; }
  .lang-opt:hover { color: #e4e4e7; background: #22252f; }
  .lang-opt.active { color: #60a5fa; background: #1a1d27; }

  .toggle {
    width: 44px;
    height: 24px;
    background: #2a2d37;
    border: none;
    border-radius: 12px;
    cursor: pointer;
    position: relative;
    transition: background 200ms;
    flex-shrink: 0;
  }

  .toggle.on {
    background: #60a5fa;
  }

  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    background: #e4e4e7;
    border-radius: 50%;
    transition: transform 200ms;
  }

  .toggle.on .toggle-thumb {
    transform: translateX(20px);
  }
</style>
