export type HabitType = "todo" | "counter" | "timer" | "counter-timer";

export interface BaseHabit {
    id: number;
    type: HabitType;
    label: string;
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
}

// Rounds model: N rounds × M seconds each
export interface CounterTimerHabit extends BaseHabit {
    type: "counter-timer";
    rounds: number;           // target number of rounds
    secondsPerRound: number;  // duration of each round
    currentRound: number;     // completed rounds (0-based)
    roundSecondsElapsed: number; // seconds elapsed in the current round
    isRunning: boolean;
}

export type Habit = TodoHabit | CounterHabit | TimerHabit | CounterTimerHabit;
