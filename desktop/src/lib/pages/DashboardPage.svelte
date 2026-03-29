<script lang="ts">
  import { onMount } from 'svelte';
  import StatusBar from '../components/StatusBar.svelte';
  import Dashboard from '../components/Dashboard.svelte';
  import { t } from '../i18n';
  import type { StatusResponse } from '../types';

  let status: StatusResponse | null = $state(null);
  let lastUpdated: Date | null = $state(null);
  let updateAvailable: { version: string } | null = $state(null);
  let updating = $state(false);

  function handleStatusUpdate(newStatus: StatusResponse) {
    status = newStatus;
    lastUpdated = new Date();
  }

  function handleDataReceived() {
    lastUpdated = new Date();
  }

  let updateInterval: ReturnType<typeof setInterval> | null = null;

  onMount(() => {
    if (!('__TAURI_INTERNALS__' in window)) return;

    // Run async init without returning promise
    (async () => {
      // Sync close-to-tray preference on startup
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const closeToTray = localStorage.getItem('vrcpulse-close-to-tray') !== 'false';
        await invoke('set_close_to_tray', { enabled: closeToTray });
      } catch {}

      // Check for updates
      try {
        const { check } = await import('@tauri-apps/plugin-updater');
        const update = await check();
        if (update?.available) {
          updateAvailable = { version: update.version };
        }
      } catch {}
    })();

    // Check for updates every 30 minutes
    updateInterval = setInterval(async () => {
      if (updateAvailable) return;
      try {
        const { check } = await import('@tauri-apps/plugin-updater');
        const update = await check();
        if (update?.available) {
          updateAvailable = { version: update.version };
        }
      } catch {}
    }, 30 * 60 * 1000);

    return () => {
      if (updateInterval) clearInterval(updateInterval);
    };
  });

  let updateError = $state('');

  async function doUpdate() {
    updating = true;
    updateError = '';
    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();
      if (update) {
        await update.downloadAndInstall();
        const { relaunch } = await import('@tauri-apps/plugin-process');
        await relaunch();
      }
    } catch (e: any) {
      updating = false;
      updateError = e?.message || 'Update failed';
    }
  }

  function dismissUpdate() {
    updateAvailable = null;
  }
</script>

{#if updateAvailable}
  <div class="update-bar">
    <span>VRCPulse {updateAvailable.version} {t('update.available')}</span>
    {#if updateError}
      <span class="update-error">{updateError}</span>
    {/if}
    <div class="update-actions">
      <button class="update-btn" onclick={doUpdate} disabled={updating}>
        {updating ? t('update.updating') : t('update.now')}
      </button>
      <button class="dismiss-btn" onclick={dismissUpdate}>{t('update.later')}</button>
    </div>
  </div>
{/if}

<StatusBar {status} {lastUpdated} />
<Dashboard onStatusUpdate={handleStatusUpdate} onDataReceived={handleDataReceived} />

<style>
  .update-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    background: rgba(96, 165, 250, 0.1);
    border-bottom: 1px solid var(--accent);
    font-size: 13px;
    color: var(--accent);
  }

  .update-actions {
    display: flex;
    gap: 8px;
  }

  .update-btn {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    padding: 4px 12px;
    background: var(--accent);
    color: var(--bg);
    border: none;
    cursor: pointer;
    font-weight: 600;
  }

  .update-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .update-error {
    font-size: 12px;
    color: var(--status-critical);
  }

  .dismiss-btn {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    padding: 4px 12px;
    background: transparent;
    color: var(--accent);
    border: 1px solid var(--accent);
    cursor: pointer;
  }
</style>
