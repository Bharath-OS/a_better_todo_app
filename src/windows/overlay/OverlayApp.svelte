<script lang="ts">
  import "../../styles/tokens.css";
  import confetti from "canvas-confetti";
  import {
    getSettings,
    getTodayTasks,
    onOverlayArm,
    onReminderDue,
    onTasksChanged,
    setOverlayExpanded,
    showMainWindow,
    toggleTask,
    type Task,
  } from "../../lib/api";
  import { applyTheme } from "../../lib/theme";

  type Mode =
    | "sliver"
    | "glimpse"
    | "count"
    | "peek"
    | "armed"
    | "expanded"
    | "reminder"
    | "overdue"
    | "allclear";

  let tasks = $state<Task[]>([]);
  let mode = $state<Mode>("glimpse");
  let armProgress = $state(0);
  let style = $state("next-task");
  let reminderTask = $state<Task | null>(null);
  let idleTimer: ReturnType<typeof setTimeout> | null = null;
  let allClearTimer: ReturnType<typeof setTimeout> | null = null;

  const open = $derived(tasks.filter((t) => !t.completed_at));
  const done = $derived(tasks.filter((t) => !!t.completed_at));
  const next = $derived(open[0] ?? null);
  const ratio = $derived(tasks.length ? done.length / tasks.length : 0);

  async function refresh() {
    tasks = await getTodayTasks();
    const settings = await getSettings();
    applyTheme(settings.theme ?? "system");
    style = settings.overlay_style ?? "next-task";
    if (mode === "expanded") return;
    if (open.length === 0 && done.length > 0) {
      enterAllClear();
    } else if (style === "count-only") {
      mode = "count";
    } else if (mode === "sliver" || mode === "allclear" || mode === "reminder") {
      /* keep */
    } else {
      mode = "glimpse";
    }
    scheduleIdle();
  }

  function scheduleIdle() {
    if (idleTimer) clearTimeout(idleTimer);
    if (mode === "expanded" || mode === "reminder" || mode === "armed" || mode === "peek") return;
    idleTimer = setTimeout(() => {
      if (open.length === 0) return;
      mode = "sliver";
    }, 30_000);
  }

  function enterAllClear() {
    mode = "allclear";
    if (allClearTimer) clearTimeout(allClearTimer);
    allClearTimer = setTimeout(() => {
      mode = "sliver";
    }, 4000);
  }

  $effect(() => {
    void refresh();
    let unsubs: Array<() => void> = [];
    void (async () => {
      unsubs.push(await onTasksChanged(refresh));
      unsubs.push(
        await onOverlayArm((p) => {
          if (mode === "expanded" || mode === "reminder") return;
          armProgress = p.progress;
          if (p.armed) mode = "armed";
          else if (p.dwelling) mode = "peek";
          else if (style === "count-only") mode = "count";
          else mode = "glimpse";
          scheduleIdle();
        }),
      );
      unsubs.push(
        await onReminderDue((t) => {
          reminderTask = t;
          mode = "reminder";
        }),
      );
    })();
    return () => unsubs.forEach((u) => u());
  });

  async function expand() {
    if (mode !== "armed" && mode !== "reminder") return;
    mode = "expanded";
    await setOverlayExpanded(true);
  }

  async function collapse() {
    mode = style === "count-only" ? "count" : "glimpse";
    armProgress = 0;
    await setOverlayExpanded(false);
    scheduleIdle();
  }

  async function complete(id: string) {
    const before = open.length;
    await toggleTask(id);
    await refresh();
    if (before === 1) {
      confetti({
        particleCount: 60,
        spread: 55,
        origin: { y: 0.25 },
        disableForReducedMotion: true,
      });
      await collapse();
      enterAllClear();
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape" && mode === "expanded") void collapse();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="root" class:expanded={mode === "expanded"}>
  {#if mode === "sliver"}
    <button class="sliver" aria-label="Todly idle" onclick={() => (mode = "glimpse")}></button>
  {:else if mode === "expanded"}
    <div class="panel">
      <div class="panel-top">
        <div class="seg">
          <button class="on">Today</button>
          <button>Week</button>
          <button>Month</button>
        </div>
        <button class="icon" onclick={() => showMainWindow()} title="Open app">⚙</button>
        <button class="icon" onclick={collapse} title="Collapse">⌃</button>
      </div>
      <div class="summary">
        Today — {open.length} left, {done.length} done
        <div class="bar"><div class="fill" style="width: {ratio * 100}%"></div></div>
      </div>
      <div class="rows">
        {#each open.slice(0, 6) as task (task.id)}
          <div class="row">
            <button class="check" onclick={() => complete(task.id)} aria-label="Complete"></button>
            <span class="title">{task.title}</span>
          </div>
        {/each}
      </div>
      <button class="open-app" onclick={() => showMainWindow()}>Open Todly</button>
    </div>
  {:else if mode === "reminder" && reminderTask}
    <div class="pill reminder" role="status">
      <div class="bell">🔔</div>
      <div class="text">
        <div class="t1">{reminderTask.title}</div>
        <div class="t2">Due soon · {reminderTask.remind_minutes_before} min lead</div>
      </div>
      <button class="done" onclick={() => complete(reminderTask!.id)}>Done</button>
      <button class="snooze" onclick={() => (mode = "glimpse")}>Snooze 10m</button>
    </div>
  {:else if mode === "allclear"}
    <div class="pill allclear">
      <span class="ok">✓</span>
      <span>All clear</span>
    </div>
  {:else if mode === "count"}
    <button
      class="pill count"
      class:peek={mode === "peek"}
      class:armed={false}
      onclick={expand}
      style="--dwell: {armProgress}"
    >
      <span class="ring" style="--p: {ratio}"></span>
      <span>{open.length} left</span>
    </button>
  {:else}
    <button
      class="pill"
      class:peek={mode === "peek"}
      class:armed={mode === "armed"}
      onclick={expand}
      style="--dwell: {armProgress}"
      aria-label="{done.length} of {tasks.length} done, next: {next?.title ?? 'none'}"
    >
      <span class="ring" style="--p: {ratio}"></span>
      <span class="frac">{done.length}/{tasks.length || 0}</span>
      <span class="divider"></span>
      <span class="next">{next?.title ?? "All clear"}</span>
      {#if next}
        <span class="chip">today</span>
      {/if}
      {#if mode === "peek"}
        <span class="tip">Hold to open</span>
      {:else if mode === "armed"}
        <span class="tip">Click to open</span>
      {/if}
    </button>
  {/if}
</div>

<style>
  .root {
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 0;
    background: transparent;
    font-family: var(--font);
  }
  .sliver {
    width: 72px;
    height: 5px;
    margin-top: 8px;
    border: 0;
    border-radius: 99px;
    background: color-mix(in srgb, var(--text) 35%, transparent);
  }
  .pill {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 38px;
    max-width: 340px;
    margin-top: 4px;
    padding: 0 12px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--island);
    box-shadow: var(--shadow);
    color: var(--text);
    font-size: 13px;
    font-weight: 500;
  }
  .pill.peek {
    transform: scale(1.03);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) calc(var(--dwell) * 100%), transparent);
  }
  .pill.armed {
    box-shadow: 0 0 0 2px var(--accent), var(--shadow);
  }
  .ring {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: conic-gradient(var(--accent) calc(var(--p) * 360deg), var(--border) 0);
    mask: radial-gradient(circle 7px, transparent 98%, #000 100%);
    -webkit-mask: radial-gradient(circle 7px, transparent 98%, #000 100%);
    flex-shrink: 0;
  }
  .frac {
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .divider {
    width: 1px;
    height: 14px;
    background: var(--border);
  }
  .next {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 160px;
  }
  .chip {
    margin-left: auto;
    font-size: 11px;
    font-weight: 600;
    background: var(--warning-soft);
    color: var(--warning-text);
    border-radius: 10px;
    padding: 2px 8px;
  }
  .tip {
    position: absolute;
    top: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
    background: var(--text);
    color: var(--surface);
    font-size: 11px;
    font-weight: 500;
    border-radius: 8px;
    padding: 4px 8px;
    white-space: nowrap;
  }
  .panel {
    width: 380px;
    margin-top: 4px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 20px;
    box-shadow: var(--shadow-lg);
    padding: 16px;
  }
  .panel-top {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .seg {
    flex: 1;
    display: flex;
    background: var(--surface-subtle);
    border-radius: 10px;
    padding: 3px;
  }
  .seg button {
    flex: 1;
    height: 28px;
    border: 0;
    background: transparent;
    border-radius: 8px;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
  }
  .seg .on {
    background: var(--surface);
    color: var(--text);
    box-shadow: var(--shadow);
  }
  .icon {
    border: 0;
    background: transparent;
    color: var(--text-secondary);
  }
  .summary {
    margin: 12px 0;
    font-size: 12px;
    color: var(--text-secondary);
  }
  .bar {
    height: 4px;
    background: var(--border);
    border-radius: 99px;
    margin-top: 6px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--accent);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 44px;
  }
  .check {
    width: 18px;
    height: 18px;
    border-radius: 6px;
    border: 1.5px solid #c5ccd8;
    background: transparent;
  }
  .title {
    font-size: 14px;
    font-weight: 500;
  }
  .open-app {
    margin-top: 8px;
    width: 100%;
    height: 36px;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--surface);
    color: var(--text-secondary);
    font-weight: 500;
  }
  .reminder {
    max-width: 420px;
    min-height: 52px;
    background: #fff8e8;
    border-color: #f5d58a;
    gap: 10px;
  }
  .bell {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--warning-soft);
    display: grid;
    place-items: center;
  }
  .t1 {
    font-size: 13px;
    font-weight: 600;
  }
  .t2 {
    font-size: 11px;
    color: var(--warning-text);
  }
  .done {
    height: 30px;
    border: 0;
    border-radius: 15px;
    padding: 0 12px;
    background: var(--success);
    color: #fff;
    font-size: 12px;
    font-weight: 600;
  }
  .snooze {
    height: 30px;
    border: 1px solid var(--border);
    border-radius: 15px;
    padding: 0 10px;
    background: #fff;
    font-size: 12px;
  }
  .allclear {
    gap: 10px;
  }
  .ok {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--success);
    color: #fff;
    display: grid;
    place-items: center;
    font-size: 12px;
  }
</style>
