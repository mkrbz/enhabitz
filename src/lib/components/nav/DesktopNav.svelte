<script lang="ts">
    import { page } from "$app/stores";
    import type { NavItem } from "./nav-items";

    let { items }: { items: NavItem[] } = $props();
</script>

<nav class="flex gap-1">
    {#each items as item}
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
