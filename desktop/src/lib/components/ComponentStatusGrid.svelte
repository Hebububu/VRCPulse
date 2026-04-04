<script lang="ts">
  import type { ComponentStatus } from '../types';
  import { t } from '../i18n';

  interface Props {
    components: ComponentStatus[];
    range: string;
  }

  let { components, range }: Props = $props();

  let userExpanded = $state(false);

  const allOperational = $derived(
    components.length > 0 && components.every(c => c.current_status === 'operational')
  );

  function statusColor(status: string): string {
    switch (status) {
      case 'operational': return 'var(--status-ok)';
      case 'degraded_performance': return 'var(--status-minor)';
      case 'partial_outage': return 'var(--status-major)';
      case 'major_outage': return 'var(--status-critical)';
      default: return '#1e2130';
    }
  }

  function statusLabel(status: string): string {
    switch (status) {
      case 'operational': return t('component.operational');
      case 'degraded_performance': return t('component.degraded');
      case 'partial_outage': return t('component.partial_outage');
      case 'major_outage': return t('component.major_outage');
      default: return t('component.unknown');
    }
  }

  function uptimePercent(buckets: string[]): number {
    if (buckets.length === 0) return 100;
    const known = buckets.filter(b => b !== 'unknown');
    if (known.length === 0) return 100;
    const operational = known.filter(b => b === 'operational').length;
    return Math.round((operational / known.length) * 100);
  }

  function rangeLabel(r: string): string {
    switch (r) {
      case '1h': return '1h';
      case '6h': return '6h';
      case '12h': return '12h';
      case '24h': return '24h';
      default: return r;
    }
  }

  function componentAriaLabel(c: ComponentStatus): string {
    return `${c.name}: ${statusLabel(c.current_status)}, ${uptimePercent(c.buckets)}% uptime in last ${rangeLabel(range)}`;
  }
</script>

{#if components.length > 0}
  <section class="component-status-section">
    {#if allOperational && !userExpanded}
      <button class="collapsed-row" onclick={() => { userExpanded = true; }}>
        <span class="status-dot" style="background: var(--status-ok);"></span>
        <span class="collapsed-text">{t('component.all_operational').replace('{n}', String(components.length))}</span>
        <span class="expand-icon">&#9662;</span>
      </button>
    {:else}
      <div class="section-header">
        <h2>{t('component.title')}</h2>
        {#if allOperational}
          <button class="collapse-btn" onclick={() => { userExpanded = false; }}>&#9652;</button>
        {/if}
      </div>
      <div class="component-grid">
        {#each components as component (component.component_id)}
          <div class="component-card" role="group" aria-label={componentAriaLabel(component)}>
            <div class="card-header">
              <span class="component-name">{component.name}</span>
              <span class="status-indicator">
                <span class="status-dot" style="background: {statusColor(component.current_status)};"></span>
                <span class="status-label" style="color: {statusColor(component.current_status)};">{statusLabel(component.current_status)}</span>
              </span>
            </div>
            <div class="bar-container">
              <div class="status-bar" aria-hidden="true">
                {#each component.buckets as bucket}
                  <div class="bucket" style="background: {statusColor(bucket)};"></div>
                {/each}
              </div>
              <div class="bar-footer">
                <span class="uptime-pct">{uptimePercent(component.buckets)}%</span>
                <span class="bar-range">{rangeLabel(range)}</span>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>
{/if}

<style>
  .component-status-section {
    margin-bottom: 16px;
  }

  .collapsed-row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 0;
    cursor: pointer;
    transition: background 150ms;
  }

  .collapsed-row:hover {
    background: var(--surface-hover);
  }

  .collapsed-text {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .expand-icon {
    margin-left: auto;
    font-size: 10px;
    color: var(--text-muted);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .section-header h2 {
    font-family: 'Geist Sans', sans-serif;
    font-size: 12px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-secondary);
    margin: 0;
  }

  .collapse-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 12px;
    padding: 2px 6px;
  }

  .collapse-btn:hover {
    color: var(--text-secondary);
  }

  .component-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .component-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 0;
    padding: 12px;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .component-name {
    font-family: 'Geist Sans', sans-serif;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-label {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
  }

  .bar-container {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .status-bar {
    display: flex;
    height: 6px;
    gap: 0;
    background: #1e2130;
  }

  .bucket {
    flex: 1;
    min-width: 0;
  }

  .bar-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .uptime-pct {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    color: var(--text-muted);
  }

  .bar-range {
    font-family: 'Geist Mono', monospace;
    font-size: 10px;
    color: var(--text-muted);
  }

  @media (max-width: 900px) {
    .component-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
