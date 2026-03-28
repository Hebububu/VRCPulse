<script lang="ts">
  import StatusBar from './lib/components/StatusBar.svelte';
  import Dashboard from './lib/components/Dashboard.svelte';
  import type { StatusResponse } from './lib/types';

  let status: StatusResponse | null = $state(null);
  let lastUpdated: Date | null = $state(null);

  function handleStatusUpdate(newStatus: StatusResponse) {
    status = newStatus;
    lastUpdated = new Date();
  }

  function handleDataReceived() {
    lastUpdated = new Date();
  }
</script>

<main>
  <StatusBar {status} {lastUpdated} />
  <Dashboard onStatusUpdate={handleStatusUpdate} onDataReceived={handleDataReceived} />
</main>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body) {
    background: #0f1117;
    color: #e4e4e7;
    font-family: 'Geist Sans', -apple-system, BlinkMacSystemFont, sans-serif;
    -webkit-font-smoothing: antialiased;
  }

  main {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }
</style>
