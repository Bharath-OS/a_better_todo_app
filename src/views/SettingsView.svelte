<script lang="ts">
  interface Props {
    settings: Record<string, string>;
    onChange: (key: string, value: string) => void;
  }

  let { settings, onChange }: Props = $props();
</script>

<header class="header">
  <h1>Settings</h1>
  <p class="sub">Overlay and reminders</p>
</header>

<section class="card">
  <h2>Overlay</h2>
  <label>
    <span>Position</span>
    <select
      value={settings.overlay_position ?? "top-center"}
      onchange={(e) => onChange("overlay_position", e.currentTarget.value)}
    >
      <option value="top-center">Top centre</option>
      <option value="top-left">Top left</option>
      <option value="top-right">Top right</option>
    </select>
  </label>
  <label>
    <span>Style</span>
    <select
      value={settings.overlay_style ?? "next-task"}
      onchange={(e) => onChange("overlay_style", e.currentTarget.value)}
    >
      <option value="next-task">Next task</option>
      <option value="count-only">Count only</option>
    </select>
  </label>
  <label>
    <span>Dwell time (ms)</span>
    <input
      type="number"
      min="1000"
      max="3000"
      step="500"
      value={settings.dwell_ms ?? "2000"}
      onchange={(e) => onChange("dwell_ms", e.currentTarget.value)}
    />
  </label>
  <label class="row">
    <span>Show overlay</span>
    <input
      type="checkbox"
      checked={(settings.overlay_visible ?? "true") === "true"}
      onchange={(e) => onChange("overlay_visible", e.currentTarget.checked ? "true" : "false")}
    />
  </label>
</section>

<section class="card">
  <h2>Reminders</h2>
  <label>
    <span>Default lead time (minutes)</span>
    <select
      value={settings.remind_minutes_before ?? "10"}
      onchange={(e) => onChange("remind_minutes_before", e.currentTarget.value)}
    >
      {#each [5, 10, 15, 30, 60] as n}
        <option value={String(n)}>{n}</option>
      {/each}
    </select>
  </label>
  <label>
    <span>Day starts at</span>
    <input
      type="time"
      value={settings.day_starts_at ?? "00:00"}
      onchange={(e) => onChange("day_starts_at", e.currentTarget.value)}
    />
  </label>
</section>

<section class="card">
  <h2>Appearance</h2>
  <label>
    <span>Theme</span>
    <select
      value={settings.theme ?? "system"}
      onchange={(e) => onChange("theme", e.currentTarget.value)}
    >
      <option value="system">System</option>
      <option value="light">Light</option>
      <option value="dark">Dark</option>
    </select>
  </label>
</section>

<style>
  .header {
    margin-bottom: 24px;
  }
  h1 {
    margin: 0;
    font-size: 28px;
    font-weight: 600;
  }
  .sub {
    color: var(--text-secondary);
    margin: 6px 0 0;
  }
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: 20px;
    margin-bottom: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    max-width: 520px;
  }
  h2 {
    margin: 0;
    font-size: 16px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    color: var(--text-secondary);
  }
  label.row {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }
  select,
  input[type="number"],
  input[type="time"] {
    height: 40px;
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 0 12px;
    background: var(--bg);
    color: var(--text);
  }
</style>
