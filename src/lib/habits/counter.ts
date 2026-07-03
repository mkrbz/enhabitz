import { find } from "./state.svelte";
import { saveLog } from "./history.svelte";
import type { CounterHabit } from "$lib/types";

const DEBOUNCE_MS = 250;
const pending = new Map<string, ReturnType<typeof setTimeout>>();

/** Coalesces rapid taps (e.g. tapping +1 twenty times in a few seconds) into
 * a single write instead of one IPC round-trip + disk write per tap — the
 * UI is already optimistic via local $state, so only the persistence needs
 * delaying. Not used for discrete, infrequent actions (toggleTodo,
 * stopTimer, manual "Set" entry) — those should save immediately. */
function debouncedSaveLog(habit: CounterHabit) {
    const existing = pending.get(habit.id);
    if (existing) clearTimeout(existing);
    pending.set(
        habit.id,
        setTimeout(() => {
            pending.delete(habit.id);
            saveLog(habit);
        }, DEBOUNCE_MS),
    );
}

export function increment(id: string) {
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
    debouncedSaveLog(h);
}

export function decrement(id: string) {
    const h = find<CounterHabit>(id, "counter");
    if (!h) return;
    if (h.count > 0) {
        h.count--;
    } else if (h.sets !== undefined && (h.completedSets ?? 0) > 0) {
        h.completedSets = (h.completedSets ?? 0) - 1;
        h.count = h.target - 1;
    }
    debouncedSaveLog(h);
}

export function setCount(id: string, count: number, completedSets?: number) {
    const h = find<CounterHabit>(id, "counter");
    if (!h) return;
    h.count = Math.max(0, count);
    if (completedSets !== undefined && h.sets !== undefined) {
        h.completedSets = Math.max(0, Math.min(completedSets, h.sets));
    }
    saveLog(h);
}
