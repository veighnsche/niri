# BUG_viewport_destination

## Symptom
`unfocus_preserves_current_size` test fails. Floating windows should preserve 200×200 size after unfocus, but window was receiving 936×1048 (tiling size).

## Reproduction
```bash
cargo test unfocus_preserves_current_size
```

## Hypothesis Log
| Chase ID | Team | Hypothesis | Result | Notes |
|----------|------|------------|--------|-------|
| 001 | TEAM_053 | FloatingSpace::refresh() sends wrong configure | BRANCH | Identified issue but couldn't fix |
| 002 | TEAM_054 | viewport.set_destination not reflected in geometry | ROOT CAUSE | Canvas2D missing floating in window search |

## Current Status
FIXED ✅

## Root Cause
**Canvas2D's `find_wl_surface()` and `find_wl_surface_mut()` methods were not searching the floating space.**

During the Canvas2D migration, `find_wl_surface()` in `canvas/operations.rs` only searched `tiled_tiles()`, not the floating space. This meant:

1. When a floating window committed (e.g., with new viewport destination), `Layout::find_window_and_output()` returned `None`
2. Therefore `window.on_commit()` was never called for floating windows
3. Therefore `Window.bbox` was never updated with the new viewport destination
4. Therefore `window.geometry().size` returned stale values (100×100 instead of 200×200)

## Fix
TEAM_054 updated:
- `Canvas2D::find_wl_surface()` to also search `self.floating.tiles()`
- `Canvas2D::find_wl_surface_mut()` to also search `self.floating.tiles_mut()`
- `Layout::find_window_and_output()` to use `canvas.find_wl_surface()` instead of iterating rows
- `Layout::find_window_and_output_mut()` to use `canvas.find_wl_surface_mut()` instead of iterating rows

## Files Modified
- `src/layout/canvas/operations.rs` - Fixed find_wl_surface and find_wl_surface_mut
- `src/layout/mod.rs` - Simplified find_window_and_output and find_window_and_output_mut
