import { habits } from "./state.svelte";
import { stopTimer } from "./timer";
import { stopCounterTimer } from "./counter-timer";
import { dbLoadHabits, dbAddHabit, dbUpdateHabit, dbDeleteHabit } from "$lib/db";
import type { Habit, TimerHabit, CounterTimerHabit } from "$lib/types";
import { emit } from "@tauri-apps/api/event";

let loadedDate = "";

function todayStr(): string {
    return new Date().toISOString().slice(0, 10);
}

export function emitHabitsChanged(): void {
    emit("habits-changed").catch(() => {});
}

// ─── Init & reset ─────────────────────────────────────────────────────────────

export async function initHabits(): Promise<void> {
    const loaded = await dbLoadHabits();
    habits.splice(0, habits.length, ...loaded);
    loadedDate = todayStr();
}

// Reload from DB while preserving running timer state in this window.
export async function refreshHabits(): Promise<void> {
    const runningState = new Map<number, number>();
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

// ─── CRUD ─────────────────────────────────────────────────────────────────────

export async function addHabit(habit: Omit<Habit, "id">): Promise<void> {
    const id = await dbAddHabit(habit);
    habits.push({ ...habit, id } as Habit);
    emitHabitsChanged();
}

export async function replaceHabit(id: number, habit: Omit<Habit, "id">): Promise<void> {
    await dbUpdateHabit(id, habit);
    // Reload from DB so today's progress is preserved alongside the new definition
    await initHabits();
    emitHabitsChanged();
}

export async function deleteHabit(id: number): Promise<void> {
    await dbDeleteHabit(id);
    const idx = habits.findIndex((h) => h.id === id);
    if (idx !== -1) habits.splice(idx, 1);
    emitHabitsChanged();
}
