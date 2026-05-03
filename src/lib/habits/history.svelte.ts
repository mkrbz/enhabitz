import { dbSaveLog, dbLoadLogHistory } from "$lib/db";
import type { Habit } from "$lib/types";
import { emitHabitsChanged } from "./manage";

// YYYY-MM-DD → completed habit labels that day
// Plain object so Svelte 5 deep-proxies it and tracks mutations.
export let dayLabels = $state<Record<string, string[]>>({});

export async function refreshHistory(): Promise<void> {
    const entries = await dbLoadLogHistory();
    // Clear existing keys
    for (const key of Object.keys(dayLabels)) {
        delete dayLabels[key];
    }
    for (const { date, label } of entries) {
        if (!dayLabels[date]) dayLabels[date] = [];
        dayLabels[date].push(label);
    }
}

export async function saveLog(habit: Habit): Promise<void> {
    await dbSaveLog(habit);
    await refreshHistory();
    emitHabitsChanged();
}
