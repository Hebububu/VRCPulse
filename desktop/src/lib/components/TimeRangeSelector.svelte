<script lang="ts">
  interface Props {
    value: string;
    onChange: (range: string) => void;
  }

  let { value, onChange }: Props = $props();

  const ranges = ['1h', '6h', '12h', '24h'];
</script>

<div class="selector">
  {#each ranges as range}
    <button
      class="range-btn"
      class:active={value === range}
      onclick={() => onChange(range)}
    >
      {range}
    </button>
  {/each}
</div>

<style>
  .selector {
    display: flex;
    gap: 0;
    border: 1px solid var(--border);
  }

  .range-btn {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    font-weight: 500;
    padding: 6px 12px;
    background: transparent;
    color: var(--text-secondary);
    border: none;
    border-right: 1px solid var(--border);
    cursor: pointer;
    transition: all 100ms;
  }

  .range-btn:last-child {
    border-right: none;
  }

  .range-btn:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .range-btn.active {
    color: var(--accent);
    background: var(--surface);
  }

  @media (max-width: 768px) {
    .selector { width: 100%; }
    .range-btn {
      flex: 1;
      min-height: 44px;
      padding: 8px 12px;
      font-size: 13px;
      text-align: center;
    }
  }
</style>
