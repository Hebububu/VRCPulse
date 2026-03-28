<script lang="ts">
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';
  import type { MetricResponse } from '../types';

  interface Props {
    data: MetricResponse | null;
    label1?: string;
    data2?: MetricResponse | null;
    label2?: string;
    color2?: string;
    title: string;
    type?: 'line' | 'area';
    unit?: string;
    thresholdValue?: number;
    thresholdColor?: string;
    hero?: boolean;
    hint?: string;
  }

  let { data, label1, data2, label2, color2 = '#a78bfa', title, type: chartType = 'line', unit = '', thresholdValue, thresholdColor, hero = false, hint = '' }: Props = $props();

  /** Dynamic precision: find enough decimals to show meaningful digits */
  function smartFormat(v: number, suffix: string = ''): string {
    if (v === 0) return `0${suffix}`;
    const abs = Math.abs(v);
    if (abs >= 1000) return `${(v / 1000).toFixed(1)}K${suffix}`;
    if (abs >= 100) return `${v.toFixed(0)}${suffix}`;
    if (abs >= 10) return `${v.toFixed(1)}${suffix}`;
    if (abs >= 1) return `${v.toFixed(2)}${suffix}`;
    // For values < 1, find first significant digit
    const digits = Math.max(0, Math.ceil(-Math.log10(abs))) + 2;
    return `${v.toFixed(Math.min(digits, 8))}${suffix}`;
  }

  function formatHeaderValue(v: number): string {
    const s = unit || '';
    return smartFormat(v, s);
  }

  let container: HTMLDivElement;
  let chart: echarts.ECharts | null = null;

  onMount(() => {
    chart = echarts.init(container, undefined, { renderer: 'canvas' });

    const observer = new ResizeObserver(() => chart?.resize());
    observer.observe(container);

    return () => {
      observer.disconnect();
      chart?.dispose();
    };
  });

  $effect(() => {
    if (!chart) return;

    if (!data || data.values.length === 0) {
      chart.clear();
      chart.setOption({
        title: {
          text: 'No data',
          left: 'center',
          top: 'center',
          textStyle: { color: '#71717a', fontSize: 14, fontFamily: 'Geist Mono, monospace' },
        },
        backgroundColor: 'transparent',
      });
      return;
    }

    const times = data.timestamps.map(t => new Date(t));
    const values = data.values;

    const suffix = unit || '';
    const formatValue = (v: number) => smartFormat(v, suffix);

    const markLine = thresholdValue != null ? {
      data: [{ yAxis: thresholdValue, label: { show: false } }],
      lineStyle: { color: thresholdColor || '#ef4444', type: 'dashed' as const, width: 1 },
      symbol: 'none',
    } : undefined;

    chart.setOption({
      backgroundColor: 'transparent',
      grid: { top: 8, right: 8, bottom: 24, left: 48 },
      xAxis: {
        type: 'time',
        axisLine: { lineStyle: { color: '#2a2d37' } },
        axisLabel: { color: '#71717a', fontSize: 10, fontFamily: 'Geist Mono, monospace' },
        splitLine: { show: false },
      },
      yAxis: {
        type: 'value',
        axisLine: { show: false },
        axisLabel: {
          color: '#71717a',
          fontSize: 10,
          fontFamily: 'Geist Mono, monospace',
          formatter: (v: number) => formatValue(v),
        },
        splitLine: { lineStyle: { color: '#2a2d37', type: 'dashed' } },
      },
      tooltip: {
        trigger: 'axis',
        backgroundColor: '#1a1d27',
        borderColor: '#2a2d37',
        textStyle: { color: '#e4e4e7', fontSize: 12, fontFamily: 'Geist Mono, monospace' },
        formatter: (params: any) => {
          const items = Array.isArray(params) ? params : [params];
          const date = new Date(items[0].value[0]);
          const time = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
          let html = time;
          for (const item of items) {
            const color = item.color || '#60a5fa';
            const name = item.seriesName || '';
            html += `<br/><span style="color:${color}">${name ? name + ': ' : ''}<strong>${formatValue(item.value[1])}</strong></span>`;
          }
          return html;
        },
      },
      series: (() => {
        const s: any[] = [{
          name: data2 ? (label1 || 'Primary') : '',
          type: 'line',
          showSymbol: false,
          smooth: true,
          lineStyle: { color: '#60a5fa', width: 2 },
          areaStyle: chartType === 'area' ? { color: 'rgba(96, 165, 250, 0.15)' } : undefined,
          data: times.map((t, i) => [t.getTime(), values[i]]),
          markLine,
          animationDuration: 300,
          animationEasing: 'cubicOut',
        }];
        if (data2 && data2.values.length > 0) {
          const times2 = data2.timestamps.map(t => new Date(t));
          s.push({
            name: label2 || 'Secondary',
            type: 'line',
            showSymbol: false,
            smooth: true,
            lineStyle: { color: color2, width: 2 },
            areaStyle: chartType === 'area' ? { color: color2.replace(')', ', 0.15)').replace('rgb', 'rgba') } : undefined,
            data: times2.map((t, i) => [t.getTime(), data2.values[i]]),
            animationDuration: 300,
            animationEasing: 'cubicOut',
          });
        }
        return s;
      })(),
    }, true);
  });
</script>

<div class="chart-card" class:hero>
  <div class="chart-header">
    <span class="chart-title">
      {title}
      {#if hint}
        <span class="hint-wrap">
          <span class="hint-icon">?</span>
          <span class="hint-tooltip">{hint}</span>
        </span>
      {/if}
    </span>
    <span class="chart-values">
      {#if data && data.values.length > 0}
        <span class="chart-value" style="color: #60a5fa">{formatHeaderValue(data.values[data.values.length - 1])}</span>
      {/if}
      {#if data2 && data2.values.length > 0}
        <span class="chart-value secondary" style="color: {color2}">{formatHeaderValue(data2.values[data2.values.length - 1])}</span>
      {/if}
    </span>
  </div>
  <div class="chart-container" bind:this={container}></div>
</div>

<style>
  .chart-card {
    background: #1a1d27;
    border: 1px solid #2a2d37;
    overflow: hidden;
    min-width: 0;
  }

  .chart-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 12px 16px 0;
  }

  .chart-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 500;
    color: #71717a;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .hint-wrap {
    position: relative;
    display: inline-flex;
  }

  .hint-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    font-size: 9px;
    font-weight: 700;
    font-family: 'Geist Mono', monospace;
    color: #71717a;
    border: 1px solid #71717a;
    border-radius: 50%;
    cursor: help;
    text-transform: none;
    letter-spacing: 0;
  }

  .hint-wrap:hover .hint-icon {
    color: #e4e4e7;
    border-color: #e4e4e7;
  }

  .hint-tooltip {
    display: none;
    position: absolute;
    top: 20px;
    left: 0;
    width: 220px;
    padding: 8px 10px;
    background: #1a1d27;
    border: 1px solid #2a2d37;
    color: #a1a1aa;
    font-size: 11px;
    font-weight: 400;
    font-family: 'Geist Sans', -apple-system, sans-serif;
    line-height: 1.4;
    text-transform: none;
    letter-spacing: 0;
    z-index: 10;
  }

  .hint-wrap:hover .hint-tooltip {
    display: block;
  }

  .chart-values {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }

  .chart-value {
    font-family: 'Geist Mono', monospace;
    font-size: 24px;
    font-weight: 700;
    color: #e4e4e7;
  }

  .chart-value.secondary {
    font-size: 16px;
  }

  .chart-container {
    width: 100%;
    height: 140px;
  }

  .hero .chart-container {
    height: 220px;
  }
</style>
