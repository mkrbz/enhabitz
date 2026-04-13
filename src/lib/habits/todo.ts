import { find } from "./state.svelte";
import { dbSaveLog } from "$lib/db";
import type { TodoHabit } from "$lib/types";

export function toggleTodo(id: number) {
    const h = find<TodoHabit>(id, "todo");
    if (!h) return;
    h.done = !h.done;
    dbSaveLog(h);
}
