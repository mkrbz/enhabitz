<script lang="ts">
    import type { TodoHabit } from "$lib/types";
    import { toggleTodo } from "$lib/habits";
    import { Checkbox } from "$lib/components/ui/checkbox";
    import { Label } from "$lib/components/ui/label";

    let { habit }: { habit: TodoHabit } = $props();
</script>

<div class="flex items-center justify-between gap-3 py-2 px-1">
    <Label
        for={`habit-${habit.id}`}
        class={`cursor-pointer text-base select-none ${habit.done ? "line-through text-muted-foreground" : ""}`}
    >
        {habit.label}
    </Label>
    <Checkbox
        id={`habit-${habit.id}`}
        checked={habit.done}
        onCheckedChange={() => toggleTodo(habit.id)}
    />
</div>
