import { find } from "./state.svelte";
import { saveLog } from "./history.svelte";
import type { TimerHabit } from "$lib/types";

export function startTimer(id: number) {
    const h = find<TimerHabit>(id, "timer");
    if (!h) return;
    // Offset startedAt so Date.now() - startedAt always equals total elapsed ms
    h.startedAt = Date.now() - h.secondsElapsed * 1000;
    h.isRunning = true;
}

export function stopTimer(id: number) {
    const h = find<TimerHabit>(id, "timer");
    if (!h) return;
    if (h.startedAt !== undefined) {
        h.secondsElapsed = Math.max(0, Math.floor((Date.now() - h.startedAt) / 1000));
        h.startedAt = undefined;
    }
    h.isRunning = false;
    saveLog(h);
}

export function setTimerElapsed(id: number, seconds: number) {
    const h = find<TimerHabit>(id, "timer");
    if (!h) return;
    h.secondsElapsed = Math.max(0, seconds);
    if (h.isRunning) h.startedAt = Date.now() - h.secondsElapsed * 1000;
    saveLog(h);
}

export function resetTimer(id: number) {
    const h = find<TimerHabit>(id, "timer");
    if (!h) return;
    h.secondsElapsed = 0;
    h.isRunning = false;
    h.startedAt = undefined;
    saveLog(h);
}
