# TEAM_055: Config Workspace to Row Migration

## Status: COMPLETE ✅

## Task
Rename all workspace-related config types and fields to row-based terminology in `niri-config/src/`.

## Files to Modify

### 1. workspace.rs → row.rs
- [x] Rename file
- [x] Rename `Workspace` struct to `RowConfig`
- [x] Rename `WorkspaceName` to `RowName`
- [x] Rename `WorkspaceLayoutPart` to `RowLayoutPart`
- [x] Update all internal references

### 2. lib.rs
- [x] Change `pub mod workspace` to `pub mod row`
- [x] Change `workspaces: Vec<Workspace>` to `rows: Vec<RowConfig>`
- [x] Update `pub use` statements
- [x] Update parsing logic for `row` node instead of `workspace`

### 3. window_rule.rs
- [x] Rename `open_on_workspace` to `open_on_row`

### 4. animations.rs
- [x] Rename `workspace_switch` to `row_switch`
- [x] Rename `WorkspaceSwitchAnim` to `RowSwitchAnim`

### 5. layout.rs
- [x] Rename `empty_workspace_above_first` to `empty_row_above_first`

## Handoff
- [x] Code compiles (`cargo check`) ✅
- [x] niri-config tests pass (`cargo test --package niri-config`) ✅
- [x] Team file complete ✅

## Summary

Successfully migrated all workspace-related config terminology to row-based terminology in `niri-config/src/`:

### Files Changed
1. **workspace.rs → row.rs**: Renamed file and all types
2. **lib.rs**: Updated module, field names, and parsing
3. **window_rule.rs**: `open_on_workspace` → `open_on_row`
4. **animations.rs**: `workspace_switch` → `row_switch`, `WorkspaceSwitchAnim` → `RowSwitchAnim`
5. **layout.rs**: `empty_workspace_above_first` → `empty_row_above_first`

### Codebase Updates
- **src/layout/mod.rs**: Updated imports and field references
- **src/layout/tests.rs**: Updated test configs and imports
- **src/layout/monitor/config.rs**: Updated field references
- **src/layout/monitor/gestures.rs**: Updated animation references
- **src/niri.rs**: Updated config.rows references
- **src/window/mod.rs**: Updated open_on_row field
- **src/handlers/xdg_shell.rs**: Updated open_on_row references
- **src/tests/window_opening.rs**: Updated test configs

### Config Syntax Changes
- `workspace "name" {}` → `row "name" {}`
- `open-on-workspace` → `open-on-row`
- `workspace-switch` animation → `row-switch` animation
- `empty-workspace-above-first` → `empty-row-above-first`

### src/layout/ Changes (Phase 2)
1. **workspace_types.rs → row_types.rs**: Renamed file and all types
   - `WorkspaceId` → `RowId`
   - `WorkspaceAddWindowTarget` → `RowAddWindowTarget`
2. **mod.rs**: Updated function names and field names
   - `find_workspace_by_name` → `find_row_by_name`
   - `ensure_named_workspace` → `ensure_named_row`
   - `last_active_workspace_id` → `last_active_row_id`
   - `workspace_id_counter` → `row_id_counter`
   - `next_workspace_id` → `next_row_id`
3. **canvas/mod.rs**: Updated `workspace_id_counter` → `row_id_counter`
4. **canvas/operations.rs**: Updated ID generation
5. **row/mod.rs**: Updated type references
6. **monitor/*.rs**: Updated imports and type references

### All Imports Updated
- src/a11y.rs
- src/protocols/ext_workspace.rs
- src/niri.rs
- src/input/spatial_movement_grab.rs
- src/ipc/server.rs
- src/handlers/mod.rs

### src/handlers/ Changes (Phase 3)
1. **xdg_shell.rs**: Updated `InitialConfigureState::Configured` field
   - `workspace_name` → `row_name` in all pattern matches
2. **compositor.rs**: Updated variable names
   - `workspace_id` → `row_id`
   - `workspace_name` → `row_name`
   - Updated `find_workspace_by_name` call to `find_row_by_name`
3. **window/unmapped.rs**: Updated struct field
   - `workspace_name` → `row_name` in `InitialConfigureState::Configured`

### Tests Changes (Phase 4)
1. **src/tests/window_opening.rs**: Updated test configs and functions
   - `workspace "ws-1" {}` → `row "ws-1" {}` in test config
   - `workspace "ws-2" {}` → `row "ws-2" {}` in test config
   - `target_output_and_workspaces()` → `target_output_and_rows()`
   - `check_target_output_and_workspace()` → `check_target_output_and_row()`
   - `final workspace:` → `final row:` in snapshot output
