<script lang="ts">
  import { push } from 'svelte-spa-router';
  import { t } from '../i18n';
  import { getIncidents, getIncidentHistory } from '../api';
  import type { Incident, IncidentSnapshotResponse } from '../types';

  interface Props {
    params: { id: string };
  }

  let { params }: Props = $props();

  let incident: Incident | null = $state(null);
  let history: IncidentSnapshotResponse[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  $effect(() => {
    loadIncident(params.id);
  });

  async function loadIncident(id: string) {
    loading = true;
    try {
      const [incData, histData] = await Promise.all([
        getIncidents('all'),
        getIncidentHistory(id),
      ]);
      incident = incData.incidents.find(i => i.id === id) ?? null;
      history = histData;
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load';
    }
    loading = false;
  }

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

  function formatDateTime(dateStr: string): string {
    const d = new Date(dateStr);
    return d.toLocaleString('en-US', {
      year: 'numeric', month: 'short', day: 'numeric',
      hour: '2-digit', minute: '2-digit',
    });
  }

  function duration(start: string, end: string | undefined): string {
    if (!end) return 'Ongoing';
    const diff = new Date(end).getTime() - new Date(start).getTime();
    const hours = Math.floor(diff / 3600000);
    const mins = Math.floor((diff % 3600000) / 60000);
    if (hours > 0) return `${hours}h ${mins}m`;
    return `${mins}m`;
  }

  const vrchatUrl = $derived(`https://status.vrchat.com/incidents/${params.id}`);

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
    <button class="back-btn" onclick={() => push('/incidents')}>{t('nav.incidents')}</button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {:else if loading}
    <div class="loading">Loading incident...</div>
  {:else if !incident}
    <div class="error">Incident not found</div>
  {:else}
    <div class="incident-header">
      <div class="title-row">
        <span class="impact-dot" style="background: {impactColor(incident.impact)}"></span>
        <h1>{incident.name}</h1>
      </div>
      <div class="meta-row">
        <span class="status-tag" style="color: {statusColor(incident.status)}">{incident.status}</span>
        <span class="impact-tag" style="color: {impactColor(incident.impact)}">{incident.impact}</span>
        <span class="date">{formatDateTime(incident.created_at)}</span>
        {#if incident.status === 'resolved'}
          <span class="duration">Duration: {duration(incident.created_at, incident.updates[0]?.created_at)}</span>
        {/if}
      </div>
      <button class="source-link" onclick={handleOpenLink}>
        {t('incidents.viewSource')} →
      </button>
    </div>

    <div class="section">
      <h2>{t('incidents.updates')}</h2>
      {#if incident.updates.length === 0}
        <p class="empty">No updates yet</p>
      {:else}
        <div class="timeline">
          {#each incident.updates as update}
            <div class="timeline-item">
              <div class="timeline-dot" style="background: {statusColor(update.status)}"></div>
              <div class="timeline-content">
                <div class="timeline-header">
                  <span class="update-status" style="color: {statusColor(update.status)}">{update.status}</span>
                  <span class="update-time">{formatDateTime(update.created_at)}</span>
                </div>
                <p class="update-body">{update.body}</p>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    {#if history.length > 1}
      <div class="section">
        <h2>{t('incidents.changeHistory')}</h2>
        <div class="history-table">
          <div class="history-header">
            <span>Time</span>
            <span>Status</span>
            <span>Impact</span>
            <span>Updates</span>
          </div>
          {#each history as snap}
            <div class="history-row">
              <span class="history-time" data-label="Time">{formatDateTime(snap.fetched_at)}</span>
              <span data-label="Status" style="color: {statusColor(snap.status)}">{snap.status}</span>
              <span data-label="Impact" style="color: {impactColor(snap.impact)}">{snap.impact}</span>
              <span data-label="Updates">{snap.update_count}</span>
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

  .page-header {
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

  .incident-header {
    margin-bottom: 32px;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
  }

  .impact-dot {
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

  .status-tag, .impact-tag {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    text-transform: capitalize;
  }

  .date, .duration {
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

  .section {
    margin-bottom: 32px;
  }

  h2 {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0 0 16px 0;
  }

  .empty { color: var(--text-secondary); font-size: 14px; }
  .error { color: #ef4444; font-size: 14px; }
  .loading { color: var(--text-secondary); font-family: 'Geist Mono', monospace; }

  .timeline {
    display: flex;
    flex-direction: column;
    gap: 0;
    border-left: 1px solid var(--border);
    margin-left: 4px;
  }

  .timeline-item {
    display: flex;
    gap: 16px;
    padding: 16px 0;
    position: relative;
  }

  .timeline-dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    flex-shrink: 0;
    margin-left: -5px;
    margin-top: 2px;
  }

  .timeline-content { flex: 1; }

  .timeline-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 6px;
  }

  .update-status {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    font-weight: 600;
    text-transform: capitalize;
  }

  .update-time {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .update-body {
    font-size: 14px;
    color: #a1a1aa;
    line-height: 1.5;
    margin: 0;
  }

  .history-table {
    border: 1px solid var(--border);
  }

  .history-header, .history-row {
    display: grid;
    grid-template-columns: 2fr 1fr 1fr 0.5fr;
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
    .timeline-item { gap: 12px; padding: 12px 0; }
    .update-body { font-size: 13px; }
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
    .source-link {
      display: inline-flex;
      min-height: 44px;
      align-items: center;
    }
  }
</style>
