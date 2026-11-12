<script lang="ts">
  import "../../styles/tokens.css";
  import { createTask, hideQuickAdd, type Horizon } from "../../lib/api";
  import { applyTheme } from "../../lib/theme";

  let title = $state("");
  let horizon = $state<Horizon>("daily");
  let duePreset = $state<string>("today-17");
  let error = $state("");

  applyTheme("system");

  function dueFromPreset(): string | null {
    const now = new Date();
    const pad = (n: number) => String(n).padStart(2, "0");
    const ymd = (d: Date) =>
      `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
    if (duePreset === "1h") {
      const d = new Date(now.getTime() + 60 * 60 * 1000);
      return `${ymd(d)}T${pad(d.getHours())}:${pad(d.getMinutes())}:00`;
    }
    if (duePreset === "today-17") return `${ymd(now)}T17:00:00`;
    if (duePreset === "tomorrow-9") {
      const d = new Date(now);
      d.setDate(d.getDate() + 1);
      return `${ymd(d)}T09:00:00`;
    }
    return null;
  }

  async function save() {
    const t = title.trim();
    if (!t) return;
    error = "";
    try {
      await createTask({
        title: t,
        horizon,
        due_at: dueFromPreset(),
        remind_minutes_before: 10,
      });
      title = "";
      await hideQuickAdd();
    } catch (e) {
      error = String(e);
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") void hideQuickAdd();
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void save();
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="wrap">
  <div class="dialog">
    <input
      class="title"
      bind:value={title}
      placeholder="What needs doing?"
      autofocus
    />
    <div class="row">
      <span class="label">Horizon</span>
      <div class="seg">
        {#each [
          ["daily", "Daily"],
          ["weekly", "Weekly"],
          ["monthly", "Monthly"],
          ["goal", "Long-term"],
        ] as [id, label]}
          <button class:active={horizon === id} onclick={() => (horizon = id as Horizon)}
            >{label}</button
          >
        {/each}
      </div>
    </div>
    <div class="row">
      <span class="label">Due</span>
      <div class="chips">
        <button class:active={duePreset === "1h"} onclick={() => (duePreset = "1h")}
          >In 1 hour</button
        >
        <button class:active={duePreset === "today-17"} onclick={() => (duePreset = "today-17")}
          >Today 5:00 PM</button
        >
        <button class:active={duePreset === "tomorrow-9"} onclick={() => (duePreset = "tomorrow-9")}
          >Tomorrow 9:00 AM</button
        >
        <button class:active={duePreset === "none"} onclick={() => (duePreset = "none")}
          >No due</button
        >
      </div>
    </div>
    <div class="remind">Remind me 10 min before</div>
    {#if error}
      <p class="error">{error}</p>
    {/if}
    <footer>
      <span class="hint">Enter to save · Esc to cancel</span>
      <button class="primary" onclick={save}>Add task</button>
    </footer>
  </div>
</div>

<style>
  .wrap {
    min-height: 100vh;
    display: grid;
    place-items: center;
    background: rgba(16, 24, 40, 0.28);
    font-family: var(--font);
  }
  .dialog {
    width: 560px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 20px;
    box-shadow: var(--shadow-lg);
    padding: 20px;
  }
  .title {
    width: 100%;
    border: 0;
    outline: none;
    font-size: 20px;
    font-weight: 500;
    color: var(--text);
    background: transparent;
    caret-color: var(--accent);
  }
  .row {
    margin-top: 16px;
  }
  .label {
    display: block;
    font-size: 12px;
    color: var(--text-tertiary);
    margin-bottom: 6px;
  }
  .seg,
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .seg {
    background: var(--surface-subtle);
    border-radius: 10px;
    padding: 3px;
  }
  .seg button,
  .chips button {
    border: 0;
    background: transparent;
    height: 32px;
    padding: 0 10px;
    border-radius: 8px;
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
  }
  .seg button.active {
    background: var(--surface);
    color: var(--accent);
    box-shadow: var(--shadow);
  }
  .chips button {
    height: 36px;
    border-radius: 10px;
    background: var(--surface-subtle);
  }
  .chips button.active {
    background: var(--accent-soft);
    color: var(--accent);
    box-shadow: inset 0 0 0 1px var(--accent);
  }
  .remind {
    margin-top: 12px;
    display: inline-flex;
    align-items: center;
    font-size: 12px;
    font-weight: 600;
    color: var(--warning-text);
    background: var(--warning-soft);
    border-radius: 8px;
    padding: 4px 8px;
  }
  footer {
    margin-top: 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .hint {
    font-size: 12px;
    color: var(--text-tertiary);
  }
  .primary {
    height: 40px;
    padding: 0 16px;
    border: 0;
    border-radius: 12px;
    background: var(--accent);
    color: #fff;
    font-weight: 600;
  }
  .error {
    color: var(--danger);
    font-size: 13px;
  }
</style>
