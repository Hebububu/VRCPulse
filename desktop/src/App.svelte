<script lang="ts">
  import Router from 'svelte-spa-router';
  import DashboardPage from './lib/pages/DashboardPage.svelte';
  import IncidentList from './lib/pages/IncidentList.svelte';
  import IncidentDetail from './lib/pages/IncidentDetail.svelte';
  import MaintenanceList from './lib/pages/MaintenanceList.svelte';
  import MaintenanceDetail from './lib/pages/MaintenanceDetail.svelte';
  import Settings from './lib/pages/Settings.svelte';

  const routes = {
    '/': DashboardPage,
    '/incidents': IncidentList,
    '/incidents/:id': IncidentDetail,
    '/maintenances': MaintenanceList,
    '/maintenances/:id': MaintenanceDetail,
    '/settings': Settings,
  };
</script>

<main>
  <Router {routes} />
</main>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body) {
    background: var(--bg);
    color: var(--text-primary);
    font-family: 'Geist Sans', -apple-system, BlinkMacSystemFont, sans-serif;
    -webkit-font-smoothing: antialiased;
  }

  main {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }
</style>
