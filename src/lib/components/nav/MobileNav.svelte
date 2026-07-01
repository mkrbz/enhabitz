<script lang="ts">
    import { page } from "$app/stores";
    import type { NavItem } from "./nav-items";

    let { items }: { items: NavItem[] } = $props();
</script>

<nav
    class="flex border-t border-border bg-background pb-[env(safe-area-inset-bottom)] shrink-0"
>
    {#each items as item}
        {@const active = $page.url.pathname === item.href}
        <a
            href={item.href}
            aria-label={item.label}
            class={`flex flex-1 items-center justify-center py-3
                ${active ? "text-primary" : "text-muted-foreground"}`}
        >
            <item.icon class="h-6 w-6" />
        </a>
    {/each}
</nav>
