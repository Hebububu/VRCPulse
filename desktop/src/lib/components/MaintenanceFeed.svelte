<script lang="ts">
  import { push } from 'svelte-spa-router';
  import { t } from '../i18n';
  import type { Maintenance } from '../types';

  interface Props {
    maintenances: Maintenance[];
  }

  let { maintenances }: Props = $props();

  const recent = $derived(maintenances.slice(0, 5));

  function timeAgo(dateStr: string): string {
    const diff = Date.now() - new Date(dateStr).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 0) {
      // Future scheduled
      const futMins = Math.abs(mins);
      if (futMins < 60) return `in ${futMins}m`;
      const hours = Math.floor(futMins / 60);
      if (hours < 24) return `in ${hours}h`;
      return `in ${Math.floor(hours / 24)}d`;
    }
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    return `${Math.floor(hours / 24)}d ago`;
  }

  function statusColor(status: string): string {
    switch (status) {
      case 'in_progress': return '#f97316';
      case 'scheduled': return '#60a5fa';
      case 'completed': return '#22c55e';
      default: return '#71717a';
    }
  }
</script>

<div class="feed">
  <div class="feed-header">
    <h3 class="feed-title">{t('maintenance.recent')}</h3>
    <button class="view-all" onclick={() => push('/maintenances')}>{t('maintenance.viewAll')}</button>
  </div>

  {#if recent.length === 0}
    <div class="empty">{t('maintenance.noRecords')}</div>
  {:else}
    {#each recent as m}
      <button
        class="item"
        class:upcoming={m.status === 'scheduled'}
        class:active={m.status === 'in_progress'}
        class:done={m.status === 'completed'}
        onclick={() => push(`/maintenances/${m.id}`)}
      >
        <div class="item-header">
          <span class="status-dot" style="background: {statusColor(m.status)}"></span>
          <span class="item-name">{m.name}</span>
        </div>
        <div class="item-meta">
          <span class="status-badge" style="color: {statusColor(m.status)}">{m.status === 'in_progress' ? 'in progress' : m.status}</span>
          <span class="item-time">{timeAgo(m.scheduled_for)}</span>
        </div>
      </button>
    {/each}
  {/if}
</div>

<style>
  .feed {
    background: var(--surface);
    border: 1px solid var(--border);
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
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0;
  }

  .view-all {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    color: var(--accent);
    background: none;
    border: 1px solid var(--border);
    padding: 4px 8px;
    cursor: pointer;
  }

  .view-all:hover { background: var(--surface-hover); }

  .empty {
    font-size: 14px;
    color: var(--text-secondary);
    font-family: 'Geist Mono', monospace;
  }

  .item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 10px 0;
    border: none;
    border-bottom: 1px solid var(--border);
    background: none;
    cursor: pointer;
    color: inherit;
    font-family: inherit;
  }

  .item:last-child { border-bottom: none; }

  .item.upcoming {
    border-left: 2px solid #60a5fa;
    padding-left: 10px;
  }

  .item.active {
    border-left: 2px solid #f97316;
    padding-left: 10px;
  }

  .item.done {
    opacity: 0.5;
  }

  .item:hover {
    background: var(--surface-hover);
    margin: 0 -16px;
    padding: 10px 16px;
    width: calc(100% + 32px);
  }

  .item-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .item-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-meta {
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

  .item-time {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    color: var(--text-secondary);
  }

  @media (max-width: 768px) {
    .feed { padding: 12px; }
    .item { padding: 12px 0; }
    .item:hover {
      margin: 0;
      padding: 12px 0;
      width: 100%;
      background: transparent;
    }
    .item:active {
      background: var(--surface-hover);
      margin: 0 -12px;
      padding: 12px;
      width: calc(100% + 24px);
    }
    .view-all { min-height: 44px; padding: 8px 16px; display: flex; align-items: center; }
    .item-name { font-size: 14px; }
  }
</style>
