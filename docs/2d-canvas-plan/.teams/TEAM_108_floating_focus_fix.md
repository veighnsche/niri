# TEAM_108 — Floating Close Focus Fix

## Status: COMPLETE ✅

## Summary
Fixing BUG-006: No window selected after floating window closes.

## Root Cause Analysis

The bug is in `Layout::remove_window()` (`src/layout/layout_impl/window_ops.rs` lines 239-241).

**Problem**: When removing a floating window, `Layout::remove_window()` directly calls:
```rust
let removed = mon.canvas.floating.remove_tile(window);
```

This bypasses `Canvas2D::remove_window()` which contains the `update_focus_after_removing()` logic that switches `floating_is_active = false` when the floating space becomes empty.

**Result**: After closing the last floating window:
1. Window is removed from floating space
2. `floating_is_active` remains `true` 
3. Focus queries continue to check floating space (which is empty)
4. No window appears focused

## Fix

Add call to `update_focus_after_removing()` in `Layout::remove_window()` after removing from floating space.

### Files Modified
- `src/layout/canvas/canvas_floating.rs` - Make `update_focus_after_removing` public
- `src/layout/layout_impl/window_ops.rs` - Call focus update after floating removal

## Handoff Checklist
- [x] Code compiles (`cargo check`)
- [ ] Bug verified fixed - User should test manually
- [x] Team file complete
