<script lang="ts">
  import { push } from 'svelte-spa-router';
  import { Wrench } from 'phosphor-svelte';
  import { t } from '../i18n';
  import type { Maintenance } from '../types';

  interface Props {
    maintenances: Maintenance[];
  }

  let { maintenances }: Props = $props();

  const active = $derived(maintenances.filter(m => m.status === 'in_progress'));

  function formatTime(dateStr: string): string {
    const d = new Date(dateStr);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', timeZone: 'UTC' }) + ' UTC';
  }
</script>

{#if active.length > 0}
  {#each active as m}
    <button class="banner" onclick={() => push(`/maintenances/${m.id}`)}>
      <div class="banner-left">
        <Wrench size={14} weight="bold" />
        <span class="banner-text">{t('maintenance.bannerText')}</span>
        <span class="banner-title">{m.name}</span>
      </div>
      <div class="banner-right">
        <span class="banner-until">{t('maintenance.scheduledUntil')} {formatTime(m.scheduled_until)}</span>
        <span class="banner-link">{t('maintenance.bannerLink')} →</span>
      </div>
    </button>
  {/each}
{/if}

<style>
  .banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 10px 16px;
    background: rgba(249, 115, 22, 0.08);
    border: 1px solid rgba(249, 115, 22, 0.25);
    border-left: 3px solid #f97316;
    color: inherit;
    font-family: inherit;
    cursor: pointer;
    text-align: left;
  }

  .banner:hover {
    background: rgba(249, 115, 22, 0.12);
  }

  .banner-left {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #f97316;
    min-width: 0;
    flex: 1;
  }

  .banner-text {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    flex-shrink: 0;
  }

  .banner-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .banner-right {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
  }

  .banner-until {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .banner-link {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    color: #f97316;
  }

  @media (max-width: 768px) {
    .banner {
      flex-direction: column;
      align-items: flex-start;
      gap: 6px;
      padding: 10px 12px;
      min-height: 44px;
    }
    .banner-right { width: 100%; justify-content: space-between; }
  }
</style>
