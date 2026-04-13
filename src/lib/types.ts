export type HabitType = "todo" | "counter" | "timer" | "counter-timer";
export type RepeatType = "daily" | "weekly" | "monthly" | "interval";

export interface BaseHabit {
    id: number;
    type: HabitType;
    label: string;
    // Scheduling — null startDate means draft/idea
    startDate: string | null;       // YYYY-MM-DD
    repeatType: RepeatType;
    repeatDays: number[] | null;    // weekly: 0–6 (Sun=0), monthly: 1–31
    repeatEvery: number | null;     // interval: every N days
    isActiveToday: boolean;         // computed by Rust
}

export interface TodoHabit extends BaseHabit {
    type: "todo";
    done: boolean;
}

export interface CounterHabit extends BaseHabit {
    type: "counter";
    count: number;        // reps in the current set
    target: number;       // reps per set
    sets?: number;        // total sets (undefined = no sets, treat as 1)
    completedSets?: number; // sets finished so far
}

export interface TimerHabit extends BaseHabit {
    type: "timer";
    targetSeconds: number;
    secondsElapsed: number;
    isRunning: boolean;
    startedAt?: number; // Date.now() offset by already-elapsed — ephemeral, never persisted
}

// Rounds model: N rounds × M seconds each
export interface CounterTimerHabit extends BaseHabit {
    type: "counter-timer";
    rounds: number;
    secondsPerRound: number;
    currentRound: number;
    roundSecondsElapsed: number;
    isRunning: boolean;
    startedAt?: number; // Date.now() offset by total elapsed — ephemeral, never persisted
}

export type Habit = TodoHabit | CounterHabit | TimerHabit | CounterTimerHabit;
