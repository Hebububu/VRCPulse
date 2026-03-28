<script lang="ts">
  import Chart from './Chart.svelte';
  import TimeRangeSelector from './TimeRangeSelector.svelte';
  import IncidentFeed from './IncidentFeed.svelte';
  import PromoBanner from './PromoBanner.svelte';
  import { getDashboard, getIncidents } from '../api';
  import type { DashboardResponse, Incident } from '../types';

  interface Props {
    onStatusUpdate: (status: DashboardResponse['status']) => void;
    onDataReceived: () => void;
  }

  let { onStatusUpdate, onDataReceived }: Props = $props();

  let range = $state('1h');
  let dashboard: DashboardResponse | null = $state(null);
  let incidents: Incident[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  async function fetchData() {
    try {
      const [dashData, incData] = await Promise.all([
        getDashboard(range),
        getIncidents('all'),
      ]);
      dashboard = dashData;
      incidents = incData.incidents;
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
  <div class="toolbar">
    <TimeRangeSelector value={range} onChange={handleRangeChange} />
  </div>

  {#if error}
    <div class="error-banner">
      <span>{error}</span>
      <button onclick={fetchData}>Retry</button>
    </div>
  {/if}

  <div class="main-area">
    <div class="charts-area">
      <Chart
        data={dashboard?.metrics?.online_users ?? null}
        title="Online Users"
        type="area"
        hero={true}
      />

      <div class="chart-grid">
        <Chart
          data={dashboard?.metrics?.api_latency ?? null}
          title="API Latency"
          unit="ms"
          thresholdValue={500}
          thresholdColor="#eab308"
        />
        <Chart
          data={dashboard?.metrics?.api_requests ?? null}
          title="API Requests"
          unit="%"
          hint="Normalized API request level relative to average capacity"
        />
      </div>

      <div class="chart-grid">
        <Chart
          data={dashboard?.metrics?.api_error_rate ?? null}
          title="Error Rate"
          type="area"
          unit="%"
          thresholdValue={5}
          thresholdColor="#ef4444"
        />
        <Chart
          data={dashboard?.metrics?.steam_auth ?? null}
          title="Steam Auth Success"
          type="area"
          unit="%"
        />
      </div>

      <div class="chart-grid">
        <Chart
          data={dashboard?.metrics?.meta_auth ?? null}
          title="Meta Auth Success"
          type="area"
          unit="%"
        />
        <Chart
          data={dashboard?.metrics?.steam_share ?? null}
          label1="Steam"
          data2={dashboard?.metrics?.meta_share ?? null}
          label2="Meta"
          color2="#a78bfa"
          title="Platform Share"
          unit="%"
          hint="Percentage of total authentications by platform (Steam vs Meta/Oculus)"
        />
      </div>
    </div>

    <div class="sidebar">
      <PromoBanner />
      <IncidentFeed {incidents} />
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
    border: 1px solid #ef4444;
    color: #ef4444;
    font-size: 14px;
  }

  .error-banner button {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    padding: 4px 12px;
    background: transparent;
    color: #ef4444;
    border: 1px solid #ef4444;
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
    .chart-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
