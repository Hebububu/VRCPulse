<script lang="ts">
  import Chart from './Chart.svelte';
  import ComponentStatusGrid from './ComponentStatusGrid.svelte';
  import InsightCard from './InsightCard.svelte';
  import MaintenanceBanner from './MaintenanceBanner.svelte';
  import MaintenanceFeed from './MaintenanceFeed.svelte';
  import TimeRangeSelector from './TimeRangeSelector.svelte';
  import IncidentFeed from './IncidentFeed.svelte';
  import PromoBanner from './PromoBanner.svelte';
  import { onMount } from 'svelte';
  import { getDashboard, getIncidents, getMaintenances, getInsight, getTranslation, getComponentStatuses } from '../api';
  import { t, getLocale } from '../i18n';
  import type { ComponentStatus, InsightBundle, DashboardResponse, Incident, Maintenance, TranslationResponse } from '../types';

  interface Props {
    onStatusUpdate: (status: DashboardResponse['status']) => void;
    onDataReceived: () => void;
  }

  let { onStatusUpdate, onDataReceived }: Props = $props();

  let isTauri = $state(false);
  let windowClass = $state<'compact' | 'standard' | 'expanded'>('standard');

  function getWindowClass(w: number, h: number): 'compact' | 'standard' | 'expanded' {
    if (w >= 1600 && h >= 900) return 'expanded';
    if (w >= 900 && h >= 600) return 'standard';
    return 'compact';
  }

  onMount(() => {
    isTauri = '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      windowClass = getWindowClass(window.innerWidth, window.innerHeight);
      let rafId: number;
      const handleResize = () => {
        cancelAnimationFrame(rafId);
        rafId = requestAnimationFrame(() => {
          windowClass = getWindowClass(window.innerWidth, window.innerHeight);
        });
      };
      window.addEventListener('resize', handleResize);
      return () => {
        window.removeEventListener('resize', handleResize);
        cancelAnimationFrame(rafId);
      };
    }
  });

  const isCompact = $derived(isTauri && windowClass === 'compact');
  const isExpanded = $derived(isTauri && windowClass === 'expanded');
  const showSidebar = $derived(!isCompact);
  const showSecondaryCharts = $derived(!isCompact);
  const insightMode = $derived<'full' | 'compact'>(
    isTauri ? (windowClass === 'expanded' ? 'full' : windowClass === 'standard' ? 'compact' : 'full') : 'full'
  );
  const showInsight = $derived(!isCompact);

  let range = $state('24h');
  let dashboard: DashboardResponse | null = $state(null);
  let incidents: Incident[] = $state([]);
  let previousIncidentIds: Set<string> = new Set();
  let insightBundle: InsightBundle | null = $state(null);
  let maintenances: Maintenance[] = $state([]);
  let components: ComponentStatus[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  // Auto-translation for Korean locale
  let incidentTranslations: Record<string, TranslationResponse> = $state({});
  let maintenanceTranslations: Record<string, TranslationResponse> = $state({});

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
      const [dashData, incData, maintData, insightData, compData] = await Promise.all([
        getDashboard(range),
        getIncidents('all'),
        getMaintenances('all').catch(() => ({ maintenances: [] })),
        getInsight().catch(() => ({ insight: null })),
        getComponentStatuses(range).catch(() => [] as ComponentStatus[]),
      ]);
      dashboard = dashData;
      checkNewIncidents(incData.incidents);
      incidents = incData.incidents;
      maintenances = maintData.maintenances;
      insightBundle = insightData.insight;
      components = compData;
      onStatusUpdate(dashData.status);
      onDataReceived();
      loading = false;
      error = '';

      // Auto-fetch translations for non-English locale
      if (getLocale() !== 'en') {
        fetchTranslations(incData.incidents, maintData.maintenances);
      }
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

  async function fetchTranslations(incs: Incident[], maints: Maintenance[]) {
    // Fetch translations sequentially to avoid rate limiting
    const newIncTranslations: Record<string, TranslationResponse> = { ...incidentTranslations };
    const newMaintTranslations: Record<string, TranslationResponse> = { ...maintenanceTranslations };

    for (const inc of incs.slice(0, 5)) {
      if (newIncTranslations[inc.id]) continue;
      try {
        const result = await getTranslation('incident', inc.id, getLocale());
        newIncTranslations[inc.id] = result;
        incidentTranslations = { ...newIncTranslations };
      } catch { break; }
    }

    for (const m of maints.slice(0, 5)) {
      if (newMaintTranslations[m.id]) continue;
      try {
        const result = await getTranslation('maintenance', m.id, getLocale());
        newMaintTranslations[m.id] = result;
        maintenanceTranslations = { ...newMaintTranslations };
      } catch { break; }
    }
  }

  function handleRangeChange(newRange: string) {
    range = newRange;
    loading = true;
  }
</script>

<div class="dashboard" class:desktop-dashboard={isTauri} class:compact={isCompact} class:expanded={isExpanded}>
  <MaintenanceBanner {maintenances} />

  {#if !isCompact}
    <div class="toolbar">
      <TimeRangeSelector value={range} onChange={handleRangeChange} />
    </div>
  {/if}

  {#if error}
    <div class="error-banner">
      <span>{error}</span>
      <button onclick={fetchData}>Retry</button>
    </div>
  {/if}

  {#if showInsight}
    <InsightCard bundle={insightBundle} mode={insightMode} />
  {/if}

  <ComponentStatusGrid {components} {range} />

  <div class="main-area">
    <div class="charts-area">
      <Chart
        data={dashboard?.metrics?.online_users ?? null}
        title={t('chart.onlineUsers')}
        type="area"
        hero={!isCompact}
      />

      {#if showSecondaryCharts}
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
      {/if}
    </div>

    {#if showSidebar}
      <div class="sidebar" class:sidebar-expanded={isExpanded}>
        <PromoBanner />
        <IncidentFeed {incidents} translations={incidentTranslations} />
        <MaintenanceFeed {maintenances} translations={maintenanceTranslations} />
      </div>
    {:else}
      <IncidentFeed {incidents} maxItems={2} compact={true} translations={incidentTranslations} />
    {/if}
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

  /* Desktop Tauri: fixed height with scroll */
  .desktop-dashboard {
    overflow-y: auto;
    overflow-x: hidden;
    height: calc(100vh - 56px); /* Viewport minus StatusBar */
  }

  .desktop-dashboard.compact {
    padding: 12px;
    gap: 12px;
  }

  .desktop-dashboard.expanded .sidebar-expanded {
    width: 360px;
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
