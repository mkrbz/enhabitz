<script lang="ts">
    import {
        habits,
        addHabit,
        replaceHabit,
        deleteHabit,
        formatTime,
    } from "$lib/habits";
    import type { Habit } from "$lib/types";
    import HabitForm from "$lib/components/HabitForm.svelte";
    import { Button } from "$lib/components/ui/button";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Pencil, Trash2, Plus } from "@lucide/svelte";

    let dialogOpen = $state(false);
    let confirmDeleteId = $state<number | null>(null);
    let editingHabit = $state<Habit | null>(null);
    let showDrafts = $state(true);

    const visibleHabits = $derived(
        showDrafts ? habits : habits.filter((h) => h.startDate !== null),
    );

    function openAdd() {
        editingHabit = null;
        dialogOpen = true;
    }

    function openEdit(habit: Habit) {
        editingHabit = habit;
        dialogOpen = true;
    }

    function handleSubmit(data: Omit<Habit, "id">) {
        if (editingHabit) {
            replaceHabit(editingHabit.id, data);
        } else {
            addHabit(data);
        }
        dialogOpen = false;
    }

    function confirmDelete(id: number) {
        confirmDeleteId = id;
    }

    function handleDelete() {
        if (confirmDeleteId !== null) {
            deleteHabit(confirmDeleteId);
            confirmDeleteId = null;
        }
    }

    function habitSummary(habit: Habit): string {
        switch (habit.type) {
            case "todo":
                return "Checkbox";
            case "counter":
                return habit.sets
                    ? `${habit.target} reps × ${habit.sets} sets`
                    : `${habit.target} reps`;
            case "timer":
                return formatTime(habit.targetSeconds);
            case "counter-timer":
                return `${habit.rounds} × ${formatTime(habit.secondsPerRound)}`;
        }
    }

    const WEEK_DAYS_SHORT = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

    function scheduleSummary(habit: Habit): string {
        if (!habit.startDate) return "";
        switch (habit.repeatType) {
            case "daily":
                return "Every day";
            case "weekly": {
                const days = (habit.repeatDays ?? []).map(
                    (d) => WEEK_DAYS_SHORT[d],
                );
                return days.length > 0 ? days.join(" · ") : "Weekly";
            }
            case "monthly": {
                const days = (habit.repeatDays ?? []).sort((a, b) => a - b);
                return days.length > 0
                    ? `Monthly: ${days.join(", ")}`
                    : "Monthly";
            }
            case "interval":
                return `Every ${habit.repeatEvery ?? 1} days`;
        }
    }

    const TYPE_LABEL: Record<Habit["type"], string> = {
        todo: "Todo",
        counter: "Counter",
        timer: "Timer",
        "counter-timer": "Sets",
    };

    const TYPE_COLOR: Record<Habit["type"], string> = {
        todo: "bg-muted text-muted-foreground",
        counter: "bg-blue-500/15 text-blue-400",
        timer: "bg-amber-500/15 text-amber-400",
        "counter-timer": "bg-purple-500/15 text-purple-400",
    };

    const draftCount = $derived(habits.filter((h) => !h.startDate).length);
</script>

<main class="flex flex-col w-full max-w-lg mx-auto px-4 py-6">
    <div class="flex items-center justify-between mb-4">
        <h1 class="text-2xl font-bold">Habits</h1>
        <Button size="sm" onclick={openAdd}>
            <Plus class="h-4 w-4 mr-1" />
            Add habit
        </Button>
    </div>

    <!-- Filter -->
    {#if draftCount > 0}
        <div class="flex items-center gap-2 mb-4">
            <button
                class={`text-xs px-2.5 py-1 rounded-full font-medium transition-colors
                    ${showDrafts ? "bg-muted text-foreground" : "text-muted-foreground hover:text-foreground"}`}
                onclick={() => (showDrafts = !showDrafts)}
            >
                {showDrafts
                    ? `Hide drafts (${draftCount})`
                    : `Show drafts (${draftCount})`}
            </button>
        </div>
    {/if}

    {#if visibleHabits.length === 0}
        <div
            class="flex flex-col items-center justify-center py-16 text-muted-foreground gap-2"
        >
            <p class="text-sm">No habits yet.</p>
            <Button variant="outline" size="sm" onclick={openAdd}
                >Add your first habit</Button
            >
        </div>
    {:else}
        <div class="flex flex-col divide-y divide-border">
            {#each visibleHabits as habit (habit.id)}
                <div class="group flex items-center gap-3 py-3 px-1">
                    <!-- Type badge -->
                    <span
                        class={`text-xs font-medium px-2 py-0.5 rounded-full shrink-0 ${TYPE_COLOR[habit.type]}`}
                    >
                        {TYPE_LABEL[habit.type]}
                    </span>

                    <div class="flex flex-col flex-1 min-w-0">
                        <div class="flex items-center gap-2">
                            <span class="text-sm font-medium truncate"
                                >{habit.label}</span
                            >
                            {#if !habit.startDate}
                                <span
                                    class="text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground shrink-0"
                                >
                                    Draft
                                </span>
                            {/if}
                        </div>
                        <span class="text-xs text-muted-foreground">
                            {habitSummary(habit)}{#if habit.startDate}
                                · {scheduleSummary(habit)}{/if}
                        </span>
                    </div>

                    <div
                        class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity"
                    >
                        <Button
                            variant="ghost"
                            size="icon"
                            class="h-7 w-7"
                            onclick={() => openEdit(habit)}
                        >
                            <Pencil class="h-3.5 w-3.5" />
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon"
                            class="h-7 w-7 text-destructive hover:text-destructive"
                            onclick={() => confirmDelete(habit.id)}
                        >
                            <Trash2 class="h-3.5 w-3.5" />
                        </Button>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</main>

<!-- Add / Edit dialog -->
<Dialog.Root bind:open={dialogOpen}>
    <Dialog.Content class="max-w-sm max-h-[90vh] overflow-y-auto">
        <Dialog.Header>
            <Dialog.Title
                >{editingHabit ? "Edit habit" : "New habit"}</Dialog.Title
            >
        </Dialog.Header>
        <HabitForm
            habit={editingHabit}
            onsubmit={handleSubmit}
            oncancel={() => (dialogOpen = false)}
        />
    </Dialog.Content>
</Dialog.Root>

<!-- Delete confirmation dialog -->
<Dialog.Root
    open={confirmDeleteId !== null}
    onOpenChange={(o) => {
        if (!o) confirmDeleteId = null;
    }}
>
    <Dialog.Content class="max-w-xs">
        <Dialog.Header>
            <Dialog.Title>Delete habit?</Dialog.Title>
            <Dialog.Description>
                "{habits.find((h) => h.id === confirmDeleteId)?.label}" will be
                permanently removed.
            </Dialog.Description>
        </Dialog.Header>
        <Dialog.Footer>
            <Button variant="outline" onclick={() => (confirmDeleteId = null)}
                >Cancel</Button
            >
            <Button variant="destructive" onclick={handleDelete}>Delete</Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
