<script lang="ts">
    import type { TimerHabit } from "$lib/types";
    import {
        startTimer,
        stopTimer,
        setTimerElapsed,
        resetTimer,
        formatTime,
    } from "$lib/habits";
    import { Button } from "$lib/components/ui/button";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Play, Pause, RotateCcw } from "@lucide/svelte";

    let { habit }: { habit: TimerHabit } = $props();

    let dialogOpen = $state(false);
    let manualMinutes = $state("0");
    let manualSeconds = $state("0");

    // Local display value — time-based, updated every 200ms while running
    let displaySeconds = $state(habit.secondsElapsed);

    const done = $derived(displaySeconds >= habit.targetSeconds);
    const progress = $derived(
        Math.min(Math.round((displaySeconds / habit.targetSeconds) * 100), 100),
    );

    $effect(() => {
        if (!habit.isRunning || habit.startedAt === undefined) {
            displaySeconds = habit.secondsElapsed;
            return;
        }
        const startedAt = habit.startedAt;
        const id = setInterval(() => {
            displaySeconds = Math.max(
                0,
                Math.floor((Date.now() - startedAt) / 1000),
            );
        }, 200);
        // Cleanup: fires when isRunning → false or component is destroyed
        return () => {
            clearInterval(id);
            if (habit.isRunning) stopTimer(habit.id);
        };
    });

    function toggleTimer() {
        if (habit.isRunning) {
            stopTimer(habit.id);
        } else {
            startTimer(habit.id);
        }
    }

    function openDialog() {
        manualMinutes = String(Math.floor(displaySeconds / 60));
        manualSeconds = String(displaySeconds % 60);
        dialogOpen = true;
    }

    function applyManual() {
        const total =
            parseInt(manualMinutes || "0") * 60 +
            parseInt(manualSeconds || "0");
        setTimerElapsed(habit.id, total);
        dialogOpen = false;
    }
</script>

<div class="flex items-center gap-3 py-2 px-1">
    <span
        class={`flex-1 text-base select-none ${done ? "line-through text-muted-foreground" : ""}`}
    >
        {habit.label}
    </span>

    <div class="flex items-center gap-2">
        <button
            class="text-sm font-mono tabular-nums hover:bg-muted rounded px-1 py-0.5 transition-colors min-w-20 text-center"
            onclick={openDialog}
        >
            <span class={done ? "text-muted-foreground" : ""}
                >{formatTime(displaySeconds)}</span
            >
            <span class="text-muted-foreground">
                / {formatTime(habit.targetSeconds)}</span
            >
        </button>

        <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7"
            onclick={toggleTimer}
        >
            {#if habit.isRunning}
                <Pause class="h-3.5 w-3.5" />
            {:else}
                <Play class="h-3.5 w-3.5" />
            {/if}
        </Button>
    </div>
</div>

<Dialog.Root bind:open={dialogOpen}>
    <Dialog.Content class="max-w-sm">
        <Dialog.Header>
            <Dialog.Title>{habit.label}</Dialog.Title>
            <Dialog.Description
                >Target: {formatTime(habit.targetSeconds)}</Dialog.Description
            >
        </Dialog.Header>

        <div class="flex flex-col items-center gap-6 py-4">
            <div class="text-6xl font-mono tabular-nums font-bold">
                {formatTime(displaySeconds)}
            </div>

            <div class="w-full h-1.5 bg-muted rounded-full overflow-hidden">
                <div
                    class="h-full bg-primary rounded-full transition-none"
                    style="width: {progress}%"
                ></div>
            </div>

            <div class="flex gap-3">
                <Button
                    variant="outline"
                    size="icon"
                    onclick={() => resetTimer(habit.id)}
                >
                    <RotateCcw class="h-4 w-4" />
                </Button>
                <Button size="icon" class="h-12 w-12" onclick={toggleTimer}>
                    {#if habit.isRunning}
                        <Pause class="h-5 w-5" />
                    {:else}
                        <Play class="h-5 w-5" />
                    {/if}
                </Button>
            </div>

            <div class="flex items-center gap-2 text-sm text-muted-foreground">
                <span>Set manually:</span>
                <input
                    type="number"
                    min="0"
                    bind:value={manualMinutes}
                    class="w-14 rounded border border-input bg-transparent px-2 py-1 text-center font-mono text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                />
                <span>m</span>
                <input
                    type="number"
                    min="0"
                    max="59"
                    bind:value={manualSeconds}
                    class="w-14 rounded border border-input bg-transparent px-2 py-1 text-center font-mono text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                />
                <span>s</span>
                <Button size="sm" onclick={applyManual}>Set</Button>
            </div>
        </div>
    </Dialog.Content>
</Dialog.Root>
