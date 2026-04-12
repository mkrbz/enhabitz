import { find } from "./state.svelte";
import type { TimerHabit } from "$lib/types";

export function startTimer(id: number) {
    const h = find<TimerHabit>(id, "timer");
    if (h) h.isRunning = true;
}

export function stopTimer(id: number) {
    const h = find<TimerHabit>(id, "timer");
    if (h) h.isRunning = false;
}

export function tickTimer(id: number) {
    const h = find<TimerHabit>(id, "timer");
    if (h) h.secondsElapsed += 1;
}

export function setTimerElapsed(id: number, seconds: number) {
    const h = find<TimerHabit>(id, "timer");
    if (h) h.secondsElapsed = Math.max(0, seconds);
}

export function resetTimer(id: number) {
    const h = find<TimerHabit>(id, "timer");
    if (h) { h.secondsElapsed = 0; h.isRunning = false; }
}
