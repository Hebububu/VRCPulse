<script lang="ts">
  import type { Incident } from '../types';

  interface Props {
    incidents: Incident[];
  }

  let { incidents }: Props = $props();

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
</script>

<div class="feed">
  <h3 class="feed-title">Incidents</h3>
  {#if incidents.length === 0}
    <div class="empty">No active incidents &#x2713;</div>
  {:else}
    {#each incidents as incident}
      <div class="incident">
        <div class="incident-header">
          <span class="impact-dot" style="background: {impactColor(incident.impact)}"></span>
          <span class="incident-name">{incident.name}</span>
          <span class="incident-time">{timeAgo(incident.created_at)}</span>
        </div>
        <div class="incident-status">{incident.status}</div>
        {#if incident.updates.length > 0}
          <div class="latest-update">{incident.updates[0].body}</div>
        {/if}
      </div>
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

  .feed-title {
    font-size: 12px;
    font-weight: 500;
    color: #71717a;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0 0 12px 0;
  }

  .empty {
    font-size: 14px;
    color: #22c55e;
    font-family: 'Geist Mono', monospace;
  }

  .incident {
    padding: 12px 0;
    border-bottom: 1px solid #2a2d37;
  }

  .incident:last-child {
    border-bottom: none;
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
    font-size: 14px;
    font-weight: 500;
    color: #e4e4e7;
    flex: 1;
  }

  .incident-time {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    color: #71717a;
  }

  .incident-status {
    font-size: 12px;
    color: #71717a;
    margin-top: 4px;
    margin-left: 14px;
    text-transform: capitalize;
  }

  .latest-update {
    font-size: 13px;
    color: #a1a1aa;
    margin-top: 6px;
    margin-left: 14px;
    line-height: 1.4;
  }
</style>
