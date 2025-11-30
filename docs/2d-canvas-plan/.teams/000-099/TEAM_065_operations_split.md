# TEAM_065: Split canvas/operations.rs

## Status: ✅ COMPLETED

## Objective
Split `canvas/operations.rs` (869 LOC) into smaller, focused modules:
- `canvas/operations/row.rs` - Row management (~200 LOC)
- `canvas/operations/window.rs` - Window operations (~200 LOC)
- `canvas/operations/tile.rs` - Tile operations (~200 LOC)
- `canvas/operations/state.rs` - State updates (~150 LOC)

## Steps

### Step 2.1: Extract Row Management
Methods to move:
- `ensure_row()`, `cleanup_empty_rows()`, `renumber_rows()`, `cleanup_and_renumber_rows()`
- `find_row_by_id()`

### Step 2.2: Extract Window Operations
Methods to move:
- `find_window()`, `find_window_mut()`, `has_window()`, `set_window_height()`
- `active_window()`, `active_window_mut()`, `windows()`, `windows_mut()`
- `find_wl_surface()`, `find_wl_surface_mut()`
- `update_window()`, `activate_window()`, `activate_window_without_raising()`

### Step 2.3: Extract Tile Operations
Methods to move:
- `add_tile()`, `add_tile_to_row()`
- `tiles()`, `tiles_mut()`, `tiled_tiles()`, `tiled_tiles_mut()`
- `tiles_with_render_positions()`
- `contains()`, `find_window_row()`

### Step 2.4: Extract State Updates
Methods to move:
- `update_config()`, `update_row_layout_config()`
- `start_window_open_animation()`, `start_open_animation()`
- `is_window_floating()`, `center_window()`, `move_floating_window()`
- `move_left()`, `move_right()`, `move_column_to_first()`, `move_column_to_last()`
- `set_fullscreen()`, `toggle_fullscreen()`, `set_maximized()`, `toggle_maximized()`
- `has_windows()`, `has_windows_or_name()`, `has_urgent_window()`
- `scroll_amount_to_activate()`, `popup_target_rect()`, `descendants_added()`
- `clean_up_workspaces()`, `active_workspace()`, `active_workspace_mut()`
- `dnd_scroll_gesture_end()`, `dnd_scroll_gesture_begin()`, `are_transitions_ongoing()`

## Progress

- [x] Create operations/ directory
- [x] Extract row.rs (89 LOC)
- [x] Extract window.rs (274 LOC)
- [x] Extract tile.rs (92 LOC)
- [x] Extract state.rs (474 LOC)
- [x] Convert operations.rs to mod.rs (18 LOC)
- [x] Verify: cargo check && cargo test layout::

## Final Results

| File | LOC | Contents |
|------|-----|----------|
| mod.rs | 18 | Module declarations |
| row.rs | 89 | ensure_row, cleanup, renumber, find_row_by_id |
| window.rs | 274 | find_window, has_window, active_window, WlSurface lookup, activation |
| tile.rs | 92 | add_tile, tiles iterators, tiled_tiles |
| state.rs | 474 | update_config, fullscreen/maximize, focus movement, DND |
| **Total** | **947** | vs 869 original (overhead from imports/docs) |

## Handoff
- [x] Code compiles (`cargo check`)
- [x] Tests pass (`cargo test layout::`) - 187 tests
- [x] Team file complete
