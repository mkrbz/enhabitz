<script lang="ts">
    import { habits, completedCount } from "$lib/habits";
    import TodoHabit from "$lib/components/habits/TodoHabit.svelte";
    import CounterHabit from "$lib/components/habits/CounterHabit.svelte";
    import TimerHabit from "$lib/components/habits/TimerHabit.svelte";
    import CounterTimerHabit from "$lib/components/habits/CounterTimerHabit.svelte";
</script>

<main class="flex flex-col w-full max-w-lg mx-auto px-4 py-6">
    <div class="flex items-baseline justify-between mb-6">
        <h1 class="text-2xl font-bold">Today</h1>
        <span class="text-sm text-muted-foreground">
            {completedCount()}/{habits.length} done
        </span>
    </div>

    <div class="flex flex-col divide-y divide-border">
        {#each habits as habit (habit.id)}
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

    <div class="mt-10 text-sm text-muted-foreground text-center">
        Heatmap coming soon
    </div>
</main>
