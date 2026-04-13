<script lang="ts">
    import type { CounterTimerHabit } from "$lib/types";
    import { startCounterTimer, stopCounterTimer, setCounterTimerRound, resetCounterTimer, formatTime } from "$lib/habits";
    import { Button } from "$lib/components/ui/button";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Play, Pause, RotateCcw } from "@lucide/svelte";

    let { habit }: { habit: CounterTimerHabit } = $props();

    let dialogOpen = $state(false);

    // Local display values — derived from Date.now() - startedAt while running
    let displayRound = $state(habit.currentRound);
    let displayRoundSeconds = $state(habit.roundSecondsElapsed);

    const done = $derived(displayRound >= habit.rounds);
    const roundProgress = $derived(
        Math.min(Math.round((displayRoundSeconds / habit.secondsPerRound) * 100), 100)
    );

    $effect(() => {
        if (!habit.isRunning || habit.startedAt === undefined) {
            displayRound = habit.currentRound;
            displayRoundSeconds = habit.roundSecondsElapsed;
            return;
        }
        const startedAt = habit.startedAt;
        const totalTarget = habit.rounds * habit.secondsPerRound;

        const id = setInterval(() => {
            const totalElapsed = Math.max(0, Math.floor((Date.now() - startedAt) / 1000));
            if (totalElapsed >= totalTarget) {
                displayRound = habit.rounds;
                displayRoundSeconds = 0;
                stopCounterTimer(habit.id); // auto-complete
                return;
            }
            displayRound = Math.floor(totalElapsed / habit.secondsPerRound);
            displayRoundSeconds = totalElapsed % habit.secondsPerRound;
        }, 200);

        return () => {
            clearInterval(id);
            if (habit.isRunning) stopCounterTimer(habit.id);
        };
    });

    function toggleTimer() {
        if (habit.isRunning) {
            stopCounterTimer(habit.id);
        } else {
            startCounterTimer(habit.id);
        }
    }
</script>

<div class="flex items-center gap-3 py-2 px-1">
    <span class={`flex-1 text-base select-none ${done ? "line-through text-muted-foreground" : ""}`}>
        {habit.label}
    </span>

    <div class="flex items-center gap-2">
        <button
            class="text-sm font-mono tabular-nums hover:bg-muted rounded px-1 py-0.5 transition-colors"
            onclick={() => (dialogOpen = true)}
        >
            <span class={done ? "text-muted-foreground" : ""}>{displayRound}</span>
            <span class="text-muted-foreground">/{habit.rounds}r</span>
            <span class="text-muted-foreground mx-1">·</span>
            <span class={done ? "text-muted-foreground" : ""}>{formatTime(displayRoundSeconds)}</span>
            <span class="text-muted-foreground">/{formatTime(habit.secondsPerRound)}</span>
        </button>

        <Button variant="ghost" size="icon" class="h-7 w-7" onclick={toggleTimer} disabled={done}>
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
            <Dialog.Description>
                {habit.rounds} rounds × {formatTime(habit.secondsPerRound)} each
            </Dialog.Description>
        </Dialog.Header>

        <div class="flex flex-col items-center gap-6 py-4">
            <div class="flex gap-2">
                {#each { length: habit.rounds } as _, i}
                    <button
                        class={`h-8 w-8 rounded-full text-sm font-semibold transition-colors
                            ${i < displayRound
                                ? "bg-primary text-primary-foreground"
                                : i === displayRound && !done
                                    ? "border-2 border-primary text-primary"
                                    : "border border-border text-muted-foreground"
                            }`}
                        onclick={() => setCounterTimerRound(habit.id, i)}
                    >
                        {i + 1}
                    </button>
                {/each}
            </div>

            <div class="text-6xl font-mono tabular-nums font-bold">
                {formatTime(displayRoundSeconds)}
            </div>
            <div class="text-sm text-muted-foreground">of {formatTime(habit.secondsPerRound)}</div>

            <div class="w-full h-1.5 bg-muted rounded-full overflow-hidden">
                <div class="h-full bg-primary rounded-full transition-none" style="width: {roundProgress}%"></div>
            </div>

            <div class="flex gap-3">
                <Button variant="outline" size="icon" onclick={() => resetCounterTimer(habit.id)}>
                    <RotateCcw class="h-4 w-4" />
                </Button>
                <Button size="icon" class="h-12 w-12" onclick={toggleTimer} disabled={done}>
                    {#if habit.isRunning}
                        <Pause class="h-5 w-5" />
                    {:else}
                        <Play class="h-5 w-5" />
                    {/if}
                </Button>
            </div>
        </div>
    </Dialog.Content>
</Dialog.Root>
