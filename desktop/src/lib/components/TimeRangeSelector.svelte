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
    border: 1px solid #2a2d37;
  }

  .range-btn {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    font-weight: 500;
    padding: 6px 12px;
    background: transparent;
    color: #71717a;
    border: none;
    border-right: 1px solid #2a2d37;
    cursor: pointer;
    transition: all 100ms;
  }

  .range-btn:last-child {
    border-right: none;
  }

  .range-btn:hover {
    color: #e4e4e7;
    background: #22252f;
  }

  .range-btn.active {
    color: #60a5fa;
    background: #1a1d27;
  }
</style>
