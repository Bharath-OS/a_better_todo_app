<script lang="ts">
  interface Props {
    open: boolean;
    onClose: () => void;
    onUnlock: () => void;
  }

  let { open, onClose, onUnlock }: Props = $props();
  let snackbar = $state("");

  function payDemo() {
    // Offline demo only — no payment APIs / no network.
    snackbar = "You paid $12 (demo). No charge was made.";
    void onUnlock();
    setTimeout(() => {
      snackbar = "";
      onClose();
    }, 2200);
  }
</script>

{#if open}
  <div class="backdrop" role="presentation" onclick={onClose}>
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.key === "Escape" && onClose()}
    >
      <header>
        <div>
          <h1>Todly Pro</h1>
          <p>Demo mode — no payment is taken.</p>
        </div>
        <button class="x" onclick={onClose} aria-label="Close">×</button>
      </header>
      <div class="plans">
        <div class="plan">
          <h2>Free</h2>
          <div class="price">$0</div>
          <ul>
            <li>Island overlay and reminders</li>
            <li>Daily and Weekly tasks</li>
            <li>Up to 20 active tasks</li>
            <li>7-day progress</li>
            <li>Quick-add hotkey</li>
          </ul>
          <button class="ghost" disabled>Current plan</button>
        </div>
        <div class="plan pro">
          <span class="badge">Recommended</span>
          <h2>Pro</h2>
          <div class="price">$12 one-time</div>
          <ul>
            <li>Everything in Free</li>
            <li>Monthly and Long-term goals</li>
            <li>Unlimited tasks</li>
            <li>Second reminders</li>
            <li>Deep progress + export</li>
          </ul>
          <button class="primary" onclick={payDemo}>Pay $12 (demo)</button>
        </div>
      </div>
    </div>
  </div>
{/if}

{#if snackbar}
  <div class="snack" role="status">{snackbar}</div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(16, 24, 40, 0.35);
    display: grid;
    place-items: center;
    z-index: 50;
    padding: 24px;
  }
  .dialog {
    width: min(880px, 100%);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 24px;
    box-shadow: var(--shadow-lg);
    padding: 32px;
  }
  header {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 24px;
  }
  h1 {
    margin: 0;
    font-size: 28px;
  }
  header p {
    margin: 6px 0 0;
    color: var(--text-secondary);
  }
  .x {
    border: 0;
    background: transparent;
    font-size: 24px;
    color: var(--text-secondary);
  }
  .plans {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }
  .plan {
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 20px;
    position: relative;
  }
  .plan.pro {
    border: 2px solid var(--accent);
    background: color-mix(in srgb, var(--accent-soft) 55%, var(--surface));
  }
  .badge {
    position: absolute;
    top: 12px;
    right: 12px;
    background: var(--accent);
    color: #fff;
    font-size: 11px;
    font-weight: 600;
    border-radius: 8px;
    padding: 4px 8px;
  }
  .price {
    font-size: 32px;
    font-weight: 600;
    margin: 8px 0 12px;
  }
  ul {
    margin: 0 0 20px;
    padding-left: 18px;
    color: var(--text-secondary);
    font-size: 14px;
  }
  .primary,
  .ghost {
    width: 100%;
    height: 40px;
    border-radius: 12px;
    border: 0;
    font-weight: 600;
  }
  .primary {
    background: var(--accent);
    color: #fff;
  }
  .ghost {
    background: var(--surface-subtle);
    color: var(--text-tertiary);
  }
  @media (max-width: 720px) {
    .plans {
      grid-template-columns: 1fr;
    }
  }
  .snack {
    position: fixed;
    left: 50%;
    bottom: 28px;
    transform: translateX(-50%);
    z-index: 80;
    background: #101828;
    color: #fff;
    padding: 12px 18px;
    border-radius: 12px;
    font-size: 14px;
    font-weight: 500;
    box-shadow: var(--shadow-lg);
  }
</style>
