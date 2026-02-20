# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Kithara is a cross-platform desktop soundboard for extracting and playing audio from the game Old World. Built with Tauri 2 (Rust backend) and Svelte 5 (TypeScript frontend).

## Tech Stack

- **Frontend:** Svelte 5 + SvelteKit + Vite + TypeScript (strict mode)
- **Backend:** Rust + Tauri 2 + Tokio
- **Database:** SQLite via rusqlite (bundled)
- **Audio:** rodio (playback), vgmstream-cli (Wwise .wem → .ogg conversion)

## Commands

```bash
npm install                # Install dependencies
npm run tauri dev          # Run full app with hot reload (port 1420)
npm run check              # TypeScript + Svelte type checking
npm run check:watch        # Watch mode type checking
npm run build && npm run tauri build  # Production build
```

For Rust-only changes: `cd src-tauri && cargo build`

## TypeScript Type Generation

TypeScript types are automatically generated from Rust structs using `ts-rs`:

```bash
npm run types:generate    # Regenerate types from Rust models
```

**How it works:**
- Rust structs in `src-tauri/src/models.rs` are annotated with `#[derive(TS)]`
- Running the test exports types to `src/lib/types/`
- `src/lib/api.ts` imports from generated types

**When to regenerate:**
- After adding/modifying structs in `models.rs`
- Types are NOT auto-generated on build (run manually)

**Adding new types:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "../../src/lib/types/"))]
#[serde(rename_all = "camelCase")]
pub struct NewType {
    pub field: String,
}
```

Then add to the test in `models.rs` and run `npm run types:generate`.

**Important:** Never edit files in `src/lib/types/` manually—they're auto-generated.

## ⚠️ Tauri Desktop Environment - NOT a Web Browser

**CRITICAL**: This is a native desktop application built with Tauri, NOT a traditional web application.

### Key Differences from Browser Environment

**What DOESN'T Work:**
- ❌ "Refresh the page" - This is a desktop app with hot-reload during development
- ❌ Browser DevTools shortcuts (F12) - Use the app's development tools
- ❌ Assuming synchronous browser APIs - Many are async in Tauri

**What DOES Work:**
- ✅ Hot reload during `npm run tauri dev` - File changes auto-update
- ✅ Tauri command invocations via `invoke()` - Frontend ↔ Rust backend communication
- ✅ Native OS dialogs - File pickers, confirmations

### Tauri-Specific API Behaviors

**Native Dialogs are Async:**
```typescript
// ❌ WRONG: Assumes browser behavior (synchronous)
const confirmed = window.confirm("Are you sure?");

// ✅ CORRECT: Tauri returns Promise
const confirmed = await window.confirm("Are you sure?");
```

**Development Workflow:**
- `npm run tauri dev` - Runs app with hot reload
- Changes to frontend auto-reload
- Changes to Rust require recompilation (automatic)
- Console logs appear in terminal, not browser DevTools

**When Troubleshooting:**
- Check terminal output for Rust panics/errors
- Check browser console for frontend errors
- Don't suggest "refresh the page" - suggest restarting dev server if needed

## Tauri Built-ins vs Web APIs

**Principle:** Prefer Tauri built-ins for type safety and native integration.

| Context | Use |
|---------|-----|
| OS integration (file pickers, dialogs, menus) | Tauri plugins |
| Security-sensitive (file system, shell) | Tauri APIs |
| Pure UI/logic (no OS interaction) | Web libraries OK |

## Architecture

**IPC Model:** Frontend → Tauri invoke() → Rust #[tauri::command] handlers → backend logic

```
src/                           # Svelte frontend
├── lib/
│   ├── api.ts                # Tauri IPC wrappers (invoke calls)
│   ├── types/                # Generated from Rust (do not edit)
│   ├── stores/               # Svelte 5 runes state ($state, $derived)
│   │   └── sounds.svelte.ts  # soundsState, filterState, playerState
│   └── components/           # UI components (mostly TODO)
├── routes/                   # SvelteKit pages
└── app.css                   # CSS custom properties (dark theme)

src-tauri/src/                # Rust backend
├── lib.rs                    # Tauri app setup
├── commands.rs               # IPC command handlers
└── models.rs                 # Serde-serializable types
```

## Key Files to Read

- **docs/architecture.md** - Complete technical specification (read this first for any significant work)
- **docs/old-world-audio-reference.md** - Wwise audio system details for extraction logic
- **src/lib/api.ts** - TypeScript interfaces and IPC function signatures
- **src-tauri/src/models.rs** - Data types shared between frontend/backend

## Svelte 5 Standards

**CRITICAL**: This project uses Svelte 5. Always use runes, not Svelte 4 patterns.

**Reactive State:**
```typescript
// ✅ CORRECT: Svelte 5 runes
let count = $state(0);
let doubled = $derived(count * 2);

// ❌ WRONG: Svelte 4 patterns
let count = 0;
$: doubled = count * 2;
```

**Props:**
```typescript
// ✅ CORRECT: Svelte 5 props
let { name, age = 0 }: { name: string; age?: number } = $props();

// ❌ WRONG: Svelte 4 export
export let name: string;
```

**Effects:**
```typescript
// ✅ CORRECT: Svelte 5 effect
$effect(() => {
  console.log('count changed:', count);
});

// ❌ WRONG: Svelte 4 reactive statement
$: console.log('count changed:', count);
```

**Effect Dependency Tracking:**

Svelte 5 `$effect` only tracks dependencies at the point they're accessed. If a reactive value is only accessed inside a conditional, it may not be tracked.

```typescript
// ✅ CORRECT: Access reactive values unconditionally
$effect(() => {
  const currentOption = option;  // Always accessed → always tracked
  if (chart && currentOption) {
    chart.setOption(currentOption);
  }
});

// ❌ WRONG: Reactive value only accessed conditionally
$effect(() => {
  if (chart) {
    chart.setOption(option);  // NOT tracked if chart is initially null
  }
});
```

## Code Quality Standards

### Null/Undefined Handling

**Data/Domain Layer (strict):**
- Use `??` for null/undefined checks
- Use explicit `!= null` checks where `0` or `""` are valid
- **NEVER** use `||` in data computation

```typescript
// ✅ CORRECT
const duration = sound.duration ?? 0;
const filtered = sounds.filter(s => s.unitType != null);

// ❌ WRONG
const duration = sound.duration || 0;  // 0 is valid!
```

**UI Rendering (pragmatic):**
- Allow `||` for display fallbacks where falsy values should show fallback

```typescript
// ✅ OK in UI
<span>{sound.displayName || "Unknown"}</span>
```

### API Layer

Use centralized `src/lib/api.ts` for all Tauri backend calls:

```typescript
import { searchSounds } from '$lib/api';

// ✅ CORRECT: Use API layer
const sounds = await searchSounds(query);

// ❌ WRONG: Direct invoke
const sounds = await invoke('search_sounds', { query });
```

## Conventions

- Tauri commands: snake_case in Rust, camelCase in TypeScript (serde converts automatically)
- State management uses Svelte 5 runes, not legacy stores
- No component library—use CSS custom properties from app.css
- Rust commands return `Result<T, String>` for error handling
- Cross-platform paths via `directories::ProjectDirs`

## Cache Locations

- macOS: `~/Library/Application Support/com.kithara.app/`
- Windows: `%APPDATA%\kithara\`
- Linux: `~/.local/share/kithara/`

## Git Workflow

**NEVER commit or push changes automatically.** Only commit/push when explicitly asked by the user. Each commit/push requires explicit direction - previous direction in the session does not carry forward. This allows the user to validate changes before committing.

Do NOT include these lines in commit messages:
- `🤖 Generated with [Claude Code](https://claude.com/claude-code)`
- `Co-Authored-By: Claude <noreply@anthropic.com>`

## Current Status

The core app is functional: extraction pipeline (BNK parsing, .wem extraction, vgmstream conversion), sound playback, music player, and catalog/search are all implemented. The frontend has working soundboard and music player views.
