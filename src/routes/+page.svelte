<script lang="ts">
    import { habits, completedCount, dayLabels, isActiveOn } from "$lib/habits";
    import { isMobile, isDesktop } from "$lib/platform";
    import TodoHabit from "$lib/components/habits/TodoHabit.svelte";
    import CounterHabit from "$lib/components/habits/CounterHabit.svelte";
    import TimerHabit from "$lib/components/habits/TimerHabit.svelte";
    import CounterTimerHabit from "$lib/components/habits/CounterTimerHabit.svelte";
    import WeekStrip from "$lib/components/today/WeekStrip.svelte";
    import { Heatmap } from "@mkrbz/svelte-ui";
    import type { HeatmapData } from "@mkrbz/svelte-ui";
    import { Check, Circle } from "@lucide/svelte";

    const activeHabits = $derived(habits.filter((h) => h.isActiveToday));

    // ── Mobile: selectable day strip ────────────────────────────────────────────

    let selectedDate = $state(new Date());
    const todayKey = localDateKey(new Date());
    const selectedKey = $derived(localDateKey(selectedDate));
    const isViewingToday = $derived(selectedKey === todayKey);

    // Past days are read-only history — isActiveToday only reflects *today's*
    // schedule, so other days need the client-side mirror of the Rust scheduler.
    const selectedHabits = $derived(
        isViewingToday
            ? activeHabits
            : habits.filter((h) => isActiveOn(h, selectedDate)),
    );

    function isDoneOn(habit: { label: string }, dateKey: string): boolean {
        return (dayLabels[dateKey] ?? []).includes(habit.label);
    }

    const selectedDoneCount = $derived(
        isViewingToday
            ? completedCount()
            : selectedHabits.filter((h) => isDoneOn(h, selectedKey)).length,
    );

    function activityFor(date: Date): "full" | "partial" | "none" {
        const scheduled = habits.filter((h) => isActiveOn(h, date));
        if (scheduled.length === 0) return "none";
        const key = localDateKey(date);
        const done = scheduled.filter((h) => isDoneOn(h, key)).length;
        if (done === 0) return "none";
        return done >= scheduled.length ? "full" : "partial";
    }

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
    <div class="flex items-baseline justify-between mb-6">
        <h1 class="text-2xl font-bold">
            {isMobile && !isViewingToday ? formatDay(selectedDate) : "Today"}
        </h1>
        <span class="text-sm text-muted-foreground">
            {isMobile
                ? `${selectedDoneCount}/${selectedHabits.length}`
                : `${completedCount()}/${activeHabits.length}`} done
        </span>
    </div>

    {#if isMobile}
        <div class="mb-6">
            <WeekStrip bind:selected={selectedDate} {activityFor} />
        </div>

        {#if selectedHabits.length === 0}
            <div
                class="flex flex-col items-center justify-center py-16 text-muted-foreground gap-2"
            >
                <p class="text-sm">
                    {isViewingToday
                        ? "No habits scheduled for today."
                        : "No habits were scheduled this day."}
                </p>
                {#if isViewingToday}
                    <p class="text-xs">
                        Add habits or set their start date in the Habits tab.
                    </p>
                {/if}
            </div>
        {:else if isViewingToday}
            <div class="flex flex-col gap-2.5">
                {#each selectedHabits as habit (habit.id)}
                    <div class="rounded-xl border border-border bg-card px-2 shadow-sm">
                        {#if habit.type === "todo"}
                            <TodoHabit {habit} />
                        {:else if habit.type === "counter"}
                            <CounterHabit {habit} />
                        {:else if habit.type === "timer"}
                            <TimerHabit {habit} />
                        {:else if habit.type === "counter-timer"}
                            <CounterTimerHabit {habit} />
                        {/if}
                    </div>
                {/each}
            </div>
        {:else}
            <div class="flex flex-col gap-2.5">
                {#each selectedHabits as habit (habit.id)}
                    {@const done = isDoneOn(habit, selectedKey)}
                    <div
                        class="flex items-center gap-3 rounded-xl border border-border bg-card px-3 py-2.5 shadow-sm"
                    >
                        {#if done}
                            <Check class="h-4 w-4 shrink-0 text-primary" />
                        {:else}
                            <Circle class="h-4 w-4 shrink-0 text-muted-foreground" />
                        {/if}
                        <span
                            class={`flex-1 text-base ${done ? "line-through text-muted-foreground" : ""}`}
                        >
                            {habit.label}
                        </span>
                    </div>
                {/each}
            </div>
        {/if}
    {:else if activeHabits.length === 0}
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

    {#if isDesktop}
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
    {/if}
</main>
