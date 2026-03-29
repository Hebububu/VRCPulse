<script lang="ts">
  import Chart from './Chart.svelte';
  import InsightCard from './InsightCard.svelte';
  import MaintenanceBanner from './MaintenanceBanner.svelte';
  import MaintenanceFeed from './MaintenanceFeed.svelte';
  import TimeRangeSelector from './TimeRangeSelector.svelte';
  import IncidentFeed from './IncidentFeed.svelte';
  import PromoBanner from './PromoBanner.svelte';
  import { onMount } from 'svelte';
  import { getDashboard, getIncidents, getMaintenances, getInsight } from '../api';
  import { t } from '../i18n';
  import type { InsightBundle, DashboardResponse, Incident, Maintenance } from '../types';

  interface Props {
    onStatusUpdate: (status: DashboardResponse['status']) => void;
    onDataReceived: () => void;
  }

  let { onStatusUpdate, onDataReceived }: Props = $props();

  let isTauri = $state(false);
  onMount(() => { isTauri = '__TAURI_INTERNALS__' in window; });

  let range = $state('24h');
  let dashboard: DashboardResponse | null = $state(null);
  let incidents: Incident[] = $state([]);
  let previousIncidentIds: Set<string> = new Set();
  let insightBundle: InsightBundle | null = $state(null);
  let maintenances: Maintenance[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  async function sendNotification(title: string, body: string) {
    if (!('__TAURI_INTERNALS__' in window)) return;
    if (localStorage.getItem('vrcpulse-notifications') === 'false') return;
    try {
      const { sendNotification, isPermissionGranted, requestPermission } = await import('@tauri-apps/plugin-notification');
      let permitted = await isPermissionGranted();
      if (!permitted) {
        const result = await requestPermission();
        permitted = result === 'granted';
      }
      if (permitted) {
        sendNotification({ title, body });
      }
    } catch {}
  }

  function checkNewIncidents(newIncidents: Incident[]) {
    if (previousIncidentIds.size === 0) {
      // First load, just store IDs
      previousIncidentIds = new Set(newIncidents.map(i => i.id));
      return;
    }
    for (const inc of newIncidents) {
      if (!previousIncidentIds.has(inc.id)) {
        sendNotification(`VRCPulse: ${inc.name}`, `Impact: ${inc.impact} — ${inc.status}`);
      }
    }
    previousIncidentIds = new Set(newIncidents.map(i => i.id));
  }

  async function fetchData() {
    try {
      const [dashData, incData, maintData, insightData] = await Promise.all([
        getDashboard(range),
        getIncidents('all'),
        getMaintenances('all').catch(() => ({ maintenances: [] })),
        isTauri ? Promise.resolve({ insight: null }) : getInsight().catch(() => ({ insight: null })),
      ]);
      dashboard = dashData;
      checkNewIncidents(incData.incidents);
      incidents = incData.incidents;
      maintenances = maintData.maintenances;
      insightBundle = insightData.insight;
      onStatusUpdate(dashData.status);
      onDataReceived();
      loading = false;
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to fetch data';
      loading = false;
    }
  }

  // Fetch on mount and when range changes
  $effect(() => {
    fetchData();
  });

  // Auto-refresh every 30 seconds
  $effect(() => {
    const interval = setInterval(fetchData, 30_000);
    return () => clearInterval(interval);
  });

  function handleRangeChange(newRange: string) {
    range = newRange;
    loading = true;
  }
</script>

<div class="dashboard">
  <MaintenanceBanner {maintenances} />

  <div class="toolbar">
    <TimeRangeSelector value={range} onChange={handleRangeChange} />
  </div>

  {#if error}
    <div class="error-banner">
      <span>{error}</span>
      <button onclick={fetchData}>Retry</button>
    </div>
  {/if}

  {#if !isTauri}
    <InsightCard bundle={insightBundle} />
  {/if}

  <div class="main-area">
    <div class="charts-area">
      <Chart
        data={dashboard?.metrics?.online_users ?? null}
        title={t('chart.onlineUsers')}
        type="area"
        hero={true}
      />

      <div class="chart-grid">
        <Chart
          data={dashboard?.metrics?.api_latency ?? null}
          title={t('chart.apiLatency')}
          unit="ms"
          thresholdValue={500}
          thresholdColor="#eab308"
        />
        <Chart
          data={dashboard?.metrics?.api_requests ?? null}
          title={t('chart.apiRequests')}
          unit="%"
          hint={t('chart.hint.apiRequests')}
        />
      </div>

      <div class="chart-grid">
        <Chart
          data={dashboard?.metrics?.api_error_rate ?? null}
          title={t('chart.errorRate')}
          type="area"
          unit="%"
          thresholdValue={5}
          thresholdColor="#ef4444"
        />
        <Chart
          data={dashboard?.metrics?.steam_auth ?? null}
          title={t('chart.steamAuth')}
          type="area"
          unit="%"
        />
      </div>

      <div class="chart-grid">
        <Chart
          data={dashboard?.metrics?.meta_auth ?? null}
          title={t('chart.metaAuth')}
          type="area"
          unit="%"
        />
        <Chart
          data={dashboard?.metrics?.steam_share ?? null}
          label1="Steam"
          data2={dashboard?.metrics?.meta_share ?? null}
          label2="Meta"
          color2="#a78bfa"
          title={t('chart.platformShare')}
          unit="%"
          hint={t('chart.hint.platformShare')}
        />
      </div>
    </div>

    <div class="sidebar">
      <PromoBanner />
      <IncidentFeed {incidents} />
      <MaintenanceFeed {maintenances} />
    </div>
  </div>
</div>

<style>
  .dashboard {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 16px;
    gap: 16px;
    overflow-y: auto;
    overflow-x: hidden;
    min-width: 0;
  }

  .toolbar {
    display: flex;
    justify-content: flex-end;
  }

  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid var(--status-critical);
    color: var(--status-critical);
    font-size: 14px;
  }

  .error-banner button {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    padding: 4px 12px;
    background: transparent;
    color: var(--status-critical);
    border: 1px solid var(--status-critical);
    cursor: pointer;
  }

  .main-area {
    display: flex;
    gap: 16px;
    flex: 1;
    min-width: 0;
  }

  .charts-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-width: 0;
  }

  .chart-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .sidebar {
    width: 320px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  @media (max-width: 1024px) {
    .main-area {
      flex-direction: column;
    }
    .sidebar {
      width: 100%;
    }
  }

  @media (max-width: 768px) {
    .dashboard {
      padding: 12px;
      gap: 12px;
    }
    .chart-grid {
      grid-template-columns: 1fr;
      gap: 12px;
    }
    .toolbar {
      justify-content: stretch;
    }
    .error-banner {
      flex-wrap: wrap;
      gap: 8px;
    }
    .error-banner button {
      min-height: 44px;
      padding: 8px 16px;
    }
  }
</style>
