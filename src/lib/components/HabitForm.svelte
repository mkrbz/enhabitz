<script lang="ts">
    import type {
        CounterHabit,
        CounterTimerHabit,
        Habit,
        HabitType,
        TimerHabit,
        TodoHabit,
    } from "$lib/types";
    import { formatTime } from "$lib/habits";
    import { Button } from "$lib/components/ui/button";
    import { Label } from "$lib/components/ui/label";

    let {
        habit = null,
        onsubmit,
        oncancel,
    }: {
        habit?: Habit | null;
        onsubmit: (data: Omit<Habit, "id">) => void;
        oncancel: () => void;
    } = $props();

    const TYPES: { value: HabitType; label: string }[] = [
        { value: "todo", label: "Todo" },
        { value: "counter", label: "Counter" },
        { value: "timer", label: "Timer" },
        { value: "counter-timer", label: "Sets" },
    ];

    // ── Form state ────────────────────────────────────────────────────────────

    // svelte-ignore state_referenced_locally
    let label = $state(habit?.label ?? "");
    // svelte-ignore state_referenced_locally
    let type = $state<HabitType>(habit?.type ?? "todo");

    // counter
    // svelte-ignore state_referenced_locally
    let target = $state(habit?.type === "counter" ? habit.target : 10);
    let hasSets = $state(
        // svelte-ignore state_referenced_locally
        habit?.type === "counter" ? (habit.sets ?? 0) > 0 : false,
    );
    // svelte-ignore state_referenced_locally
    let sets = $state(habit?.type === "counter" ? (habit.sets ?? 2) : 2);

    // timer
    let timerMinutes = $state(
        // svelte-ignore state_referenced_locally
        habit?.type === "timer" ? Math.floor(habit.targetSeconds / 60) : 2,
    );
    let timerSeconds = $state(
        // svelte-ignore state_referenced_locally
        habit?.type === "timer" ? habit.targetSeconds % 60 : 0,
    );

    // counter-timer
    // svelte-ignore state_referenced_locally
    let rounds = $state(habit?.type === "counter-timer" ? habit.rounds : 3);
    let roundMinutes = $state(
        // svelte-ignore state_referenced_locally
        habit?.type === "counter-timer"
            ? Math.floor(habit.secondsPerRound / 60)
            : 0,
    );
    let roundSeconds = $state(
        // svelte-ignore state_referenced_locally
        habit?.type === "counter-timer" ? habit.secondsPerRound % 60 : 30,
    );

    // ── Derived ───────────────────────────────────────────────────────────────

    const totalTimerSeconds = $derived(timerMinutes * 60 + timerSeconds);
    const totalRoundSeconds = $derived(roundMinutes * 60 + roundSeconds);
    const totalCounterTimerSeconds = $derived(rounds * totalRoundSeconds);

    const isValid = $derived(
        label.trim().length > 0 &&
            (type !== "timer" || totalTimerSeconds > 0) &&
            (type !== "counter-timer" || totalRoundSeconds > 0) &&
            (type !== "counter" || target > 0),
    );

    // ── Submit ────────────────────────────────────────────────────────────────

    function submit() {
        if (!isValid) return;

        let data: Omit<Habit, "id">;

        switch (type) {
            case "todo":
                data = { type, label: label.trim(), done: false } as TodoHabit;
                break;
            case "counter":
                data = {
                    type,
                    label: label.trim(),
                    count: 0,
                    target,
                    ...(hasSets && sets > 1 ? { sets, completedSets: 0 } : {}),
                } as CounterHabit;
                break;
            case "timer":
                data = {
                    type,
                    label: label.trim(),
                    targetSeconds: totalTimerSeconds,
                    secondsElapsed: 0,
                    isRunning: false,
                } as TimerHabit;
                break;
            case "counter-timer":
                data = {
                    type,
                    label: label.trim(),
                    rounds,
                    secondsPerRound: totalRoundSeconds,
                    currentRound: 0,
                    roundSecondsElapsed: 0,
                    isRunning: false,
                } as CounterTimerHabit;
                break;
        }

        onsubmit(data!);
    }
</script>

<div class="flex flex-col gap-5">
    <!-- Label -->
    <div class="flex flex-col gap-1.5">
        <Label for="habit-label">Name</Label>
        <input
            id="habit-label"
            type="text"
            bind:value={label}
            placeholder="e.g. Morning run"
            class="rounded-md border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-ring"
            onkeydown={(e) => e.key === "Enter" && submit()}
        />
    </div>

    <!-- Type selector -->
    <div class="flex flex-col gap-1.5">
        <Label>Type</Label>
        <div class="grid grid-cols-4 gap-1 rounded-lg border border-border p-1">
            {#each TYPES as t}
                <button
                    class={`rounded-md py-1.5 text-sm font-medium transition-colors
                        ${
                            type === t.value
                                ? "bg-primary text-primary-foreground"
                                : "text-muted-foreground hover:text-foreground hover:bg-muted"
                        }`}
                    onclick={() => (type = t.value)}
                >
                    {t.label}
                </button>
            {/each}
        </div>
    </div>

    <!-- Type-specific fields -->
    {#if type === "counter"}
        <div class="flex flex-col gap-3">
            <div class="flex items-center gap-3">
                <div class="flex flex-col gap-1.5 flex-1">
                    <Label for="counter-target">Target reps</Label>
                    <input
                        id="counter-target"
                        type="number"
                        min="1"
                        bind:value={target}
                        class="rounded-md border border-input bg-transparent px-3 py-2 text-sm text-center font-mono focus:outline-none focus:ring-1 focus:ring-ring"
                    />
                </div>
            </div>

            <div class="flex items-center gap-2">
                <input
                    id="has-sets"
                    type="checkbox"
                    bind:checked={hasSets}
                    class="h-4 w-4 rounded border border-input"
                />
                <Label for="has-sets" class="cursor-pointer">Track sets</Label>
            </div>

            {#if hasSets}
                <div class="flex flex-col gap-1.5">
                    <Label for="counter-sets">Number of sets</Label>
                    <input
                        id="counter-sets"
                        type="number"
                        min="2"
                        bind:value={sets}
                        class="rounded-md border border-input bg-transparent px-3 py-2 text-sm text-center font-mono focus:outline-none focus:ring-1 focus:ring-ring"
                    />
                </div>
            {/if}
        </div>
    {:else if type === "timer"}
        <div class="flex flex-col gap-1.5">
            <Label>Target duration</Label>
            <div class="flex items-center gap-2">
                <input
                    type="number"
                    min="0"
                    bind:value={timerMinutes}
                    class="w-20 rounded-md border border-input bg-transparent px-3 py-2 text-sm text-center font-mono focus:outline-none focus:ring-1 focus:ring-ring"
                />
                <span class="text-muted-foreground text-sm">min</span>
                <input
                    type="number"
                    min="0"
                    max="59"
                    bind:value={timerSeconds}
                    class="w-20 rounded-md border border-input bg-transparent px-3 py-2 text-sm text-center font-mono focus:outline-none focus:ring-1 focus:ring-ring"
                />
                <span class="text-muted-foreground text-sm">sec</span>
                {#if totalTimerSeconds > 0}
                    <span class="text-xs text-muted-foreground ml-1"
                        >= {formatTime(totalTimerSeconds)}</span
                    >
                {/if}
            </div>
        </div>
    {:else if type === "counter-timer"}
        <div class="flex flex-col gap-3">
            <div class="flex flex-col gap-1.5">
                <Label for="ct-rounds">Rounds</Label>
                <input
                    id="ct-rounds"
                    type="number"
                    min="1"
                    bind:value={rounds}
                    class="w-20 rounded-md border border-input bg-transparent px-3 py-2 text-sm text-center font-mono focus:outline-none focus:ring-1 focus:ring-ring"
                />
            </div>

            <div class="flex flex-col gap-1.5">
                <Label>Duration per round</Label>
                <div class="flex items-center gap-2">
                    <input
                        type="number"
                        min="0"
                        bind:value={roundMinutes}
                        class="w-20 rounded-md border border-input bg-transparent px-3 py-2 text-sm text-center font-mono focus:outline-none focus:ring-1 focus:ring-ring"
                    />
                    <span class="text-muted-foreground text-sm">min</span>
                    <input
                        type="number"
                        min="0"
                        max="59"
                        bind:value={roundSeconds}
                        class="w-20 rounded-md border border-input bg-transparent px-3 py-2 text-sm text-center font-mono focus:outline-none focus:ring-1 focus:ring-ring"
                    />
                    <span class="text-muted-foreground text-sm">sec</span>
                </div>
            </div>

            {#if totalRoundSeconds > 0 && rounds > 0}
                <p class="text-xs text-muted-foreground">
                    {rounds} × {formatTime(totalRoundSeconds)} = {formatTime(
                        totalCounterTimerSeconds,
                    )} total
                </p>
            {/if}
        </div>
    {/if}

    <!-- Actions -->
    <div class="flex justify-end gap-2 pt-1">
        <Button variant="outline" onclick={oncancel}>Cancel</Button>
        <Button onclick={submit} disabled={!isValid}>
            {habit ? "Save" : "Add habit"}
        </Button>
    </div>
</div>
