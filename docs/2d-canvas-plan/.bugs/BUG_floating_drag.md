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
INVESTIGATING

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

## Recommended Next Steps for CHASE_003
1. Add logging to `xdg_shell.rs::move_request()` to see if it's even called
2. Add logging to `MoveGrab::new()` to see if grab is created
3. Add logging to `MoveGrab::begin_move()` to see if move starts
4. Check if `interactive_move_update()` is being called with correct deltas
5. Compare with main branch's floating drag behavior
