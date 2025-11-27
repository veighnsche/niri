# Global TODO List — 2D Canvas Refactor

> **Check this file first** before starting work.
> This is the single source of truth for what needs to be done.

**Last updated**: TEAM_042

---

# 📊 CURRENT STATUS

| Metric | Value |
|--------|-------|
| **Build** | ✅ Compiles |
| **Tests** | 201 passed, 67 failed (75%) |
| **Golden Tests** | ✅ 84/84 pass |
| **TODOs in codebase** | 84 total |

---

# 🎯 PHASE 1: Config Migration (CURRENT PRIORITY)

> **Goal**: Replace all `workspace` terminology with `row`
> **Decision**: Remove immediately, no deprecation period

## Config Changes Needed

### niri-config/src/

- [ ] **workspace.rs** → rename to `row.rs`
  - [ ] Rename `Workspace` struct to `RowConfig`
  - [ ] Rename `WorkspaceName` to `RowName`
  - [ ] Update all references

- [ ] **lib.rs**
  - [ ] Change `workspaces: Vec<Workspace>` to `rows: Vec<RowConfig>`
  - [ ] Update `pub use` statements

- [ ] **window_rule.rs** (or wherever window rules are)
  - [ ] Rename `open-on-workspace` to `open-on-row`

- [ ] **animations.rs**
  - [ ] Rename `workspace_switch` to `row_switch` (or remove if not needed)

- [ ] **layout.rs**
  - [ ] Rename `empty_workspace_above_first` to `empty_row_above_first`

### src/layout/

- [ ] **workspace_types.rs** → rename to `row_types.rs`
  - [ ] Rename `WorkspaceId` to `RowId`
  - [ ] Rename `WorkspaceAddWindowTarget` to `RowAddWindowTarget`
  - [ ] Update all imports across codebase

- [ ] **mod.rs**
  - [ ] Rename `find_workspace_by_name` to `find_row_by_name`
  - [ ] Rename `ensure_named_workspace` to `ensure_named_row`
  - [ ] Rename `last_active_workspace_id` to `last_active_row_id`
  - [ ] Rename `workspace_id_counter` to `row_id_counter`

### src/handlers/

- [ ] **xdg_shell.rs**
  - [ ] Update `workspace_name` variable to `row_name`
  - [ ] Update `InitialConfigureState::Configured` fields

- [ ] **compositor.rs**
  - [ ] Update `workspace_id` to `row_id`

### Tests

- [ ] **src/tests/window_opening.rs**
  - [ ] Update test configs to use `row` syntax
  - [ ] Rename test functions if needed

---

# 🔴 BLOCKING: Animation System Bug

> **Status**: Move animations created but don't interpolate
> **Impact**: 67 test failures related to animations

## Problem

Move animations for tiles are created correctly but `Animation::value()` returns 0 instead of interpolating.

```
DEBUG: Creating move animation for tile 1 with delta 50.0
render_offset: move_y exists=true, value=0, offset.y=0  ← Should be interpolating!
```

## Investigation Needed

- [ ] Compare `animate_move_y_from_with_config()` vs working resize animations
- [ ] Check `Animation::new` parameters differences
- [ ] Verify clock advancement for move animations
- [ ] Check animation config differences

## Files to Investigate

- `src/layout/tile.rs` - `animate_move_y_from_with_config()`
- `src/layout/column/sizing/tile_sizes.rs` - move animation creation
- `src/animation/mod.rs` - `Animation::value()` implementation

---

# 📋 REMAINING TODOs FROM CODEBASE

## High Priority (Core Functionality)

### src/layout/mod.rs

| Line | TODO | Description |
|------|------|-------------|
| 735 | TEAM_023 | Apply workspace config to canvas rows |
| 787 | TEAM_024 | Implement canvas cleanup logic |
| 1033 | TEAM_023 | Implement window height setting on canvas/row |
| 1050 | TEAM_023 | Implement proper workspace ID to row mapping |
| 1251 | TEAM_020 | Remove workspace check entirely |
| 1510 | TEAM_020 | Remove workspace check entirely |
| 3021 | TEAM_020 | Remove workspace config entirely |
| 3565 | TEAM_032 | original_output field doesn't exist in Row |
| 3579 | TEAM_032 | original_output field doesn't exist in Row |
| 3622 | TEAM_020 | Remove workspace check entirely |
| 3647 | TEAM_020 | Remove workspace check entirely |
| 3701 | TEAM_020 | Remove workspace check entirely |
| 3726 | TEAM_020 | Remove workspace check entirely |
| 4635 | TEAM_018 | Implement proper duplicate name checking |

### src/layout/monitor/

| File | Line | TODO | Description |
|------|------|------|-------------|
| render.rs | 45 | TEAM_022 | Implement proper insert hint rendering |
| hit_test.rs | 22 | TEAM_023 | Implement proper row geometry calculation |
| hit_test.rs | 41 | TEAM_023 | Implement proper row geometry |
| mod.rs | 300 | TEAM_033 | Properly merge rows |
| mod.rs | 322 | TEAM_022 | Implement proper column addition to canvas |
| navigation.rs | 59 | TEAM_022 | Implement previous row tracking |
| navigation.rs | 67 | TEAM_022 | Implement previous row tracking |
| navigation.rs | 124 | - | Implement move window between rows |
| navigation.rs | 129 | - | Implement move window between rows |
| navigation.rs | 139 | - | Implement move window between rows |
| navigation.rs | 148 | - | Implement move column between rows |
| navigation.rs | 153 | - | Implement move column between rows |
| navigation.rs | 158 | - | Implement move column between rows |
| gestures.rs | 142 | TEAM_024 | Get workspace ID from canvas row |
| gestures.rs | 146 | TEAM_024 | Set active workspace index in canvas |
| config.rs | 28 | TEAM_024 | Implement row removal in canvas |

### src/layout/canvas/

| File | Line | TODO | Description |
|------|------|------|-------------|
| operations.rs | 108 | TEAM_019 | Implement proper active window handling |
| operations.rs | 123 | TEAM_019 | Implement proper active window handling |
| operations.rs | 194 | TEAM_019 | Implement layout_config for Row |
| operations.rs | 217 | TEAM_019 | Implement start_open_animation for Row |
| operations.rs | 240 | TEAM_019 | Implement proper centering for tiled windows |
| operations.rs | 324 | TEAM_019 | Implement actual column reordering |
| operations.rs | 340 | TEAM_019 | Implement actual column reordering |
| operations.rs | 448 | TEAM_020 | Implement proper window update |
| operations.rs | 468 | TEAM_020 | Properly activate in row |
| operations.rs | 487 | TEAM_020 | Implement fullscreen setting |
| operations.rs | 492 | TEAM_020 | Implement fullscreen toggle |
| operations.rs | 497 | TEAM_020 | Implement maximized setting |
| operations.rs | 502 | TEAM_020 | Implement maximized toggle |
| operations.rs | 513 | TEAM_020 | Implement proper scroll calculation |
| operations.rs | 520 | TEAM_021 | Implement proper popup positioning |
| render.rs | 25 | TEAM_007 | Apply camera offset to render elements |
| mod.rs | 312 | TEAM_025 | Implement proper row removal |
| floating.rs | 163 | TEAM_009 | Add close animation for tiled windows |
| navigation.rs | 79 | TEAM_007 | Add vertical_view_movement config |
| navigation.rs | 324 | TEAM_018 | Implement back-and-forth logic |
| navigation.rs | 331 | TEAM_018 | Implement previous row tracking |

### src/layout/row/

| File | Line | TODO | Description |
|------|------|------|-------------|
| mod.rs | 395 | TEAM_027 | Calculate proper extra_size |
| mod.rs | 999 | TEAM_024 | Implement column width expansion |
| mod.rs | 1057 | - | Implement cancel_resize_for_column |
| mod.rs | 1068 | - | Implement consume_or_expel_window_right |
| mod.rs | 1176 | TEAM_022 | Rows should get output from monitor/canvas |
| mod.rs | 1183 | TEAM_022 | Implement active window logic |

### src/layout/workspace_types.rs

| Line | TODO | Description |
|------|------|-------------|
| 10 | - | Remove when external systems updated |
| 46 | - | Remove when external systems updated |
| 78 | - | Move to more appropriate location |

### Other src/ files

| File | Line | TODO | Description |
|------|------|------|-------------|
| niri.rs | 3813 | TEAM_023 | Get output from monitor |
| niri.rs | 4502 | TEAM_023 | Update render elements handling |
| niri.rs | 4533 | TEAM_023 | Update workspace-specific rendering |
| niri.rs | 4541 | TEAM_023 | Fix workspace-specific element rendering |
| a11y.rs | 122 | TEAM_023 | Implement proper row ID generation |

---

# 🔮 FUTURE PHASES (After Phase 1)

## Phase 2: Row System
- Row naming (any row can be named)
- Row lifecycle (creation/deletion rules)
- Global row ID counter
- Active row tracking

## Phase 3: Row Spanning
- `row_span` field on Tile
- Cross-row occupancy tracking
- `increase-row-span` / `decrease-row-span` actions

## Phase 4: Camera System
- Camera struct with (x, y, zoom)
- Auto-zoom for row span
- Zoom gestures (Mod+Scroll)
- Render transform pipeline

## Phase 5: Camera Bookmarks
- Save/restore camera positions
- `Mod+1/2/3` for bookmarks
- Optional row name reference

## Phase 6: Navigation & Polish
- Geometric navigation (find nearest tile)
- Origin-based leading edges
- Spawn direction based on quadrant
- Documentation

---

# 📝 FOLLOW-UP QUESTIONS

## From TEAM_042 Questionnaire

1. **Row 0 naming**: Can row 0 be renamed, or is "origin" special?
   - Decision: Any row can be named ✅

2. **Negative rows**: Rows can go negative (above origin)?
   - Decision: Yes, rows are indexed ..., -2, -1, 0, 1, 2, ... ✅

3. **Window spanning**: How does a window's row assignment work when spanning?
   - Decision: Top-left corner (0,0 point) determines the row ✅

4. **Zoom behavior**: When zoomed out, how does focus work?
   - Open question: Need to define focus behavior at different zoom levels

5. **Config migration**: How to handle users with old `workspace` configs?
   - Decision: Remove immediately, no deprecation ✅

---

# 🗄️ ARCHIVED (Completed Work)

<details>
<summary>Click to expand completed work history</summary>

## Compilation Fixes (TEAM_030-040)
- All MonitorSet::NoOutputs patterns updated
- All method call parens fixed
- All workspace field accesses migrated to canvas
- All Monitor/Row methods implemented
- All type mismatches resolved
- All borrow checker issues fixed

## Core Migration (TEAM_021)
- workspace.rs (1,997 lines) DELETED
- workspace_compat.rs (302 lines) DELETED
- workspace_ops.rs DELETED
- Canvas2D is sole layout system

## Row Implementation (TEAM_036-040)
- `window_under()`, `resize_edges_under()` implemented
- `activate_window()`, `is_urgent()` implemented
- `set_fullscreen()`, `toggle_fullscreen()` implemented
- `set_maximized()`, `toggle_maximized()` implemented
- `configure_new_window()`, `update_window()` implemented
- `toggle_width()`, `toggle_window_width/height()` implemented
- `find_wl_surface()`, `find_wl_surface_mut()` implemented

## Animation System (TEAM_039)
- Move animation creation logic implemented
- Old position calculation fixed
- Delta calculation working
- Animation parameters fixed (0,1,0 → 1,0,0)
- Rendering integration confirmed

</details>

---

*Check `phases/` for detailed phase documentation.*
*Check `.questions/` for architecture decisions.*
*Check `.teams/` for team handoff notes.*
