# Enhabitz

A habit tracking desktop app built with Tauri 2, SvelteKit, and TypeScript.

## Stack

- **[Tauri 2](https://tauri.app/)** — desktop shell
- **[SvelteKit](https://kit.svelte.dev/) + Svelte 5** — frontend with runes-based reactivity
- **[shadcn-svelte](https://shadcn-svelte.com/)** — UI components
- **[Tailwind CSS v4](https://tailwindcss.com/)** — styling
- **[Bun](https://bun.sh/)** — package manager and runtime

## Habit types

| Type | Description | Display |
|---|---|---|
| `todo` | Simple checkbox | ☐ label |
| `counter` | Count toward a target, optionally in sets | `3/20` or `1/3s · 3/20` |
| `timer` | Stopwatch with a target duration | `1:23 / 2:00` |
| `counter-timer` | N rounds, each timed | `2/3r · 0:18/0:30` |

## Dev setup

```bash
bun install
bun tauri dev
```

## Recommended IDE

VS Code with the [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode), [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode), and [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extensions.
