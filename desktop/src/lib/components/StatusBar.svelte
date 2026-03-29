<script lang="ts">
  import type { StatusResponse } from '../types';
  import { onMount } from 'svelte';
  import { push } from 'svelte-spa-router';
  import { t, getLocale, toggleLocale } from '../i18n';
  import logoImg from '../../assets/logo.png';

  let isTauriApp = $state(false);
  onMount(() => { isTauriApp = '__TAURI_INTERNALS__' in window; });

  interface Props {
    status: StatusResponse | null;
    lastUpdated: Date | null;
  }

  let { status, lastUpdated }: Props = $props();

  let elapsed = $state('...');

  $effect(() => {
    const interval = setInterval(() => {
      if (lastUpdated) {
        const secs = Math.floor((Date.now() - lastUpdated.getTime()) / 1000);
        if (secs < 60) elapsed = `${secs}s ago`;
        else elapsed = `${Math.floor(secs / 60)}m ago`;
      }
    }, 1000);
    return () => clearInterval(interval);
  });

  const statusColor = $derived.by(() => {
    if (!status) return '#71717a';
    switch (status.indicator) {
      case 'none': return '#22c55e';
      case 'minor': return '#eab308';
      case 'major': return '#f97316';
      case 'critical': return '#ef4444';
      default: return '#71717a';
    }
  });

  const statusLabel = $derived.by(() => {
    if (!status) return t('status.connecting');
    switch (status.indicator) {
      case 'none': return t('status.operational');
      case 'minor': return t('status.minor');
      case 'major': return t('status.major');
      case 'critical': return t('status.critical');
      default: return status.indicator;
    }
  });

  const langLabel = getLocale() === 'en' ? 'KO' : 'EN';
</script>

<header class="status-bar">
  <div class="left">
    <img src={logoImg} alt="VRCPulse" class="app-logo" />
    <span class="app-name">VRCPulse</span>
    <span class="separator">|</span>
    <span class="server-label">{t('status.server')}</span>
    <span class="status-dot" style="background: {statusColor}"></span>
    <span class="status-label" style="color: {statusColor}">{statusLabel}</span>
  </div>
  <div class="right">
    <a href="https://discord.gg/JW3XrskcpK" target="_blank" rel="noopener" class="social-link" title="Discord" aria-label="Discord">
      <svg width="18" height="14" viewBox="0 0 71 55" fill="currentColor" aria-hidden="true">
        <path d="M60.1 4.9A58.5 58.5 0 0045.4.2a.2.2 0 00-.2.1 40.7 40.7 0 00-1.8 3.7 54 54 0 00-16.2 0A26.4 26.4 0 0025.4.3a.2.2 0 00-.2-.1A58.4 58.4 0 0010.5 4.9a.2.2 0 00-.1.1C1.5 18.7-.9 32.2.3 45.5v.2a58.9 58.9 0 0017.7 9 .2.2 0 00.3-.1 42 42 0 003.6-5.9.2.2 0 00-.1-.3 38.8 38.8 0 01-5.5-2.7.2.2 0 01.5-.3l1 .8a42 42 0 0036 0l1.1-.9a.2.2 0 01.4.4 36.4 36.4 0 01-5.5 2.6.2.2 0 00-.1.4 47.2 47.2 0 003.7 5.8.2.2 0 00.2.1 58.7 58.7 0 0017.8-9 .2.2 0 00.1-.1c1.4-15-2.3-28.4-9.8-40.1a.2.2 0 00-.1-.1zM23.7 37.3c-3.5 0-6.3-3.2-6.3-7.1s2.8-7.1 6.3-7.1 6.4 3.2 6.3 7.1c0 3.9-2.8 7.1-6.3 7.1zm23.3 0c-3.5 0-6.3-3.2-6.3-7.1s2.8-7.1 6.3-7.1 6.4 3.2 6.3 7.1c0 3.9-2.7 7.1-6.3 7.1z"/>
      </svg>
    </a>
    <a href="https://github.com/Hebububu/VRCPulse" target="_blank" rel="noopener" class="social-link" title="GitHub" aria-label="GitHub">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
      </svg>
    </a>
    {#if isTauriApp}
      <button class="social-link" onclick={() => push('/settings')} title="Settings" aria-label={t('settings.title')}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
          <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 00.12-.61l-1.92-3.32a.49.49 0 00-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.48.48 0 00-.48-.41h-3.84a.48.48 0 00-.48.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96a.49.49 0 00-.59.22L2.74 8.87a.48.48 0 00.12.61l2.03 1.58c-.05.3-.07.63-.07.94s.02.64.07.94l-2.03 1.58a.49.49 0 00-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.48-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6A3.6 3.6 0 1115.6 12 3.6 3.6 0 0112 15.6z"/>
        </svg>
      </button>
    {/if}
    <button class="lang-btn" onclick={toggleLocale} aria-label={t('settings.languageDesc')}>{langLabel}</button>
    <span class="updated">{t('status.lastUpdated')} {elapsed}</span>
  </div>
</header>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 56px;
    padding: 0 16px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .app-logo {
    width: 28px;
    height: 28px;
  }

  .app-name {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .separator {
    color: var(--border);
    font-size: 16px;
  }

  .server-label {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    transition: background 200ms ease-out;
    animation: dot-glow 3s ease-in-out infinite;
  }

  @keyframes dot-glow {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }

  @media (prefers-reduced-motion: reduce) {
    .status-dot { animation: none; }
  }

  .status-label {
    font-size: 14px;
    font-weight: 500;
    transition: color 200ms ease-out;
  }

  .right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .social-link {
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    transition: color 150ms;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
  }

  .social-link:hover {
    color: var(--text-primary);
  }

  .lang-btn {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    font-weight: 600;
    padding: 3px 8px;
    color: var(--text-secondary);
    background: transparent;
    border: 1px solid var(--border);
    cursor: pointer;
    transition: all 150ms;
  }

  .lang-btn:hover {
    color: var(--text-primary);
    border-color: var(--text-primary);
  }

  .updated {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    color: var(--text-secondary);
  }

  @media (max-width: 768px) {
    .status-bar {
      flex-wrap: wrap;
      height: auto;
      padding: 10px 12px;
      gap: 6px;
    }
    .left {
      width: 100%;
      gap: 8px;
    }
    .separator { display: none; }
    .server-label { display: none; }
    .app-logo { width: 24px; height: 24px; }
    .app-name { font-size: 14px; }
    .right {
      width: 100%;
      justify-content: space-between;
      gap: 8px;
    }
    .social-link {
      min-width: 44px;
      min-height: 44px;
      justify-content: center;
    }
    .lang-btn {
      min-height: 36px;
      padding: 6px 12px;
      font-size: 12px;
    }
    .updated {
      font-size: 11px;
      flex: 1;
      text-align: right;
    }
  }
</style>
