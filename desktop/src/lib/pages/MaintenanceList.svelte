<script lang="ts">
  import { push } from 'svelte-spa-router';
  import { t, getLocale } from '../i18n';
  import { getMaintenances, getTranslation } from '../api';
  import type { Maintenance, TranslationResponse } from '../types';

  let maintenances: Maintenance[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let filter = $state('all');
  const isKorean = getLocale() === 'ko';
  let translations: Record<string, TranslationResponse> = $state({});

  async function fetchData() {
    loading = true;
    try {
      const data = await getMaintenances(filter === 'upcoming' ? 'upcoming' : filter);
      maintenances = data.maintenances;
      error = '';
      if (isKorean) fetchTranslations(data.maintenances);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load';
    }
    loading = false;
  }

  async function fetchTranslations(items: Maintenance[]) {
    const updated = { ...translations };
    for (const m of items.slice(0, 20)) {
      if (updated[m.id]) continue;
      try {
        updated[m.id] = await getTranslation('maintenance', m.id, 'ko');
        translations = { ...updated };
      } catch { break; }
    }
  }

  function getName(m: Maintenance): string {
    if (isKorean && translations[m.id]) return translations[m.id].translated_name;
    return m.name;
  }

  $effect(() => {
    fetchData();
  });

  function statusColor(status: string): string {
    switch (status) {
      case 'in_progress': return '#f97316';
      case 'scheduled': return '#60a5fa';
      case 'completed': return '#22c55e';
      default: return '#71717a';
    }
  }

  function formatDate(dateStr: string): string {
    const d = new Date(dateStr);
    return d.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric', timeZone: 'UTC' });
  }

  function formatTime(dateStr: string): string {
    const d = new Date(dateStr);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', timeZone: 'UTC' });
  }
</script>

<div class="page">
  <div class="page-header">
    <button class="back-btn" onclick={() => push('/')}>{t('nav.dashboard')}</button>
    <h1>{t('maintenance.history')}</h1>
  </div>

  <div class="filters">
    {#each ['all', 'upcoming', 'scheduled', 'in_progress', 'completed'] as f}
      <button
        class="filter-btn"
        class:active={filter === f}
        onclick={() => { filter = f; }}
      >
        {f === 'in_progress' ? 'in progress' : f}
      </button>
    {/each}
  </div>

  {#if error}
    <div class="error">{error}</div>
  {:else if loading}
    <div class="loading">{t('error.loading')}</div>
  {:else if maintenances.length === 0}
    <div class="empty">{t('maintenance.noRecords')}</div>
  {:else}
    <div class="list">
      {#each maintenances as m}
        <button class="row" onclick={() => push(`/maintenances/${m.id}`)}>
          <div class="row-left">
            <span class="status-dot" style="background: {statusColor(m.status)}"></span>
            <div class="row-info">
              <span class="row-name">{getName(m)}</span>
              <span class="row-date">{formatDate(m.scheduled_for)} {formatTime(m.scheduled_for)} — {formatTime(m.scheduled_until)} UTC</span>
            </div>
          </div>
          <div class="row-right">
            <span class="status-tag" style="color: {statusColor(m.status)}">{m.status === 'in_progress' ? 'in progress' : m.status}</span>
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

  .back-btn:hover { background: var(--surface-hover); }

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

  .list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--border);
    border: 1px solid var(--border);
  }

  .row {
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

  .row:hover { background: var(--surface-hover); }

  .row-left {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    min-width: 0;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .row-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .row-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-date {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .row-right {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .status-tag {
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
    .row {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
      padding: 14px 12px;
    }
    .row-name { white-space: normal; }
  }
</style>
