# BUG_floating_close_animation

## Symptom
Floating windows disappear instantly when closed instead of playing the close animation.

## Reproduction
1. Open a terminal
2. Press Mod+Shift+Space to make it floating
3. Close the window (click X or Mod+Shift+Q)
4. **Expected**: Window should fade/shrink with animation
5. **Actual**: Window disappears instantly

## Hypothesis Log
| Chase ID | Team | Hypothesis | Result | Notes |
|----------|------|------------|--------|-------|
| 001 | TEAM_106 | Layout::start_close_animation_for_window doesn't check floating | INCONCLUSIVE | Added floating check but not tested |

## Current Status
**FIXED** by TEAM_107

## Investigation Notes (TEAM_106)

### What Was Attempted
Modified `Layout::start_close_animation_for_window()` in `src/layout/mod.rs` to check floating space:
```rust
// Check floating first
if mon.canvas.floating.has_window(window) {
    mon.canvas.floating.start_close_animation_for_window(renderer, window, blocker);
    return;
}
```

### Code Path
1. Window close triggers `handlers/xdg_shell.rs` or `handlers/compositor.rs`
2. These call `layout.start_close_animation_for_window()`
3. Original code only checked rows, not floating space
4. Fix adds floating check before row check

### Files Involved
- `src/layout/mod.rs` - start_close_animation_for_window (lines 1764-1797)
- `src/layout/canvas/floating/render.rs` - FloatingSpace::start_close_animation_for_window

### Potential Issues
1. The fix was added but not tested
2. `FloatingSpace::start_close_animation_for_window` might have its own issues
3. The closing_windows list in FloatingSpace might not be rendered

## Root Cause (TEAM_107)

The bug was in `Layout::store_unmap_snapshot` - it only checked rows, not the floating space.

**Code path**:
1. Window closes → `handlers/xdg_shell.rs` calls `layout.store_unmap_snapshot()`
2. `store_unmap_snapshot` checked rows but **NOT floating space** ❌
3. No snapshot stored for floating windows
4. Later `start_close_animation_for_window` was called (TEAM_106 fix)
5. `tile.take_unmap_snapshot()` returned `None` → no animation

## Fix (TEAM_107)

1. Added `store_unmap_snapshot_if_empty` and `clear_unmap_snapshot` to `FloatingSpace`
2. Updated `Layout::store_unmap_snapshot` to check floating first
3. Updated `Layout::clear_unmap_snapshot` to check floating first

## Files Modified
- `src/layout/canvas/floating/render.rs` - Added snapshot methods
- `src/layout/mod.rs` - Added floating checks to both snapshot methods
