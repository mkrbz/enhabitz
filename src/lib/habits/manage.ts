import { habits, initHabits } from "./state.svelte";
import { dbAddHabit, dbUpdateHabit, dbDeleteHabit } from "$lib/db";
import type { Habit } from "$lib/types";

export { initHabits };

export async function addHabit(habit: Omit<Habit, "id">): Promise<void> {
    const id = await dbAddHabit(habit);
    habits.push({ ...habit, id } as Habit);
}

export async function replaceHabit(id: number, habit: Omit<Habit, "id">): Promise<void> {
    await dbUpdateHabit(id, habit);
    const idx = habits.findIndex((h) => h.id === id);
    if (idx !== -1) habits[idx] = { ...habit, id } as Habit;
}

export async function deleteHabit(id: number): Promise<void> {
    await dbDeleteHabit(id);
    const idx = habits.findIndex((h) => h.id === id);
    if (idx !== -1) habits.splice(idx, 1);
}
