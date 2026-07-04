import {
    isPermissionGranted,
    requestPermission,
    sendNotification,
    cancel,
    Schedule,
} from "@tauri-apps/plugin-notification";
import { habits } from "./habits/state.svelte";
import { isHabitDone } from "./habits/derived.svelte";
import { isActiveOn } from "./habits/schedule";

// The notification plugin requires a stable 32-bit int id per notification,
// but habit ids are UUID strings — derive one deterministically (FNV-1a,
// masked to 31 bits to stay a safe positive int32 across the JS/Kotlin/Rust
// boundary) rather than maintaining a separate id mapping table.
function notificationIdFor(habitId: string): number {
    let hash = 0x811c9dc5;
    for (let i = 0; i < habitId.length; i++) {
        hash ^= habitId.charCodeAt(i);
        hash = Math.imul(hash, 0x01000193);
    }
    return hash & 0x7fffffff;
}

function reminderDateTimeToday(reminderTime: string, now: Date): Date {
    const [hours, minutes] = reminderTime.split(":").map(Number);
    const date = new Date(now);
    date.setHours(hours, minutes, 0, 0);
    return date;
}

/** Requests notification permission if not already granted. Call this from
 * a clear, contextual user action (e.g. setting a reminder time on a
 * habit) — not eagerly on app launch. */
export async function ensureNotificationPermission(): Promise<boolean> {
    if (await isPermissionGranted()) return true;
    const permission = await requestPermission();
    return permission === "granted";
}

/** Reconciles every habit's reminder against current state: schedules a
 * notification for habits that are active today, not yet done, and whose
 * reminder time hasn't passed yet; cancels it otherwise (already done, not
 * scheduled today, or the time already passed). Idempotent and cheap to
 * call after any habit mutation — see the call sites in manage.ts and
 * history.svelte.ts. No-ops quickly if permission was never granted, so
 * it's safe to call unconditionally before the user has touched this
 * feature at all. */
export async function syncReminders(): Promise<void> {
    if (!(await isPermissionGranted())) return;

    const now = new Date();
    for (const h of habits) {
        if (!h.reminderTime) continue;

        const id = notificationIdFor(h.id);
        const reminderAt = reminderDateTimeToday(h.reminderTime, now);
        const shouldRemind = isActiveOn(h, now) && !isHabitDone(h) && reminderAt > now;

        if (shouldRemind) {
            await sendNotification({
                id,
                title: "Reminder",
                body: `${h.label} isn't done yet`,
                schedule: Schedule.at(reminderAt, false, true),
            });
        } else {
            await cancel([id]);
        }
    }
}
