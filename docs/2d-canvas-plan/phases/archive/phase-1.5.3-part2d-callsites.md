# Phase 1.5.3 Part 2D: Monitor Modular Refactor - Complete Analysis

> **Status**: PENDING
> **Type**: MODULAR REFACTOR + WORKSPACE REMOVAL
> **Created by**: TEAM_012

---

## Overview

This document provides a complete analysis for refactoring `src/layout/monitor.rs` 
(2255 lines monolith) into a proper modular structure while removing workspace code.

**This is NOT just a rename - it's a full modular refactor.**

**Scope**: 
- ~490 workspace references to REMOVE (not migrate)
- ~800 lines of kept code to MODULARIZE into 7 files
- New row navigation methods to CREATE

---

## Target Module Structure

```
src/layout/monitor/
├── mod.rs          - Core struct, new(), output/canvas accessors (~200 lines)
├── operations.rs   - Window operations: add_window, add_tile (~150 lines)
├── navigation.rs   - Row navigation (NEW: focus_row_down/up, move_to_row) (~100 lines)
├── render.rs       - Rendering: render_elements (~200 lines)
├── hit_test.rs     - Geometry: window_under, resize_edges_under (~100 lines)
├── config.rs       - Config: update_config, update_output_size (~50 lines)
└── insert_hint.rs  - Insert hint types and rendering (~100 lines)
```

---

## 1. What Gets MIGRATED (to new modules)

### 1.1 Core Struct Fields → mod.rs

| Field | Notes |
|-------|-------|
| `output: Output` | Output reference |
| `output_name: String` | Cached name |
| `scale: Scale` | Output scale |
| `view_size: Size` | View dimensions |
| `working_area: Rectangle` | Usable area |
| `canvas: Canvas2D<W>` | **Primary layout primitive** |
| `clock: Clock` | Animation clock |
| `base_options: Rc<Options>` | Base config |
| `options: Rc<Options>` | Current config |
| `layout_config: Option<LayoutPart>` | Layout config |
| `insert_hint` | Insert hint state |
| `insert_hint_element` | Cached hint element |

### 1.2 Methods → mod.rs

| Method | Notes |
|--------|-------|
| `new()` | Simplified (no workspaces) |
| `output()`, `output_name()` | Accessors |
| `canvas()`, `canvas_mut()` | Canvas accessors |
| `scale()`, `view_size()`, `working_area()` | Geometry accessors |
| `windows()`, `has_window()` | Delegates to canvas |
| `advance_animations()` | Simplified |
| `are_animations_ongoing()` | Simplified |

### 1.3 Methods → operations.rs

| Method | Notes |
|--------|-------|
| `add_window()` | Window placement |
| `add_tile()` | Tile placement |
| `add_column()` | Column placement |
| `remove_window()` | Window removal |

### 1.4 Methods → navigation.rs (NEW)

| New Method | Replaces | Notes |
|------------|----------|-------|
| `focus_row_down()` | `switch_workspace_down()` | Navigate to row below |
| `focus_row_up()` | `switch_workspace_up()` | Navigate to row above |
| `focus_row(idx)` | `switch_workspace(idx)` | Focus specific row |
| `focus_previous_position()` | `switch_workspace_previous()` | Browser-like back |
| `move_window_to_row_down(focus)` | `move_to_workspace_down()` | Move window down |
| `move_window_to_row_up(focus)` | `move_to_workspace_up()` | Move window up |
| `move_column_to_row_down(activate)` | `move_column_to_workspace_down()` | Move column down |
| `move_column_to_row_up(activate)` | `move_column_to_workspace_up()` | Move column up |
| `move_row_down()` | `move_workspace_down()` | Reorder row down |
| `move_row_up()` | `move_workspace_up()` | Reorder row up |
| `move_row_to_index(idx)` | `move_workspace_to_idx()` | Move row to index |

### 1.5 Methods → render.rs

| Method | Notes |
|--------|-------|
| `render_elements()` | Simplified (no workspace switching) |
| `render_above_top_layer()` | Above layer rendering |

### 1.6 Methods → hit_test.rs

| Method | Notes |
|--------|-------|
| `window_under()` | Window hit testing |
| `resize_edges_under()` | Resize edge detection |

### 1.7 Methods → config.rs

| Method | Notes |
|--------|-------|
| `update_config()` | Config updates |
| `update_output_size()` | Output resize handling |
| `set_scale()` | Scale changes |

### 1.8 Types/Methods → insert_hint.rs

| Item | Notes |
|------|-------|
| `InsertHint` struct | Without workspace variant |
| `InsertPosition` enum | Simplified |
| Insert hint rendering | |

---

## 2. What Gets REMOVED (NOT migrated)

### 2.1 Types to DELETE

| Type | Line | Notes |
|------|------|-------|
| `WorkspaceSwitch` | ~50 | Workspace animation enum |
| `WorkspaceSwitchGesture` | ~80 | Gesture tracking struct |
| `InsertWorkspace` | ~120 | Workspace-based insert enum |
| `OverviewProgress` | ~140 | Overview mode enum |

### 2.2 Constants to DELETE

| Constant | Notes |
|----------|-------|
| `WORKSPACE_GESTURE_MOVEMENT` | Gesture threshold |
| `WORKSPACE_GESTURE_RUBBER_BAND` | Rubber band factor |
| `WORKSPACE_DND_EDGE_SCROLL_MOVEMENT` | DnD scroll threshold |

### 2.3 Fields to DELETE from Monitor struct

| Field | Notes |
|-------|-------|
| `workspaces: Vec<Workspace<W>>` | **Primary removal target** |
| `active_workspace_idx: usize` | No longer needed |
| `previous_workspace_id: Option<WorkspaceId>` | No longer needed |
| `workspace_switch: Option<WorkspaceSwitch>` | No longer needed |
| `overview_open: bool` | Overview removed |
| `overview_progress: Option<OverviewProgress>` | Overview removed |

### 2.4 Methods to DELETE (50+ methods)

#### Workspace Accessors (DELETE)
| Method | Line |
|--------|------|
| `into_workspaces()` | 387 |
| `active_workspace_idx()` | 423 |
| `active_workspace_ref()` | 427 |
| `active_workspace()` | 447 |
| `find_named_workspace()` | 431 |
| `find_named_workspace_index()` | 439 |

#### Workspace Management (DELETE)
| Method | Line |
|--------|------|
| `add_workspace_at()` | 474 |
| `add_workspace_top()` | 493 |
| `add_workspace_bottom()` | 497 |
| `activate_workspace()` | 501 |
| `activate_workspace_with_anim_config()` | 505 |
| `clean_up_workspaces()` | 698 |
| `unname_workspace()` | 729 |
| `remove_workspace_by_idx()` | 743 |
| `insert_workspace()` | 768 |
| `append_workspaces()` | 797 |

#### Workspace Navigation (DELETE - replaced by row navigation)
| Method | Line |
|--------|------|
| `move_down_or_to_workspace_down()` | 834 |
| `move_up_or_to_workspace_up()` | 840 |
| `focus_window_or_workspace_down()` | 846 |
| `focus_window_or_workspace_up()` | 852 |
| `move_to_workspace_up()` | 858 |
| `move_to_workspace_down()` | 892 |
| `move_to_workspace()` | 926 |
| `move_column_to_workspace_up()` | 985 |
| `move_column_to_workspace_down()` | 1006 |
| `move_column_to_workspace()` | 1027 |
| `switch_workspace_up()` | 1053 |
| `switch_workspace_down()` | 1067 |
| `switch_workspace()` | 1086 |
| `switch_workspace_auto_back_and_forth()` | 1090 |
| `switch_workspace_previous()` | 1102 |
| `move_workspace_down()` | 1317 |
| `move_workspace_up()` | 1343 |
| `move_workspace_to_idx()` | 1369 |

#### Workspace Rendering (DELETE)
| Method | Line |
|--------|------|
| `workspace_render_idx()` | 1474 |
| `workspaces_render_geo()` | 1549 |
| `workspaces_with_render_geo()` | 1573 |
| `workspaces_with_render_geo_idx()` | 1584 |
| `workspaces_with_render_geo_mut()` | 1595 |
| `workspace_under()` | 1607 |
| `workspace_under_narrow()` | 1622 |
| `render_insert_hint_between_workspaces()` | 1714 |
| `render_workspace_shadows()` | 1841 |

#### Workspace Gestures (DELETE)
| Method | Line |
|--------|------|
| `workspace_switch_gesture_begin()` | 1871 |
| `workspace_switch_gesture_update()` | 1921 |
| `workspace_switch_gesture_end()` | 2058 |
| `dnd_scroll_gesture_begin()` | ~2100 |
| `dnd_scroll_gesture_update()` | ~2120 |
| `dnd_scroll_gesture_end()` | ~2140 |

---

## 3. Execution Steps

### Step 1: Create module structure
```bash
mkdir -p src/layout/monitor
touch src/layout/monitor/{mod.rs,operations.rs,navigation.rs,render.rs,hit_test.rs,config.rs,insert_hint.rs}
```

### Step 2: Create mod.rs with core struct
1. Define `Monitor` struct with ONLY kept fields
2. Move `new()` - simplified without workspace initialization
3. Move output/canvas accessors
4. Add `pub mod` declarations for submodules

### Step 3: Create operations.rs
1. Move `add_window()`, `add_tile()`, `add_column()`
2. Update to use canvas directly (no workspace indirection)

### Step 4: Create navigation.rs (NEW CODE)
1. Create new row navigation methods
2. Delegate to `Canvas2D` for actual row operations
3. This is NEW code, not migrated code

### Step 5: Create render.rs
1. Move `render_elements()` - simplified for single canvas
2. Remove workspace switching visual effects
3. Remove overview rendering

### Step 6: Create hit_test.rs
1. Move `window_under()`, `resize_edges_under()`
2. Simplify (no workspace selection)

### Step 7: Create config.rs
1. Move `update_config()`, `update_output_size()`, `set_scale()`

### Step 8: Create insert_hint.rs
1. Move `InsertHint` struct (without workspace variant)
2. Move `InsertPosition` enum (simplified)
3. Move insert hint rendering

### Step 9: Update src/layout/mod.rs
1. Change `mod monitor;` to use new module
2. Update all imports

### Step 10: Delete old monitor.rs
1. Verify all needed code migrated
2. Delete the 2255-line monolith

---

## 4. Callers to Update

### 4.1 src/layout/mod.rs

All calls to Monitor methods need to be updated to use new method names:

| Old Call | New Call |
|----------|----------|
| `monitor.switch_workspace_down()` | `monitor.focus_row_down()` |
| `monitor.switch_workspace_up()` | `monitor.focus_row_up()` |
| `monitor.move_to_workspace_down(focus)` | `monitor.move_window_to_row_down(focus)` |
| `monitor.move_to_workspace_up(focus)` | `monitor.move_window_to_row_up(focus)` |
| `monitor.move_column_to_workspace_down(act)` | `monitor.move_column_to_row_down(act)` |
| `monitor.move_column_to_workspace_up(act)` | `monitor.move_column_to_row_up(act)` |
| `monitor.move_workspace_down()` | `monitor.move_row_down()` |
| `monitor.move_workspace_up()` | `monitor.move_row_up()` |
| ... | ... |

### 4.2 src/input/mod.rs

| Old Call | New Call |
|----------|----------|
| `mon.switch_workspace_down()` | `mon.focus_row_down()` |
| `mon.switch_workspace_up()` | `mon.focus_row_up()` |

---

## 5. Verification

After each step:
1. `cargo check` — must compile
2. After Step 10: `cargo test` — tests must pass
3. After Step 10: `./scripts/verify-golden.sh` — golden tests must pass

---

## 6. Dependencies

### Must Complete First:
- Part 2C (Layout methods) - because layout/mod.rs calls monitor methods

### Can Be Done After:
- Part 2E (Tests) - tests call layout methods, not monitor methods directly

---

## Summary

| Category | Count |
|----------|-------|
| New module files | 7 |
| Methods to migrate | ~20 |
| Methods to DELETE | ~50 |
| New methods to CREATE | ~12 |
| Lines removed | ~1400 |
| Lines remaining | ~800 |

---

*TEAM_012: Phase 1.5.3 Part 2D — Modular Refactor Analysis*
