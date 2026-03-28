<script lang="ts">
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';
  import type { MetricResponse } from '../types';

  interface Props {
    data: MetricResponse | null;
    title: string;
    type?: 'line' | 'area';
    unit?: string;
    thresholdValue?: number;
    thresholdColor?: string;
    hero?: boolean;
  }

  let { data, title, type: chartType = 'line', unit = '', thresholdValue, thresholdColor, hero = false }: Props = $props();

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

    const formatValue = (v: number) => {
      if (unit === '%') return `${v.toFixed(1)}%`;
      if (unit === 'ms') return `${v.toFixed(0)}ms`;
      if (v >= 1000) return `${(v / 1000).toFixed(1)}K`;
      return v.toFixed(0);
    };

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
          const p = Array.isArray(params) ? params[0] : params;
          const date = new Date(p.value[0]);
          const time = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
          return `${time}<br/><strong>${formatValue(p.value[1])}</strong>`;
        },
      },
      series: [{
        type: 'line',
        showSymbol: false,
        smooth: true,
        lineStyle: { color: '#60a5fa', width: 2 },
        areaStyle: chartType === 'area' ? { color: 'rgba(96, 165, 250, 0.15)' } : undefined,
        data: times.map((t, i) => [t.getTime(), values[i]]),
        markLine,
        animationDuration: 300,
        animationEasing: 'cubicOut',
      }],
    }, true);
  });
</script>

<div class="chart-card" class:hero>
  <div class="chart-header">
    <span class="chart-title">{title}</span>
    {#if data && data.values.length > 0}
      <span class="chart-value">
        {#if unit === '%'}
          {data.values[data.values.length - 1].toFixed(1)}%
        {:else if unit === 'ms'}
          {data.values[data.values.length - 1].toFixed(0)}ms
        {:else if data.values[data.values.length - 1] >= 1000}
          {(data.values[data.values.length - 1] / 1000).toFixed(1)}K
        {:else}
          {data.values[data.values.length - 1].toFixed(0)}
        {/if}
      </span>
    {/if}
  </div>
  <div class="chart-container" bind:this={container}></div>
</div>

<style>
  .chart-card {
    background: #1a1d27;
    border: 1px solid #2a2d37;
    overflow: hidden;
  }

  .chart-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 12px 16px 0;
  }

  .chart-title {
    font-size: 12px;
    font-weight: 500;
    color: #71717a;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .chart-value {
    font-family: 'Geist Mono', monospace;
    font-size: 24px;
    font-weight: 700;
    color: #e4e4e7;
  }

  .chart-container {
    width: 100%;
    height: 140px;
  }

  .hero .chart-container {
    height: 220px;
  }
</style>
