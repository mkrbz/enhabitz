<script lang="ts">
    import type { CounterHabit } from "$lib/types";
    import { increment, decrement, setCount } from "$lib/habits";
    import { Button } from "$lib/components/ui/button";
    import * as Dialog from "$lib/components/ui/dialog";

    let { habit }: { habit: CounterHabit } = $props();

    let dialogOpen = $state(false);
    let manualCount = $state("");
    let manualSets = $state("");

    const hasSets = $derived(habit.sets !== undefined && habit.sets > 1);
    const completedSets = $derived(habit.completedSets ?? 0);
    // done = all sets complete (with sets) or at/above target (no sets, visual only)
    const done = $derived(
        hasSets ? completedSets >= habit.sets! : habit.count >= habit.target,
    );
    // + is only blocked when sets are all complete; without sets it's always enabled
    const canIncrement = $derived(hasSets ? !done : true);

    function openDialog() {
        manualCount = String(habit.count);
        manualSets = String(completedSets);
        dialogOpen = true;
    }

    function applyManual() {
        const count = parseInt(manualCount, 10);
        const sets = parseInt(manualSets, 10);
        setCount(
            habit.id,
            isNaN(count) ? 0 : count,
            isNaN(sets) ? undefined : sets,
        );
        dialogOpen = false;
    }
</script>

<div class="flex items-center gap-3 py-2 px-1">
    <span
        class={`flex-1 text-base select-none ${done ? "line-through text-muted-foreground" : ""}`}
    >
        {habit.label}
    </span>

    <div class="flex items-center gap-1">
        <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7"
            disabled={habit.count <= 0 && completedSets <= 0}
            onclick={() => decrement(habit.id)}
        >
            <span class="text-lg leading-none">−</span>
        </Button>

        <button
            class="min-w-fit text-center text-sm font-mono tabular-nums hover:bg-muted rounded px-1.5 py-0.5 transition-colors"
            onclick={openDialog}
        >
            {#if hasSets}
                <span class={done ? "text-muted-foreground" : ""}
                    >{completedSets}</span
                >
                <span class="text-muted-foreground">/{habit.sets}s</span>
                <span class="text-muted-foreground mx-1">·</span>
                <span class={done ? "text-muted-foreground" : ""}
                    >{habit.count}</span
                >
                <span class="text-muted-foreground">/{habit.target}</span>
            {:else}
                <span class={done ? "text-muted-foreground" : ""}
                    >{habit.count}</span
                >
                <span class="text-muted-foreground">/{habit.target}</span>
            {/if}
        </button>

        <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7"
            disabled={!canIncrement}
            onclick={() => increment(habit.id)}
        >
            <span class="text-lg leading-none">+</span>
        </Button>
    </div>
</div>

<Dialog.Root bind:open={dialogOpen}>
    <Dialog.Content class="max-w-xs">
        <Dialog.Header>
            <Dialog.Title>{habit.label}</Dialog.Title>
            <Dialog.Description>Enter values manually.</Dialog.Description>
        </Dialog.Header>

        <div class="flex flex-col gap-3 py-2">
            {#if hasSets}
                <div class="flex items-center gap-2">
                    <span class="text-sm text-muted-foreground w-20"
                        >Sets done</span
                    >
                    <input
                        type="number"
                        min="0"
                        max={habit.sets}
                        bind:value={manualSets}
                        class="flex-1 rounded-md border border-input bg-transparent px-3 py-2 text-center text-xl font-mono tabular-nums focus:outline-none focus:ring-1 focus:ring-ring"
                    />
                    <span class="text-muted-foreground">/ {habit.sets}</span>
                </div>
            {/if}
            <div class="flex items-center gap-2">
                <span class="text-sm text-muted-foreground w-20"
                    >{hasSets ? "Current" : "Count"}</span
                >
                <input
                    type="number"
                    min="0"
                    bind:value={manualCount}
                    class="flex-1 rounded-md border border-input bg-transparent px-3 py-2 text-center text-xl font-mono tabular-nums focus:outline-none focus:ring-1 focus:ring-ring"
                    onkeydown={(e) => e.key === "Enter" && applyManual()}
                />
                <span class="text-muted-foreground">/ {habit.target}</span>
            </div>
        </div>

        <Dialog.Footer>
            <Button variant="outline" onclick={() => (dialogOpen = false)}
                >Cancel</Button
            >
            <Button onclick={applyManual}>Set</Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
