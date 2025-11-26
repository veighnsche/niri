# Phase 1.5.3 Part 2C: Complete Call Site Analysis - Layout Methods

> **Status**: PENDING
> **Type**: BREAKING CHANGE — Workspace → Row transformation
> **Created by**: TEAM_012

---

## Overview

This document provides a complete analysis of all layout methods in `src/layout/mod.rs`
that need to be renamed for the Workspace → Row transformation.

**Scope**: ~600 workspace references in layout/mod.rs

---

## 1. Public Methods to Rename

### 1.1 Focus/Navigation Methods

| Line | Current Method | New Method | Callers |
|------|----------------|------------|---------|
| 2071 | `focus_window_or_workspace_down()` | `focus_window_or_row_down()` | input/mod.rs |
| 2078 | `focus_window_or_workspace_up()` | `focus_window_or_row_up()` | input/mod.rs |
| 2179 | `switch_workspace_up()` | `focus_row_up()` | input/mod.rs |
| 2186 | `switch_workspace_down()` | `focus_row_down()` | input/mod.rs |
| 2193 | `switch_workspace(idx)` | `focus_row(idx)` | input/mod.rs, tests.rs |
| 2200 | `switch_workspace_auto_back_and_forth(idx)` | `focus_row_auto_back_and_forth(idx)` | input/mod.rs |
| 2207 | `switch_workspace_previous()` | `focus_previous_position()` | input/mod.rs |

### 1.2 Move Window/Column Methods

| Line | Current Method | New Method | Callers |
|------|----------------|------------|---------|
| 1869 | `move_down_or_to_workspace_down()` | `move_down_or_to_row_down()` | input/mod.rs |
| 1876 | `move_up_or_to_workspace_up()` | `move_up_or_to_row_up()` | input/mod.rs |
| 2113 | `move_to_workspace_up(focus)` | `move_to_row_up(focus)` | input/mod.rs |
| 2120 | `move_to_workspace_down(focus)` | `move_to_row_down(focus)` | input/mod.rs |
| 2127 | `move_to_workspace(window, idx, activate)` | `move_to_row(window, idx, activate)` | input/mod.rs, tests.rs |
| 2158 | `move_column_to_workspace_up(activate)` | `move_column_to_row_up(activate)` | input/mod.rs |
| 2165 | `move_column_to_workspace_down(activate)` | `move_column_to_row_down(activate)` | input/mod.rs |
| 2172 | `move_column_to_workspace(idx, activate)` | `move_column_to_row(idx, activate)` | input/mod.rs, tests.rs |

### 1.3 Reorder Row Methods

| Line | Current Method | New Method | Callers |
|------|----------------|------------|---------|
| 4470 | `move_workspace_down()` | `move_row_down()` | input/mod.rs |
| 4477 | `move_workspace_up()` | `move_row_up()` | input/mod.rs |
| 4484 | `move_workspace_to_idx(res, new_idx)` | `move_row_to_index(res, new_idx)` | input/mod.rs, tests.rs |

### 1.4 Row Naming Methods

| Line | Current Method | New Method | Callers |
|------|----------------|------------|---------|
| 4515 | `set_workspace_name(name, ref)` | `set_row_name(name)` | input/mod.rs, tests.rs |
| 4565 | `unset_workspace_name(ref)` | `unset_row_name()` | input/mod.rs, tests.rs |
| 1335 | `unname_workspace(name)` | `unname_row(name)` | internal |
| 1339 | `unname_workspace_by_ref(ref)` | REMOVE (uses WorkspaceReference) | internal |
| 1346 | `unname_workspace_by_id(id)` | `unname_row_by_id(id)` | internal |

### 1.5 Move Row to Output Methods

| Line | Current Method | New Method | Callers |
|------|----------------|------------|---------|
| 3415 | `move_workspace_to_output(output)` | `move_row_to_output(output)` | input/mod.rs |
| 3430 | `move_workspace_to_output_by_id(idx, old_output, new_output)` | `move_row_to_output_by_id(...)` | input/mod.rs, handlers/mod.rs |

### 1.6 Gesture Methods

| Line | Current Method | New Method | Callers |
|------|----------------|------------|---------|
| 3584 | `workspace_switch_gesture_begin(output, is_touchpad)` | `row_pan_gesture_begin(...)` | input/scroll_swipe_gesture.rs |
| 3601 | `workspace_switch_gesture_update(delta, timestamp, is_touchpad)` | `row_pan_gesture_update(...)` | input/scroll_swipe_gesture.rs |
| 3627 | `workspace_switch_gesture_end(is_touchpad)` | `row_pan_gesture_end(...)` | input/scroll_swipe_gesture.rs |

---

## 2. Query/Lookup Methods (KEEP or RENAME)

### 2.1 Methods to Keep (Internal Use)

These methods are used internally and may be kept with workspace naming until
the Workspace type itself is renamed to Row:

| Line | Current Method | Decision | Notes |
|------|----------------|----------|-------|
| 1259 | `find_workspace_by_id(id)` | KEEP | Returns Workspace type |
| 1285 | `find_workspace_by_name(name)` | KEEP | Returns Workspace type |
| 1314 | `find_workspace_by_ref(ref)` | REMOVE | Uses WorkspaceReference |
| 1606 | `active_workspace()` | KEEP | Returns Workspace type |
| 1620 | `active_workspace_mut()` | KEEP | Returns Workspace type |
| 1784 | `monitor_for_workspace(name)` | KEEP | Internal use |
| 2351 | `workspace_under(point)` | KEEP | Internal use |
| 2900 | `ensure_named_workspace(config)` | KEEP | Config handling |
| 4625 | `toggle_overview_to_workspace(idx)` | KEEP | Overview handling |
| 4885 | `workspaces()` | KEEP | Iterator over Workspace type |
| 4919 | `workspaces_mut()` | KEEP | Iterator over Workspace type |

### 2.2 Methods to Remove (Use WorkspaceReference)

| Line | Method | Reason |
|------|--------|--------|
| 1314 | `find_workspace_by_ref(reference)` | Uses WorkspaceReference type |
| 1339 | `unname_workspace_by_ref(reference)` | Uses WorkspaceReference type |

---

## 3. Callers to Update

### 3.1 src/input/mod.rs

| Line | Current Call | New Call |
|------|--------------|----------|
| ~985 | `layout.move_down_or_to_workspace_down()` | `layout.move_down_or_to_row_down()` |
| ~996 | `layout.move_up_or_to_workspace_up()` | `layout.move_up_or_to_row_up()` |
| ~1225 | `layout.focus_window_or_workspace_down()` | `layout.focus_window_or_row_down()` |
| ~1233 | `layout.focus_window_or_workspace_up()` | `layout.focus_window_or_row_up()` |
| ~1267 | `layout.move_to_workspace_down(focus)` | `layout.move_to_row_down(focus)` |
| ~1274 | `layout.move_to_workspace_up(focus)` | `layout.move_to_row_up(focus)` |
| ~1281 | `layout.move_column_to_workspace_down(focus)` | `layout.move_column_to_row_down(focus)` |
| ~1288 | `layout.move_column_to_workspace_up(focus)` | `layout.move_column_to_row_up(focus)` |
| ~1301 | `layout.switch_workspace_down()` | `layout.focus_row_down()` |
| ~1320 | `layout.switch_workspace_up()` | `layout.focus_row_up()` |
| ~1339 | `layout.switch_workspace_previous()` | `layout.focus_previous_position()` |
| ~1347 | `layout.move_workspace_down()` | `layout.move_row_down()` |
| ~1353 | `layout.move_workspace_up()` | `layout.move_row_up()` |
| ~1360 | `layout.move_workspace_to_idx(...)` | `layout.move_row_to_index(...)` |
| ~1366 | `layout.set_workspace_name(...)` | `layout.set_row_name(...)` |
| ~1370 | `layout.unset_workspace_name(...)` | `layout.unset_row_name()` |
| ~1873+ | `layout.move_workspace_to_output(...)` | `layout.move_row_to_output(...)` |

### 3.2 src/handlers/mod.rs

| Line | Current Call | New Call |
|------|--------------|----------|
| ~576 | `find_output_and_workspace_index(reference)` | Keep (uses WorkspaceReference) |
| ~596 | `find_output_and_workspace_index(reference)` | Keep (uses WorkspaceReference) |
| ~591 | `layout.move_workspace_to_output_by_id(...)` | `layout.move_row_to_output_by_id(...)` |

### 3.3 src/niri.rs

| Line | Current Call | New Call |
|------|--------------|----------|
| ~3783 | `find_output_and_workspace_index(reference)` | Keep (uses WorkspaceReference) |

### 3.4 src/layout/tests.rs

| Line | Current Call | New Call |
|------|--------------|----------|
| ~1138 | `layout.focus_window_or_workspace_down()` | `layout.focus_window_or_row_down()` |
| ~1139 | `layout.focus_window_or_workspace_up()` | `layout.focus_window_or_row_up()` |
| ~1169 | `layout.move_down_or_to_workspace_down()` | `layout.move_down_or_to_row_down()` |
| ~1170 | `layout.move_up_or_to_workspace_up()` | `layout.move_up_or_to_row_up()` |
| ~1190 | `layout.switch_workspace_down()` | `layout.focus_row_down()` |
| ~1191 | `layout.switch_workspace_up()` | `layout.focus_row_up()` |
| ~1192 | `layout.switch_workspace(idx)` | `layout.focus_row(idx)` |
| ~1194 | `layout.switch_workspace_auto_back_and_forth(idx)` | `layout.focus_row_auto_back_and_forth(idx)` |
| ~1196 | `layout.switch_workspace_previous()` | `layout.focus_previous_position()` |
| ~1197 | `layout.move_to_workspace_down(focus)` | `layout.move_to_row_down(focus)` |
| ~1198 | `layout.move_to_workspace_up(focus)` | `layout.move_to_row_up(focus)` |
| ~1204 | `layout.move_to_workspace(...)` | `layout.move_to_row(...)` |
| ~1206 | `layout.move_column_to_workspace_down(focus)` | `layout.move_column_to_row_down(focus)` |
| ~1207 | `layout.move_column_to_workspace_up(focus)` | `layout.move_column_to_row_up(focus)` |
| ~1208 | `layout.move_column_to_workspace(idx, focus)` | `layout.move_column_to_row(idx, focus)` |
| ~1241 | `layout.move_workspace_down()` | `layout.move_row_down()` |
| ~1242 | `layout.move_workspace_up()` | `layout.move_row_up()` |
| ~1268 | `layout.move_workspace_to_idx(...)` | `layout.move_row_to_index(...)` |
| ~1273 | `layout.move_workspace_to_idx(...)` | `layout.move_row_to_index(...)` |
| ~1282 | `layout.move_workspace_to_output(...)` | `layout.move_row_to_output(...)` |
| ~1313 | `layout.move_workspace_to_output_by_id(...)` | `layout.move_row_to_output_by_id(...)` |
| ~1518 | `layout.move_workspace_to_output(...)` | `layout.move_row_to_output(...)` |

### 3.5 src/input/scroll_swipe_gesture.rs

| Line | Current Call | New Call |
|------|--------------|----------|
| TBD | `layout.workspace_switch_gesture_begin(...)` | `layout.row_pan_gesture_begin(...)` |
| TBD | `layout.workspace_switch_gesture_update(...)` | `layout.row_pan_gesture_update(...)` |
| TBD | `layout.workspace_switch_gesture_end(...)` | `layout.row_pan_gesture_end(...)` |

---

## 4. Internal Method Calls (Within layout/mod.rs)

These are calls within layout/mod.rs that delegate to Monitor methods:

| Line | Current Call | Delegates To |
|------|--------------|--------------|
| ~1873 | `monitor.move_down_or_to_workspace_down()` | monitor.rs |
| ~1880 | `monitor.move_up_or_to_workspace_up()` | monitor.rs |
| ~2075 | `monitor.focus_window_or_workspace_down()` | monitor.rs |
| ~2082 | `monitor.focus_window_or_workspace_up()` | monitor.rs |
| ~2117 | `monitor.move_to_workspace_up(focus)` | monitor.rs |
| ~2124 | `monitor.move_to_workspace_down(focus)` | monitor.rs |
| ~2155 | `monitor.move_to_workspace(...)` | monitor.rs |
| ~2162 | `monitor.move_column_to_workspace_up(activate)` | monitor.rs |
| ~2169 | `monitor.move_column_to_workspace_down(activate)` | monitor.rs |
| ~2176 | `monitor.move_column_to_workspace(idx, activate)` | monitor.rs |
| ~2183 | `monitor.switch_workspace_up()` | monitor.rs |
| ~2190 | `monitor.switch_workspace_down()` | monitor.rs |
| ~2197 | `monitor.switch_workspace(idx)` | monitor.rs |
| ~2204 | `monitor.switch_workspace_auto_back_and_forth(idx)` | monitor.rs |
| ~2211 | `monitor.switch_workspace_previous()` | monitor.rs |
| ~4474 | `monitor.move_workspace_down()` | monitor.rs |
| ~4481 | `monitor.move_workspace_up()` | monitor.rs |
| ~4512 | `monitor.move_workspace_to_idx(old_idx, new_idx)` | monitor.rs |

---

## 5. Execution Plan

### Step 1: Rename Public Methods in layout/mod.rs
1. Rename all methods listed in Section 1
2. Update method bodies to call renamed Monitor methods

### Step 2: Update Callers
1. Update src/input/mod.rs calls
2. Update src/handlers/mod.rs calls
3. Update src/layout/tests.rs calls
4. Update src/input/scroll_swipe_gesture.rs calls

### Step 3: Remove WorkspaceReference-dependent Methods
1. Remove `find_workspace_by_ref()`
2. Remove `unname_workspace_by_ref()`
3. Update `set_workspace_name()` to not take reference parameter
4. Update `unset_workspace_name()` to not take reference parameter

### Step 4: Verify
1. `cargo check` - must compile
2. Continue to Part 2D (Monitor methods)

---

## Summary

### Total Methods to Rename: ~25
### Total Call Sites to Update: ~50+
### Files Affected: 5

1. `src/layout/mod.rs` - Method definitions
2. `src/input/mod.rs` - Action handlers
3. `src/handlers/mod.rs` - Protocol handlers
4. `src/layout/tests.rs` - Test operations
5. `src/input/scroll_swipe_gesture.rs` - Gesture handlers

---

*TEAM_012: Phase 1.5.3 Part 2C Call Site Analysis*
