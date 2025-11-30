# TEAM_103: Technical Debt Stub Cleanup

## Status: COMPLETED ✅

## Objective
Remove broken stub methods and fix technical debt identified in the codebase.

## Bugs Fixed

| Bug | Location | Fix |
|-----|----------|-----|
| `set_window_floating` was no-op | `layout/mod.rs:1048` | Now properly routes to Canvas2D |
| `current_output()` broken | `spatial_movement_grab.rs:100` | Fixed to use `output_for_workspace_by_id()` |
| `popup_target_rect()` would panic | `row/mod.rs` | Implemented properly |
| `scrolling_insert_position()` wrong | `row/mod.rs` | Implemented proper column detection |
| `tiles_with_ipc_layouts()` fake data | `row/mod.rs` | Implemented with real positions |
| `swap_window_in_direction()` no-op | `row/mod.rs` | Implemented column swapping |
| `center_visible_columns()` no-op | `row/mod.rs` | Implemented with animation |
| `expel_from_column()` no-op | `navigation.rs` | Fixed to use Canvas2D toggle |
| Snapshot methods no-ops | `row/mod.rs` | Implemented proper tile delegation |
| Close animation broken | `row/mod.rs` | Implemented `start_close_animation_for_window()` |

## Dead Code Removed

- `Monitor::active_workspace()` / `active_workspace_ref()` - aliases removed
- `Monitor::move_workspace_to_idx()` - empty stub removed
- `Row::make_tile()` - never called
- `Row::toggle_window_floating()` / `set_window_floating()` - stubs removed

## Call Sites Updated

- `src/handlers/xdg_shell.rs` (5 places) - now uses `canvas().active_row()`
- `src/input/gesture.rs` - fixed monitor lookup
- `src/input/pointer.rs` - fixed monitor lookup
- `src/layout/mod.rs` - uses `canvas_mut().active_row_mut()`
- `src/layout/monitor/hit_test.rs` - uses `canvas.active_row()`

## Files Modified

- `src/layout/row/mod.rs` - implemented stub methods
- `src/layout/mod.rs` - fixed set_window_floating routing
- `src/layout/layout_impl/navigation.rs` - fixed expel_from_column
- `src/layout/layout_impl/row_management.rs` - added output_for_workspace_by_id
- `src/layout/monitor/mod.rs` - removed dead stubs
- `src/handlers/xdg_shell.rs` - updated call sites
- `src/input/gesture.rs` - fixed monitor lookup
- `src/input/pointer.rs` - fixed monitor lookup
- `src/input/spatial_movement_grab.rs` - fixed gesture check

## Remaining TODOs

- `move_row_to_index()` - needs Canvas2D implementation for arbitrary row reordering

## Handoff Checklist

- [x] Code compiles (`cargo check`)
- [x] Tests pass (`cargo test`)
- [x] Team file complete
- [x] Remaining TODOs documented

## Code Comments Added

All changes marked with `// TEAM_103:` where applicable.
