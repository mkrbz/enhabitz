<script lang="ts">
    import { habits, dayLabels, currentStreak, bestStreak, completionRate } from "$lib/habits";
    import { formatDay } from "$lib/date";
    import { Flame, Trophy } from "@lucide/svelte";
    import { Heatmap } from "@mkrbz/svelte-ui";
    import type { HeatmapData } from "@mkrbz/svelte-ui";

    const RATE_WINDOW_DAYS = 30;

    // Drafts (no start date) have no schedule to measure — exclude them.
    const trackedHabits = $derived(habits.filter((h) => h.startDate));

    const rows = $derived(
        trackedHabits
            .map((h) => ({
                habit: h,
                current: currentStreak(h),
                best: bestStreak(h),
                rate: completionRate(h, RATE_WINDOW_DAYS),
            }))
            .sort((a, b) => b.current - a.current || b.best - a.best),
    );

    // ── Heatmap (moved here from Today — this is where historical view belongs) ─

    const heatmapData = $derived<HeatmapData[]>(
        Object.entries(dayLabels).map(([date, labels]) => ({
            date: new Date(date + "T00:00:00"),
            count: labels.length,
        })),
    );

    function labelsForDay(date: Date): string[] {
        const y = date.getFullYear();
        const m = String(date.getMonth() + 1).padStart(2, "0");
        const d = String(date.getDate()).padStart(2, "0");
        return dayLabels[`${y}-${m}-${d}`] ?? [];
    }

    type Theme = "green" | "orange" | "red";

    const THEMES: { value: Theme; dot: string }[] = [
        { value: "green", dot: "bg-emerald-500" },
        { value: "orange", dot: "bg-orange-500" },
        { value: "red", dot: "bg-rose-500" },
    ];

    const SCALES: Record<Theme, string[]> = {
        green: ["bg-emerald-950", "bg-emerald-700", "bg-emerald-500", "bg-emerald-300"],
        orange: ["bg-orange-950", "bg-orange-700", "bg-orange-500", "bg-orange-300"],
        red: ["bg-rose-950", "bg-rose-700", "bg-rose-500", "bg-rose-300"],
    };

    const THEME_KEY = "heatmap-theme";

    function savedTheme(): Theme {
        const v = localStorage.getItem(THEME_KEY);
        return v === "green" || v === "orange" || v === "red" ? v : "green";
    }

    let theme = $state<Theme>(savedTheme());

    $effect(() => {
        localStorage.setItem(THEME_KEY, theme);
    });

    function colorScale(count: number, future: boolean): string {
        if (future) return "bg-muted border border-border";
        if (count === 0) return "bg-muted/50 border border-border/40";
        const scale = SCALES[theme];
        if (count === 1) return scale[0];
        if (count === 2) return scale[1];
        if (count === 3) return scale[2];
        return scale[3];
    }
</script>

<main class="flex flex-col w-full max-w-lg mx-auto px-4 py-6">
    <h1 class="text-2xl font-bold mb-6">Statistics</h1>

    {#if trackedHabits.length === 0}
        <div class="flex flex-col items-center justify-center py-16 text-muted-foreground gap-2">
            <p class="text-sm">No habits scheduled yet.</p>
            <p class="text-xs">Add habits and set their start date to see stats here.</p>
        </div>
    {:else}
        <div class="flex flex-col gap-2.5 mb-10">
            {#each rows as { habit, current, best, rate } (habit.id)}
                <div class="rounded-xl border border-border bg-card p-3 shadow-sm flex flex-col gap-2">
                    <span class="font-medium">{habit.label}</span>

                    <div class="flex items-center gap-4 text-sm">
                        <div class="flex items-center gap-1.5">
                            <Flame class={`h-4 w-4 ${current > 0 ? "text-orange-500" : "text-muted-foreground"}`} />
                            <span>{current} day{current === 1 ? "" : "s"}</span>
                        </div>
                        <div class="flex items-center gap-1.5 text-muted-foreground">
                            <Trophy class="h-4 w-4" />
                            <span>Best: {best}</span>
                        </div>
                    </div>

                    {#if rate !== null}
                        <div class="flex items-center gap-2">
                            <div class="h-1.5 flex-1 rounded-full bg-muted overflow-hidden">
                                <div
                                    class="h-full bg-primary rounded-full"
                                    style="width: {Math.round(rate * 100)}%"
                                ></div>
                            </div>
                            <span class="text-xs text-muted-foreground tabular-nums w-9 text-right">
                                {Math.round(rate * 100)}%
                            </span>
                        </div>
                        <span class="text-xs text-muted-foreground">Last {RATE_WINDOW_DAYS} days</span>
                    {/if}
                </div>
            {/each}
        </div>

        <div>
            <div class="flex items-center justify-between mb-2">
                <span class="text-xs text-muted-foreground">Habits completed</span>
                <div class="flex gap-1.5">
                    {#each THEMES as t}
                        <button
                            onclick={() => (theme = t.value)}
                            class={`w-4 h-4 rounded-full ${t.dot} transition-all
                                ${theme === t.value ? "ring-2 ring-offset-1 ring-offset-background ring-current scale-110" : "opacity-40 hover:opacity-70"}`}
                            aria-label={t.value}
                        ></button>
                    {/each}
                </div>
            </div>
            <div class="overflow-x-auto -mx-4 px-4">
                <Heatmap
                    data={heatmapData}
                    weeks={26}
                    todaysWeek={-1}
                    label=""
                    options={{ colorScale }}
                >
                    {#snippet tooltip(date)}
                        {@const labels = labelsForDay(date)}
                        <p class="font-medium mb-1">{formatDay(date)}</p>
                        {#if labels.length === 0}
                            <p class="text-muted-foreground">Nothing completed</p>
                        {:else}
                            <ul class="flex flex-col gap-0.5">
                                {#each labels as label}
                                    <li>· {label}</li>
                                {/each}
                            </ul>
                        {/if}
                    {/snippet}
                </Heatmap>
            </div>
        </div>
    {/if}
</main>
