# BUG_alacritty_invisible

## Symptom
- Pressing Mod+T in the 2d-canvas branch (winit backend) should spawn an Alacritty terminal.
- Logs show that `Action::Spawn(["alacritty"])` runs, `spawn_sync` executes `"alacritty"`, and an xdg-toplevel with `app_id="Alacritty"` and `title="Alacritty"` is mapped.
- Layout logs show the window is added to output `winit`, row_idx=0, with a large tile at a camera-adjusted position `(x≈16, y≈16, w≈707, h≈1346)` and `row_visible=true`.
- Despite this, no visible Alacritty window appears in the winit window.

## Reproduction
1. Checkout the 2d-canvas branch.
2. Build and run:
   ```bash
   cargo run -- --config resources/default-config.kdl
   ```
3. Ensure you are on the winit backend (running under an existing Wayland/X11 session).
4. Press `Mod+T` several times.
5. Observe:
   - Logs like:
     - `Spawn action triggered with command: ["alacritty"]`
     - `spawn_sync: executing command "alacritty"`
     - `mapped new toplevel window app_id="Alacritty" title="Alacritty"`
     - `monitor: new window render position output=winit row_idx=0 x=16.0 y=16.0 w=707.0 h=1346.0 is_active_tile=true`
   - But the winit window remains visually empty / no terminal is shown.

## Hypothesis Log
| Chase ID                         | Team     | Hypothesis                                                             | Result    | Notes |
|----------------------------------|----------|-------------------------------------------------------------------------|----------|-------|
| BUG_alacritty_invisible_CHASE_001 | TEAM_047 | Layout is misplacing Alacritty (off-camera row or wrong coordinates)    | DEAD END | Logs show row_idx=0, row_visible=true, and camera-adjusted tile `(x≈16,y≈16,w≈707,h≈1346)` well within a typical 0..view_width,0..view_height viewport. Layout placement appears correct. |
| BUG_alacritty_invisible_CHASE_002 | TEAM_047 | Rendering pipeline/canvas rendering is failing to draw the mapped tile | BRANCH   | Layout claims an on-screen tile, so likely issue is in render path (canvas/row/column render or Gles/winit damage). |

## Current Status
INVESTIGATING

## Root Cause (if found)
- TODO: Not yet identified. Suspect is downstream of `Monitor::add_window`, likely in the render pipeline.
