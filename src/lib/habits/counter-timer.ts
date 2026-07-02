import { find } from "./state.svelte";
import { saveLog } from "./history.svelte";
import type { CounterTimerHabit } from "$lib/types";

function totalElapsedSeconds(h: CounterTimerHabit): number {
    return h.currentRound * h.secondsPerRound + h.roundSecondsElapsed;
}

function roundStateAt(
    h: CounterTimerHabit,
    elapsedMs: number,
): { currentRound: number; roundSecondsElapsed: number } {
    const elapsed = Math.max(0, Math.floor(elapsedMs / 1000));
    const totalTarget = h.rounds * h.secondsPerRound;
    const clamped = Math.min(elapsed, totalTarget);
    const currentRound = Math.min(Math.floor(clamped / h.secondsPerRound), h.rounds);
    const roundSecondsElapsed = currentRound >= h.rounds ? 0 : clamped % h.secondsPerRound;
    return { currentRound, roundSecondsElapsed };
}

export function startCounterTimer(id: string) {
    const h = find<CounterTimerHabit>(id, "counter-timer");
    if (!h || h.currentRound >= h.rounds) return;
    h.startedAt = Date.now() - totalElapsedSeconds(h) * 1000;
    h.isRunning = true;
}

export function stopCounterTimer(id: string) {
    const h = find<CounterTimerHabit>(id, "counter-timer");
    if (!h) return;
    if (h.startedAt !== undefined) {
        Object.assign(h, roundStateAt(h, Date.now() - h.startedAt));
        h.startedAt = undefined;
    }
    h.isRunning = false;
    saveLog(h);
}

/** Snapshots current round progress to disk without stopping — same
 * rationale as persistTimerProgress() in timer.ts. */
export function persistCounterTimerProgress(id: string) {
    const h = find<CounterTimerHabit>(id, "counter-timer");
    if (!h || !h.isRunning || h.startedAt === undefined) return;
    saveLog({ ...h, ...roundStateAt(h, Date.now() - h.startedAt) });
}

export function setCounterTimerRound(id: string, round: number) {
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
    saveLog(h);
}

export function resetCounterTimer(id: string) {
    const h = find<CounterTimerHabit>(id, "counter-timer");
    if (!h) return;
    h.currentRound = 0;
    h.roundSecondsElapsed = 0;
    h.isRunning = false;
    h.startedAt = undefined;
    saveLog(h);
}
