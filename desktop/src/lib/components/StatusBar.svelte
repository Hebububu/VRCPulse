<script lang="ts">
  import type { StatusResponse } from '../types';
  import logoImg from '../../assets/logo.png';

  interface Props {
    status: StatusResponse | null;
    lastUpdated: Date | null;
  }

  let { status, lastUpdated }: Props = $props();

  let elapsed = $state('...');

  $effect(() => {
    const interval = setInterval(() => {
      if (lastUpdated) {
        const secs = Math.floor((Date.now() - lastUpdated.getTime()) / 1000);
        if (secs < 60) elapsed = `${secs}s ago`;
        else elapsed = `${Math.floor(secs / 60)}m ago`;
      }
    }, 1000);
    return () => clearInterval(interval);
  });

  const statusColor = $derived.by(() => {
    if (!status) return '#71717a';
    switch (status.indicator) {
      case 'none': return '#22c55e';
      case 'minor': return '#eab308';
      case 'major': return '#f97316';
      case 'critical': return '#ef4444';
      default: return '#71717a';
    }
  });

  const statusLabel = $derived.by(() => {
    if (!status) return 'Connecting...';
    switch (status.indicator) {
      case 'none': return 'Operational';
      case 'minor': return 'Minor Issues';
      case 'major': return 'Major Outage';
      case 'critical': return 'Critical Outage';
      default: return status.indicator;
    }
  });
</script>

<header class="status-bar">
  <div class="left">
    <img src={logoImg} alt="VRCPulse" class="app-logo" />
    <span class="app-name">VRCPulse</span>
    <span class="status-dot" style="background: {statusColor}"></span>
    <span class="status-label" style="color: {statusColor}">{statusLabel}</span>
  </div>
  <div class="right">
    <span class="updated">Last updated {elapsed}</span>
  </div>
</header>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 56px;
    padding: 0 16px;
    background: #1a1d27;
    border-bottom: 1px solid #2a2d37;
    flex-shrink: 0;
  }

  .left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .app-logo {
    width: 28px;
    height: 28px;
  }

  .app-name {
    font-size: 16px;
    font-weight: 600;
    color: #e4e4e7;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    transition: background 200ms ease-out;
  }

  .status-label {
    font-size: 14px;
    font-weight: 500;
    transition: color 200ms ease-out;
  }

  .right {
    display: flex;
    align-items: center;
  }

  .updated {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    color: #71717a;
  }
</style>
