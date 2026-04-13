import { habits } from "./state.svelte";
import type { Habit } from "$lib/types";

export function isHabitDone(habit: Habit): boolean {
    switch (habit.type) {
        case "todo":          return habit.done;
        case "counter":       return habit.sets
                                ? (habit.completedSets ?? 0) >= habit.sets
                                : habit.count >= habit.target;
        case "timer":         return habit.secondsElapsed >= habit.targetSeconds;
        case "counter-timer": return habit.currentRound >= habit.rounds;
    }
}

export function completedCount(): number {
    return habits.filter((h) => h.isActiveToday && isHabitDone(h)).length;
}
