<script lang="ts">
  import { push } from 'svelte-spa-router';
  import { t } from '../i18n';
  import { getMaintenanceById, getMaintenanceHistory } from '../api';
  import type { Maintenance, MaintenanceSnapshotResponse } from '../types';

  interface Props {
    params: { id: string };
  }

  let { params }: Props = $props();

  let maintenance: Maintenance | null = $state(null);
  let history: MaintenanceSnapshotResponse[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  $effect(() => {
    loadMaintenance(params.id);
  });

  async function loadMaintenance(id: string) {
    loading = true;
    try {
      const [mData, histData] = await Promise.all([
        getMaintenanceById(id),
        getMaintenanceHistory(id),
      ]);
      maintenance = mData;
      history = histData;
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load';
    }
    loading = false;
  }

  function statusColor(status: string): string {
    switch (status) {
      case 'in_progress': return '#f97316';
      case 'scheduled': return '#60a5fa';
      case 'completed': return '#22c55e';
      default: return '#71717a';
    }
  }

  function formatDateTime(dateStr: string): string {
    const d = new Date(dateStr);
    return d.toLocaleString('en-US', {
      year: 'numeric', month: 'short', day: 'numeric',
      hour: '2-digit', minute: '2-digit',
      timeZone: 'UTC',
    }) + ' UTC';
  }

  function formatTimeWindow(from: string, to: string): string {
    const d1 = new Date(from);
    const d2 = new Date(to);
    const dateStr = d1.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric', timeZone: 'UTC' });
    const t1 = d1.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', timeZone: 'UTC' });
    const t2 = d2.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', timeZone: 'UTC' });
    return `${dateStr} ${t1} — ${t2} UTC`;
  }

  const vrchatUrl = $derived(`https://status.vrchat.com`);

  async function handleOpenLink() {
    try {
      const { open } = await import('@tauri-apps/plugin-shell');
      await open(vrchatUrl);
    } catch {
      window.open(vrchatUrl, '_blank');
    }
  }
</script>

<div class="page">
  <div class="page-header">
    <button class="back-btn" onclick={() => push('/maintenances')}>← {t('maintenance.recent')}</button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {:else if loading}
    <div class="loading">{t('error.loading')}</div>
  {:else if !maintenance}
    <div class="error">{t('maintenance.notFound')}</div>
  {:else}
    <div class="detail-header">
      <div class="title-row">
        <span class="status-dot" style="background: {statusColor(maintenance.status)}"></span>
        <h1>{maintenance.name}</h1>
      </div>
      <div class="meta-row">
        <span class="status-tag" style="color: {statusColor(maintenance.status)}">
          {maintenance.status === 'in_progress' ? 'in progress' : maintenance.status}
        </span>
        <span class="schedule">{formatTimeWindow(maintenance.scheduled_for, maintenance.scheduled_until)}</span>
      </div>
      <button class="source-link" onclick={handleOpenLink}>
        {t('maintenance.viewSource')} →
      </button>
    </div>

    {#if maintenance.description}
      <div class="section">
        <h2>{t('maintenance.description')}</h2>
        <p class="description">{maintenance.description}</p>
      </div>
    {/if}

    {#if history.length > 0}
      <div class="section">
        <h2>{t('maintenance.changeHistory')}</h2>
        <div class="history-table">
          <div class="history-header">
            <span>{t('incidents.time')}</span>
            <span>{t('incidents.status')}</span>
            <span>{t('maintenance.scheduledFor')}</span>
            <span>{t('maintenance.scheduledUntil')}</span>
          </div>
          {#each history as snap}
            <div class="history-row">
              <span class="history-time" data-label="Time">{formatDateTime(snap.fetched_at)}</span>
              <span data-label="Status" style="color: {statusColor(snap.status)}">{snap.status === 'in_progress' ? 'in progress' : snap.status}</span>
              <span data-label="From">{formatDateTime(snap.scheduled_for)}</span>
              <span data-label="Until">{formatDateTime(snap.scheduled_until)}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
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

  .page-header { margin-bottom: 24px; }

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

  .detail-header { margin-bottom: 32px; }

  .title-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  h1 {
    font-size: 24px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .meta-row {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 8px;
  }

  .status-tag {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    text-transform: capitalize;
  }

  .schedule {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .source-link {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    color: var(--accent);
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
  }

  .source-link:hover { text-decoration: underline; }

  .section { margin-bottom: 32px; }

  h2 {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0 0 16px 0;
  }

  .description {
    font-size: 14px;
    color: var(--text-primary);
    line-height: 1.6;
    margin: 0;
  }

  .error { color: #ef4444; font-size: 14px; }
  .loading { color: var(--text-secondary); font-family: 'Geist Mono', monospace; }

  .history-table { border: 1px solid var(--border); }

  .history-header, .history-row {
    display: grid;
    grid-template-columns: 2fr 1fr 1.5fr 1.5fr;
    padding: 8px 12px;
  }

  .history-header {
    background: var(--surface);
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    color: var(--text-secondary);
    text-transform: uppercase;
    border-bottom: 1px solid var(--border);
  }

  .history-row {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    color: var(--text-primary);
    border-bottom: 1px solid var(--border);
  }

  .history-row:last-child { border-bottom: none; }
  .history-time { color: var(--text-secondary); }

  @media (max-width: 768px) {
    .page { padding: 12px; }
    h1 { font-size: 18px; word-break: break-word; }
    .title-row { gap: 8px; }
    .meta-row { flex-wrap: wrap; gap: 6px; }
    .history-header { display: none; }
    .history-row {
      display: flex;
      flex-direction: column;
      gap: 2px;
      padding: 10px 12px;
    }
    .history-row span[data-label]::before {
      content: attr(data-label) ': ';
      font-size: 10px;
      color: #52525b;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      font-family: 'Geist Mono', monospace;
    }
    .source-link { display: inline-flex; min-height: 44px; align-items: center; }
  }
</style>
