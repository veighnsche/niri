# Phase 1.5.3 Part 2E: Complete Call Site Analysis - Tests

> **Status**: PENDING
> **Type**: BREAKING CHANGE — Workspace → Row transformation
> **Created by**: TEAM_012

---

## Overview

This document provides a complete analysis of all test operations and test functions
in `src/layout/tests.rs` that need to be modified for the Workspace → Row transformation.

**Scope**: ~143 workspace references in tests.rs

---

## 1. Op Enum Variants to Rename

### 1.1 Focus Operations

| Line | Current Variant | New Variant |
|------|-----------------|-------------|
| 490 | `FocusWindowOrWorkspaceDown` | `FocusWindowOrRowDown` |
| 491 | `FocusWindowOrWorkspaceUp` | `FocusWindowOrRowUp` |
| 528 | `FocusWorkspaceDown` | `FocusRowDown` |
| 529 | `FocusWorkspaceUp` | `FocusRowUp` |
| 530 | `FocusWorkspace(usize)` | `FocusRow(usize)` |
| 531 | `FocusWorkspaceAutoBackAndForth(usize)` | `FocusRowAutoBackAndForth(usize)` |
| 532 | `FocusWorkspacePrevious` | `FocusPreviousPosition` |

### 1.2 Move Window Operations

| Line | Current Variant | New Variant |
|------|-----------------|-------------|
| 507 | `MoveWindowDownOrToWorkspaceDown` | `MoveWindowDownOrToRowDown` |
| 508 | `MoveWindowUpOrToWorkspaceUp` | `MoveWindowUpOrToRowUp` |
| 533 | `MoveWindowToWorkspaceDown(bool)` | `MoveWindowToRowDown(bool)` |
| 534 | `MoveWindowToWorkspaceUp(bool)` | `MoveWindowToRowUp(bool)` |
| 535-540 | `MoveWindowToWorkspace { ... }` | `MoveWindowToRow { ... }` |

### 1.3 Move Column Operations

| Line | Current Variant | New Variant |
|------|-----------------|-------------|
| 541 | `MoveColumnToWorkspaceDown(bool)` | `MoveColumnToRowDown(bool)` |
| 542 | `MoveColumnToWorkspaceUp(bool)` | `MoveColumnToRowUp(bool)` |
| 543 | `MoveColumnToWorkspace(usize, bool)` | `MoveColumnToRow(usize, bool)` |

### 1.4 Reorder Row Operations

| Line | Current Variant | New Variant |
|------|-----------------|-------------|
| 544 | `MoveWorkspaceDown` | `MoveRowDown` |
| 545 | `MoveWorkspaceUp` | `MoveRowUp` |
| 546-551 | `MoveWorkspaceToIndex { ... }` | `MoveRowToIndex { ... }` |
| 552-557 | `MoveWorkspaceToMonitor { ... }` | `MoveRowToMonitor { ... }` |
| 666 | `MoveWorkspaceToOutput(usize)` | `MoveRowToOutput(usize)` |

### 1.5 Row Naming Operations

| Line | Current Variant | New Variant |
|------|-----------------|-------------|
| 558-563 | `SetWorkspaceName { ... }` | `SetRowName { ... }` |
| 564-567 | `UnsetWorkspaceName { ... }` | `UnsetRowName { ... }` |

### 1.6 Workspace Management Operations (KEEP or RENAME)

| Line | Current Variant | Decision | Notes |
|------|-----------------|----------|-------|
| 434-441 | `AddNamedWorkspace { ... }` | KEEP | Creates workspace/row |
| 442-445 | `UnnameWorkspace { ws_name }` | KEEP | Internal |
| 446-459 | `UpdateWorkspaceLayoutConfig { ... }` | KEEP | Config |
| 460-466 | `AddWindowToNamedWorkspace { ... }` | KEEP | Window placement |

### 1.7 Gesture Operations

| Line | Current Variant | New Variant |
|------|-----------------|-------------|
| ~668 | `WorkspaceSwitchGestureBegin { ... }` | `RowPanGestureBegin { ... }` |
| ~674 | `WorkspaceSwitchGestureUpdate { ... }` | `RowPanGestureUpdate { ... }` |
| ~680 | `WorkspaceSwitchGestureEnd { ... }` | `RowPanGestureEnd { ... }` |

---

## 2. Op Handler Match Arms to Update

### 2.1 Focus Handlers

| Line | Current Match | New Match | Layout Call |
|------|---------------|-----------|-------------|
| 1138 | `Op::FocusWindowOrWorkspaceDown` | `Op::FocusWindowOrRowDown` | `focus_window_or_row_down()` |
| 1139 | `Op::FocusWindowOrWorkspaceUp` | `Op::FocusWindowOrRowUp` | `focus_window_or_row_up()` |
| 1190 | `Op::FocusWorkspaceDown` | `Op::FocusRowDown` | `focus_row_down()` |
| 1191 | `Op::FocusWorkspaceUp` | `Op::FocusRowUp` | `focus_row_up()` |
| 1192 | `Op::FocusWorkspace(idx)` | `Op::FocusRow(idx)` | `focus_row(idx)` |
| 1193-1195 | `Op::FocusWorkspaceAutoBackAndForth(idx)` | `Op::FocusRowAutoBackAndForth(idx)` | `focus_row_auto_back_and_forth(idx)` |
| 1196 | `Op::FocusWorkspacePrevious` | `Op::FocusPreviousPosition` | `focus_previous_position()` |

### 2.2 Move Window Handlers

| Line | Current Match | New Match | Layout Call |
|------|---------------|-----------|-------------|
| 1169 | `Op::MoveWindowDownOrToWorkspaceDown` | `Op::MoveWindowDownOrToRowDown` | `move_down_or_to_row_down()` |
| 1170 | `Op::MoveWindowUpOrToWorkspaceUp` | `Op::MoveWindowUpOrToRowUp` | `move_up_or_to_row_up()` |
| 1197 | `Op::MoveWindowToWorkspaceDown(focus)` | `Op::MoveWindowToRowDown(focus)` | `move_to_row_down(focus)` |
| 1198 | `Op::MoveWindowToWorkspaceUp(focus)` | `Op::MoveWindowToRowUp(focus)` | `move_to_row_up(focus)` |
| 1199-1205 | `Op::MoveWindowToWorkspace { ... }` | `Op::MoveWindowToRow { ... }` | `move_to_row(...)` |

### 2.3 Move Column Handlers

| Line | Current Match | New Match | Layout Call |
|------|---------------|-----------|-------------|
| 1206 | `Op::MoveColumnToWorkspaceDown(focus)` | `Op::MoveColumnToRowDown(focus)` | `move_column_to_row_down(focus)` |
| 1207 | `Op::MoveColumnToWorkspaceUp(focus)` | `Op::MoveColumnToRowUp(focus)` | `move_column_to_row_up(focus)` |
| 1208 | `Op::MoveColumnToWorkspace(idx, focus)` | `Op::MoveColumnToRow(idx, focus)` | `move_column_to_row(idx, focus)` |

### 2.4 Reorder Row Handlers

| Line | Current Match | New Match | Layout Call |
|------|---------------|-----------|-------------|
| 1241 | `Op::MoveWorkspaceDown` | `Op::MoveRowDown` | `move_row_down()` |
| 1242 | `Op::MoveWorkspaceUp` | `Op::MoveRowUp` | `move_row_up()` |
| 1243-1273 | `Op::MoveWorkspaceToIndex { ... }` | `Op::MoveRowToIndex { ... }` | `move_row_to_index(...)` |
| 1274-1315 | `Op::MoveWorkspaceToMonitor { ... }` | `Op::MoveRowToMonitor { ... }` | `move_row_to_output(...)` |
| 1512-1520 | `Op::MoveWorkspaceToOutput(id)` | `Op::MoveRowToOutput(id)` | `move_row_to_output(...)` |

### 2.5 Row Naming Handlers

| Line | Current Match | New Match | Layout Call |
|------|---------------|-----------|-------------|
| 891-898 | `Op::SetWorkspaceName { ... }` | `Op::SetRowName { ... }` | `set_row_name(...)` |
| 899-903 | `Op::UnsetWorkspaceName { ... }` | `Op::UnsetRowName { ... }` | `unset_row_name()` |

### 2.6 Gesture Handlers

| Line | Current Match | New Match | Layout Call |
|------|---------------|-----------|-------------|
| 1542-1552 | `Op::WorkspaceSwitchGestureBegin { ... }` | `Op::RowPanGestureBegin { ... }` | `row_pan_gesture_begin(...)` |
| 1553-1559 | `Op::WorkspaceSwitchGestureUpdate { ... }` | `Op::RowPanGestureUpdate { ... }` | `row_pan_gesture_update(...)` |
| 1560-1565 | `Op::WorkspaceSwitchGestureEnd { ... }` | `Op::RowPanGestureEnd { ... }` | `row_pan_gesture_end(...)` |

---

## 3. Test Functions to Update

### 3.1 Test Functions with Workspace in Name

| Line | Current Function | New Function |
|------|------------------|--------------|
| 2021 | `move_to_workspace_by_idx_does_not_leave_empty_workspaces()` | `move_to_row_by_idx_does_not_leave_empty_rows()` |
| 2260 | `move_workspace_to_output()` | `move_row_to_output()` |
| 3293 | `move_workspace_to_same_monitor_doesnt_reorder()` | `move_row_to_same_monitor_doesnt_reorder()` |
| 3424 | `move_column_to_workspace_unfocused_with_multiple_monitors()` | `move_column_to_row_unfocused_with_multiple_monitors()` |
| 3487 | `move_column_to_workspace_down_focus_false_on_floating_window()` | `move_column_to_row_down_focus_false_on_floating_window()` |
| 3510 | `move_column_to_workspace_focus_false_on_floating_window()` | `move_column_to_row_focus_false_on_floating_window()` |
| 3597 | `move_column_to_workspace_maximize_and_fullscreen()` | `move_column_to_row_maximize_and_fullscreen()` |

### 3.2 Golden Test Functions

| Line | Current Function | New Function |
|------|------------------|--------------|
| 651 | `golden_w3_focus_window_or_workspace_down()` | `golden_w3_focus_window_or_row_down()` |

---

## 4. Test Data Arrays to Update

### 4.1 OPS_AFFECTING_WORKSPACE_LAYOUT

This array lists operations that affect workspace layout and needs to be renamed
and updated:

| Line | Current Name | New Name |
|------|--------------|----------|
| ~1680 | `OPS_AFFECTING_WORKSPACE_LAYOUT` | `OPS_AFFECTING_ROW_LAYOUT` |

Contents need to be updated to use new Op variant names.

### 4.2 OPS_AFFECTING_FLOATING_LAYOUT

May contain workspace-related operations that need updating.

---

## 5. Imports to Update

| Line | Current Import | New Import |
|------|----------------|------------|
| 7 | `WorkspaceReference` | REMOVE (if no longer used) |

---

## 6. Execution Plan

### Step 1: Rename Op Enum Variants
1. Update all variant names in the Op enum definition
2. This will cause compiler errors at all usage sites

### Step 2: Update Op Handlers
1. Update all match arms in the apply_op() function
2. Update layout method calls to use new names

### Step 3: Update Test Functions
1. Rename test functions
2. Update test bodies to use new Op variants

### Step 4: Update Test Data Arrays
1. Rename OPS_AFFECTING_WORKSPACE_LAYOUT
2. Update array contents

### Step 5: Verify
1. `cargo test --lib` - all tests must pass
2. `./scripts/verify-golden.sh` - golden tests must pass

---

## 7. Dependencies

### Must Complete First:
- Part 2C (Layout methods) - tests call layout methods
- Part 2D (Monitor methods) - layout methods call monitor methods

### Verification After:
- Golden snapshot tests must still pass
- All unit tests must still pass

---

## Summary

### Total Op Variants to Rename: ~25
### Total Test Functions to Rename: ~8
### Total Match Arms to Update: ~30+
### Files Affected: 1

1. `src/layout/tests.rs` - All changes

---

*TEAM_012: Phase 1.5.3 Part 2E Call Site Analysis*
