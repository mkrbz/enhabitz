import { habits } from "./state.svelte";
import type { Habit } from "$lib/types";

export function addHabit(habit: Omit<Habit, "id">) {
    const id = habits.length > 0 ? Math.max(...habits.map((h) => h.id)) + 1 : 1;
    habits.push({ ...habit, id } as Habit);
}

export function replaceHabit(id: number, habit: Omit<Habit, "id">) {
    const idx = habits.findIndex((h) => h.id === id);
    if (idx !== -1) habits[idx] = { ...habit, id } as Habit;
}

export function deleteHabit(id: number) {
    const idx = habits.findIndex((h) => h.id === id);
    if (idx !== -1) habits.splice(idx, 1);
}
