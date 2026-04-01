<script lang="ts">
  import { push } from 'svelte-spa-router';
  import { t, getLocale } from '../i18n';
  import type { Maintenance, TranslationResponse } from '../types';

  interface Props {
    maintenances: Maintenance[];
    translations?: Record<string, TranslationResponse>;
  }

  let { maintenances, translations = {} }: Props = $props();

  const recent = $derived(maintenances.slice(0, 5));
  const needsTranslation = getLocale() !== 'en';

  let showOriginal: Record<string, boolean> = $state({});

  function toggleOriginal(e: Event, id: string) {
    e.stopPropagation();
    showOriginal[id] = !showOriginal[id];
  }

  function getDisplayName(m: Maintenance): string {
    if (needsTranslation && translations[m.id] && !showOriginal[m.id]) {
      return translations[m.id].translated_name;
    }
    return m.name;
  }

  function isTranslated(id: string): boolean {
    return needsTranslation && !!translations[id] && !showOriginal[id];
  }

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
          <span class="item-name">{getDisplayName(m)}</span>
          {#if isTranslated(m.id)}
            <span
              class="translate-btn translated"
              role="button"
              tabindex="0"
              onclick={(e) => toggleOriginal(e, m.id)}
              onkeydown={(e) => e.key === 'Enter' && toggleOriginal(e, m.id)}
              aria-label={t('translate.showOriginal')}
            >
              <svg width="14" height="14" viewBox="0 0 256 256" fill="currentColor">
                <path d="M208,144a15.78,15.78,0,0,1-10.42,14.94l-51.65,19.06L126.87,229.65a16,16,0,0,1-30.08-.57l-17.64-48.18L31,163.26a16,16,0,0,1,.57-30.08L79.68,115.1l19.06-51.65a15.78,15.78,0,0,1,29.86.36l18.64,48.42,48.42,18.64A15.78,15.78,0,0,1,208,144ZM152,48h16V64a8,8,0,0,0,16,0V48h16a8,8,0,0,0,0-16H184V16a8,8,0,0,0-16,0V32H152a8,8,0,0,0,0,16Zm88,32h-8V72a8,8,0,0,0-16,0v8h-8a8,8,0,0,0,0,16h8v8a8,8,0,0,0,16,0V96h8a8,8,0,0,0,0-16Z"/>
              </svg>
              <span class="translate-badge">{t('translate.aiTranslated')}</span>
            </span>
          {:else if needsTranslation && showOriginal[m.id]}
            <span
              class="translate-btn"
              role="button"
              tabindex="0"
              onclick={(e) => toggleOriginal(e, m.id)}
              onkeydown={(e) => e.key === 'Enter' && toggleOriginal(e, m.id)}
              aria-label={t('translate.button')}
            >
              <svg width="14" height="14" viewBox="0 0 256 256" fill="currentColor">
                <path d="M208,144a15.78,15.78,0,0,1-10.42,14.94l-51.65,19.06L126.87,229.65a16,16,0,0,1-30.08-.57l-17.64-48.18L31,163.26a16,16,0,0,1,.57-30.08L79.68,115.1l19.06-51.65a15.78,15.78,0,0,1,29.86.36l18.64,48.42,48.42,18.64A15.78,15.78,0,0,1,208,144ZM152,48h16V64a8,8,0,0,0,16,0V48h16a8,8,0,0,0,0-16H184V16a8,8,0,0,0-16,0V32H152a8,8,0,0,0,0,16Zm88,32h-8V72a8,8,0,0,0-16,0v8h-8a8,8,0,0,0,0,16h8v8a8,8,0,0,0,16,0V96h8a8,8,0,0,0,0-16Z"/>
              </svg>
            </span>
          {/if}
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

  .translate-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 28px;
    height: 28px;
    justify-content: center;
    background: none;
    border: none;
    color: rgba(96, 165, 250, 0.4);
    cursor: pointer;
    flex-shrink: 0;
    padding: 0;
    transition: color 200ms;
  }
  .translate-btn:hover { color: var(--accent); }
  .translate-btn.translated { color: var(--accent); width: auto; }
  .translate-badge {
    font-family: 'Geist Sans', sans-serif;
    font-size: 11px;
    font-weight: 500;
    background: rgba(96, 165, 250, 0.2);
    padding: 1px 6px;
    white-space: nowrap;
  }
  @keyframes sparkle-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
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
