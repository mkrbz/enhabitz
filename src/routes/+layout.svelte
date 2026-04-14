<script lang="ts">
    import "../app.css";
    import { onMount } from "svelte";
    import { page } from "$app/stores";
    import { CalendarCheck, ListTodo, Settings } from "@lucide/svelte";
    import {
        initHabits,
        checkAndResetIfNewDay,
        refreshHistory,
    } from "$lib/habits";

    let { children } = $props();

    let dark = $state(localStorage.getItem("theme") !== "light");

    $effect(() => {
        document.documentElement.classList.toggle("dark", dark);
        localStorage.setItem("theme", dark ? "dark" : "light");
    });

    onMount(() => {
        initHabits();
        refreshHistory();
        window.addEventListener("focus", checkAndResetIfNewDay);
        const interval = setInterval(checkAndResetIfNewDay, 60_000);
        return () => {
            window.removeEventListener("focus", checkAndResetIfNewDay);
            clearInterval(interval);
        };
    });

    const NAV = [
        { href: "/", label: "Today", icon: CalendarCheck },
        { href: "/habits", label: "Habits", icon: ListTodo },
        { href: "/settings", label: "Settings", icon: Settings },
    ];

    const isWidget = $derived($page.url.pathname === "/widget");
</script>

{#if isWidget}
    <div class="h-screen overflow-hidden">
        {@render children()}
    </div>
{:else}
    <div class="flex flex-col h-screen">
        <header
            class="border-b border-border px-4 py-3 flex items-center justify-between shrink-0"
        >
            <span class="font-bold tracking-tight">Enhabitz</span>
            <nav class="flex gap-1">
                {#each NAV as item}
                    {@const active = $page.url.pathname === item.href}
                    <a
                        href={item.href}
                        class={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm font-medium transition-colors
                            ${
                                active
                                    ? "bg-primary text-primary-foreground"
                                    : "text-muted-foreground hover:text-foreground hover:bg-muted"
                            }`}
                    >
                        <item.icon class="h-3.5 w-3.5" />
                        {item.label}
                    </a>
                {/each}
            </nav>
        </header>

        <div class="flex-1 overflow-y-auto">
            {@render children()}
        </div>
    </div>
{/if}
