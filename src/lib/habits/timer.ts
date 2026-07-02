import { find } from "./state.svelte";
import { saveLog } from "./history.svelte";
import type { TimerHabit } from "$lib/types";

export function startTimer(id: string) {
    const h = find<TimerHabit>(id, "timer");
    if (!h) return;
    // Offset startedAt so Date.now() - startedAt always equals total elapsed ms
    h.startedAt = Date.now() - h.secondsElapsed * 1000;
    h.isRunning = true;
}

export function stopTimer(id: string) {
    const h = find<TimerHabit>(id, "timer");
    if (!h) return;
    if (h.startedAt !== undefined) {
        h.secondsElapsed = Math.max(0, Math.floor((Date.now() - h.startedAt) / 1000));
        h.startedAt = undefined;
    }
    h.isRunning = false;
    saveLog(h);
}

export function setTimerElapsed(id: string, seconds: number) {
    const h = find<TimerHabit>(id, "timer");
    if (!h) return;
    h.secondsElapsed = Math.max(0, seconds);
    if (h.isRunning) h.startedAt = Date.now() - h.secondsElapsed * 1000;
    saveLog(h);
}

export function resetTimer(id: string) {
    const h = find<TimerHabit>(id, "timer");
    if (!h) return;
    h.secondsElapsed = 0;
    h.isRunning = false;
    h.startedAt = undefined;
    saveLog(h);
}

/** Snapshots current elapsed time to disk without stopping — a running
 * timer's progress otherwise only lives in memory until an explicit pause,
 * but Android can kill a backgrounded process at any time to reclaim
 * memory, silently discarding it. Called on visibilitychange, see
 * persistRunningProgress() in manage.ts. */
export function persistTimerProgress(id: string) {
    const h = find<TimerHabit>(id, "timer");
    if (!h || !h.isRunning || h.startedAt === undefined) return;
    const elapsed = Math.max(0, Math.floor((Date.now() - h.startedAt) / 1000));
    saveLog({ ...h, secondsElapsed: elapsed });
}
