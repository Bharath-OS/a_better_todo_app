import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Horizon = "daily" | "weekly" | "monthly" | "goal";

export interface Task {
  id: string;
  title: string;
  horizon: Horizon;
  due_at: string | null;
  remind_minutes_before: number;
  completed_at: string | null;
  created_at: string;
  notes: string | null;
  progress: number | null;
  second_remind_minutes_before: number | null;
}

export interface DayStats {
  date: string;
  total: number;
  completed: number;
}

export interface EntitlementInfo {
  plan: string;
  active_tasks: number;
  active_cap: number | null;
}

const inTauri = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** Dev fallback so Vite preview works without the Rust host. */
const mockTasks: Task[] = [
  {
    id: "mock-1",
    title: "Send invoice to Acme Studio",
    horizon: "daily",
    due_at: null,
    remind_minutes_before: 10,
    completed_at: null,
    created_at: "2026-11-12T09:00:00",
    notes: null,
    progress: null,
    second_remind_minutes_before: null,
  },
  {
    id: "mock-2",
    title: "Deep work block: 2 hours",
    horizon: "daily",
    due_at: null,
    remind_minutes_before: 10,
    completed_at: null,
    created_at: "2026-11-12T09:05:00",
    notes: null,
    progress: null,
    second_remind_minutes_before: null,
  },
];

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!inTauri()) {
    throw new Error(`not in tauri: ${cmd}`);
  }
  return invoke<T>(cmd, args);
}

export async function getTodayTasks(): Promise<Task[]> {
  try {
    return await call<Task[]>("get_today_tasks");
  } catch {
    return mockTasks;
  }
}

export async function getTasks(horizon?: Horizon): Promise<Task[]> {
  try {
    return await call<Task[]>("get_tasks", { horizon: horizon ?? null });
  } catch {
    return mockTasks.filter((t) => !horizon || t.horizon === horizon);
  }
}

export async function createTask(input: {
  title: string;
  horizon: Horizon;
  due_at?: string | null;
  notes?: string | null;
  remind_minutes_before?: number;
}): Promise<Task> {
  try {
    return await call<Task>("create_task", {
      title: input.title,
      horizon: input.horizon,
      due_at: input.due_at ?? null,
      notes: input.notes ?? null,
      remind_minutes_before: input.remind_minutes_before ?? 10,
    });
  } catch (e) {
    if (!inTauri()) {
      const t: Task = {
        id: crypto.randomUUID(),
        title: input.title,
        horizon: input.horizon,
        due_at: input.due_at ?? null,
        remind_minutes_before: input.remind_minutes_before ?? 10,
        completed_at: null,
        created_at: new Date().toISOString().slice(0, 19),
        notes: input.notes ?? null,
        progress: null,
        second_remind_minutes_before: null,
      };
      mockTasks.unshift(t);
      return t;
    }
    throw e;
  }
}

export async function toggleTask(id: string): Promise<Task> {
  try {
    return await call<Task>("toggle_task", { id });
  } catch {
    const t = mockTasks.find((x) => x.id === id)!;
    t.completed_at = t.completed_at ? null : new Date().toISOString().slice(0, 19);
    return t;
  }
}

export async function deleteTask(id: string): Promise<void> {
  try {
    await call("delete_task", { id });
  } catch {
    const i = mockTasks.findIndex((x) => x.id === id);
    if (i >= 0) mockTasks.splice(i, 1);
  }
}

export async function getSettings(): Promise<Record<string, string>> {
  try {
    return await call("get_settings");
  } catch {
    return {
      theme: "light",
      entitlement: "free",
      dwell_ms: "2000",
      overlay_position: "top-center",
      day_starts_at: "00:00",
    };
  }
}

export async function updateSetting(key: string, value: string): Promise<void> {
  try {
    await call("update_setting", { key, value });
  } catch {
    /* preview */
  }
}

export async function getDayStats(days = 7): Promise<DayStats[]> {
  try {
    return await call("get_day_stats", { days });
  } catch {
    return [];
  }
}

export async function getEntitlement(): Promise<EntitlementInfo> {
  try {
    return await call("get_entitlement");
  } catch {
    return { plan: "free", active_tasks: mockTasks.length, active_cap: 20 };
  }
}

export async function unlockProDemo(): Promise<void> {
  await call("unlock_pro_demo");
}

export async function setOverlayExpanded(expanded: boolean): Promise<void> {
  try {
    await call("set_overlay_expanded", { expanded });
  } catch {
    /* preview */
  }
}

export async function showMainWindow(): Promise<void> {
  try {
    await call("show_main_window");
  } catch {
    /* preview */
  }
}

export async function hideQuickAdd(): Promise<void> {
  try {
    await call("hide_quick_add");
  } catch {
    /* preview */
  }
}

export async function onTasksChanged(cb: () => void): Promise<UnlistenFn> {
  if (!inTauri()) return () => {};
  return listen("tasks-changed", () => cb());
}

export async function onSettingsChanged(cb: () => void): Promise<UnlistenFn> {
  if (!inTauri()) return () => {};
  return listen("settings-changed", () => cb());
}

export async function onOverlayArm(
  cb: (p: { armed: boolean; dwelling: boolean; progress: number }) => void,
): Promise<UnlistenFn> {
  if (!inTauri()) return () => {};
  return listen("overlay-arm", (e) => cb(e.payload as never));
}

export async function onReminderDue(cb: (t: Task) => void): Promise<UnlistenFn> {
  if (!inTauri()) return () => {};
  return listen("reminder-due", (e) => cb(e.payload as Task));
}
