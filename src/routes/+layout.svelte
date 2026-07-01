<script lang="ts">
    import "../app.css";
    import { onMount } from "svelte";
    import { page } from "$app/stores";
    import { listen } from "@tauri-apps/api/event";
    import { isMobile } from "$lib/platform";
    import { themeStore } from "$lib/theme.svelte";
    import { NAV } from "$lib/components/nav/nav-items";
    import DesktopNav from "$lib/components/nav/DesktopNav.svelte";
    import MobileNav from "$lib/components/nav/MobileNav.svelte";
    import {
        initHabits,
        refreshHabits,
        checkAndResetIfNewDay,
        refreshHistory,
    } from "$lib/habits";

    let { children } = $props();

    $effect(() => {
        document.documentElement.classList.toggle("dark", themeStore.isDark);
    });

    onMount(() => {
        initHabits();
        refreshHistory();
        window.addEventListener("focus", checkAndResetIfNewDay);
        const interval = setInterval(checkAndResetIfNewDay, 60_000);

        let unlistenHabitsChanged: (() => void) | undefined;
        listen("habits-changed", async () => {
            await refreshHabits();
            await refreshHistory();
        }).then((fn) => {
            unlistenHabitsChanged = fn;
        });

        return () => {
            window.removeEventListener("focus", checkAndResetIfNewDay);
            clearInterval(interval);
            unlistenHabitsChanged?.();
        };
    });

    const isWidget = $derived($page.url.pathname === "/widget");
</script>

{#if isWidget}
    <div class="h-screen overflow-hidden">
        {@render children()}
    </div>
{:else if isMobile}
    <div class="flex flex-col h-screen">
        <div class="flex-1 overflow-y-auto pt-[env(safe-area-inset-top)]">
            {@render children()}
        </div>
        <MobileNav items={NAV} />
    </div>
{:else}
    <div class="flex flex-col h-screen">
        <header
            class="border-b border-border px-4 py-3 flex items-center justify-between shrink-0"
        >
            <span class="font-bold tracking-tight">Enhabitz</span>
            <DesktopNav items={NAV} />
        </header>

        <div class="flex-1 overflow-y-auto">
            {@render children()}
        </div>
    </div>
{/if}
