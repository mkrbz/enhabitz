import { find } from "./state.svelte";
import { dbSaveLog } from "$lib/db";
import type { CounterTimerHabit } from "$lib/types";

function totalElapsedSeconds(h: CounterTimerHabit): number {
    return h.currentRound * h.secondsPerRound + h.roundSecondsElapsed;
}

export function startCounterTimer(id: number) {
    const h = find<CounterTimerHabit>(id, "counter-timer");
    if (!h || h.currentRound >= h.rounds) return;
    h.startedAt = Date.now() - totalElapsedSeconds(h) * 1000;
    h.isRunning = true;
}

export function stopCounterTimer(id: number) {
    const h = find<CounterTimerHabit>(id, "counter-timer");
    if (!h) return;
    if (h.startedAt !== undefined) {
        const elapsed = Math.max(0, Math.floor((Date.now() - h.startedAt) / 1000));
        const totalTarget = h.rounds * h.secondsPerRound;
        const clamped = Math.min(elapsed, totalTarget);
        h.currentRound = Math.min(Math.floor(clamped / h.secondsPerRound), h.rounds);
        h.roundSecondsElapsed = h.currentRound >= h.rounds ? 0 : clamped % h.secondsPerRound;
        h.startedAt = undefined;
    }
    h.isRunning = false;
    dbSaveLog(h);
}

export function setCounterTimerRound(id: number, round: number) {
    const h = find<CounterTimerHabit>(id, "counter-timer");
    if (!h) return;
    h.currentRound = Math.max(0, Math.min(round, h.rounds));
    h.roundSecondsElapsed = 0;
    if (h.currentRound >= h.rounds) {
        h.isRunning = false;
        h.startedAt = undefined;
    } else if (h.isRunning) {
        h.startedAt = Date.now() - totalElapsedSeconds(h) * 1000;
    }
    dbSaveLog(h);
}

export function resetCounterTimer(id: number) {
    const h = find<CounterTimerHabit>(id, "counter-timer");
    if (!h) return;
    h.currentRound = 0;
    h.roundSecondsElapsed = 0;
    h.isRunning = false;
    h.startedAt = undefined;
    dbSaveLog(h);
}
