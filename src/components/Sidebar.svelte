<script lang="ts">
  import type { EntitlementInfo } from "../lib/api";

  interface Props {
    active: string;
    entitlement: EntitlementInfo;
    onNavigate: (view: string) => void;
    onUpgrade: () => void;
  }

  let { active, entitlement, onNavigate, onUpgrade }: Props = $props();

  const items = [
    { id: "today", label: "Today" },
    { id: "week", label: "Week" },
    { id: "month", label: "Month" },
    { id: "goals", label: "Long-term" },
    { id: "progress", label: "Progress" },
  ];
</script>

<aside class="sidebar">
  <div class="brand">
    <div class="logo" aria-hidden="true">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
        <path
          d="M3.5 8.2 6.2 11l6.3-6.5"
          stroke="#fff"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </div>
    <span class="wordmark">Todly</span>
  </div>

  <nav class="nav">
    {#each items as item}
      <button
        class="nav-item"
        class:active={active === item.id}
        onclick={() => onNavigate(item.id)}
      >
        <span class="dot"></span>
        {item.label}
      </button>
    {/each}
  </nav>

  <div class="footer">
    <button
      class="nav-item"
      class:active={active === "settings"}
      onclick={() => onNavigate("settings")}
    >
      Settings
    </button>
    <div class="plan-card">
      <div class="plan-title">{entitlement.plan === "pro" ? "Pro plan" : "Free plan"}</div>
      <div class="plan-meta">
        {entitlement.active_tasks}
        {#if entitlement.active_cap}
          of {entitlement.active_cap}
        {/if}
        active tasks
      </div>
      {#if entitlement.active_cap}
        <div class="bar">
          <div
            class="fill"
            style="width: {Math.min(100, (entitlement.active_tasks / entitlement.active_cap) * 100)}%"
          ></div>
        </div>
      {/if}
      {#if entitlement.plan !== "pro"}
        <button class="upgrade" onclick={onUpgrade}>Upgrade to Pro</button>
      {/if}
    </div>
  </div>
</aside>

<style>
  .sidebar {
    width: 232px;
    flex-shrink: 0;
    background: var(--surface);
    border-right: 1px solid var(--border);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 4px 12px;
  }
  .logo {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    background: var(--accent);
    display: grid;
    place-items: center;
  }
  .wordmark {
    font-size: 16px;
    font-weight: 600;
  }
  .nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
  }
  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 40px;
    padding: 0 12px;
    border: 0;
    border-radius: 10px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 14px;
    font-weight: 500;
    text-align: left;
  }
  .nav-item:hover {
    background: var(--surface-subtle);
    color: var(--text);
  }
  .nav-item.active {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: currentColor;
    opacity: 0.45;
  }
  .footer {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .plan-card {
    background: var(--bg);
    border-radius: 12px;
    padding: 12px;
  }
  .plan-title {
    font-size: 12px;
    font-weight: 600;
  }
  .plan-meta {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 4px;
  }
  .bar {
    height: 4px;
    background: var(--border);
    border-radius: 99px;
    margin-top: 8px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--accent);
  }
  .upgrade {
    margin-top: 8px;
    border: 0;
    background: transparent;
    color: var(--accent);
    font-size: 12px;
    font-weight: 600;
    padding: 0;
  }
</style>
