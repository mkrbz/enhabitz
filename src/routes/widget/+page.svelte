<script lang="ts">
    import { onMount } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { habits, completedCount } from "$lib/habits";
    import TodoHabit from "$lib/components/habits/TodoHabit.svelte";
    import CounterHabit from "$lib/components/habits/CounterHabit.svelte";
    import TimerHabit from "$lib/components/habits/TimerHabit.svelte";
    import CounterTimerHabit from "$lib/components/habits/CounterTimerHabit.svelte";
    import { X } from "@lucide/svelte";

    const activeHabits = $derived(habits.filter((h) => h.isActiveToday));

    const today = new Intl.DateTimeFormat("en", {
        weekday: "short",
        month: "short",
        day: "numeric",
    }).format(new Date());

    onMount(() => {
        const win = getCurrentWindow();
        let unlisten: (() => void) | undefined;
        win.onFocusChanged(({ payload: focused }) => {
            if (!focused) win.hide();
        }).then((fn) => {
            unlisten = fn;
        });
        return () => unlisten?.();
    });

    function close() {
        getCurrentWindow().hide();
    }
</script>

<div class="p-2 h-screen box-border">
    <div
        class="flex flex-col h-full bg-background text-foreground border border-border rounded-lg overflow-hidden"
    >
        <!-- Header -->
        <div
            class="flex items-center justify-between px-4 py-3 border-b border-border shrink-0"
        >
            <div>
                <p class="text-xs text-muted-foreground">{today}</p>
                <p class="text-sm font-semibold leading-tight">
                    {completedCount()}/{activeHabits.length} completed
                </p>
            </div>
            <button
                onclick={close}
                class="text-muted-foreground hover:text-foreground transition-colors"
            >
                <X class="h-4 w-4" />
            </button>
        </div>

        <!-- Habits list -->
        <div class="flex-1 overflow-y-auto">
            {#if activeHabits.length === 0}
                <div
                    class="flex items-center justify-center h-full text-muted-foreground text-sm"
                >
                    No habits for today
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
        </div>
    </div>
</div>
