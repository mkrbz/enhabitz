import { find } from "./state.svelte";
import { dbSaveLog } from "$lib/db";
import type { CounterHabit } from "$lib/types";

export function increment(id: number) {
    const h = find<CounterHabit>(id, "counter");
    if (!h) return;
    if (h.sets !== undefined) {
        const completed = h.completedSets ?? 0;
        if (completed >= h.sets) return;
        h.count++;
        if (h.count >= h.target) {
            h.completedSets = completed + 1;
            if (h.completedSets < h.sets) h.count = 0;
        }
    } else {
        h.count++;
    }
    dbSaveLog(h);
}

export function decrement(id: number) {
    const h = find<CounterHabit>(id, "counter");
    if (!h) return;
    if (h.count > 0) {
        h.count--;
    } else if (h.sets !== undefined && (h.completedSets ?? 0) > 0) {
        h.completedSets = (h.completedSets ?? 0) - 1;
        h.count = h.target - 1;
    }
    dbSaveLog(h);
}

export function setCount(id: number, count: number, completedSets?: number) {
    const h = find<CounterHabit>(id, "counter");
    if (!h) return;
    h.count = Math.max(0, count);
    if (completedSets !== undefined && h.sets !== undefined) {
        h.completedSets = Math.max(0, Math.min(completedSets, h.sets));
    }
    dbSaveLog(h);
}
