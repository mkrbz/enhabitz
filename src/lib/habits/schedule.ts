import type { Habit } from "$lib/types";

function stripTime(date: Date): Date {
    return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

/** Mirrors is_active_today() in src-tauri/src/db.rs, generalized to any date
 * so the frontend can answer "was this habit scheduled on day X" for history
 * views without a round-trip to Rust. Keep in sync with the Rust version. */
export function isActiveOn(habit: Habit, date: Date): boolean {
    if (!habit.startDate) return false; // draft / idea

    const start = stripTime(new Date(habit.startDate + "T00:00:00"));
    const day = stripTime(date);
    if (day < start) return false; // hasn't started yet

    switch (habit.repeatType) {
        case "daily":
            return true;
        case "weekly":
            // repeatDays: 0–6 where 0 = Sunday, matches Date.getDay()
            return (habit.repeatDays ?? []).includes(day.getDay());
        case "monthly":
            return (habit.repeatDays ?? []).includes(day.getDate());
        case "interval": {
            const every = Math.max(habit.repeatEvery ?? 1, 1);
            const diffDays = Math.round(
                (day.getTime() - start.getTime()) / 86_400_000,
            );
            return diffDays % every === 0;
        }
        default:
            return true;
    }
}
