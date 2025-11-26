# Phase 1.5.3 Part 2B: Complete Call Site Analysis - Input Handlers

> **Status**: ANALYSIS COMPLETE
> **Type**: BREAKING CHANGE — Workspace → Row transformation
> **Created by**: TEAM_012

---

## Overview

This document provides a complete analysis of all call sites in `src/input/mod.rs`
that need to be modified for the Workspace → Row transformation.

---

## 1. Action Handler Match Arms

### 1.1 Focus Actions (DONE ✅)

| Line | Old Action | New Action | Layout Call | Status |
|------|------------|------------|-------------|--------|
| ~1223-1229 | `Action::FocusWindowOrWorkspaceDown` | `Action::FocusWindowOrRowDown` | `layout.focus_window_or_workspace_down()` | ✅ Action renamed |
| ~1231-1237 | `Action::FocusWindowOrWorkspaceUp` | `Action::FocusWindowOrRowUp` | `layout.focus_window_or_workspace_up()` | ✅ Action renamed |
| ~1299-1306 | `Action::FocusWorkspaceDown` | `Action::FocusRowDown` | `layout.switch_workspace_down()` | ✅ Action renamed |
| ~1307-1316 | `Action::FocusWorkspaceDownUnderMouse` | `Action::FocusRowDownUnderMouse` | `mon.switch_workspace_down()` | ✅ Action renamed |
| ~1318-1325 | `Action::FocusWorkspaceUp` | `Action::FocusRowUp` | `layout.switch_workspace_up()` | ✅ Action renamed |
| ~1326-1335 | `Action::FocusWorkspaceUpUnderMouse` | `Action::FocusRowUpUnderMouse` | `mon.switch_workspace_up()` | ✅ Action renamed |
| ~1337-1343 | `Action::FocusWorkspacePrevious` | `Action::FocusPreviousPosition` | `layout.switch_workspace_previous()` | ✅ Action renamed |

### 1.2 Move Window Actions (DONE ✅)

| Line | Old Action | New Action | Layout Call | Status |
|------|------------|------------|-------------|--------|
| ~980-989 | `Action::MoveWindowDownOrToWorkspaceDown` | `Action::MoveWindowDownOrToRowDown` | `layout.move_down_or_to_workspace_down()` | ✅ Action renamed |
| ~991-1000 | `Action::MoveWindowUpOrToWorkspaceUp` | `Action::MoveWindowUpOrToRowUp` | `layout.move_up_or_to_workspace_up()` | ✅ Action renamed |
| ~1265-1270 | `Action::MoveWindowToWorkspaceDown(focus)` | `Action::MoveWindowToRowDown(focus)` | `layout.move_to_workspace_down(focus)` | ✅ Action renamed |
| ~1272-1277 | `Action::MoveWindowToWorkspaceUp(focus)` | `Action::MoveWindowToRowUp(focus)` | `layout.move_to_workspace_up(focus)` | ✅ Action renamed |

### 1.3 Move Column Actions (DONE ✅)

| Line | Old Action | New Action | Layout Call | Status |
|------|------------|------------|-------------|--------|
| ~1279-1284 | `Action::MoveColumnToWorkspaceDown(focus)` | `Action::MoveColumnToRowDown(focus)` | `layout.move_column_to_workspace_down(focus)` | ✅ Action renamed |
| ~1286-1291 | `Action::MoveColumnToWorkspaceUp(focus)` | `Action::MoveColumnToRowUp(focus)` | `layout.move_column_to_workspace_up(focus)` | ✅ Action renamed |

### 1.4 Move/Reorder Row Actions (DONE ✅)

| Line | Old Action | New Action | Layout Call | Status |
|------|------------|------------|-------------|--------|
| ~1345-1349 | `Action::MoveWorkspaceDown` | `Action::MoveRowDown` | `layout.move_workspace_down()` | ✅ Action renamed |
| ~1351-1355 | `Action::MoveWorkspaceUp` | `Action::MoveRowUp` | `layout.move_workspace_up()` | ✅ Action renamed |
| ~1357-1362 | `Action::MoveWorkspaceToIndex(idx)` | `Action::MoveRowToIndex(idx)` | `layout.move_workspace_to_idx()` | ✅ Action renamed |

### 1.5 Row Naming Actions (DONE ✅)

| Line | Old Action | New Action | Layout Call | Status |
|------|------------|------------|-------------|--------|
| ~1364-1366 | `Action::SetWorkspaceName(name)` | `Action::SetRowName(name)` | `layout.set_workspace_name()` | ✅ Action renamed |
| ~1368-1370 | `Action::UnsetWorkspaceName` | `Action::UnsetRowName` | `layout.unset_workspace_name()` | ✅ Action renamed |

### 1.6 Move Row to Monitor Actions (DONE ✅)

| Line | Old Action | New Action | Layout Call | Status |
|------|------------|------------|-------------|--------|
| ~1870-1877 | `Action::MoveWorkspaceToMonitorLeft` | `Action::MoveRowToMonitorLeft` | `layout.move_workspace_to_output()` | ✅ Action renamed |
| ~1879-1886 | `Action::MoveWorkspaceToMonitorRight` | `Action::MoveRowToMonitorRight` | `layout.move_workspace_to_output()` | ✅ Action renamed |
| ~1888-1895 | `Action::MoveWorkspaceToMonitorDown` | `Action::MoveRowToMonitorDown` | `layout.move_workspace_to_output()` | ✅ Action renamed |
| ~1897-1904 | `Action::MoveWorkspaceToMonitorUp` | `Action::MoveRowToMonitorUp` | `layout.move_workspace_to_output()` | ✅ Action renamed |
| ~1906-1913 | `Action::MoveWorkspaceToMonitorPrevious` | `Action::MoveRowToMonitorPrevious` | `layout.move_workspace_to_output()` | ✅ Action renamed |
| ~1915-1922 | `Action::MoveWorkspaceToMonitorNext` | `Action::MoveRowToMonitorNext` | `layout.move_workspace_to_output()` | ✅ Action renamed |
| ~1924-1932 | `Action::MoveWorkspaceToMonitor(output)` | `Action::MoveRowToMonitor(output)` | `layout.move_workspace_to_output()` | ✅ Action renamed |

---

## 2. Removed Actions (No Longer Exist)

These actions were removed because they reference specific workspaces by index/name/id:

| Old Action | Reason for Removal |
|------------|-------------------|
| `Action::FocusWorkspace(WorkspaceReference)` | No jump to specific row by reference |
| `Action::MoveWindowToWorkspace(ref, focus)` | No move to specific row by reference |
| `Action::MoveWindowToWorkspaceById { ... }` | No move by window ID to specific row |
| `Action::MoveColumnToWorkspace(ref, focus)` | No move to specific row by reference |
| `Action::MoveWorkspaceToIndexByRef { ... }` | No reference-based reordering |
| `Action::SetWorkspaceNameByRef { ... }` | No reference-based naming |
| `Action::UnsetWorkSpaceNameByRef(ref)` | No reference-based unnaming |
| `Action::MoveWorkspaceToMonitorByRef { ... }` | No reference-based monitor move |

---

## 3. Other Input Handler Locations

### 3.1 Wheel Scroll Binds (DONE ✅)

| Line | Old Bind | New Bind | Status |
|------|----------|----------|--------|
| ~2966 | `Action::FocusWorkspaceUpUnderMouse` | `Action::FocusRowUpUnderMouse` | ✅ |
| ~2978 | `Action::FocusWorkspaceDownUnderMouse` | `Action::FocusRowDownUnderMouse` | ✅ |

### 3.2 Screenshot UI Action List (DONE ✅)

| Line | Old Action | New Action | Status |
|------|------------|------------|--------|
| ~4384 | `Action::MoveWindowUpOrToWorkspaceUp` | `Action::MoveWindowUpOrToRowUp` | ✅ |
| ~4387 | `Action::MoveWindowDownOrToWorkspaceDown` | `Action::MoveWindowDownOrToRowDown` | ✅ |

### 3.3 Overview Keybinds (DONE ✅)

| Line | Old Action | New Action | Status |
|------|------------|------------|--------|
| ~4422 | `Action::FocusWindowOrWorkspaceUp` | `Action::FocusWindowOrRowUp` | ✅ |
| ~4423 | `Action::FocusWindowOrWorkspaceDown` | `Action::FocusWindowOrRowDown` | ✅ |

---

## 4. Layout Method Calls (DEFERRED TO PART 2C)

The action handlers now use the new Row action names, but they still call the old
workspace-named layout methods. These will be renamed in Part 2C:

| Current Layout Call | Future Layout Call (Part 2C) |
|---------------------|------------------------------|
| `layout.focus_window_or_workspace_down()` | `layout.focus_window_or_row_down()` |
| `layout.focus_window_or_workspace_up()` | `layout.focus_window_or_row_up()` |
| `layout.move_down_or_to_workspace_down()` | `layout.move_down_or_to_row_down()` |
| `layout.move_up_or_to_workspace_up()` | `layout.move_up_or_to_row_up()` |
| `layout.move_to_workspace_down(focus)` | `layout.move_to_row_down(focus)` |
| `layout.move_to_workspace_up(focus)` | `layout.move_to_row_up(focus)` |
| `layout.move_column_to_workspace_down(focus)` | `layout.move_column_to_row_down(focus)` |
| `layout.move_column_to_workspace_up(focus)` | `layout.move_column_to_row_up(focus)` |
| `layout.switch_workspace_down()` | `layout.focus_row_down()` |
| `layout.switch_workspace_up()` | `layout.focus_row_up()` |
| `layout.switch_workspace_previous()` | `layout.focus_previous_position()` |
| `layout.move_workspace_down()` | `layout.move_row_down()` |
| `layout.move_workspace_up()` | `layout.move_row_up()` |
| `layout.move_workspace_to_idx()` | `layout.move_row_to_index()` |
| `layout.set_workspace_name()` | `layout.set_row_name()` |
| `layout.unset_workspace_name()` | `layout.unset_row_name()` |
| `layout.move_workspace_to_output()` | `layout.move_row_to_output()` |
| `mon.switch_workspace_down()` | `mon.focus_row_down()` |
| `mon.switch_workspace_up()` | `mon.focus_row_up()` |

---

## 5. Gesture Handlers (FUTURE WORK)

These are in `src/input/scroll_swipe_gesture.rs` and related files:

| File | Method | Notes |
|------|--------|-------|
| `scroll_swipe_gesture.rs` | `workspace_switch_gesture_*` | Rename to `row_pan_gesture_*` |
| `touch_overview_grab.rs` | Various | May be removed with overview |

**Status**: Not in scope for Part 2B - will be addressed separately.

---

## Summary

### Part 2B Status: ✅ COMPLETE (Action Handlers)

All action handler match arms in `src/input/mod.rs` have been updated to use the
new Row action names.

### What's Done:
- All `Action::*Workspace*` patterns → `Action::*Row*` patterns
- Wheel scroll binds updated
- Screenshot UI action list updated
- Overview keybinds updated

### What's Deferred:
- Layout method renames (Part 2C)
- Monitor method renames (Part 2D)
- Gesture handler renames (separate task)

---

*TEAM_012: Phase 1.5.3 Part 2B Call Site Analysis*
