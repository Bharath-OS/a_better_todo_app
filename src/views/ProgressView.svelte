<script lang="ts">
  import type { DayStats } from "../lib/api";

  interface Props {
    stats: DayStats[];
    isPro: boolean;
  }

  let { stats, isPro }: Props = $props();

  const max = $derived(Math.max(1, ...stats.map((s) => s.completed)));
</script>

<header class="header">
  <h1>Progress</h1>
  <p class="sub">Last 7 days · local day boundaries</p>
</header>

<section class="card">
  <h2>Tasks completed</h2>
  <div class="chart">
    {#each stats as day}
      <div class="col">
        <div
          class="bar"
          style="height: {(day.completed / max) * 100}%"
          title="{day.date}: {day.completed}"
        ></div>
        <span>{day.date.slice(5)}</span>
      </div>
    {/each}
  </div>
</section>

{#if !isPro}
  <p class="hint">Heatmap, 30/90-day views and export unlock with Pro (demo).</p>
{:else}
  <section class="card muted">
    <h2>12-week heatmap</h2>
    <p class="hint">Stub — cells will reflect local completion density.</p>
  </section>
{/if}

<style>
  .header {
    margin-bottom: 24px;
  }
  h1 {
    margin: 0;
    font-size: 28px;
    font-weight: 600;
  }
  .sub,
  .hint {
    color: var(--text-secondary);
    font-size: 14px;
  }
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: 20px;
  }
  .card.muted {
    margin-top: 16px;
  }
  h2 {
    margin: 0 0 16px;
    font-size: 16px;
  }
  .chart {
    display: flex;
    align-items: flex-end;
    gap: 12px;
    height: 160px;
  }
  .col {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100%;
    justify-content: flex-end;
    gap: 8px;
  }
  .bar {
    width: 100%;
    min-height: 2px;
    background: var(--accent);
    border-radius: 6px 6px 2px 2px;
  }
  .col span {
    font-size: 11px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }
</style>
