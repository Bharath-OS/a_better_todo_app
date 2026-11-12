<script lang="ts">
  import TaskRow from "../components/TaskRow.svelte";
  import type { Task } from "../lib/api";
  import { createTask } from "../lib/api";

  interface Props {
    tasks: Task[];
    onToggle: (id: string) => void;
    onOpen: (task: Task) => void;
    onChanged: () => void;
  }

  let { tasks, onToggle, onOpen, onChanged }: Props = $props();
  let draft = $state("");
  let error = $state("");

  const open = $derived(tasks.filter((t) => !t.completed_at));
  const done = $derived(tasks.filter((t) => t.completed_at));

  async function add() {
    const title = draft.trim();
    if (!title) return;
    error = "";
    try {
      await createTask({ title, horizon: "daily" });
      draft = "";
      onChanged();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<header class="header">
  <div>
    <h1>Today</h1>
    <p class="sub">{open.length} left · {done.length} done</p>
  </div>
</header>

<div class="composer">
  <input
    bind:value={draft}
    placeholder="Add a task for today…"
    onkeydown={(e) => e.key === "Enter" && add()}
  />
  <button class="primary" onclick={add}>Add</button>
</div>
{#if error}
  <p class="error">{error}</p>
{/if}

<section class="list">
  {#each open as task (task.id)}
    <TaskRow {task} {onToggle} {onOpen} />
  {/each}
  {#if open.length === 0}
    <p class="empty">All clear for today. Add something above, or use Ctrl+Alt+T.</p>
  {/if}
</section>

{#if done.length}
  <h2 class="section-title">Done</h2>
  <section class="list">
    {#each done as task (task.id)}
      <TaskRow {task} {onToggle} {onOpen} />
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
  .composer {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }
  input {
    flex: 1;
    height: 44px;
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    padding: 0 14px;
    background: var(--surface);
    color: var(--text);
  }
  .primary {
    height: 44px;
    padding: 0 18px;
    border: 0;
    border-radius: var(--radius-control);
    background: var(--accent);
    color: #fff;
    font-weight: 600;
  }
  .error {
    color: var(--danger);
    font-size: 13px;
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
    font-size: 14px;
  }
  .section-title {
    margin: 28px 0 12px;
    font-size: 16px;
    font-weight: 600;
  }
</style>
