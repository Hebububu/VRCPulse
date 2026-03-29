<script lang="ts">
  import { push } from 'svelte-spa-router';
  import { t, getLocale } from '../i18n';
  import { getIncidents, getTranslation } from '../api';
  import type { Incident, TranslationResponse } from '../types';

  let incidents: Incident[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let filter = $state('all');
  const isKorean = getLocale() === 'ko';
  let translations: Record<string, TranslationResponse> = $state({});

  async function fetchIncidents() {
    loading = true;
    try {
      const data = await getIncidents(filter);
      incidents = data.incidents;
      error = '';
      if (isKorean) fetchTranslations(data.incidents);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load';
    }
    loading = false;
  }

  async function fetchTranslations(incs: Incident[]) {
    const updated = { ...translations };
    for (const inc of incs.slice(0, 20)) {
      if (updated[inc.id]) continue;
      try {
        updated[inc.id] = await getTranslation('incident', inc.id, 'ko');
        translations = { ...updated };
      } catch { break; }
    }
  }

  function getName(inc: Incident): string {
    if (isKorean && translations[inc.id]) return translations[inc.id].translated_name;
    return inc.name;
  }

  $effect(() => {
    fetchIncidents();
  });

  function impactColor(impact: string): string {
    switch (impact) {
      case 'critical': return '#ef4444';
      case 'major': return '#f97316';
      case 'minor': return '#eab308';
      default: return '#71717a';
    }
  }

  function statusColor(status: string): string {
    switch (status) {
      case 'resolved': return '#22c55e';
      case 'monitoring': return '#60a5fa';
      case 'identified': return '#eab308';
      case 'investigating': return '#f97316';
      default: return '#71717a';
    }
  }

  function formatDate(dateStr: string): string {
    const d = new Date(dateStr);
    return d.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
  }

  function formatTime(dateStr: string): string {
    const d = new Date(dateStr);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
</script>

<div class="page">
  <div class="page-header">
    <button class="back-btn" onclick={() => push('/')}>{t('nav.dashboard')}</button>
    <h1>{t('incidents.history')}</h1>
  </div>

  <div class="filters">
    {#each ['all', 'resolved', 'investigating', 'monitoring'] as f}
      <button
        class="filter-btn"
        class:active={filter === f}
        onclick={() => { filter = f; }}
      >
        {f}
      </button>
    {/each}
  </div>

  {#if error}
    <div class="error">{error}</div>
  {:else if loading}
    <div class="loading">Loading incidents...</div>
  {:else if incidents.length === 0}
    <div class="empty">No incidents found</div>
  {:else}
    <div class="incident-list">
      {#each incidents as incident}
        <button class="incident-row" onclick={() => push(`/incidents/${incident.id}`)}>
          <div class="incident-left">
            <span class="impact-dot" style="background: {impactColor(incident.impact)}"></span>
            <div class="incident-info">
              <span class="incident-name">{getName(incident)}</span>
              <span class="incident-date">{formatDate(incident.created_at)} at {formatTime(incident.created_at)}</span>
            </div>
          </div>
          <div class="incident-right">
            <span class="impact-tag" style="color: {impactColor(incident.impact)}">{incident.impact}</span>
            <span class="status-tag" style="color: {statusColor(incident.status)}">{incident.status}</span>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page {
    padding: 24px;
    width: 100%;
    max-width: 900px;
    margin: 0 auto;
    min-height: calc(100vh - 56px);
  }

  .page-header {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 24px;
  }

  .back-btn {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    color: var(--accent);
    background: none;
    border: 1px solid var(--border);
    padding: 6px 12px;
    cursor: pointer;
  }

  .back-btn:hover {
    background: var(--surface-hover);
  }

  h1 {
    font-size: 20px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .filters {
    display: flex;
    gap: 0;
    border: 1px solid var(--border);
    margin-bottom: 16px;
    width: fit-content;
  }

  .filter-btn {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    padding: 6px 16px;
    background: transparent;
    color: var(--text-secondary);
    border: none;
    border-right: 1px solid #2a2d37;
    cursor: pointer;
    text-transform: capitalize;
  }

  .filter-btn:last-child { border-right: none; }
  .filter-btn:hover { color: var(--text-primary); background: var(--surface-hover); }
  .filter-btn.active { color: var(--accent); background: var(--surface); }

  .error { color: #ef4444; font-size: 14px; }
  .loading, .empty { color: var(--text-secondary); font-family: 'Geist Mono', monospace; font-size: 14px; }

  .incident-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--border);
    border: 1px solid var(--border);
  }

  .incident-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
    background: var(--surface);
    border: none;
    cursor: pointer;
    color: inherit;
    font-family: inherit;
    text-align: left;
    width: 100%;
  }

  .incident-row:hover { background: var(--surface-hover); }

  .incident-left {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    min-width: 0;
  }

  .impact-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .incident-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .incident-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .incident-date {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .incident-right {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-shrink: 0;
  }

  .impact-tag, .status-tag {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    text-transform: capitalize;
  }

  @media (max-width: 768px) {
    .page { padding: 12px; }
    .page-header { margin-bottom: 16px; gap: 12px; }
    h1 { font-size: 18px; }
    .back-btn { min-height: 44px; display: flex; align-items: center; }
    .filters {
      width: 100%;
      overflow-x: auto;
      -webkit-overflow-scrolling: touch;
      margin-bottom: 12px;
    }
    .filter-btn {
      flex: 1;
      min-height: 44px;
      padding: 8px 12px;
      white-space: nowrap;
    }
    .incident-row {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
      padding: 14px 12px;
    }
    .incident-right {
      width: 100%;
      justify-content: flex-start;
      gap: 12px;
    }
    .incident-name { white-space: normal; }
  }
</style>
