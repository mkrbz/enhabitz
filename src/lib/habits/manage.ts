import { habits } from "./state.svelte";
import { stopTimer, persistTimerProgress } from "./timer";
import { stopCounterTimer, persistCounterTimerProgress } from "./counter-timer";
import { dbLoadHabits, dbAddHabit, dbUpdateHabit, dbDeleteHabit } from "$lib/db";
import type { Habit, TimerHabit, CounterTimerHabit } from "$lib/types";
import { emit } from "@tauri-apps/api/event";
import { syncReminders } from "$lib/notifications";

let loadedDate = "";

// Local date, matching the SQL side's date('now', 'localtime') — toISOString()
// is UTC and would disagree with it near a day boundary.
function todayStr(): string {
    const d = new Date();
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${y}-${m}-${day}`;
}

let ignoreNextChangeEvent = false;

export function emitHabitsChanged(): void {
    // Every caller of this already updated `habits` locally before emitting —
    // the event exists to tell *other* windows (desktop main+widget) to catch
    // up, not to make this window redundantly re-fetch what it just wrote.
    // Skipping that self-echo also removes an async reload that could race
    // with (and clobber) a state change made after the emit but before the
    // round-trip resolves — e.g. immediately resuming a timer right after
    // pausing it.
    ignoreNextChangeEvent = true;
    emit("habits-changed").catch(() => {});
}

/** Consumes the flag set by emitHabitsChanged(): true if this event is our
 * own echo, so the caller should skip reloading. */
export function isOwnChangeEvent(): boolean {
    if (!ignoreNextChangeEvent) return false;
    ignoreNextChangeEvent = false;
    return true;
}

// ─── Init & reset ─────────────────────────────────────────────────────────────

export async function initHabits(): Promise<void> {
    const loaded = await dbLoadHabits();
    habits.splice(0, habits.length, ...loaded);
    loadedDate = todayStr();
    syncReminders();
}

// Reload from DB while preserving running timer state in this window.
export async function refreshHabits(): Promise<void> {
    const runningState = new Map<string, number>();
    for (const h of habits) {
        if ((h.type === "timer" || h.type === "counter-timer") && h.isRunning && h.startedAt !== undefined) {
            runningState.set(h.id, h.startedAt);
        }
    }
    const loaded = await dbLoadHabits();
    habits.splice(0, habits.length, ...loaded);
    loadedDate = todayStr();
    for (const h of habits) {
        if ((h.type === "timer" || h.type === "counter-timer") && runningState.has(h.id)) {
            h.isRunning = true;
            (h as TimerHabit | CounterTimerHabit).startedAt = runningState.get(h.id);
        }
    }
    syncReminders();
}

export async function checkAndResetIfNewDay(): Promise<void> {
    if (loadedDate === todayStr()) return;
    // Stop any running timers so they save their progress before the day rolls over
    for (const h of habits) {
        if (h.type === "timer" && h.isRunning) stopTimer(h.id);
        if (h.type === "counter-timer" && h.isRunning) stopCounterTimer(h.id);
    }
    await initHabits();
}

/** Flushes any running timer/counter-timer's live elapsed progress to disk
 * without stopping it. Call this on visibilitychange (app backgrounding) —
 * Android can kill a cached/backgrounded process at any time to reclaim
 * memory, and a running timer's progress otherwise only lives in memory
 * until an explicit pause, so a kill mid-run would silently lose it. */
export function persistRunningProgress(): void {
    for (const h of habits) {
        if (h.type === "timer" && h.isRunning) persistTimerProgress(h.id);
        if (h.type === "counter-timer" && h.isRunning) persistCounterTimerProgress(h.id);
    }
}

// ─── CRUD ─────────────────────────────────────────────────────────────────────

export async function addHabit(habit: Omit<Habit, "id" | "updatedAt" | "deviceId">): Promise<void> {
    await dbAddHabit(habit);
    // Reload from DB so the sync metadata Rust just assigned (id, updatedAt, deviceId) is accurate
    await initHabits();
    emitHabitsChanged();
}

export async function replaceHabit(id: string, habit: Omit<Habit, "id" | "updatedAt" | "deviceId">): Promise<void> {
    await dbUpdateHabit(id, habit);
    // Reload from DB so today's progress is preserved alongside the new definition
    await initHabits();
    emitHabitsChanged();
}

export async function deleteHabit(id: string): Promise<void> {
    await dbDeleteHabit(id);
    const idx = habits.findIndex((h) => h.id === id);
    if (idx !== -1) habits.splice(idx, 1);
    emitHabitsChanged();
    syncReminders();
}
