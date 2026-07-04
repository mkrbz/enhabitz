# Enhabitz

A habit tracking app for desktop and Android, built with Tauri 2, SvelteKit, and TypeScript.

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

## Android

Android support goes through the same Tauri project — `src-tauri/gen/android` is generated once and checked in, so there's no separate app/repo to maintain.

### Prerequisites

- Android SDK + NDK installed, with `ANDROID_HOME` (and `NDK_HOME`, if not auto-detected) set.
- The Rust Android targets:
  ```bash
  rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
  ```
- A device with USB debugging enabled and connected (`adb devices` should list it), or an emulator.

### Dev build

```bash
bun tauri android dev
```

Builds, installs, and launches a debug build on the connected device/emulator with live reload from the Vite dev server — same workflow as `bun tauri dev` on desktop.

### Release build

```bash
bun tauri android build
```

Produces per-ABI and universal APKs (and an AAB) under `src-tauri/gen/android/app/build/outputs/`. This uses the release Cargo profile (LTO, stripped, R8/ProGuard-minified) for a meaningfully smaller/lighter build than the dev one — see `tasks/00-overview.md` for the full battery/performance optimization rationale.

The release build type currently signs with the same debug keystore as `bun tauri android dev` (see the comment in `src-tauri/gen/android/app/build.gradle.kts`), so it installs over an existing dev install with no data loss:

```bash
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
```

Switch to a real release keystore (see [Tauri's Android signing docs](https://v2.tauri.app/distribute/sign/android/)) before ever publishing this or installing across multiple devices that need to receive updates independently.

## Recommended IDE

VS Code with the [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode), [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode), and [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extensions.
