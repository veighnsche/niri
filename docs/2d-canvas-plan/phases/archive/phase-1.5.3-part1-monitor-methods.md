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

## Step 1.2: Add Simple Tile Iterators to Column ✅

Column's `tiles()` returns `(tile, position)`. We need a simpler version:

| Method | Purpose | Status |
|--------|---------|--------|
| `tiles_iter()` | Returns `&Tile` only | ✅ Added |
| `tiles_iter_mut()` | Returns `&mut Tile` only | ✅ Added |

**Note**: These had to be added outside the `#[cfg(test)]` block to be available in non-test builds.

---

## Step 1.3: Migrate Monitor Query Methods ✅

These methods only READ data, so they're safe to migrate first:

| Method | Current | Target | Status |
|--------|---------|--------|--------|
| `windows()` | `workspaces.iter().flat_map(ws.windows())` | `canvas.windows()` | ✅ |
| `has_window()` | `windows().any(...)` | `canvas.contains_any()` | ✅ |

**Migration Strategy**: During the transition, these methods check BOTH canvas AND workspaces.
This ensures existing tests pass while we incrementally migrate window operations.

```rust
// windows() chains both sources
canvas_windows.chain(workspace_windows)

// has_window() checks both
self.canvas.contains_any(window) || self.workspaces.iter().any(...)
```

After full migration (Part 4), the workspace checks will be removed.

---

## Step 1.4: Migrate Monitor Mutation Methods ✅

These methods MODIFY state and are more complex:

| Method | Current | Target | Status |
|--------|---------|--------|--------|
| `add_window()` | Uses `workspaces[0].make_tile()` | Uses `canvas.make_tile()` | ✅ |
| `add_tile()` | Uses `workspaces[idx]` | Still uses workspaces (TODO) | ⏳ |
| `remove_window()` | Uses `workspaces[idx]` | Still uses workspaces (TODO) | ⏳ |

**Progress**:
- ✅ Added `make_tile()` to Canvas2D
- ✅ `add_window()` now uses `canvas.make_tile()` instead of `workspaces[0].make_tile()`
- ⏳ `add_tile()` still routes through workspaces (complex workspace-specific logic)
- ⏳ `remove_window()` still routes through workspaces

**Note**: Full migration of `add_tile()` requires removing workspace-specific logic like:
- `add_workspace_bottom()` / `add_workspace_top()`
- `activate_workspace()`
- `workspace.original_output`

These will be removed in Parts 2-4.

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
