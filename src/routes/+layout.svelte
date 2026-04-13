<script lang="ts">
    import "../app.css";
    import { onMount } from "svelte";
    import { page } from "$app/stores";
    import { CalendarCheck, ListTodo } from "@lucide/svelte";
    import { initHabits } from "$lib/habits";

    let { children } = $props();

    let dark = $state(true);

    $effect(() => {
        document.documentElement.classList.toggle("dark", dark);
    });

    onMount(() => {
        initHabits();
    });

    const NAV = [
        { href: "/", label: "Today", icon: CalendarCheck },
        { href: "/habits", label: "Habits", icon: ListTodo },
    ];
</script>

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
