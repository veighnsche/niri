# Phase 1.5.3 Part 2D: Complete Call Site Analysis - Monitor Methods

> **Status**: PENDING
> **Type**: BREAKING CHANGE — Workspace → Row transformation
> **Created by**: TEAM_012

---

## Overview

This document provides a complete analysis of all monitor methods in `src/layout/monitor.rs`
that need to be renamed for the Workspace → Row transformation.

**Scope**: ~490 workspace references in monitor.rs

---

## 1. Public Methods to Rename

### 1.1 Focus/Navigation Methods

| Line | Current Method | New Method | Callers |
|------|----------------|------------|---------|
| 846 | `focus_window_or_workspace_down()` | `focus_window_or_row_down()` | layout/mod.rs |
| 852 | `focus_window_or_workspace_up()` | `focus_window_or_row_up()` | layout/mod.rs |
| 1053 | `switch_workspace_up()` | `focus_row_up()` | layout/mod.rs, input/mod.rs |
| 1067 | `switch_workspace_down()` | `focus_row_down()` | layout/mod.rs, input/mod.rs |
| 1086 | `switch_workspace(idx)` | `focus_row(idx)` | layout/mod.rs |
| 1090 | `switch_workspace_auto_back_and_forth(idx)` | `focus_row_auto_back_and_forth(idx)` | layout/mod.rs |
| 1102 | `switch_workspace_previous()` | `focus_previous_position()` | layout/mod.rs |

### 1.2 Move Window/Column Methods

| Line | Current Method | New Method | Callers |
|------|----------------|------------|---------|
| 834 | `move_down_or_to_workspace_down()` | `move_down_or_to_row_down()` | layout/mod.rs |
| 840 | `move_up_or_to_workspace_up()` | `move_up_or_to_row_up()` | layout/mod.rs |
| 858 | `move_to_workspace_up(focus)` | `move_to_row_up(focus)` | layout/mod.rs |
| 892 | `move_to_workspace_down(focus)` | `move_to_row_down(focus)` | layout/mod.rs |
| 926 | `move_to_workspace(window, idx, activate)` | `move_to_row(window, idx, activate)` | layout/mod.rs |
| 985 | `move_column_to_workspace_up(activate)` | `move_column_to_row_up(activate)` | layout/mod.rs |
| 1006 | `move_column_to_workspace_down(activate)` | `move_column_to_row_down(activate)` | layout/mod.rs |
| 1027 | `move_column_to_workspace(idx, activate)` | `move_column_to_row(idx, activate)` | layout/mod.rs |

### 1.3 Reorder Row Methods

| Line | Current Method | New Method | Callers |
|------|----------------|------------|---------|
| 1317 | `move_workspace_down()` | `move_row_down()` | layout/mod.rs |
| 1343 | `move_workspace_up()` | `move_row_up()` | layout/mod.rs |
| 1369 | `move_workspace_to_idx(old_idx, new_idx)` | `move_row_to_index(old_idx, new_idx)` | layout/mod.rs |

### 1.4 Gesture Methods

| Line | Current Method | New Method | Callers |
|------|----------------|------------|---------|
| 1871 | `workspace_switch_gesture_begin(is_touchpad)` | `row_pan_gesture_begin(is_touchpad)` | layout/mod.rs |
| 1921 | `workspace_switch_gesture_update(delta, timestamp, is_touchpad)` | `row_pan_gesture_update(...)` | layout/mod.rs |
| 2058 | `workspace_switch_gesture_end(is_touchpad)` | `row_pan_gesture_end(is_touchpad)` | layout/mod.rs |

---

## 2. Internal/Helper Methods (KEEP or RENAME)

### 2.1 Methods to Keep (Internal Use)

These methods work with the Workspace type and may be kept until the type is renamed:

| Line | Current Method | Decision | Notes |
|------|----------------|----------|-------|
| 387 | `into_workspaces()` | KEEP | Returns Vec<Workspace> |
| 423 | `active_workspace_idx()` | KEEP | Returns index |
| 427 | `active_workspace_ref()` | KEEP | Returns &Workspace |
| 431 | `find_named_workspace(name)` | KEEP | Returns Option<&Workspace> |
| 439 | `find_named_workspace_index(name)` | KEEP | Returns Option<usize> |
| 447 | `active_workspace()` | KEEP | Returns &mut Workspace |
| 474 | `add_workspace_at(idx)` | KEEP | Creates Workspace |
| 493 | `add_workspace_top()` | KEEP | Creates Workspace |
| 497 | `add_workspace_bottom()` | KEEP | Creates Workspace |
| 501 | `activate_workspace(idx)` | KEEP | Internal |
| 505 | `activate_workspace_with_anim_config(...)` | KEEP | Internal |
| 610 | `add_column(workspace_idx, column, activate)` | KEEP | Internal |
| 698 | `clean_up_workspaces()` | KEEP | Internal |
| 729 | `unname_workspace(id)` | KEEP | Internal |
| 743 | `remove_workspace_by_idx(idx)` | KEEP | Internal |
| 768 | `insert_workspace(ws, idx, activate)` | KEEP | Internal |
| 797 | `append_workspaces(workspaces)` | KEEP | Internal |
| 1474 | `workspace_render_idx()` | KEEP | Rendering |
| 1549 | `workspaces_render_geo()` | KEEP | Rendering |
| 1573 | `workspaces_with_render_geo()` | KEEP | Rendering |
| 1584 | `workspaces_with_render_geo_idx()` | KEEP | Rendering |
| 1595 | `workspaces_with_render_geo_mut()` | KEEP | Rendering |
| 1607 | `workspace_under(point)` | KEEP | Hit testing |
| 1622 | `workspace_under_narrow(point)` | KEEP | Hit testing |
| 1714 | `render_insert_hint_between_workspaces(...)` | KEEP | Rendering |
| 1841 | `render_workspace_shadows(...)` | KEEP | Rendering |

---

## 3. Callers to Update

### 3.1 src/layout/mod.rs (Primary Caller)

| Line | Current Call | New Call |
|------|--------------|----------|
| ~1873 | `monitor.move_down_or_to_workspace_down()` | `monitor.move_down_or_to_row_down()` |
| ~1880 | `monitor.move_up_or_to_workspace_up()` | `monitor.move_up_or_to_row_up()` |
| ~2075 | `monitor.focus_window_or_workspace_down()` | `monitor.focus_window_or_row_down()` |
| ~2082 | `monitor.focus_window_or_workspace_up()` | `monitor.focus_window_or_row_up()` |
| ~2117 | `monitor.move_to_workspace_up(focus)` | `monitor.move_to_row_up(focus)` |
| ~2124 | `monitor.move_to_workspace_down(focus)` | `monitor.move_to_row_down(focus)` |
| ~2155 | `monitor.move_to_workspace(...)` | `monitor.move_to_row(...)` |
| ~2162 | `monitor.move_column_to_workspace_up(activate)` | `monitor.move_column_to_row_up(activate)` |
| ~2169 | `monitor.move_column_to_workspace_down(activate)` | `monitor.move_column_to_row_down(activate)` |
| ~2176 | `monitor.move_column_to_workspace(idx, activate)` | `monitor.move_column_to_row(idx, activate)` |
| ~2183 | `monitor.switch_workspace_up()` | `monitor.focus_row_up()` |
| ~2190 | `monitor.switch_workspace_down()` | `monitor.focus_row_down()` |
| ~2197 | `monitor.switch_workspace(idx)` | `monitor.focus_row(idx)` |
| ~2204 | `monitor.switch_workspace_auto_back_and_forth(idx)` | `monitor.focus_row_auto_back_and_forth(idx)` |
| ~2211 | `monitor.switch_workspace_previous()` | `monitor.focus_previous_position()` |
| ~4474 | `monitor.move_workspace_down()` | `monitor.move_row_down()` |
| ~4481 | `monitor.move_workspace_up()` | `monitor.move_row_up()` |
| ~4512 | `monitor.move_workspace_to_idx(old_idx, new_idx)` | `monitor.move_row_to_index(old_idx, new_idx)` |
| ~3588 | `monitor.workspace_switch_gesture_begin(is_touchpad)` | `monitor.row_pan_gesture_begin(is_touchpad)` |
| ~3605 | `monitor.workspace_switch_gesture_update(...)` | `monitor.row_pan_gesture_update(...)` |
| ~3631 | `monitor.workspace_switch_gesture_end(is_touchpad)` | `monitor.row_pan_gesture_end(is_touchpad)` |

### 3.2 src/input/mod.rs (Direct Monitor Calls)

| Line | Current Call | New Call |
|------|--------------|----------|
| ~1311 | `mon.switch_workspace_down()` | `mon.focus_row_down()` |
| ~1330 | `mon.switch_workspace_up()` | `mon.focus_row_up()` |

---

## 4. Internal Method Calls (Within monitor.rs)

These are internal calls within monitor.rs that need to be updated:

| Line | Current Call | New Call |
|------|--------------|----------|
| ~836 | `self.move_to_workspace_down(true)` | `self.move_to_row_down(true)` |
| ~842 | `self.move_to_workspace_up(true)` | `self.move_to_row_up(true)` |
| ~848 | `self.switch_workspace_down()` | `self.focus_row_down()` |
| ~854 | `self.switch_workspace_up()` | `self.focus_row_up()` |
| ~1055-1065 | Internal workspace switching logic | Update method names |
| ~1069-1084 | Internal workspace switching logic | Update method names |
| ~1092-1100 | Auto back-and-forth logic | Update method names |
| ~1104 | `self.switch_workspace(idx)` | `self.focus_row(idx)` |

---

## 5. Fields and Types (KEEP FOR NOW)

These are struct fields and type references that will be renamed in a future phase
when the Workspace type itself is renamed to Row:

| Location | Current | Future | Notes |
|----------|---------|--------|-------|
| struct Monitor | `workspaces: Vec<Workspace<W>>` | `rows: Vec<Row<W>>` | Major refactor |
| struct Monitor | `active_workspace_idx: usize` | `active_row_idx: usize` | Field rename |
| struct Monitor | `previous_workspace_id: Option<WorkspaceId>` | `previous_row_id: Option<RowId>` | Type rename |
| struct Monitor | `workspace_switch: Option<WorkspaceSwitch>` | `row_switch: Option<RowSwitch>` | Type rename |

---

## 6. Execution Plan

### Step 1: Rename Public Methods in monitor.rs
1. Rename all methods listed in Section 1
2. Update internal calls within monitor.rs

### Step 2: Update Callers in layout/mod.rs
1. Update all delegation calls to Monitor methods

### Step 3: Update Callers in input/mod.rs
1. Update direct Monitor method calls

### Step 4: Verify
1. `cargo check` - must compile
2. Continue to Part 2E (Tests)

---

## 7. Dependencies

### Must Complete First:
- Part 2C (Layout methods) - because layout/mod.rs calls monitor methods

### Can Be Done After:
- Part 2E (Tests) - tests call layout methods, not monitor methods directly

---

## Summary

### Total Methods to Rename: ~15
### Total Call Sites to Update: ~30+
### Files Affected: 3

1. `src/layout/monitor.rs` - Method definitions
2. `src/layout/mod.rs` - Delegation calls
3. `src/input/mod.rs` - Direct monitor calls

---

*TEAM_012: Phase 1.5.3 Part 2D Call Site Analysis*
