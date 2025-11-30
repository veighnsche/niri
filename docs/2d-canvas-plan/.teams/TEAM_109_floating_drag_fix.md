# TEAM_109 — Floating Window Drag Fix

## Status: COMPLETE ✅

## Summary
Fixing BUG-003: Cannot drag floating windows by title bar.

## Root Cause Analysis

The bug is in `src/input/move_grab.rs` lines 159-168.

**Problem**: When determining if a window is floating during move gesture recognition, the code uses `layout.workspaces()` which only iterates over rows (tiled windows), NOT the floating space.

```rust
let is_floating = data
    .niri
    .layout
    .workspaces()
    .find_map(|(_, _, ws)| {
        ws.windows()
            .any(|w| w.window == self.window)
            .then(|| ws.is_floating(&self.window))
    })
    .unwrap_or(false);  // Always false for floating windows!
```

**Result**: For floating windows:
1. `is_floating = false` (wrong!)
2. Horizontal drag → `is_view_offset = true`
3. `begin_view_offset()` called, searches `workspaces()`, doesn't find floating window
4. Returns `false` → grab ends, no move happens

**Note**: Vertical drags might work because `is_view_offset = false` leads to `begin_move()`.

## Fix

1. Added `Layout::is_window_floating(&smithay::desktop::Window)` method to `window_ops.rs`
2. Method checks all monitors' floating spaces using `is_wl_surface()` comparison
3. Updated `move_grab.rs` to use the new method instead of the broken `workspaces()` search

### Files Modified
- `src/layout/layout_impl/window_ops.rs` - Added `is_window_floating` method
- `src/input/move_grab.rs` - Use new method for `is_floating` check

## Handoff Checklist
- [x] Code compiles (`cargo check`)
- [ ] Bug verified fixed - User should test manually
- [x] Team file complete
