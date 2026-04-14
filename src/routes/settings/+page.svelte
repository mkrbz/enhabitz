<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { Button } from "$lib/components/ui/button";
    import { Keyboard } from "@lucide/svelte";

    type ShortcutTarget = "widget" | "main";

    interface ShortcutState {
        current: string;
        input: string;
        recording: boolean;
        status: "idle" | "saved" | "error";
        error: string;
    }

    const defaults: Record<ShortcutTarget, string> = {
        widget: "CommandOrControl+Shift+H",
        main: "CommandOrControl+Shift+E",
    };

    let shortcuts = $state<Record<ShortcutTarget, ShortcutState>>({
        widget: {
            current: "",
            input: "",
            recording: false,
            status: "idle",
            error: "",
        },
        main: {
            current: "",
            input: "",
            recording: false,
            status: "idle",
            error: "",
        },
    });

    onMount(async () => {
        for (const target of ["widget", "main"] as ShortcutTarget[]) {
            const val = await invoke<string>("get_shortcut", { target });
            shortcuts[target].current = val;
            shortcuts[target].input = val;
        }
    });

    function startRecording(target: ShortcutTarget) {
        // Stop any other active recording
        for (const t of ["widget", "main"] as ShortcutTarget[]) {
            if (t !== target) shortcuts[t].recording = false;
        }
        shortcuts[target].input = "";
        shortcuts[target].recording = true;
        shortcuts[target].status = "idle";
    }

    function stopRecording(target: ShortcutTarget) {
        shortcuts[target].recording = false;
        if (!shortcuts[target].input)
            shortcuts[target].input = shortcuts[target].current;
    }

    function keydownHandler(e: KeyboardEvent) {
        const target = (["widget", "main"] as ShortcutTarget[]).find(
            (t) => shortcuts[t].recording,
        );
        if (!target) return;
        e.preventDefault();

        if (["Control", "Shift", "Alt", "Meta", "Super"].includes(e.key))
            return;

        const parts: string[] = [];
        if (e.ctrlKey || e.metaKey) parts.push("CommandOrControl");
        if (e.altKey) parts.push("Alt");
        if (e.shiftKey) parts.push("Shift");
        parts.push(mapKey(e.key));

        shortcuts[target].input = parts.join("+");
        shortcuts[target].recording = false;
    }

    function mapKey(key: string): string {
        const map: Record<string, string> = {
            " ": "Space",
            ArrowUp: "Up",
            ArrowDown: "Down",
            ArrowLeft: "Left",
            ArrowRight: "Right",
            Escape: "Escape",
            Enter: "Return",
            Backspace: "Backspace",
            Delete: "Delete",
            Tab: "Tab",
            Home: "Home",
            End: "End",
            PageUp: "PageUp",
            PageDown: "PageDown",
        };
        if (map[key]) return map[key];
        if (/^F\d+$/.test(key)) return key;
        if (key.length === 1) return key.toUpperCase();
        return key;
    }

    async function save(target: ShortcutTarget) {
        try {
            await invoke("set_shortcut", {
                target,
                shortcut: shortcuts[target].input,
            });
            shortcuts[target].current = shortcuts[target].input;
            shortcuts[target].status = "saved";
            setTimeout(() => (shortcuts[target].status = "idle"), 2000);
        } catch (e) {
            shortcuts[target].error = String(e);
            shortcuts[target].status = "error";
        }
    }

    async function reset(target: ShortcutTarget) {
        shortcuts[target].input = defaults[target];
        await save(target);
    }
</script>

<svelte:window onkeydown={keydownHandler} />

<div class="p-6 max-w-lg space-y-8">
    <h1 class="text-xl font-semibold">Settings</h1>

    {#each [{ target: "widget" as ShortcutTarget, label: "Widget Shortcut", desc: "Toggle the floating widget." }, { target: "main" as ShortcutTarget, label: "Main Window Shortcut", desc: "Toggle the main Enhabitz window." }] as item}
        {@const sc = shortcuts[item.target]}
        <section class="space-y-3">
            <div
                class="flex items-center gap-2 text-sm font-medium text-muted-foreground uppercase tracking-wider"
            >
                <Keyboard class="h-4 w-4" />
                {item.label}
            </div>

            <p class="text-sm text-muted-foreground">{item.desc}</p>

            <div class="flex items-center gap-3">
                <button
                    class={`flex-1 px-4 py-2 rounded-md border text-sm font-mono text-center transition-colors select-none
                        ${
                            sc.recording
                                ? "border-primary bg-primary/10 text-primary animate-pulse cursor-crosshair"
                                : "border-border bg-muted text-foreground cursor-pointer hover:border-primary"
                        }`}
                    onclick={() =>
                        sc.recording
                            ? stopRecording(item.target)
                            : startRecording(item.target)}
                >
                    {sc.recording
                        ? "Press your shortcut…"
                        : sc.input || "Click to record"}
                </button>

                <Button
                    onclick={() => save(item.target)}
                    disabled={sc.input === sc.current || !sc.input}
                >
                    Save
                </Button>

                <Button
                    variant="outline"
                    onclick={() => reset(item.target)}
                    title="Reset to default"
                >
                    Reset
                </Button>
            </div>

            {#if sc.status === "saved"}
                <p class="text-sm text-green-500">Shortcut saved.</p>
            {:else if sc.status === "error"}
                <p class="text-sm text-red-500">Error: {sc.error}</p>
            {/if}

            <p class="text-xs text-muted-foreground">
                Current: <span class="font-mono">{sc.current}</span>
            </p>
        </section>
    {/each}
</div>
