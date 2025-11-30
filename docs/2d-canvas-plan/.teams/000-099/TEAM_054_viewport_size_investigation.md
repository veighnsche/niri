# TEAM_054: Viewport Size Investigation & Test Fixes

## Status: COMPLETED ✅

## Mission
1. Deep investigation of smithay viewport behavior to understand why `viewport.set_destination()` is not reflected in `window.geometry().size`.
2. Fix remaining failing tests from the Canvas2D migration.

## Bug ID
`BUG_viewport_destination`

## Root Cause Found
The issue was NOT with smithay's viewport handling. The root cause was that during the Canvas2D migration, `find_wl_surface()` and `find_wl_surface_mut()` in `canvas/operations.rs` were not searching the floating space.

## Fixes Applied

### 1. Floating Window Search (BUG_viewport_destination)
- `Canvas2D::find_wl_surface()` - Added floating space search
- `Canvas2D::find_wl_surface_mut()` - Added floating space search
- `Layout::find_window_and_output()` - Use canvas.find_wl_surface()
- `Layout::find_window_and_output_mut()` - Use canvas.find_wl_surface_mut()

### 2. Floating Window Size Handling
- `Layout::set_window_width()` - Handle floating windows properly
- `Layout::set_window_height()` - Handle floating windows properly
- `FloatingSpace::set_window_width()` - Use expected_size() for pending height
- `FloatingSpace::set_window_height()` - Use expected_size() for pending width

### 3. Layout::windows() Iterator
- Include floating windows in the iterator (was only returning tiled)

### 4. Fullscreen/Maximize from Floating
- `Canvas2D::set_fullscreen()` - Move floating to tiled, set floating_is_active=false
- `Canvas2D::toggle_fullscreen()` - Same fix
- `Canvas2D::set_maximized()` - Same fix
- `Canvas2D::toggle_maximized()` - Same fix

### 5. Row Fullscreen/Maximize with Multiple Tiles
- `Row::set_fullscreen()` - Extract window from column before fullscreening
- `Row::set_maximized()` - Extract window from column before maximizing

## Test Results
- **Before**: 29 failing tests
- **After**: 21 failing tests
- **Fixed**: 8 tests

## Remaining Failing Tests (21)
- Animation tests (2): View offset/position issues
- Golden tests (2): Expand to available width issues
- Workspace/move tests (8): Canvas2D migration issues
- Floating interactive move tests (4): Complex state tracking
- Floating fullscreen/maximize tests (4): Windowed fullscreen handling
- Window opening test (1): Target workspace issues

## Files Modified
- `src/layout/canvas/operations.rs`
- `src/layout/canvas/floating.rs`
- `src/layout/floating.rs`
- `src/layout/mod.rs`
- `src/layout/row/mod.rs`

## Handoff
- [x] Code compiles (`cargo check`)
- [x] 251 tests pass, 21 still failing
- [x] Team file complete
- [x] Bug file updated with root cause

## Notes for Next Team
The remaining 21 failing tests are more complex issues:
1. **Animation tests**: View offset calculations differ between Row and original ScrollingSpace
2. **Workspace tests**: Named workspace preservation logic needs Canvas2D adaptation
3. **Interactive move tests**: Complex state tracking during moves
4. **Windowed fullscreen tests**: Need to track pre-fullscreen floating state
