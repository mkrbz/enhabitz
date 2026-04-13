import type { Habit } from "$lib/types";
import { dbLoadHabits } from "$lib/db";

export let habits = $state<Habit[]>([]);

export async function initHabits(): Promise<void> {
    const loaded = await dbLoadHabits();
    habits.splice(0, habits.length, ...loaded);
}

export function find<T extends Habit>(id: number, type: T["type"]): T | undefined {
    return habits.find((h) => h.id === id && h.type === type) as T | undefined;
}
