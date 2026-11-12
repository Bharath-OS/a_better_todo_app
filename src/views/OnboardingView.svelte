<script lang="ts">
  interface Props {
    onContinue: () => void;
    onSkip: () => void;
  }

  let { onContinue, onSkip }: Props = $props();
</script>

<div class="onboard">
  <div class="left">
    <div class="brand">
      <div class="logo"></div>
      <strong>Todly</strong>
    </div>
    <div class="step">Step 1 of 3</div>
    <h1>Your tasks, one glance away</h1>
    <p>
      A small island sits at the top of your screen. Rest your pointer on it for two seconds, then
      click to open your list.
    </p>
    <div class="actions">
      <button class="primary" onclick={onContinue}>Continue</button>
      <button class="ghost" onclick={onSkip}>Skip</button>
    </div>
    <div class="dots" aria-hidden="true">
      <span class="on"></span><span></span><span></span>
    </div>
  </div>
  <div class="right">
    <div class="demo">
      <div class="pill">
        <span class="ring">3/8</span>
        <span class="title">Send invoice to Acme Studio</span>
        <span class="chip">in 18 min</span>
      </div>
      <p class="caption">Hover for 2 seconds, then click</p>
    </div>
  </div>
</div>

<style>
  .onboard {
    min-height: 100vh;
    display: grid;
    grid-template-columns: 1fr 1fr;
    background: var(--surface);
    padding: 48px;
    gap: 32px;
  }
  .left {
    display: flex;
    flex-direction: column;
    justify-content: center;
    max-width: 400px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 32px;
  }
  .logo {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    background: var(--accent);
  }
  .step {
    color: var(--accent);
    font-size: 12px;
    font-weight: 600;
  }
  h1 {
    margin: 8px 0;
    font-size: 32px;
    line-height: 38px;
  }
  p {
    color: var(--text-secondary);
    font-size: 15px;
    line-height: 24px;
  }
  .actions {
    display: flex;
    gap: 12px;
    margin-top: 32px;
  }
  .primary {
    height: 44px;
    padding: 0 18px;
    border: 0;
    border-radius: 12px;
    background: var(--accent);
    color: #fff;
    font-weight: 600;
  }
  .ghost {
    height: 44px;
    border: 0;
    background: transparent;
    color: var(--text-secondary);
  }
  .dots {
    display: flex;
    gap: 6px;
    margin-top: 40px;
  }
  .dots span {
    width: 8px;
    height: 8px;
    border-radius: 99px;
    background: var(--border);
  }
  .dots .on {
    width: 20px;
    background: var(--accent);
  }
  .right {
    background: var(--surface-subtle);
    border-radius: 24px;
    display: grid;
    place-items: center;
  }
  .demo {
    text-align: center;
  }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 8px 12px;
    box-shadow: var(--shadow);
    font-size: 13px;
  }
  .ring {
    color: var(--text-secondary);
    font-weight: 600;
    font-size: 12px;
  }
  .chip {
    background: var(--warning-soft);
    color: var(--warning-text);
    border-radius: 8px;
    padding: 2px 8px;
    font-size: 11px;
    font-weight: 600;
  }
  .caption {
    margin-top: 16px;
    font-size: 13px;
    color: var(--text-secondary);
  }
  @media (max-width: 800px) {
    .onboard {
      grid-template-columns: 1fr;
    }
  }
</style>
