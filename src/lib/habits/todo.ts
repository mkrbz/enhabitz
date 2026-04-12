import { find } from "./state.svelte";
import type { TodoHabit } from "$lib/types";

export function toggleTodo(id: number) {
    const h = find<TodoHabit>(id, "todo");
    if (h) h.done = !h.done;
}
