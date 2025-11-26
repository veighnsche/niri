# Phase 1.5.3 Part 2A: Complete Call Site Analysis

> **Status**: ANALYSIS COMPLETE
> **Type**: BREAKING CHANGE — Workspace → Row transformation
> **Created by**: TEAM_012

---

## Overview

This document provides a complete analysis of all call sites that need to be modified
for the Workspace → Row transformation in the config/IPC layer.

---

## 1. niri-ipc/src/lib.rs

### 1.1 Action Enum Variants (DONE ✅)

| Line | Old Variant | New Variant | Status |
|------|-------------|-------------|--------|
| ~366 | `FocusWindowOrWorkspaceDown` | `FocusWindowOrRowDown` | ✅ |
| ~368 | `FocusWindowOrWorkspaceUp` | `FocusWindowOrRowUp` | ✅ |
| ~402 | `MoveWindowDownOrToWorkspaceDown` | `MoveWindowDownOrToRowDown` | ✅ |
| ~404 | `MoveWindowUpOrToWorkspaceUp` | `MoveWindowUpOrToRowUp` | ✅ |
| ~462 | `FocusWorkspaceDown` | `FocusRowDown` | ✅ |
| ~464 | `FocusWorkspaceUp` | `FocusRowUp` | ✅ |
| ~466 | `FocusWorkspace { reference }` | REMOVED | ✅ |
| ~472 | `FocusWorkspacePrevious` | `FocusPreviousPosition` | ✅ |
| ~468-502 | `MoveWindowToWorkspace*` | `MoveWindowToRow*` | ✅ |
| ~486-502 | `MoveColumnToWorkspace*` | `MoveColumnToRow*` | ✅ |
| ~504-516 | `MoveWorkspaceDown/Up/ToIndex` | `MoveRowDown/Up/ToIndex` | ✅ |
| ~518-528 | `SetWorkspaceName/UnsetWorkspaceName` | `SetRowName/UnsetRowName` | ✅ |
| ~700-720 | `MoveWorkspaceToMonitor*` | `MoveRowToMonitor*` | ✅ |

### 1.2 Types (DONE ✅)

| Line | Old Type | New Type | Status |
|------|----------|----------|--------|
| ~878-888 | `WorkspaceReferenceArg` | REMOVED | ✅ |
| ~1537-1552 | `FromStr for WorkspaceReferenceArg` | REMOVED | ✅ |

### 1.3 Remaining Workspace References (NOT in scope for Part 2A)

These are data structures, not actions, and will be addressed in later phases:

| Line | Item | Notes |
|------|------|-------|
| ~1217 | `Window::workspace_id` | IPC data structure - keep for now |
| ~1307-1346 | `Workspace` struct | IPC data structure - keep for now |
| ~1400-1440 | `WorkspaceActivated/Removed` events | IPC events - keep for now |

---

## 2. niri-config/src/binds.rs

### 2.1 Action Enum Variants (DONE ✅)

| Line | Old Variant | New Variant | Status |
|------|-------------|-------------|--------|
| ~179 | `FocusWindowOrWorkspaceDown` | `FocusWindowOrRowDown` | ✅ |
| ~180 | `FocusWindowOrWorkspaceUp` | `FocusWindowOrRowUp` | ✅ |
| ~194 | `MoveWindowDownOrToWorkspaceDown` | `MoveWindowDownOrToRowDown` | ✅ |
| ~195 | `MoveWindowUpOrToWorkspaceUp` | `MoveWindowUpOrToRowUp` | ✅ |
| ~213-228 | `FocusWorkspace*` | `FocusRow*` / REMOVED | ✅ |
| ~221-238 | `MoveWindowToWorkspace*` | `MoveWindowToRow*` / REMOVED | ✅ |
| ~233-238 | `MoveColumnToWorkspace*` | `MoveColumnToRow*` / REMOVED | ✅ |
| ~239-252 | `MoveWorkspace*` | `MoveRow*` | ✅ |
| ~253-261 | `SetWorkspaceName*` | `SetRowName` / REMOVED | ✅ |
| ~325-330 | `MoveWorkspaceToMonitor*` | `MoveRowToMonitor*` | ✅ |

### 2.2 From<niri_ipc::Action> Implementation (DONE ✅)

| Line Range | Changes | Status |
|------------|---------|--------|
| ~422-423 | `FocusWindowOrWorkspace*` → `FocusWindowOrRow*` | ✅ |
| ~441-444 | `MoveWindowDownOrToWorkspace*` → `MoveWindowDownOrToRow*` | ✅ |
| ~467-486 | `FocusWorkspace*` → `FocusRow*` / REMOVED | ✅ |
| ~470-501 | `MoveWindowToWorkspace*` → `MoveWindowToRow*` / REMOVED | ✅ |
| ~493-501 | `MoveColumnToWorkspace*` → `MoveColumnToRow*` / REMOVED | ✅ |
| ~502-519 | `MoveWorkspace*` → `MoveRow*` / REMOVED | ✅ |
| ~504-519 | `SetWorkspaceName*` → `SetRowName` / REMOVED | ✅ |
| ~591-620 | `MoveWorkspaceToMonitor*` → `MoveRowToMonitor*` | ✅ |

### 2.3 WorkspaceReference Type (KEPT FOR INTERNAL USE)

| Line | Item | Status | Notes |
|------|------|--------|-------|
| ~612-619 | `WorkspaceReference` enum | KEPT | Still used by src/layout, src/niri, src/handlers |

**Remaining internal users of WorkspaceReference:**
- `src/niri.rs:19` - import
- `src/niri.rs:3783-3790` - `find_output_and_workspace_index()`
- `src/layout/mod.rs:42` - import
- `src/layout/mod.rs:1316-1340` - `find_workspace_by_ref()`, `unname_workspace_by_ref()`
- `src/layout/mod.rs:4515` - `set_workspace_name()`
- `src/layout/mod.rs:4565` - `unset_workspace_name()`
- `src/layout/tests.rs:7` - import
- `src/layout/tests.rs:896,901` - test helpers
- `src/handlers/mod.rs:576,596` - `activate_workspace()`, `assign_workspace()`

**Decision**: Keep `WorkspaceReference` until Part 2C/2D when layout methods are refactored.

---

## 3. src/ui/hotkey_overlay.rs (DONE ✅)

### 3.1 action_name() Function

| Line | Old Match | New Match | Status |
|------|-----------|-----------|--------|
| ~467 | `FocusWorkspaceDown` | `FocusRowDown` | ✅ |
| ~468 | `FocusWorkspaceUp` | `FocusRowUp` | ✅ |
| ~469 | `MoveColumnToWorkspaceDown` | `MoveColumnToRowDown` | ✅ |
| ~470 | `MoveColumnToWorkspaceUp` | `MoveColumnToRowUp` | ✅ |
| ~471 | `MoveWindowToWorkspaceDown` | `MoveWindowToRowDown` | ✅ |
| ~472 | `MoveWindowToWorkspaceUp` | `MoveWindowToRowUp` | ✅ |

### 3.2 important_actions() Function

| Line | Old Action | New Action | Status |
|------|------------|------------|--------|
| ~219 | `FocusWorkspaceDown` | `FocusRowDown` | ✅ |
| ~220 | `FocusWorkspaceUp` | `FocusRowUp` | ✅ |
| ~226 | `MoveColumnToWorkspaceDown` | `MoveColumnToRowDown` | ✅ |
| ~231 | `MoveWindowToWorkspaceDown` | `MoveWindowToRowDown` | ✅ |
| ~233 | `MoveWindowToWorkspaceDown(true)` | `MoveWindowToRowDown(true)` | ✅ |
| ~235 | `MoveColumnToWorkspaceDown(true)` | `MoveColumnToRowDown(true)` | ✅ |
| ~241 | `MoveColumnToWorkspaceUp` | `MoveColumnToRowUp` | ✅ |
| ~246 | `MoveWindowToWorkspaceUp` | `MoveWindowToRowUp` | ✅ |
| ~248 | `MoveWindowToWorkspaceUp(true)` | `MoveWindowToRowUp(true)` | ✅ |
| ~250 | `MoveColumnToWorkspaceUp(true)` | `MoveColumnToRowUp(true)` | ✅ |

---

## 4. src/input/mod.rs (DONE ✅)

### 4.1 Action Handlers

| Line | Old Handler | New Handler | Status |
|------|-------------|-------------|--------|
| ~980-1000 | `MoveWindowDownOrToWorkspaceDown/Up` | `MoveWindowDownOrToRowDown/Up` | ✅ |
| ~1223-1237 | `FocusWindowOrWorkspaceDown/Up` | `FocusWindowOrRowDown/Up` | ✅ |
| ~1265-1291 | `MoveWindowToWorkspaceDown/Up` | `MoveWindowToRowDown/Up` | ✅ |
| ~1279-1291 | `MoveColumnToWorkspaceDown/Up` | `MoveColumnToRowDown/Up` | ✅ |
| ~1299-1343 | `FocusWorkspaceDown/Up/UnderMouse` | `FocusRowDown/Up/UnderMouse` | ✅ |
| ~1337-1343 | `FocusWorkspacePrevious` | `FocusPreviousPosition` | ✅ |
| ~1345-1371 | `MoveWorkspaceDown/Up/ToIndex` | `MoveRowDown/Up/ToIndex` | ✅ |
| ~1364-1371 | `SetWorkspaceName/UnsetWorkspaceName` | `SetRowName/UnsetRowName` | ✅ |
| ~1870-1932 | `MoveWorkspaceToMonitor*` | `MoveRowToMonitor*` | ✅ |
| ~2966,2978 | Wheel scroll binds | Updated to Row | ✅ |
| ~4384-4387 | Screenshot UI action list | Updated to Row | ✅ |
| ~4422-4423 | Overview keybinds | Updated to Row | ✅ |

### 4.2 Layout Method Calls (NOT RENAMED YET)

These still call the old workspace method names - will be addressed in Part 2C:

| Line | Current Call | Future Call (Part 2C) |
|------|--------------|----------------------|
| ~985 | `layout.move_down_or_to_workspace_down()` | `layout.move_down_or_to_row_down()` |
| ~996 | `layout.move_up_or_to_workspace_up()` | `layout.move_up_or_to_row_up()` |
| ~1225 | `layout.focus_window_or_workspace_down()` | `layout.focus_window_or_row_down()` |
| ~1233 | `layout.focus_window_or_workspace_up()` | `layout.focus_window_or_row_up()` |
| ~1267 | `layout.move_to_workspace_down()` | `layout.move_to_row_down()` |
| ~1274 | `layout.move_to_workspace_up()` | `layout.move_to_row_up()` |
| ~1281 | `layout.move_column_to_workspace_down()` | `layout.move_column_to_row_down()` |
| ~1288 | `layout.move_column_to_workspace_up()` | `layout.move_column_to_row_up()` |
| ~1301 | `layout.switch_workspace_down()` | `layout.focus_row_down()` |
| ~1311,1330 | `mon.switch_workspace_down/up()` | `mon.focus_row_down/up()` |
| ~1320 | `layout.switch_workspace_up()` | `layout.focus_row_up()` |
| ~1339 | `layout.switch_workspace_previous()` | `layout.focus_previous_position()` |
| ~1347 | `layout.move_workspace_down()` | `layout.move_row_down()` |
| ~1353 | `layout.move_workspace_up()` | `layout.move_row_up()` |
| ~1360 | `layout.move_workspace_to_idx()` | `layout.move_row_to_index()` |
| ~1366 | `layout.set_workspace_name()` | `layout.set_row_name()` |
| ~1370 | `layout.unset_workspace_name()` | `layout.unset_row_name()` |
| ~1873,1882,... | `layout.move_workspace_to_output()` | `layout.move_row_to_output()` |

---

## Summary

### Part 2A Status: ✅ COMPLETE (Action Variants)

All action variant renames in niri-ipc and niri-config are done.

### Deferred to Later Parts:

1. **Part 2C**: Layout method renames (600+ references in layout/mod.rs)
2. **Part 2D**: Monitor method renames (490+ references in monitor.rs)
3. **Part 2E**: Test updates (143+ references in tests.rs)
4. **Future Phase**: IPC data structures (Workspace struct, events, Window.workspace_id)

---

*TEAM_012: Phase 1.5.3 Part 2A Call Site Analysis*
