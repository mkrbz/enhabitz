<script lang="ts">
    let { children } = $props();

    import "../app.css";
    let dark = $state(true);

    $effect(() => {
        document.documentElement.classList.toggle("dark", dark);
    });

    export function toggleTheme() {
        dark = !dark;
    }
</script>

{@render children()}
