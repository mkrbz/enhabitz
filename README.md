# Enhabitz

A habit tracking desktop app built with Tauri 2, SvelteKit, and TypeScript.

## Stack

- **[Tauri 2](https://tauri.app/)** — desktop shell with Rust backend
- **[SvelteKit](https://kit.svelte.dev/) + Svelte 5** — frontend with runes-based reactivity
- **[shadcn-svelte](https://shadcn-svelte.com/)** — UI components
- **[Tailwind CSS v4](https://tailwindcss.com/)** — styling
- **[Bun](https://bun.sh/)** — package manager and runtime
- **[rusqlite](https://github.com/rusqlite/rusqlite)** — SQLite persistence via Rust (bundled, no external dependency)

## Habit types

| Type | Description | Display |
|---|---|---|
| `todo` | Simple checkbox | ☐ label |
| `counter` | Count toward a target, optionally in sets | `3 / 20` or `1/3 sets · 3/20` |
| `timer` | Stopwatch with a target duration | `1:23 / 2:00` |
| `counter-timer` | N timed rounds | `Round 2/3 · 0:18 / 0:30` |

## Scheduling

Every habit has a **start date** and a **repeat rule**:

| Repeat | Behaviour |
|---|---|
| Daily | Active every day from start date |
| Weekly | Active on selected days of the week (Sun–Sat) |
| Monthly | Active on selected days of the month (1–31) |
| Every N days | Active every N days counting from the start date |

Habits with no start date are saved as **drafts** — they appear in the Habits list but not on the Today view.

## Architecture

The Rust backend owns the SQLite database. The frontend never touches the DB directly — all reads and writes go through typed Tauri commands (`load_habits`, `add_habit`, `update_habit`, `delete_habit`, `save_log`). Daily scheduling (`is_active_today`) is computed in Rust using `chrono` and returned with each habit record.

## Dev setup

```bash
bun install
bun tauri dev
```

## Build

```bash
bun tauri build
```

### macOS — "damaged and can't be opened"

Downloaded builds are blocked by Gatekeeper because the app isn't notarized. Strip the quarantine attribute to fix it:

```bash
xattr -cr /Applications/enhabitz.app
```

## Recommended IDE

VS Code with the [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode), [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode), and [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extensions.
