<script lang="ts">
  import { Sparkle } from 'phosphor-svelte';
  import { onDestroy } from 'svelte';
  import type { AiInsightResponse } from '../types';

  interface Props {
    insight: AiInsightResponse | null;
  }

  let { insight }: Props = $props();
  let prevInsightId = $state<number | null>(null);
  let streaming = $state(false);
  let activeTimers: ReturnType<typeof setTimeout>[] = [];

  // Streaming text state
  let displayHeadline = $state('');
  let displayBullets = $state<string[]>([]);
  let showFooter = $state(true);

  const CHAR_DELAY = 18;
  const BULLET_CHAR_DELAY = 12;
  const BULLET_GAP = 80;

  // Detect new insight and start streaming
  $effect(() => {
    if (!insight) return;
    if (insight.id === prevInsightId) return;

    prevInsightId = insight.id;
    startStreaming(insight);
  });

  // Cleanup only on component destroy
  onDestroy(() => clearAllTimers());

  function clearAllTimers() {
    activeTimers.forEach(clearTimeout);
    activeTimers = [];
  }

  function startStreaming(data: AiInsightResponse) {
    clearAllTimers();

    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      displayHeadline = data.headline;
      displayBullets = [...data.summary.bullets];
      showFooter = true;
      return;
    }

    streaming = true;
    displayHeadline = '';
    displayBullets = [];
    showFooter = false;

    const headline = data.headline;
    const bullets = data.summary.bullets;

    // Stream headline
    for (let i = 0; i <= headline.length; i++) {
      activeTimers.push(setTimeout(() => {
        displayHeadline = headline.slice(0, i);
      }, i * CHAR_DELAY));
    }

    const headlineDone = headline.length * CHAR_DELAY;

    // Stream bullets sequentially
    let bulletOffset = headlineDone + BULLET_GAP;
    for (let b = 0; b < bullets.length; b++) {
      const bullet = bullets[b];
      const slotTime = bulletOffset;

      activeTimers.push(setTimeout(() => {
        displayBullets = [...displayBullets, ''];
      }, slotTime));

      for (let c = 0; c <= bullet.length; c++) {
        activeTimers.push(setTimeout(() => {
          displayBullets = displayBullets.map((existing, idx) =>
            idx === b ? bullet.slice(0, c) : existing
          );
        }, slotTime + c * BULLET_CHAR_DELAY));
      }

      bulletOffset += bullet.length * BULLET_CHAR_DELAY + BULLET_GAP;
    }

    // Show footer after streaming completes
    activeTimers.push(setTimeout(() => {
      showFooter = true;
      streaming = false;
    }, bulletOffset));
  }

  function timeAgo(dateStr: string): string {
    const diff = Date.now() - new Date(dateStr).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return 'just now';
    if (mins < 60) return `${mins}min ago`;
    const hours = Math.floor(mins / 60);
    return `${hours}h ago`;
  }

  function timeUntil(dateStr: string): string {
    const diff = new Date(dateStr).getTime() - Date.now();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return 'soon';
    if (mins < 60) return `${mins}min`;
    const hours = Math.floor(mins / 60);
    return `${hours}h`;
  }

  function confidenceLabel(value: number): string {
    if (value >= 0.8) return '높음';
    if (value >= 0.5) return '보통';
    return '낮음';
  }

  let severityClass = $derived(
    insight?.summary?.severity === 'critical' ? 'critical' :
    insight?.summary?.severity === 'warning' ? 'warning' : ''
  );

  let badgeLabel = $derived(
    insight?.scope === 'incident' ? 'incident detected' :
    insight?.scope === 'maintenance' ? 'maintenance' :
    insight?.summary?.severity ?? 'stable'
  );
</script>

{#if insight}
  <div class="insight-card {severityClass}" role="region" aria-label="AI 서버 상태 분석">
    <div class="insight-header">
      <div class="insight-title-row">
        <div class="sparkle-icon" class:streaming>
          <Sparkle size={16} weight="fill" />
        </div>
        <span class="insight-label">AI Insight</span>
        <span class="insight-badge {severityClass}">{badgeLabel}</span>
        <span class="insight-confidence">신뢰도 {confidenceLabel(insight.confidence)}</span>
      </div>
      <div class="insight-meta-inline">
        <span>{timeAgo(insight.created_at)}</span>
        <span class="separator" aria-hidden="true">·</span>
        <span>{insight.model_id}</span>
      </div>
    </div>

    <div class="insight-headline">
      {displayHeadline}{#if streaming && displayBullets.length === 0}<span class="cursor" aria-hidden="true"></span>{/if}
    </div>

    {#if displayBullets.length > 0}
      <ul class="insight-bullets">
        {#each displayBullets as bullet, i}
          <li>
            {bullet}{#if streaming && i === displayBullets.length - 1 && bullet !== (insight.summary.bullets[i] ?? '')}<span class="cursor" aria-hidden="true"></span>{/if}
          </li>
        {/each}
      </ul>
    {/if}

    {#if showFooter}
      <div class="insight-footer">
        <span>다음 분석: {timeUntil(insight.expires_at)}</span>
        <span class="insight-basis">24시간 기준</span>
      </div>
    {/if}
  </div>
{/if}

<style>
  .insight-card {
    background: linear-gradient(180deg, rgba(96, 165, 250, 0.06) 0%, #1a1d27 100%);
    border: 1px solid rgba(96, 165, 250, 0.2);
    border-top: 2px solid #60a5fa;
    padding: 12px 16px;
    position: relative;
    transition: border-color 200ms, background 200ms;
  }

  .insight-card.warning {
    background: linear-gradient(180deg, rgba(234, 179, 8, 0.05) 0%, #1a1d27 100%);
    border-color: rgba(234, 179, 8, 0.2);
    border-top-color: #eab308;
    border-left: 3px solid #f97316;
  }

  .insight-card.critical {
    background: linear-gradient(180deg, rgba(239, 68, 68, 0.05) 0%, #1a1d27 100%);
    border-color: rgba(239, 68, 68, 0.2);
    border-top-color: #ef4444;
    border-left: 3px solid #ef4444;
  }

  .insight-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .insight-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sparkle-icon {
    color: #60a5fa;
    display: flex;
    align-items: center;
    transition: color 200ms;
  }

  .insight-card.warning .sparkle-icon { color: #eab308; }
  .insight-card.critical .sparkle-icon { color: #ef4444; }

  @keyframes sparkle-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .sparkle-icon.streaming {
    animation: sparkle-pulse 1s ease-in-out infinite;
  }

  .insight-label {
    font-size: 12px;
    font-weight: 700;
    color: #60a5fa;
    font-family: 'Geist Sans', sans-serif;
    letter-spacing: 0.01em;
    transition: color 200ms;
  }

  .insight-card.warning .insight-label { color: #eab308; }
  .insight-card.critical .insight-label { color: #ef4444; }

  .insight-badge {
    font-size: 11px;
    padding: 2px 8px;
    background: rgba(34, 197, 94, 0.12);
    color: #22c55e;
    font-family: 'Geist Mono', monospace;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    transition: color 200ms, background 200ms;
  }

  .insight-badge.warning {
    background: rgba(234, 179, 8, 0.12);
    color: #eab308;
  }

  .insight-badge.critical {
    background: rgba(239, 68, 68, 0.12);
    color: #ef4444;
  }

  .insight-confidence {
    font-size: 11px;
    color: #52525b;
    font-family: 'Geist Mono', monospace;
  }

  .insight-meta-inline {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: #52525b;
    font-family: 'Geist Mono', monospace;
    flex-wrap: wrap;
  }

  .separator { color: #3f3f46; }

  .insight-headline {
    font-size: 16px;
    font-weight: 600;
    margin-bottom: 8px;
    line-height: 1.4;
    color: #f4f4f5;
    min-height: 24px;
  }

  .insight-bullets {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .insight-bullets li {
    font-size: 14px;
    color: #a1a1aa;
    padding: 4px 0;
    padding-left: 16px;
    position: relative;
    line-height: 1.5;
    min-height: 20px;
  }

  .insight-bullets li::before {
    content: '';
    position: absolute;
    left: 4px;
    top: 11px;
    width: 4px;
    height: 4px;
    background: #60a5fa;
    opacity: 0.5;
  }

  .insight-card.warning .insight-bullets li::before {
    background: #eab308;
  }

  .insight-card.critical .insight-bullets li::before {
    background: #ef4444;
  }

  .cursor {
    display: inline-block;
    width: 2px;
    height: 14px;
    background: #60a5fa;
    margin-left: 1px;
    vertical-align: text-bottom;
    animation: blink 600ms step-end infinite;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0; }
  }

  .insight-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid #2a2d37;
  }

  .insight-footer span {
    font-size: 11px;
    color: #52525b;
    font-family: 'Geist Mono', monospace;
  }

  .insight-basis {
    color: rgba(96, 165, 250, 0.6) !important;
  }

  @media (prefers-reduced-motion: reduce) {
    .sparkle-icon.streaming {
      animation: none;
    }
    .cursor {
      animation: none;
      opacity: 0;
    }
  }

  @media (max-width: 768px) {
    .insight-card { padding: 10px 12px; }
    .insight-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 6px;
      margin-bottom: 6px;
    }
    .insight-title-row { flex-wrap: wrap; gap: 6px; }
    .insight-headline { font-size: 14px; margin-bottom: 6px; }
    .insight-bullets li {
      font-size: 13px;
      padding: 3px 0;
      padding-left: 14px;
      line-height: 1.4;
    }
    .insight-footer { margin-top: 6px; padding-top: 6px; }
    .insight-confidence { display: none; }
  }
</style>
