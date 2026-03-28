<script lang="ts">
  import { onMount } from 'svelte';
  import StatusBar from '../components/StatusBar.svelte';
  import Dashboard from '../components/Dashboard.svelte';
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

  onMount(async () => {
    if (!('__TAURI__' in window)) return;
    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();
      if (update?.available) {
        updateAvailable = { version: update.version };
      }
    } catch {
      // Updater not configured or network error, ignore
    }
  });

  async function doUpdate() {
    updating = true;
    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();
      if (update) {
        await update.downloadAndInstall();
        const { relaunch } = await import('@tauri-apps/plugin-process');
        await relaunch();
      }
    } catch {
      updating = false;
    }
  }

  function dismissUpdate() {
    updateAvailable = null;
  }
</script>

{#if updateAvailable}
  <div class="update-bar">
    <span>VRCPulse {updateAvailable.version} is available</span>
    <div class="update-actions">
      <button class="update-btn" onclick={doUpdate} disabled={updating}>
        {updating ? 'Updating...' : 'Update Now'}
      </button>
      <button class="dismiss-btn" onclick={dismissUpdate}>Later</button>
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
    border-bottom: 1px solid #60a5fa;
    font-size: 13px;
    color: #60a5fa;
  }

  .update-actions {
    display: flex;
    gap: 8px;
  }

  .update-btn {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    padding: 4px 12px;
    background: #60a5fa;
    color: #0f1117;
    border: none;
    cursor: pointer;
    font-weight: 600;
  }

  .update-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .dismiss-btn {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    padding: 4px 12px;
    background: transparent;
    color: #60a5fa;
    border: 1px solid #60a5fa;
    cursor: pointer;
  }
</style>
