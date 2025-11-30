# BUG_floating_drag

## Symptom
Cannot drag floating windows by their title bar. Title bar buttons (close, maximize, minimize) work correctly, but dragging does not move the window.

## Reproduction
1. Open a terminal window
2. Press Mod+Shift+Space to toggle it to floating
3. Try to drag the window by its title bar
4. Window does not move

## Hypothesis Log
| Chase ID | Team | Hypothesis | Result | Notes |
|----------|------|------------|--------|-------|
| 001 | TEAM_106 | window_under returns wrong HitType | DEAD END | Fixed to use HitType::hit_tile, buttons now work |
| 002 | TEAM_106 | interactive_move_begin not finding floating | INCONCLUSIVE | Added logging, needs testing |

## Current Status
**FIXED** by TEAM_109

## Investigation Notes (TEAM_106)

### What Was Fixed
- `FloatingSpace::window_under()` now uses `HitType::hit_tile()` properly
- This fixed title bar buttons (close/maximize/minimize)

### What Still Doesn't Work
- Dragging by title bar doesn't move the window

### Code Path Analysis
1. Title bar drag triggers `xdg_shell.rs::move_request()`
2. `move_request()` calls `layout.find_window_and_output(wl_surface)`
3. If found, creates `MoveGrab::new()` 
4. `MoveGrab::begin_move()` calls `layout.interactive_move_begin()`
5. `interactive_move_begin()` should find the window and start the move

### Debug Logging Added
- `interactive_move_begin()` has BUG003 logging at entry and exit
- Logs show which WindowLocation type was found

### Suspected Issues
1. **Serial mismatch**: The `move_request` might not match the grab serial
2. **Focus issue**: The pointer grab focus might not match the floating window surface
3. **interactive_move_update not being called**: The move might start but updates aren't processed

### Files Involved
- `src/handlers/xdg_shell.rs` - move_request handler
- `src/input/move_grab.rs` - MoveGrab implementation
- `src/layout/layout_impl/interactive_move.rs` - interactive_move_begin/update/end
- `src/layout/canvas/floating/mod.rs` - FloatingSpace::window_under

## Root Cause (TEAM_109)

The bug was in `src/input/move_grab.rs` lines 159-168.

**Problem**: When recognizing a drag gesture, the code checked `is_floating` using `layout.workspaces()` which only iterates over rows (tiled windows), NOT the floating space.

```rust
let is_floating = data.niri.layout.workspaces()
    .find_map(|(_, _, ws)| {
        ws.windows().any(|w| w.window == self.window)
            .then(|| ws.is_floating(&self.window))
    })
    .unwrap_or(false);  // Always false for floating windows!
```

**Result**:
1. `is_floating = false` for floating windows (wrong!)
2. Horizontal drag → `is_view_offset = true`
3. `begin_view_offset()` searches `workspaces()`, doesn't find floating window
4. Returns `false` → grab ends, no move happens

## Fix (TEAM_109)

1. Added `Layout::is_window_floating(&smithay::desktop::Window)` method to `window_ops.rs`
2. Method checks all monitors' floating spaces using `is_wl_surface()` comparison
3. Updated `move_grab.rs` to use the new method

## Files Modified
- `src/layout/layout_impl/window_ops.rs` - Added `is_window_floating` method
- `src/input/move_grab.rs` - Use new method for `is_floating` check
