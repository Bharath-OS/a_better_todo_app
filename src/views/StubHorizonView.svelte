<script lang="ts">
  import TaskRow from "../components/TaskRow.svelte";
  import type { Task } from "../lib/api";

  interface Props {
    title: string;
    subtitle: string;
    tasks: Task[];
    onToggle: (id: string) => void;
    onOpen: (task: Task) => void;
    proLocked?: boolean;
    onUpgrade?: () => void;
  }

  let { title, subtitle, tasks, onToggle, onOpen, proLocked = false, onUpgrade }: Props =
    $props();
</script>

<header class="header">
  <h1>{title}</h1>
  <p class="sub">{subtitle}</p>
</header>

{#if proLocked}
  <div class="lock">
    <p>Monthly and Long-term goals are part of Todly Pro.</p>
    <button class="primary" onclick={() => onUpgrade?.()}>Unlock Pro (demo)</button>
  </div>
{:else}
  <section class="list">
    {#each tasks as task (task.id)}
      <TaskRow {task} {onToggle} {onOpen} />
    {:else}
      <p class="empty">Nothing here yet.</p>
    {/each}
  </section>
{/if}

<style>
  .header {
    margin-bottom: 24px;
  }
  h1 {
    margin: 0;
    font-size: 28px;
    line-height: 34px;
    font-weight: 600;
  }
  .sub {
    margin: 6px 0 0;
    color: var(--text-secondary);
    font-size: 14px;
  }
  .list {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: 8px;
  }
  .empty {
    margin: 0;
    padding: 20px 12px;
    color: var(--text-secondary);
  }
  .lock {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: 32px;
    max-width: 420px;
  }
  .primary {
    margin-top: 12px;
    height: 40px;
    padding: 0 16px;
    border: 0;
    border-radius: 12px;
    background: var(--accent);
    color: #fff;
    font-weight: 600;
  }
</style>
