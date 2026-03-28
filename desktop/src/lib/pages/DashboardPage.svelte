<script lang="ts">
  import StatusBar from '../components/StatusBar.svelte';
  import Dashboard from '../components/Dashboard.svelte';
  import type { StatusResponse } from '../types';

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

<StatusBar {status} {lastUpdated} />
<Dashboard onStatusUpdate={handleStatusUpdate} onDataReceived={handleDataReceived} />
