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
</script>

<main class="flex flex-col w-full max-w-lg mx-auto px-4 py-6">
    <div class="flex items-center justify-between mb-6">
        <h1 class="text-2xl font-bold">Habits</h1>
        <Button size="sm" onclick={openAdd}>
            <Plus class="h-4 w-4 mr-1" />
            Add habit
        </Button>
    </div>

    {#if habits.length === 0}
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
            {#each habits as habit (habit.id)}
                <div class="group flex items-center gap-3 py-3 px-1">
                    <span
                        class={`text-xs font-medium px-2 py-0.5 rounded-full shrink-0 ${TYPE_COLOR[habit.type]}`}
                    >
                        {TYPE_LABEL[habit.type]}
                    </span>

                    <div class="flex flex-col flex-1 min-w-0">
                        <span class="text-sm font-medium truncate"
                            >{habit.label}</span
                        >
                        <span class="text-xs text-muted-foreground"
                            >{habitSummary(habit)}</span
                        >
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
    <Dialog.Content class="max-w-sm">
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
