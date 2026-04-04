<script lang="ts">
  import type { ComponentStatus } from '../types';
  import { t, tDynamic } from '../i18n';

  interface Props {
    components: ComponentStatus[];
    range: string;
  }

  let { components, range }: Props = $props();

  let userExpanded = $state(false);

  // VRChat status API component hierarchy
  const GROUP_API_WEBSITE = '64b3rr3cxgk5';
  const GROUP_REALTIME_NETWORKING = 't1jm7fqqq43h';
  const API_WEBSITE_CHILDREN = new Set(['ll3syftt0xwm', 'fcb1zgxm9b3s', '6yydlg6mdf01', 'ftp7mrsh0fwm']);
  const REALTIME_NETWORKING_CHILDREN = new Set(['sc8glkrd3yr4', '76vv54mp1zfz', 'yxhq0fcg5lkj', '3rv208r2qv7z']);

  interface ComponentGroup {
    parent: ComponentStatus | null;
    children: ComponentStatus[];
  }

  const componentMap = $derived(
    new Map(components.map(c => [c.component_id, c]))
  );

  const groups = $derived.by((): ComponentGroup[] => {
    const result: ComponentGroup[] = [];
    const assigned = new Set<string>();

    const apiParent = componentMap.get(GROUP_API_WEBSITE);
    if (apiParent) {
      assigned.add(GROUP_API_WEBSITE);
      const children: ComponentStatus[] = [];
      for (const id of API_WEBSITE_CHILDREN) {
        assigned.add(id);
        const c = componentMap.get(id);
        if (c) children.push(c);
      }
      result.push({ parent: apiParent, children });
    }

    const netParent = componentMap.get(GROUP_REALTIME_NETWORKING);
    if (netParent) {
      assigned.add(GROUP_REALTIME_NETWORKING);
      const children: ComponentStatus[] = [];
      for (const id of REALTIME_NETWORKING_CHILDREN) {
        assigned.add(id);
        const c = componentMap.get(id);
        if (c) children.push(c);
      }
      result.push({ parent: netParent, children });
    }

    const ungrouped = components.filter(c => !assigned.has(c.component_id));
    if (ungrouped.length > 0) {
      result.push({ parent: null, children: ungrouped });
    }

    return result;
  });

  const allOperational = $derived(
    components.length > 0 && components.every(c => c.current_status === 'operational')
  );

  function statusColor(status: string): string {
    switch (status) {
      case 'operational': return 'var(--status-ok)';
      case 'degraded_performance': return 'var(--status-minor)';
      case 'partial_outage': return 'var(--status-major)';
      case 'major_outage': return 'var(--status-critical)';
      default: return 'var(--surface-deep)';
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

  function bucketTooltip(index: number, total: number, rangeStr: string, status: string): string {
    const rangeMinutes: Record<string, number> = { '1h': 60, '6h': 360, '12h': 720, '24h': 1440 };
    const totalMins = rangeMinutes[rangeStr] ?? 1440;
    const bucketMins = totalMins / total;
    const startAgo = Math.round((total - index) * bucketMins);
    const endAgo = Math.round((total - index - 1) * bucketMins);

    const fmt = (mins: number): string => {
      if (mins === 0) return t('component.now');
      if (mins >= 60) return `${Math.round(mins / 60)}h ${t('component.ago')}`;
      return `${mins}m ${t('component.ago')}`;
    };

    return `${fmt(startAgo)} – ${fmt(endAgo)}: ${statusLabel(status)}`;
  }

  function componentName(name: string): string {
    const key = `component.name.${name}`;
    const translated = tDynamic(key);
    return translated === key ? name : translated;
  }

  function componentAriaLabel(c: ComponentStatus): string {
    return `${componentName(c.name)}: ${statusLabel(c.current_status)}, ${uptimePercent(c.buckets)}% uptime in last ${rangeLabel(range)}`;
  }

  const legendStatuses = [
    { key: 'operational', label: () => t('component.operational') },
    { key: 'degraded_performance', label: () => t('component.degraded') },
    { key: 'partial_outage', label: () => t('component.partial_outage') },
    { key: 'major_outage', label: () => t('component.major_outage') },
  ];
</script>

{#if components.length > 0}
  <section class="component-panel" aria-label={t('component.title')}>
    <!-- Header bar — always visible, acts as toggle when collapsible -->
    {#if allOperational}
      <button
        class="panel-header"
        onclick={() => { userExpanded = !userExpanded; }}
        aria-label={userExpanded ? t('component.collapse') : t('component.expand')}
        aria-expanded={userExpanded}
      >
        <div class="header-left">
          <span class="status-dot" style="background: var(--status-ok);"></span>
          {#if !userExpanded}
            <span class="header-text">{t('component.all_operational').replace('{n}', String(components.length))}</span>
          {:else}
            <span class="header-label">{t('component.title')}</span>
          {/if}
        </div>
        <div class="header-right">
          {#if userExpanded}
            <div class="legend" aria-label="Status legend">
              {#each legendStatuses as ls}
                <span class="legend-item">
                  <span class="legend-dot" style="background: {statusColor(ls.key)};"></span>
                  <span class="legend-label">{ls.label()}</span>
                </span>
              {/each}
            </div>
          {/if}
          <span class="chevron" class:expanded={userExpanded} aria-hidden="true">&#9662;</span>
        </div>
      </button>
    {:else}
      <!-- Non-collapsible header when issues exist -->
      <div class="panel-header static">
        <div class="header-left">
          <span class="header-label">{t('component.title')}</span>
        </div>
        <div class="header-right">
          <div class="legend" aria-label="Status legend">
            {#each legendStatuses as ls}
              <span class="legend-item">
                <span class="legend-dot" style="background: {statusColor(ls.key)};"></span>
                <span class="legend-label">{ls.label()}</span>
              </span>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    <!-- Expanded body -->
    {#if userExpanded || !allOperational}
      <div class="panel-body">
        {#each groups as group}
          <div class="component-group">
            {#if group.parent}
              <div class="group-header">
                <span class="group-name">{componentName(group.parent.name)}</span>
                <span class="status-indicator">
                  <span class="status-dot" style="background: {statusColor(group.parent.current_status)};"></span>
                  <span class="status-label" style="color: {statusColor(group.parent.current_status)};">{statusLabel(group.parent.current_status)}</span>
                </span>
              </div>
            {/if}
            <div class="component-grid">
              {#each group.children as component (component.component_id)}
                <div class="component-card" role="group" aria-label={componentAriaLabel(component)}>
                  <div class="card-header">
                    <span class="component-name">{componentName(component.name)}</span>
                    <span class="status-indicator">
                      <span class="status-dot" style="background: {statusColor(component.current_status)};"></span>
                      <span class="status-label" style="color: {statusColor(component.current_status)};">{statusLabel(component.current_status)}</span>
                    </span>
                  </div>
                  <div class="bar-container">
                    <div class="status-bar" role="img" aria-label="{component.name} history: {uptimePercent(component.buckets)}% uptime">
                      {#each component.buckets as bucket, i}
                        <div
                          class="bucket status-{bucket}"
                          style="background: {statusColor(bucket)};"
                          title={bucketTooltip(i, component.buckets.length, range, bucket)}
                        ></div>
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
          </div>
        {/each}
      </div>
    {/if}
  </section>
{/if}

<style>
  /* Panel container */
  .component-panel {
    border: 1px solid var(--border);
    background: var(--surface);
  }

  /* Header bar — shared between collapsed and expanded */
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    min-height: 44px;
    padding: 10px 12px;
    background: var(--surface);
    border: none;
    border-radius: 0;
    cursor: pointer;
    transition: background 150ms;
    gap: 12px;
  }

  .panel-header:hover {
    background: var(--surface-hover);
  }

  .panel-header.static {
    cursor: default;
  }

  .panel-header.static:hover {
    background: var(--surface);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .header-text {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .header-label {
    font-family: 'Geist Sans', sans-serif;
    font-size: 12px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-secondary);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-left: auto;
  }

  .chevron {
    font-size: 10px;
    color: var(--text-muted);
    transition: transform 200ms ease-out;
  }

  .chevron.expanded {
    transform: rotate(180deg);
  }

  /* Legend */
  .legend {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .legend-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .legend-label {
    font-family: 'Geist Mono', monospace;
    font-size: 10px;
    color: var(--text-muted);
  }

  /* Panel body */
  .panel-body {
    border-top: 1px solid var(--border);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  /* Group hierarchy */
  .component-group:last-child {
    margin-bottom: 0;
  }

  .group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 0;
    margin-bottom: 8px;
  }

  .group-name {
    font-family: 'Geist Sans', sans-serif;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: 0.01em;
  }

  /* Card grid */
  .component-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }

  .component-card {
    background: var(--bg);
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
    color: var(--text-primary);
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

  /* History bar */
  .bar-container {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .status-bar {
    display: flex;
    height: 6px;
    gap: 0;
    background: var(--surface-deep);
  }

  .bucket {
    flex: 1;
    min-width: 0;
    cursor: default;
  }

  /* Colorblind-safe patterns for non-operational statuses */
  .bucket.status-degraded_performance {
    background-image: repeating-linear-gradient(
      45deg,
      transparent,
      transparent 2px,
      rgba(0, 0, 0, 0.2) 2px,
      rgba(0, 0, 0, 0.2) 3px
    );
  }

  .bucket.status-partial_outage {
    background-image: repeating-linear-gradient(
      90deg,
      transparent,
      transparent 2px,
      rgba(0, 0, 0, 0.25) 2px,
      rgba(0, 0, 0, 0.25) 3px
    );
  }

  .bucket.status-major_outage {
    background-image: repeating-linear-gradient(
      -45deg,
      transparent,
      transparent 1px,
      rgba(0, 0, 0, 0.3) 1px,
      rgba(0, 0, 0, 0.3) 3px
    );
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

  /* Responsive */
  @media (max-width: 900px) {
    .component-grid {
      grid-template-columns: 1fr;
    }

    .legend {
      display: none;
    }
  }

  @media (max-width: 768px) {
    .panel-header {
      min-height: 48px;
      padding: 12px;
    }

    .status-bar {
      height: 10px;
    }

    .panel-body {
      padding: 12px 8px;
    }
  }
</style>
