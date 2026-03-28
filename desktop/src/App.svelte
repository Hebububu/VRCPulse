<script lang="ts">
  import Router from 'svelte-spa-router';
  import DashboardPage from './lib/pages/DashboardPage.svelte';
  import IncidentList from './lib/pages/IncidentList.svelte';
  import IncidentDetail from './lib/pages/IncidentDetail.svelte';
  import Settings from './lib/pages/Settings.svelte';

  const routes = {
    '/': DashboardPage,
    '/incidents': IncidentList,
    '/incidents/:id': IncidentDetail,
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
