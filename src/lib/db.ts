import { invoke } from "@tauri-apps/api/core";
import type { Habit, TodoHabit, CounterHabit, TimerHabit, CounterTimerHabit } from "./types";

// ─── Wire types (mirror Rust HabitRecord / HabitData / LogData) ───────────────

interface HabitRecord {
    id: number;
    type: Habit["type"];
    label: string;
    // counter
    target: number | null;
    sets: number | null;
    // timer
    target_seconds: number | null;
    // counter-timer
    rounds: number | null;
    seconds_per_round: number | null;
    // today's log
    done: boolean | null;
    count: number | null;
    completed_sets: number | null;
    seconds_elapsed: number | null;
    current_round: number | null;
    round_seconds_elapsed: number | null;
}

interface HabitData {
    type: Habit["type"];
    label: string;
    target: number | null;
    sets: number | null;
    target_seconds: number | null;
    rounds: number | null;
    seconds_per_round: number | null;
}

interface LogData {
    habit_id: number;
    done: boolean | null;
    count: number | null;
    completed_sets: number | null;
    seconds_elapsed: number | null;
    current_round: number | null;
    round_seconds_elapsed: number | null;
}

// ─── Commands ─────────────────────────────────────────────────────────────────

export async function dbLoadHabits(): Promise<Habit[]> {
    const records = await invoke<HabitRecord[]>("load_habits");
    return records.map(recordToHabit);
}

export async function dbAddHabit(habit: Omit<Habit, "id">): Promise<number> {
    return invoke<number>("add_habit", { data: habitToData(habit) });
}

export async function dbUpdateHabit(id: number, habit: Omit<Habit, "id">): Promise<void> {
    return invoke("update_habit", { id, data: habitToData(habit) });
}

export async function dbDeleteHabit(id: number): Promise<void> {
    return invoke("delete_habit", { id });
}

export async function dbSaveLog(habit: Habit): Promise<void> {
    return invoke("save_log", { log: habitToLog(habit) });
}

// ─── Mapping helpers ──────────────────────────────────────────────────────────

function recordToHabit(r: HabitRecord): Habit {
    const base = { id: r.id, label: r.label };
    switch (r.type) {
        case "todo":
            return { ...base, type: "todo", done: r.done ?? false } as TodoHabit;
        case "counter":
            return {
                ...base, type: "counter",
                count: r.count ?? 0,
                target: r.target!,
                sets: r.sets ?? undefined,
                completedSets: r.completed_sets ?? 0,
            } as CounterHabit;
        case "timer":
            return {
                ...base, type: "timer",
                targetSeconds: r.target_seconds!,
                secondsElapsed: r.seconds_elapsed ?? 0,
                isRunning: false,
            } as TimerHabit;
        case "counter-timer":
            return {
                ...base, type: "counter-timer",
                rounds: r.rounds!,
                secondsPerRound: r.seconds_per_round!,
                currentRound: r.current_round ?? 0,
                roundSecondsElapsed: r.round_seconds_elapsed ?? 0,
                isRunning: false,
            } as CounterTimerHabit;
    }
}

function habitToData(habit: Omit<Habit, "id">): HabitData {
    return {
        type: habit.type,
        label: habit.label,
        target: habit.type === "counter" ? (habit as CounterHabit).target : null,
        sets: habit.type === "counter" ? ((habit as CounterHabit).sets ?? null) : null,
        target_seconds: habit.type === "timer" ? (habit as TimerHabit).targetSeconds : null,
        rounds: habit.type === "counter-timer" ? (habit as CounterTimerHabit).rounds : null,
        seconds_per_round: habit.type === "counter-timer" ? (habit as CounterTimerHabit).secondsPerRound : null,
    };
}

function habitToLog(habit: Habit): LogData {
    return {
        habit_id: habit.id,
        done: habit.type === "todo" ? habit.done : null,
        count: habit.type === "counter" ? habit.count : null,
        completed_sets: habit.type === "counter" ? (habit.completedSets ?? null) : null,
        seconds_elapsed: habit.type === "timer" ? habit.secondsElapsed : null,
        current_round: habit.type === "counter-timer" ? habit.currentRound : null,
        round_seconds_elapsed: habit.type === "counter-timer" ? habit.roundSecondsElapsed : null,
    };
}
