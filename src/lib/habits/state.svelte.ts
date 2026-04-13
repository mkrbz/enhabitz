import type { Habit } from "$lib/types";

export let habits = $state<Habit[]>([]);

export function find<T extends Habit>(id: number, type: T["type"]): T | undefined {
    return habits.find((h) => h.id === id && h.type === type) as T | undefined;
}
