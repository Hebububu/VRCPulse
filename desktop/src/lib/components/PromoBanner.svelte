<script lang="ts">
  import { onMount } from 'svelte';

  let showDownload = $state(true);

  onMount(() => {
    // __TAURI_INTERNALS__ is injected by the actual Tauri runtime, not by Vite
    showDownload = !('__TAURI_INTERNALS__' in window);
  });

  const discordUrl = 'https://discord.com/oauth2/authorize?client_id=1456912795462275166&permissions=49152&scope=bot%20applications.commands';
  const downloadUrl = 'https://github.com/Hebububu/VRCPulse/releases/latest';
</script>

<div class="promo-cards">
    <a href={discordUrl} target="_blank" rel="noopener" class="promo-card discord">
      <div class="card-icon">
        <svg width="20" height="16" viewBox="0 0 71 55" fill="currentColor">
          <path d="M60.1 4.9A58.5 58.5 0 0045.4.2a.2.2 0 00-.2.1 40.7 40.7 0 00-1.8 3.7 54 54 0 00-16.2 0A26.4 26.4 0 0025.4.3a.2.2 0 00-.2-.1A58.4 58.4 0 0010.5 4.9a.2.2 0 00-.1.1C1.5 18.7-.9 32.2.3 45.5v.2a58.9 58.9 0 0017.7 9 .2.2 0 00.3-.1 42 42 0 003.6-5.9.2.2 0 00-.1-.3 38.8 38.8 0 01-5.5-2.7.2.2 0 01.5-.3l1 .8a42 42 0 0036 0l1.1-.9a.2.2 0 01.4.4 36.4 36.4 0 01-5.5 2.6.2.2 0 00-.1.4 47.2 47.2 0 003.7 5.8.2.2 0 00.2.1 58.7 58.7 0 0017.8-9 .2.2 0 00.1-.1c1.4-15-2.3-28.4-9.8-40.1a.2.2 0 00-.1-.1zM23.7 37.3c-3.5 0-6.3-3.2-6.3-7.1s2.8-7.1 6.3-7.1 6.4 3.2 6.3 7.1c0 3.9-2.8 7.1-6.3 7.1zm23.3 0c-3.5 0-6.3-3.2-6.3-7.1s2.8-7.1 6.3-7.1 6.4 3.2 6.3 7.1c0 3.9-2.7 7.1-6.3 7.1z"/>
        </svg>
      </div>
      <div class="card-text">
        <span class="card-title">Get Discord Alerts</span>
        <span class="card-desc">Add bot to your server</span>
      </div>
      <span class="card-arrow">→</span>
    </a>

    {#if showDownload}
    <a href={downloadUrl} target="_blank" rel="noopener" class="promo-card download">
      <div class="card-icon">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
      </div>
      <div class="card-text">
        <span class="card-title">Desktop App</span>
        <span class="card-desc">System tray + notifications</span>
      </div>
      <span class="card-arrow">→</span>
    </a>
    {/if}
  </div>

<style>
  .promo-cards {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .promo-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    text-decoration: none;
    border: 1px solid #2a2d37;
    background: #1a1d27;
    cursor: pointer;
    transition: all 150ms ease-out;
  }

  .promo-card:hover {
    background: #22252f;
    border-color: #3a3d47;
  }

  .discord:hover {
    border-color: #5865f2;
  }

  .download:hover {
    border-color: #60a5fa;
  }

  .card-icon {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .discord .card-icon {
    color: #5865f2;
    background: rgba(88, 101, 242, 0.1);
  }

  .download .card-icon {
    color: #60a5fa;
    background: rgba(96, 165, 250, 0.1);
  }

  .card-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    flex: 1;
    min-width: 0;
  }

  .card-title {
    font-size: 13px;
    font-weight: 600;
    color: #e4e4e7;
  }

  .card-desc {
    font-size: 11px;
    color: #71717a;
  }

  .card-arrow {
    font-family: 'Geist Mono', monospace;
    font-size: 14px;
    color: #71717a;
    flex-shrink: 0;
    transition: transform 150ms ease-out;
  }

  .promo-card:hover .card-arrow {
    transform: translateX(3px);
    color: #e4e4e7;
  }
</style>
