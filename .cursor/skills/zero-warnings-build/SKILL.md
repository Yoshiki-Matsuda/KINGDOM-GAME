---
name: zero-warnings-build
description: >-
  Ensures Rust (cargo) and TypeScript (npm) builds finish with zero warnings
  before completing a task. Use after editing server/ or src/ code, when the
  user mentions compiler warnings, cargo warnings, tsc warnings, or asks to
  leave no warnings.
---

# Zero-Warnings Build

## Rule

**Do not finish a coding task while compiler warnings remain.** Fix warnings in the same change set — do not defer them and do not ask the user to clean up later.

## When To Run

Run both checks when you touched the matching area:

| Changed paths | Command |
|---------------|---------|
| `server/` | `cargo build -p kingdom-server` |
| `src/`, `package.json`, `tsconfig*` | `npm run build` |
| Both | Run **both** commands |

Working directory for cargo: `server/`. Working directory for npm: repo root.

## Verification Workflow

1. Make code changes.
2. Run the build command(s) above.
3. If **any warning or error** appears → fix it → rebuild.
4. Repeat until output shows `Finished` with **no `warning:` lines**.
5. Only then report the task as done.

Do not rely on "it compiles" from the IDE alone — run the commands.

## How To Fix (preferred order)

1. **Delete** unused imports, functions, modules, and variables.
2. **Simplify** redundant logic (e.g. `changed = true` that is always true).
3. **Wire up** code that should be used instead of leaving it dead.
4. **Merge** duplicate modules (e.g. a file that only re-exports one symbol).

## Do Not

- Leave `warning: unused import`, `dead_code`, or `unused_assignments` for the user.
- Add `#[allow(...)]`, `#![allow(...)]`, `@ts-ignore`, or `eslint-disable` unless the user explicitly requests suppression.
- Create stub modules or placeholder exports "for later".
- Treat warnings as acceptable if the user's build log shows them.

## Kingdom-Game Notes

- Server crate: `kingdom-server` (`server/Cargo.toml`).
- Common Rust warnings here: unused imports in `model_actions/mod.rs`, dead helpers in `ai_actions.rs`, redundant flags in `world_scheduler.rs`.
- Client build: `npm run build` runs `tsc && vite build` — TypeScript errors block the build; fix all reported issues.

## Completion Checklist

```
- [ ] cargo build -p kingdom-server — zero warnings
- [ ] npm run build — success (if client changed)
- [ ] No new allow/suppress attributes without user request
```
