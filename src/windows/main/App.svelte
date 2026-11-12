<script lang="ts">
  import "../../styles/tokens.css";
  import Sidebar from "../../components/Sidebar.svelte";
  import TodayView from "../../views/TodayView.svelte";
  import StubHorizonView from "../../views/StubHorizonView.svelte";
  import ProgressView from "../../views/ProgressView.svelte";
  import SettingsView from "../../views/SettingsView.svelte";
  import UpgradeDialog from "../../views/UpgradeDialog.svelte";
  import TaskDetailsDialog from "../../views/TaskDetailsDialog.svelte";
  import OnboardingView from "../../views/OnboardingView.svelte";
  import {
    getDayStats,
    getEntitlement,
    getSettings,
    getTasks,
    getTodayTasks,
    onSettingsChanged,
    onTasksChanged,
    toggleTask,
    unlockProDemo,
    updateSetting,
    type DayStats,
    type EntitlementInfo,
    type Task,
  } from "../../lib/api";
  import { applyTheme } from "../../lib/theme";

  let view = $state("today");
  let tasks = $state<Task[]>([]);
  let weekTasks = $state<Task[]>([]);
  let monthTasks = $state<Task[]>([]);
  let goalTasks = $state<Task[]>([]);
  let entitlement = $state<EntitlementInfo>({
    plan: "free",
    active_tasks: 0,
    active_cap: 20,
  });
  let settings = $state<Record<string, string>>({});
  let stats = $state<DayStats[]>([]);
  let showUpgrade = $state(false);
  let editing = $state<Task | null>(null);
  let showOnboarding = $state(false);

  async function refresh() {
    const [today, weekly, monthly, goals, ent, set, dayStats] = await Promise.all([
      getTodayTasks(),
      getTasks("weekly"),
      getTasks("monthly"),
      getTasks("goal"),
      getEntitlement(),
      getSettings(),
      getDayStats(7),
    ]);
    tasks = today;
    weekTasks = weekly;
    monthTasks = monthly;
    goalTasks = goals;
    entitlement = ent;
    settings = set;
    stats = dayStats;
    applyTheme(set.theme ?? "system");
    showOnboarding = (set.onboarding_done ?? "false") !== "true";
  }

  $effect(() => {
    void refresh();
    let unsubs: Array<() => void> = [];
    void (async () => {
      unsubs.push(await onTasksChanged(refresh));
      unsubs.push(await onSettingsChanged(refresh));
    })();
    return () => unsubs.forEach((u) => u());
  });

  async function handleToggle(id: string) {
    await toggleTask(id);
    await refresh();
  }

  async function handleSetting(key: string, value: string) {
    await updateSetting(key, value);
    if (key === "theme") applyTheme(value);
    await refresh();
  }

  async function handleUnlock() {
    try {
      await unlockProDemo();
    } catch {
      await updateSetting("entitlement", "pro");
    }
    showUpgrade = false;
    await refresh();
  }

  async function finishOnboarding() {
    await updateSetting("onboarding_done", "true");
    showOnboarding = false;
  }
</script>

{#if showOnboarding}
  <OnboardingView onContinue={finishOnboarding} onSkip={finishOnboarding} />
{:else}
  <div class="shell">
    <Sidebar
      active={view}
      {entitlement}
      onNavigate={(v) => (view = v)}
      onUpgrade={() => (showUpgrade = true)}
    />
    <main class="content">
      {#if view === "today"}
        <TodayView
          {tasks}
          onToggle={handleToggle}
          onOpen={(t) => (editing = t)}
          onChanged={refresh}
        />
      {:else if view === "week"}
        <StubHorizonView
          title="Week"
          subtitle="Tasks with a weekly horizon"
          tasks={weekTasks}
          onToggle={handleToggle}
          onOpen={(t) => (editing = t)}
        />
      {:else if view === "month"}
        <StubHorizonView
          title="Month"
          subtitle="Calendar grid ships next; list works now"
          tasks={monthTasks}
          onToggle={handleToggle}
          onOpen={(t) => (editing = t)}
          proLocked={entitlement.plan !== "pro"}
          onUpgrade={() => (showUpgrade = true)}
        />
      {:else if view === "goals"}
        <StubHorizonView
          title="Long-term goals"
          subtitle="Quarterly and yearly goals with manual progress"
          tasks={goalTasks}
          onToggle={handleToggle}
          onOpen={(t) => (editing = t)}
          proLocked={entitlement.plan !== "pro"}
          onUpgrade={() => (showUpgrade = true)}
        />
      {:else if view === "progress"}
        <ProgressView {stats} isPro={entitlement.plan === "pro"} />
      {:else if view === "settings"}
        <SettingsView {settings} onChange={handleSetting} />
      {/if}
    </main>
  </div>
{/if}

<UpgradeDialog
  open={showUpgrade}
  onClose={() => (showUpgrade = false)}
  onUnlock={handleUnlock}
/>
<TaskDetailsDialog
  task={editing}
  isPro={entitlement.plan === "pro"}
  onClose={() => (editing = null)}
  onChanged={refresh}
/>

<style>
  :global(body) {
    margin: 0;
  }
  .shell {
    display: flex;
    min-height: 100vh;
    background: var(--bg);
  }
  .content {
    flex: 1;
    padding: 28px 32px 32px;
    overflow: auto;
  }
</style>
