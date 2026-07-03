export function localDateKey(date: Date): string {
    const y = date.getFullYear();
    const m = String(date.getMonth() + 1).padStart(2, "0");
    const d = String(date.getDate()).padStart(2, "0");
    return `${y}-${m}-${d}`;
}

export function stripTime(date: Date): Date {
    return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

export function addDays(date: Date, n: number): Date {
    const d = new Date(date);
    d.setDate(d.getDate() + n);
    return d;
}

const MONTHS = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

export function formatDay(date: Date): string {
    return `${MONTHS[date.getMonth()]} ${date.getDate()}`;
}

/** Schedules `callback` to fire once just after the next local midnight,
 * then reschedules itself for the following one — a single precisely-timed
 * wakeup instead of a recurring poll (e.g. a 60s setInterval) that would
 * keep the JS engine busy for as long as the app is alive, fighting
 * Android's Doze/App Standby power management for no benefit (the check
 * only ever needs to fire once a day). Returns an unsubscribe function. */
export function onNextMidnight(callback: () => void): () => void {
    let timeout: ReturnType<typeof setTimeout>;

    function schedule() {
        const now = new Date();
        const next = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1, 0, 0, 5);
        timeout = setTimeout(() => {
            callback();
            schedule();
        }, next.getTime() - now.getTime());
    }

    schedule();
    return () => clearTimeout(timeout);
}
