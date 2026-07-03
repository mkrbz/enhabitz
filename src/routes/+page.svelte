<script lang="ts">
    import { onMount } from "svelte";
    import { habits, completedCount, dayLabels, isActiveOn } from "$lib/habits";
    import { isMobile } from "$lib/platform";
    import { localDateKey, formatDay, onNextMidnight } from "$lib/date";
    import TodoHabit from "$lib/components/habits/TodoHabit.svelte";
    import CounterHabit from "$lib/components/habits/CounterHabit.svelte";
    import TimerHabit from "$lib/components/habits/TimerHabit.svelte";
    import CounterTimerHabit from "$lib/components/habits/CounterTimerHabit.svelte";
    import WeekStrip from "$lib/components/today/WeekStrip.svelte";
    import { Check, Circle } from "@lucide/svelte";

    const activeHabits = $derived(habits.filter((h) => h.isActiveToday));

    // ── Mobile: selectable day strip ────────────────────────────────────────────

    // `todayDate` tracks the real day, refreshed on focus/interval below —
    // a plain const here would freeze at whatever day the app happened to
    // load on, silently turning "today" into a read-only past day (with no
    // click handlers at all) after the app has been open across midnight.
    let todayDate = $state(new Date());
    // null = "following today"; a Date = the user explicitly picked a day.
    let userSelected = $state<Date | null>(null);
    const selectedDate = $derived(userSelected ?? todayDate);

    onMount(() => {
        function refreshToday() {
            // If the explicit pick was "today" as of the last check, keep
            // following the new day rather than getting stuck on what is
            // now yesterday.
            if (userSelected && localDateKey(userSelected) === localDateKey(todayDate)) {
                userSelected = null;
            }
            todayDate = new Date();
        }
        window.addEventListener("focus", refreshToday);
        const stopMidnightCheck = onNextMidnight(refreshToday);
        return () => {
            window.removeEventListener("focus", refreshToday);
            stopMidnightCheck();
        };
    });

    const todayKey = $derived(localDateKey(todayDate));
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
            <WeekStrip
                selected={selectedDate}
                {todayKey}
                onSelect={(d) => (userSelected = d)}
                {activityFor}
            />
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
</main>
