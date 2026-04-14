<script lang="ts">
    import { habits, completedCount, dayLabels } from "$lib/habits";
    import TodoHabit from "$lib/components/habits/TodoHabit.svelte";
    import CounterHabit from "$lib/components/habits/CounterHabit.svelte";
    import TimerHabit from "$lib/components/habits/TimerHabit.svelte";
    import CounterTimerHabit from "$lib/components/habits/CounterTimerHabit.svelte";
    import { Heatmap } from "@mkrbz/svelte-ui";
    import type { HeatmapData } from "@mkrbz/svelte-ui";

    const activeHabits = $derived(habits.filter((h) => h.isActiveToday));

    const heatmapData = $derived<HeatmapData[]>(
        Object.entries(dayLabels).map(([date, labels]) => ({
            date: new Date(date + "T00:00:00"),
            count: labels.length,
        })),
    );

    const MONTHS = [
        "Jan",
        "Feb",
        "Mar",
        "Apr",
        "May",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "Oct",
        "Nov",
        "Dec",
    ];

    function formatDay(date: Date): string {
        return `${MONTHS[date.getMonth()]} ${date.getDate()}`;
    }

    function localDateKey(date: Date): string {
        const y = date.getFullYear();
        const m = String(date.getMonth() + 1).padStart(2, "0");
        const d = String(date.getDate()).padStart(2, "0");
        return `${y}-${m}-${d}`;
    }

    function labelsForDay(date: Date): string[] {
        return dayLabels[localDateKey(date)] ?? [];
    }

    // ── Heatmap color themes ──────────────────────────────────────────────────

    type Theme = "green" | "orange" | "red";

    const THEMES: { value: Theme; dot: string }[] = [
        { value: "green", dot: "bg-emerald-500" },
        { value: "orange", dot: "bg-orange-500" },
        { value: "red", dot: "bg-rose-500" },
    ];

    const SCALES: Record<Theme, string[]> = {
        green: [
            "bg-emerald-950",
            "bg-emerald-700",
            "bg-emerald-500",
            "bg-emerald-300",
        ],
        orange: [
            "bg-orange-950",
            "bg-orange-700",
            "bg-orange-500",
            "bg-orange-300",
        ],
        red: ["bg-rose-950", "bg-rose-700", "bg-rose-500", "bg-rose-300"],
    };

    let theme = $state<Theme>("green");

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
    <div class="flex items-baseline justify-between mb-6">
        <h1 class="text-2xl font-bold">Today</h1>
        <span class="text-sm text-muted-foreground">
            {completedCount()}/{activeHabits.length} done
        </span>
    </div>

    {#if activeHabits.length === 0}
        <div
            class="flex flex-col items-center justify-center py-16 text-muted-foreground gap-2"
        >
            <p class="text-sm">No habits scheduled for today.</p>
            <p class="text-xs">
                Add habits or set their start date in the Habits tab.
            </p>
        </div>
    {:else}
        <div class="flex flex-col divide-y divide-border">
            {#each activeHabits as habit (habit.id)}
                {#if habit.type === "todo"}
                    <TodoHabit {habit} />
                {:else if habit.type === "counter"}
                    <CounterHabit {habit} />
                {:else if habit.type === "timer"}
                    <TimerHabit {habit} />
                {:else if habit.type === "counter-timer"}
                    <CounterTimerHabit {habit} />
                {/if}
            {/each}
        </div>
    {/if}

    <div class="mt-10">
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
</main>
