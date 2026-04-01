<script lang="ts">
  import { push } from 'svelte-spa-router';
  import { t, getLocale } from '../i18n';
  import type { Incident, TranslationResponse } from '../types';

  interface Props {
    incidents: Incident[];
    maxItems?: number;
    compact?: boolean;
    translations?: Record<string, TranslationResponse>;
  }

  let { incidents, maxItems = 5, compact = false, translations = {} }: Props = $props();

  const recent = $derived(incidents.slice(0, maxItems));
  const needsTranslation = getLocale() !== 'en';

  // Track which incidents user toggled to show original
  let showOriginal: Record<string, boolean> = $state({});

  function toggleOriginal(e: Event, id: string) {
    e.stopPropagation();
    showOriginal[id] = !showOriginal[id];
  }

  function getDisplayName(incident: Incident): string {
    if (needsTranslation && translations[incident.id] && !showOriginal[incident.id]) {
      return translations[incident.id].translated_name;
    }
    return incident.name;
  }

  function isTranslated(id: string): boolean {
    return needsTranslation && !!translations[id] && !showOriginal[id];
  }

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
    {#if !compact}
      <button class="view-all" onclick={() => push('/incidents')}>{t('incidents.viewAll')}</button>
    {/if}
  </div>

  {#if recent.length === 0}
    <div class="empty">{t('incidents.noRecords')}</div>
  {:else}
    {#each recent as incident}
      <button class="incident" onclick={() => push(`/incidents/${incident.id}`)}>
        <div class="incident-header">
          <span class="impact-dot" style="background: {impactColor(incident.impact)}"></span>
          <span class="incident-name">{getDisplayName(incident)}</span>
          {#if isTranslated(incident.id)}
            <span
              class="translate-btn translated"
              role="button"
              tabindex="0"
              onclick={(e) => toggleOriginal(e, incident.id)}
              onkeydown={(e) => e.key === 'Enter' && toggleOriginal(e, incident.id)}
              aria-label={t('translate.showOriginal')}
            >
              <svg width="14" height="14" viewBox="0 0 256 256" fill="currentColor">
                <path d="M208,144a15.78,15.78,0,0,1-10.42,14.94l-51.65,19.06L126.87,229.65a16,16,0,0,1-30.08-.57l-17.64-48.18L31,163.26a16,16,0,0,1,.57-30.08L79.68,115.1l19.06-51.65a15.78,15.78,0,0,1,29.86.36l18.64,48.42,48.42,18.64A15.78,15.78,0,0,1,208,144ZM152,48h16V64a8,8,0,0,0,16,0V48h16a8,8,0,0,0,0-16H184V16a8,8,0,0,0-16,0V32H152a8,8,0,0,0,0,16Zm88,32h-8V72a8,8,0,0,0-16,0v8h-8a8,8,0,0,0,0,16h8v8a8,8,0,0,0,16,0V96h8a8,8,0,0,0,0-16Z"/>
              </svg>
              <span class="translate-badge">{t('translate.aiTranslated')}</span>
            </span>
          {:else if needsTranslation && showOriginal[incident.id]}
            <span
              class="translate-btn"
              role="button"
              tabindex="0"
              onclick={(e) => toggleOriginal(e, incident.id)}
              onkeydown={(e) => e.key === 'Enter' && toggleOriginal(e, incident.id)}
              aria-label={t('translate.button')}
            >
              <svg width="14" height="14" viewBox="0 0 256 256" fill="currentColor">
                <path d="M208,144a15.78,15.78,0,0,1-10.42,14.94l-51.65,19.06L126.87,229.65a16,16,0,0,1-30.08-.57l-17.64-48.18L31,163.26a16,16,0,0,1,.57-30.08L79.68,115.1l19.06-51.65a15.78,15.78,0,0,1,29.86.36l18.64,48.42,48.42,18.64A15.78,15.78,0,0,1,208,144ZM152,48h16V64a8,8,0,0,0,16,0V48h16a8,8,0,0,0,0-16H184V16a8,8,0,0,0-16,0V32H152a8,8,0,0,0,0,16Zm88,32h-8V72a8,8,0,0,0-16,0v8h-8a8,8,0,0,0,0,16h8v8a8,8,0,0,0,16,0V96h8a8,8,0,0,0,0-16Z"/>
              </svg>
            </span>
          {/if}
        </div>
        <div class="incident-meta">
          <span class="status-badge" style="color: {statusBadgeColor(incident.status)}">{incident.status}</span>
          <span class="incident-time">{timeAgo(incident.created_at)}</span>
        </div>
      </button>
    {/each}
  {/if}
  <div class="translate-announce" aria-live="polite"></div>
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

  .view-all:hover {
    background: var(--surface-hover);
  }

  .empty {
    font-size: 14px;
    color: var(--text-secondary);
    font-family: 'Geist Mono', monospace;
  }

  .incident {
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

  .incident:last-child {
    border-bottom: none;
  }

  .incident:hover {
    background: var(--surface-hover);
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
    color: var(--text-primary);
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

  .translate-btn:hover {
    color: var(--accent);
  }

  .translate-btn.translated {
    color: var(--accent);
    width: auto;
    gap: 4px;
  }

  .translate-badge {
    font-family: 'Geist Sans', sans-serif;
    font-size: 11px;
    font-weight: 500;
    background: rgba(96, 165, 250, 0.2);
    padding: 1px 6px;
    white-space: nowrap;
  }

  .translate-announce {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
  }

  @keyframes sparkle-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  @media (max-width: 768px) {
    .feed { padding: 12px; }
    .incident {
      padding: 12px 0;
    }
    .incident:hover {
      margin: 0;
      padding: 12px 0;
      width: 100%;
      background: transparent;
    }
    .incident:active {
      background: var(--surface-hover);
      margin: 0 -12px;
      padding: 12px;
      width: calc(100% + 24px);
    }
    .view-all {
      min-height: 44px;
      padding: 8px 16px;
      display: flex;
      align-items: center;
    }
    .incident-name { font-size: 14px; }
  }
</style>
