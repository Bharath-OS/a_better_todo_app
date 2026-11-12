<script lang="ts">
  import type { Horizon, Task } from "../lib/api";
  import { deleteTask } from "../lib/api";
  import { invoke } from "@tauri-apps/api/core";

  interface Props {
    task: Task | null;
    isPro: boolean;
    onClose: () => void;
    onChanged: () => void;
  }

  let { task, isPro, onClose, onChanged }: Props = $props();

  let title = $state("");
  let notes = $state("");
  let horizon = $state<Horizon>("daily");
  let dueDate = $state("");
  let dueTime = $state("");
  let remind = $state(10);

  $effect(() => {
    if (!task) return;
    title = task.title;
    notes = task.notes ?? "";
    horizon = task.horizon;
    remind = task.remind_minutes_before;
    if (task.due_at) {
      const [d, t] = task.due_at.split("T");
      dueDate = d ?? "";
      dueTime = (t ?? "").slice(0, 5);
    } else {
      dueDate = "";
      dueTime = "";
    }
  });

  async function save() {
    if (!task) return;
    const due_at =
      dueDate && dueTime ? `${dueDate}T${dueTime}:00` : dueDate ? `${dueDate}T17:00:00` : null;
    try {
      await invoke("update_task", {
        args: {
          id: task.id,
          title: title.trim() || task.title,
          horizon,
          due_at,
          clear_due: due_at === null,
          notes: notes || null,
          remind_minutes_before: remind,
          progress: null,
        },
      });
    } catch {
      /* preview mode */
    }
    onChanged();
    onClose();
  }

  async function remove() {
    if (!task) return;
    await deleteTask(task.id);
    onChanged();
    onClose();
  }
</script>

{#if task}
  <div class="backdrop" role="presentation" onclick={onClose}>
    <div class="dialog" role="dialog" aria-modal="true" onclick={(e) => e.stopPropagation()}>
      <header>
        <h1>Edit task</h1>
        <button class="x" onclick={onClose} aria-label="Close">×</button>
      </header>

      <label>
        <span>Title</span>
        <input bind:value={title} />
      </label>
      <label>
        <span>Notes</span>
        <textarea rows="3" bind:value={notes}></textarea>
      </label>
      <div class="seg">
        <span>Horizon</span>
        <div class="track">
          {#each ["daily", "weekly", "monthly", "goal"] as h}
            <button class:active={horizon === h} onclick={() => (horizon = h as Horizon)}>
              {h === "goal" ? "Long-term" : h[0].toUpperCase() + h.slice(1)}
            </button>
          {/each}
        </div>
      </div>
      <div class="row2">
        <label>
          <span>Deadline date</span>
          <input type="date" bind:value={dueDate} />
        </label>
        <label>
          <span>Deadline time</span>
          <input type="time" bind:value={dueTime} />
        </label>
      </div>
      <label>
        <span>Remind me</span>
        <select bind:value={remind}>
          {#each [5, 10, 15, 30, 60] as n}
            <option value={n}>{n} minutes before</option>
          {/each}
        </select>
      </label>
      <div class="second">
        <span>Second reminder</span>
        <span class="pro">{isPro ? "ON" : "PRO"}</span>
        <input type="checkbox" disabled={!isPro} />
      </div>

      <footer>
        <button class="danger" onclick={remove}>Delete</button>
        <div class="spacer"></div>
        <button class="ghost" onclick={onClose}>Cancel</button>
        <button class="primary" onclick={save}>Save</button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(16, 24, 40, 0.35);
    display: grid;
    place-items: center;
    z-index: 40;
    padding: 24px;
  }
  .dialog {
    width: min(560px, 100%);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 20px;
    box-shadow: var(--shadow-lg);
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  h1 {
    margin: 0;
    font-size: 20px;
  }
  .x {
    border: 0;
    background: transparent;
    font-size: 22px;
    color: var(--text-secondary);
  }
  label,
  .seg {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
  }
  input,
  textarea,
  select {
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 10px 12px;
    background: var(--surface);
    color: var(--text);
  }
  .track {
    display: flex;
    background: var(--surface-subtle);
    border-radius: 10px;
    padding: 3px;
    gap: 2px;
  }
  .track button {
    flex: 1;
    border: 0;
    background: transparent;
    height: 32px;
    border-radius: 8px;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
  }
  .track button.active {
    background: var(--surface);
    color: var(--accent);
    box-shadow: var(--shadow);
  }
  .row2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .second {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }
  .pro {
    font-size: 10px;
    font-weight: 700;
    color: var(--accent);
    background: var(--accent-soft);
    border-radius: 6px;
    padding: 2px 6px;
  }
  footer {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-top: 8px;
  }
  .spacer {
    flex: 1;
  }
  .primary,
  .ghost,
  .danger {
    height: 40px;
    padding: 0 14px;
    border-radius: 12px;
    border: 0;
    font-weight: 600;
  }
  .primary {
    background: var(--accent);
    color: #fff;
  }
  .ghost {
    background: transparent;
    color: var(--text-secondary);
  }
  .danger {
    background: var(--danger-soft);
    color: var(--danger-text);
  }
</style>
