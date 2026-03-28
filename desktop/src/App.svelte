<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let greeting = $state('');

  async function greet() {
    greeting = await invoke('greet', { name: 'VRChat' });
  }

  greet();
</script>

<main>
  <div class="status-bar">
    <span class="app-name">VRCPulse</span>
    <span class="status-text">Connecting...</span>
  </div>

  <div class="content">
    <p class="greeting">{greeting || 'Loading...'}</p>
    <p class="subtitle">Desktop + Web dashboard coming soon</p>
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: #0f1117;
    color: #e4e4e7;
    font-family: 'Geist Sans', -apple-system, sans-serif;
  }

  main {
    min-height: 100vh;
  }

  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 56px;
    padding: 0 16px;
    background: #1a1d27;
    border-bottom: 1px solid #2a2d37;
  }

  .app-name {
    font-size: 16px;
    font-weight: 600;
  }

  .status-text {
    font-size: 14px;
    color: #71717a;
  }

  .content {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: calc(100vh - 56px);
    gap: 8px;
  }

  .greeting {
    font-family: 'Geist Mono', monospace;
    font-size: 20px;
    font-weight: 600;
    color: #60a5fa;
  }

  .subtitle {
    font-size: 14px;
    color: #71717a;
  }
</style>
