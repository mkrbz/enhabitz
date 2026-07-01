export type Theme = "device" | "light" | "dark";

const STORAGE_KEY = "theme";

function loadTheme(): Theme {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored === "light" || stored === "dark" ? stored : "device";
}

const media = window.matchMedia("(prefers-color-scheme: dark)");

// A class instance is Svelte 5's pattern for module state that mixes a
// mutable field with a derived one — exporting a reassigned $state binding
// or a bare $derived directly from a module is disallowed, but the instance
// reference itself is stable, so its reactive fields work from any importer.
class ThemeStore {
    current = $state<Theme>(loadTheme());
    systemPrefersDark = $state(media.matches);

    /** Resolves "device" against the live OS preference, so it keeps
     * tracking OS-level theme changes for as long as the user hasn't picked
     * light/dark explicitly (only set() persists a choice). */
    isDark = $derived(
        this.current === "device" ? this.systemPrefersDark : this.current === "dark",
    );

    set(next: Theme) {
        this.current = next;
        localStorage.setItem(STORAGE_KEY, next);
    }
}

export const themeStore = new ThemeStore();

media.addEventListener("change", (e) => {
    themeStore.systemPrefersDark = e.matches;
});
