import type { Habit } from "$lib/types";
import { isActiveOn } from "./schedule";
import { dayLabels } from "./history.svelte";
import { localDateKey, stripTime, addDays } from "$lib/date";

function isDoneOn(habit: Habit, dateKey: string): boolean {
    return (dayLabels[dateKey] ?? []).includes(habit.label);
}

/** Consecutive scheduled days completed, walking back from today. Today
 * itself doesn't break the streak if it's not done yet — there's still time
 * left in the day. */
export function currentStreak(habit: Habit, today: Date = new Date()): number {
    if (!habit.startDate) return 0;
    const start = stripTime(new Date(habit.startDate + "T00:00:00"));
    let cursor = stripTime(today);

    if (isActiveOn(habit, cursor) && !isDoneOn(habit, localDateKey(cursor))) {
        cursor = addDays(cursor, -1);
    }

    let streak = 0;
    while (cursor >= start) {
        if (isActiveOn(habit, cursor)) {
            if (isDoneOn(habit, localDateKey(cursor))) {
                streak++;
            } else {
                break;
            }
        }
        cursor = addDays(cursor, -1);
    }
    return streak;
}

/** Longest streak ever achieved, from the habit's start date through today. */
export function bestStreak(habit: Habit, today: Date = new Date()): number {
    if (!habit.startDate) return 0;
    const start = stripTime(new Date(habit.startDate + "T00:00:00"));
    const end = stripTime(today);

    let cursor = start;
    let running = 0;
    let best = 0;
    while (cursor <= end) {
        if (isActiveOn(habit, cursor)) {
            if (isDoneOn(habit, localDateKey(cursor))) {
                running++;
                if (running > best) best = running;
            } else {
                running = 0;
            }
        }
        cursor = addDays(cursor, 1);
    }
    return best;
}

/** Fraction (0–1) of scheduled days completed in the trailing `days`-day
 * window ending today. Null when nothing was scheduled in that window. */
export function completionRate(
    habit: Habit,
    days: number,
    today: Date = new Date(),
): number | null {
    const end = stripTime(today);
    const start = addDays(end, -(days - 1));

    let cursor = start;
    let scheduled = 0;
    let done = 0;
    while (cursor <= end) {
        if (isActiveOn(habit, cursor)) {
            scheduled++;
            if (isDoneOn(habit, localDateKey(cursor))) done++;
        }
        cursor = addDays(cursor, 1);
    }
    return scheduled === 0 ? null : done / scheduled;
}
