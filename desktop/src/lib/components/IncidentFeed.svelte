<script lang="ts">
  import { push } from 'svelte-spa-router';
  import { t } from '../i18n';
  import type { Incident } from '../types';

  interface Props {
    incidents: Incident[];
  }

  let { incidents }: Props = $props();

  const recent = $derived(incidents.slice(0, 5));

  function timeAgo(dateStr: string): string {
    const diff = Date.now() - new Date(dateStr).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    return `${Math.floor(hours / 24)}d ago`;
  }

  function impactColor(impact: string): string {
    switch (impact) {
      case 'critical': return '#ef4444';
      case 'major': return '#f97316';
      case 'minor': return '#eab308';
      default: return '#71717a';
    }
  }

  function statusBadgeColor(status: string): string {
    switch (status) {
      case 'resolved': return '#22c55e';
      case 'monitoring': return '#60a5fa';
      case 'identified': return '#eab308';
      case 'investigating': return '#f97316';
      default: return '#71717a';
    }
  }
</script>

<div class="feed">
  <div class="feed-header">
    <h3 class="feed-title">{t('incidents.recent')}</h3>
    <button class="view-all" onclick={() => push('/incidents')}>{t('incidents.viewAll')}</button>
  </div>

  {#if recent.length === 0}
    <div class="empty">{t('incidents.noRecords')}</div>
  {:else}
    {#each recent as incident}
      <button class="incident" onclick={() => push(`/incidents/${incident.id}`)}>
        <div class="incident-header">
          <span class="impact-dot" style="background: {impactColor(incident.impact)}"></span>
          <span class="incident-name">{incident.name}</span>
        </div>
        <div class="incident-meta">
          <span class="status-badge" style="color: {statusBadgeColor(incident.status)}">{incident.status}</span>
          <span class="incident-time">{timeAgo(incident.created_at)}</span>
        </div>
      </button>
    {/each}
  {/if}
</div>

<style>
  .feed {
    background: #1a1d27;
    border: 1px solid #2a2d37;
    padding: 16px;
    overflow-y: auto;
  }

  .feed-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .feed-title {
    font-size: 12px;
    font-weight: 500;
    color: #71717a;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0;
  }

  .view-all {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    color: #60a5fa;
    background: none;
    border: 1px solid #2a2d37;
    padding: 4px 8px;
    cursor: pointer;
  }

  .view-all:hover {
    background: #22252f;
  }

  .empty {
    font-size: 14px;
    color: #71717a;
    font-family: 'Geist Mono', monospace;
  }

  .incident {
    display: block;
    width: 100%;
    text-align: left;
    padding: 10px 0;
    border: none;
    border-bottom: 1px solid #2a2d37;
    background: none;
    cursor: pointer;
    color: inherit;
    font-family: inherit;
  }

  .incident:last-child {
    border-bottom: none;
  }

  .incident:hover {
    background: #22252f;
    margin: 0 -16px;
    padding: 10px 16px;
    width: calc(100% + 32px);
  }

  .incident-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .impact-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .incident-name {
    font-size: 13px;
    font-weight: 500;
    color: #e4e4e7;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .incident-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 4px;
    margin-left: 14px;
  }

  .status-badge {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    text-transform: capitalize;
  }

  .incident-time {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    color: #71717a;
  }
</style>
