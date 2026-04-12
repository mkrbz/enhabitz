import { find } from "./state.svelte";
import type { CounterTimerHabit } from "$lib/types";

export function startCounterTimer(id: number) {
    const h = find<CounterTimerHabit>(id, "counter-timer");
    if (h && h.currentRound < h.rounds) h.isRunning = true;
}

export function stopCounterTimer(id: number) {
    const h = find<CounterTimerHabit>(id, "counter-timer");
    if (h) h.isRunning = false;
}

export function tickCounterTimer(id: number) {
    const h = find<CounterTimerHabit>(id, "counter-timer");
    if (!h) return;
    h.roundSecondsElapsed += 1;
    if (h.roundSecondsElapsed >= h.secondsPerRound) {
        h.currentRound = Math.min(h.currentRound + 1, h.rounds);
        h.roundSecondsElapsed = 0;
        if (h.currentRound >= h.rounds) h.isRunning = false;
    }
}

export function setCounterTimerRound(id: number, round: number) {
    const h = find<CounterTimerHabit>(id, "counter-timer");
    if (!h) return;
    h.currentRound = Math.max(0, Math.min(round, h.rounds));
    h.roundSecondsElapsed = 0;
    if (h.currentRound >= h.rounds) h.isRunning = false;
}

export function resetCounterTimer(id: number) {
    const h = find<CounterTimerHabit>(id, "counter-timer");
    if (h) { h.currentRound = 0; h.roundSecondsElapsed = 0; h.isRunning = false; }
}
