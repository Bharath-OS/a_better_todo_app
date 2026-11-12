<script lang="ts">
  import type { Task } from "../lib/api";

  interface Props {
    task: Task;
    onToggle: (id: string) => void;
    onOpen?: (task: Task) => void;
  }

  let { task, onToggle, onOpen }: Props = $props();

  function dueLabel(due: string | null): string | null {
    if (!due) return null;
    const d = new Date(due);
    if (Number.isNaN(d.getTime())) return due;
    return d.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
  }
</script>

<div class="row" class:done={!!task.completed_at}>
  <button
    class="check"
    class:checked={!!task.completed_at}
    aria-label={task.completed_at ? "Mark incomplete" : "Complete task"}
    onclick={() => onToggle(task.id)}
  >
    {#if task.completed_at}
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
        <path
          d="M2.5 6.2 4.8 8.5 9.5 3.5"
          stroke="#fff"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    {/if}
  </button>
  <button class="title" onclick={() => onOpen?.(task)}>{task.title}</button>
  {#if dueLabel(task.due_at)}
    <span class="chip">{dueLabel(task.due_at)}</span>
  {/if}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 48px;
    padding: 0 8px;
    border-radius: 12px;
  }
  .row:hover {
    background: var(--surface-subtle);
  }
  .row.done .title {
    color: var(--text-tertiary);
    text-decoration: line-through;
  }
  .check {
    width: 20px;
    height: 20px;
    border-radius: 6px;
    border: 1.5px solid #c5ccd8;
    background: transparent;
    display: grid;
    place-items: center;
    padding: 0;
    flex-shrink: 0;
  }
  .check.checked {
    background: var(--success);
    border-color: var(--success);
  }
  .title {
    flex: 1;
    border: 0;
    background: transparent;
    text-align: left;
    color: var(--text);
    font-size: 14px;
    font-weight: 500;
    padding: 8px 0;
  }
  .chip {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    background: var(--surface-subtle);
    border-radius: 8px;
    padding: 4px 8px;
    font-variant-numeric: tabular-nums;
  }
</style>
