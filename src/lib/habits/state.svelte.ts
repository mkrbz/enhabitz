import type { Habit } from "$lib/types";

export let habits = $state<Habit[]>([
    {
        id: 1,
        type: "todo",
        label: "Morning meditation",
        done: false,
    },
    {
        id: 2,
        type: "counter",
        label: "Pushups",
        count: 0,
        target: 20,
    },
    {
        id: 3,
        type: "counter",
        label: "Pushup sets",
        count: 0,
        target: 20,
        sets: 3,
        completedSets: 0,
    },
    {
        id: 4,
        type: "timer",
        label: "Cold shower",
        targetSeconds: 120,
        secondsElapsed: 0,
        isRunning: false,
    },
    {
        id: 5,
        type: "counter-timer",
        label: "Plank sets",
        rounds: 3,
        secondsPerRound: 30,
        currentRound: 0,
        roundSecondsElapsed: 0,
        isRunning: false,
    },
]);

export function find<T extends Habit>(id: number, type: T["type"]): T | undefined {
    return habits.find((h) => h.id === id && h.type === type) as T | undefined;
}
