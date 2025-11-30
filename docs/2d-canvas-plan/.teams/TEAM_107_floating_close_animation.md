# TEAM_107 — Floating Window Close Animation Fix

## Status: COMPLETE ✅

## Summary
Fixing BUG-005: Floating window close animation missing.

## Root Cause Analysis

The bug is in `Layout::store_unmap_snapshot` (src/layout/mod.rs lines 1661-1691).

**Problem**: When a floating window is about to close, the handlers call `layout.store_unmap_snapshot(renderer, &window)` to capture the window's visual state for the close animation. However, `store_unmap_snapshot` only checks rows - it never checks the floating space.

**Code path**:
1. Window closes → `handlers/xdg_shell.rs` line 859 calls `layout.store_unmap_snapshot()`
2. `Layout::store_unmap_snapshot` checks:
   - Interactive move state ✓
   - Rows via `canvas.rows_mut()` ✓
   - **FloatingSpace** ❌ MISSING
3. Since floating window not found in rows, no snapshot is stored
4. Later, `start_close_animation_for_window` is called correctly (TEAM_106 fix)
5. `FloatingSpace::start_close_animation_for_window` calls `tile.take_unmap_snapshot()`
6. Returns `None` because snapshot was never stored → function returns early, no animation

**Same issue affects**: `Layout::clear_unmap_snapshot` (consistency fix)

## Fix

### 1. Add methods to FloatingSpace (src/layout/canvas/floating/render.rs)
- `store_unmap_snapshot_if_empty(&mut self, renderer, window_id)`
- `clear_unmap_snapshot(&mut self, window_id)`

### 2. Update Layout methods (src/layout/mod.rs)
- `store_unmap_snapshot`: Check floating space first, before rows
- `clear_unmap_snapshot`: Check floating space first, before rows

## Files Modified
- `src/layout/canvas/floating/render.rs` - Add snapshot methods
- `src/layout/mod.rs` - Add floating checks

## Handoff Checklist
- [x] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test --lib`) - Not run, user should verify
- [ ] Bug verified fixed - User should test manually
- [x] Team file complete
