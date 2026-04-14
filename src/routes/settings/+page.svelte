<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { Button } from "$lib/components/ui/button";
    import { Keyboard } from "@lucide/svelte";

    let currentShortcut = $state("");
    let inputValue = $state("");
    let recording = $state(false);
    let status = $state<"idle" | "saved" | "error">("idle");
    let errorMsg = $state("");

    onMount(async () => {
        currentShortcut = await invoke<string>("get_shortcut");
        inputValue = currentShortcut;
    });

    function startRecording() {
        inputValue = "";
        recording = true;
        status = "idle";
    }

    function stopRecording() {
        recording = false;
        if (!inputValue) inputValue = currentShortcut;
    }

    function keydownHandler(e: KeyboardEvent) {
        if (!recording) return;
        e.preventDefault();

        // Ignore lone modifier keys
        if (["Control", "Shift", "Alt", "Meta", "Super"].includes(e.key))
            return;

        const parts: string[] = [];
        if (e.ctrlKey || e.metaKey) parts.push("CommandOrControl");
        if (e.altKey) parts.push("Alt");
        if (e.shiftKey) parts.push("Shift");

        // Map key to Tauri accelerator key name
        const key = mapKey(e.key, e.code);
        parts.push(key);

        inputValue = parts.join("+");
        recording = false;
    }

    function mapKey(key: string, code: string): string {
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
        // F-keys
        if (/^F\d+$/.test(key)) return key;
        // Single printable char → uppercase
        if (key.length === 1) return key.toUpperCase();
        return key;
    }

    async function save() {
        try {
            await invoke("set_shortcut", { shortcut: inputValue });
            currentShortcut = inputValue;
            status = "saved";
            setTimeout(() => (status = "idle"), 2000);
        } catch (e) {
            errorMsg = String(e);
            status = "error";
        }
    }

    async function reset() {
        inputValue = "CommandOrControl+Shift+H";
        await save();
    }
</script>

<svelte:window onkeydown={keydownHandler} />

<div class="p-6 max-w-lg">
    <h1 class="text-xl font-semibold mb-6">Settings</h1>

    <section class="space-y-4">
        <div
            class="flex items-center gap-2 text-sm font-medium text-muted-foreground uppercase tracking-wider"
        >
            <Keyboard class="h-4 w-4" />
            Global Shortcut
        </div>

        <p class="text-sm text-muted-foreground">
            Press this shortcut anywhere to toggle the widget.
        </p>

        <div class="flex items-center gap-3">
            <button
                class={`flex-1 px-4 py-2 rounded-md border text-sm font-mono text-center transition-colors select-none
                    ${
                        recording
                            ? "border-primary bg-primary/10 text-primary animate-pulse cursor-crosshair"
                            : "border-border bg-muted text-foreground cursor-pointer hover:border-primary"
                    }`}
                onclick={recording ? stopRecording : startRecording}
            >
                {recording
                    ? "Press your shortcut…"
                    : inputValue || "Click to record"}
            </button>

            <Button
                onclick={save}
                disabled={inputValue === currentShortcut || !inputValue}
            >
                Save
            </Button>

            <Button variant="outline" onclick={reset} title="Reset to default">
                Reset
            </Button>
        </div>

        {#if status === "saved"}
            <p class="text-sm text-green-500">Shortcut saved.</p>
        {:else if status === "error"}
            <p class="text-sm text-red-500">Error: {errorMsg}</p>
        {/if}

        <p class="text-xs text-muted-foreground">
            Current: <span class="font-mono">{currentShortcut}</span>
        </p>
    </section>
</div>
