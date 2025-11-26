# Phase 1.5.3 Part 1: Migrate Monitor Methods to Canvas2D

> **Status**: IN PROGRESS (TEAM_010)
> **Prerequisite**: Canvas2D field added to Monitor ✅

---

## Overview

Before removing workspace fields, we must migrate all Monitor methods to use Canvas2D.
This ensures the code compiles and tests pass at each step.

---

## Step 1.1: Add Missing Methods to Canvas2D/Row ✅

Canvas2D needs these methods to support Monitor migration:

| Method | Location | Status |
|--------|----------|--------|
| `tiles()` | `canvas/operations.rs` | ✅ Added |
| `tiles_mut()` | `canvas/operations.rs` | ✅ Added |
| `windows()` | `canvas/operations.rs` | ✅ Added |
| `windows_mut()` | `canvas/operations.rs` | ✅ Added |
| `tiles()` | `row/mod.rs` | ✅ Added |
| `tiles_mut()` | `row/mod.rs` | ✅ Added |

Row needs `tiles()` and `tiles_mut()` to iterate over all tiles in columns.
Column already has `tiles()` returning `(tile, position)`.

---

## Step 1.2: Add Simple Tile Iterators to Column

Column's `tiles()` returns `(tile, position)`. We need a simpler version:

| Method | Purpose | Status |
|--------|---------|--------|
| `tiles_iter()` | Returns `&Tile` only | ⏳ TODO |
| `tiles_iter_mut()` | Returns `&mut Tile` only | ⏳ TODO |

---

## Step 1.3: Migrate Monitor Query Methods

These methods only READ data, so they're safe to migrate first:

| Method | Current | Target | Status |
|--------|---------|--------|--------|
| `windows()` | `workspaces.iter().flat_map(ws.windows())` | `canvas.windows()` | ⏳ |
| `has_window()` | `windows().any(...)` | Uses new `windows()` | ⏳ |

---

## Step 1.4: Migrate Monitor Mutation Methods

These methods MODIFY state and are more complex:

| Method | Current | Target | Status |
|--------|---------|--------|--------|
| `add_window()` | Uses `workspaces[idx]` | Use `canvas.add_window()` | ⏳ |
| `add_tile()` | Uses `workspaces[idx]` | Use `canvas.add_tile()` | ⏳ |
| `remove_window()` | Uses `workspaces[idx]` | Use `canvas.remove_window()` | ⏳ |

---

## Step 1.5: Handle Workspace-Specific Logic

Some Monitor methods have workspace-specific logic that needs rethinking:

| Logic | Current Behavior | New Behavior |
|-------|------------------|--------------|
| `add_workspace_bottom()` | Creates empty workspace | **REMOVE** — no workspaces |
| `add_workspace_top()` | Creates empty workspace | **REMOVE** — no workspaces |
| `activate_workspace()` | Switches workspace | **REMOVE** — use row navigation |
| `workspace.original_output` | Tracks output | **REMOVE** — canvas has one output |

---

## Verification

After each step:
1. `cargo check` — must compile
2. `cargo test --lib` — all 284 tests must pass
3. `./scripts/verify-golden.sh` — all 91 golden tests must pass

---

## Files Modified

- `src/layout/canvas/operations.rs` — Added tiles/windows methods
- `src/layout/row/mod.rs` — Added tiles/tiles_mut methods
- `src/layout/column/mod.rs` — TODO: Add simple tile iterators
- `src/layout/monitor.rs` — TODO: Migrate methods

---

*TEAM_010: Phase 1.5.3 Part 1*
