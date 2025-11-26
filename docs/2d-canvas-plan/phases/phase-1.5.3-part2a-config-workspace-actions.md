# Phase 1.5.3 Part 2A: Replace Workspace Actions with Row Actions in Config

> **Status**: PENDING
> **Type**: BREAKING CHANGE — Workspace → Row transformation
> **Prerequisite**: None (first step in Part 2)

---

## Overview

Replace workspace-related Action variants with row-based equivalents in `niri-config/src/binds.rs`.

**Strategy**: Don't just remove — replace with Row equivalents where applicable.

---

## Action Variants: Remove vs Replace

### Focus: REPLACE with Row
| Old Variant | New Variant | Notes |
|-------------|-------------|-------|
| `FocusWorkspaceDown` | `FocusRowDown` | Navigate to row below |
| `FocusWorkspaceUp` | `FocusRowUp` | Navigate to row above |
| `FocusWorkspaceDownUnderMouse` | `FocusRowDownUnderMouse` | |
| `FocusWorkspaceUpUnderMouse` | `FocusRowUpUnderMouse` | |
| `FocusWorkspace(WorkspaceReference)` | **REMOVE** | No jump to specific row |
| `FocusWorkspacePrevious` | `FocusPreviousPosition` | Browser-like back |

### Move Window to Workspace: REPLACE with Row
| Old Variant | New Variant | Notes |
|-------------|-------------|-------|
| `MoveWindowToWorkspaceDown(bool)` | `MoveWindowToRowDown(bool)` | |
| `MoveWindowToWorkspaceUp(bool)` | `MoveWindowToRowUp(bool)` | |
| `MoveWindowToWorkspace(ref, bool)` | **REMOVE** | No jump to specific row |
| `MoveWindowToWorkspaceById { ... }` | **REMOVE** | |

### Move Column to Workspace: REPLACE with Row
| Old Variant | New Variant | Notes |
|-------------|-------------|-------|
| `MoveColumnToWorkspaceDown(bool)` | `MoveColumnToRowDown(bool)` | |
| `MoveColumnToWorkspaceUp(bool)` | `MoveColumnToRowUp(bool)` | |
| `MoveColumnToWorkspace(ref, bool)` | **REMOVE** | |

### Move Workspace: REPLACE with Row
| Old Variant | New Variant | Notes |
|-------------|-------------|-------|
| `MoveWorkspaceDown` | `MoveRowDown` | Reorder row |
| `MoveWorkspaceUp` | `MoveRowUp` | Reorder row |
| `MoveWorkspaceToIndex(usize)` | `MoveRowToIndex(usize)` | |
| `MoveWorkspaceToMonitor*` | `MoveRowToMonitor*` | Shrinks spanning windows |
| `MoveWorkspaceToOutput(String)` | `MoveRowToOutput(String)` | |

### Workspace Naming: REPLACE with Row
| Old Variant | New Variant | Notes |
|-------------|-------------|-------|
| `SetWorkspaceName(String)` | `SetRowName(String)` | Rows can be named |
| `UnsetWorkspaceName` | `UnsetRowName` | |

### Combined Actions: SIMPLIFY
| Old Variant | New Variant | Notes |
|-------------|-------------|-------|
| `FocusWindowOrWorkspaceDown` | `FocusWindowOrRowDown` | At edge → focus row |
| `FocusWindowOrWorkspaceUp` | `FocusWindowOrRowUp` | At edge → focus row |
| `MoveWindowDownOrToWorkspaceDown` | `MoveWindowDownOrToRowDown` | At edge → move to row |
| `MoveWindowUpOrToWorkspaceUp` | `MoveWindowUpOrToRowUp` | At edge → move to row |

---

## Types to Replace

| Old Type | New Type | Notes |
|----------|----------|-------|
| `WorkspaceReference` | **REMOVE** | Rows use index only |

---

## Parsing to Update

| Old Parser | New Parser | Notes |
|------------|------------|-------|
| `focus-workspace-down` | `focus-row-down` | |
| `focus-workspace-up` | `focus-row-up` | |
| `move-window-to-workspace-down` | `move-window-to-row-down` | |
| `move-window-to-workspace-up` | `move-window-to-row-up` | |
| `move-column-to-workspace-down` | `move-column-to-row-down` | |
| `move-column-to-workspace-up` | `move-column-to-row-up` | |
| `move-workspace-down` | `move-row-down` | |
| `move-workspace-up` | `move-row-up` | |
| `move-workspace-to-index` | `move-row-to-index` | |
| `set-workspace-name` | `set-row-name` | |

---

## Hotkey Overlay Updates

| File | Changes |
|------|---------|
| `src/ui/hotkey_overlay.rs` | Replace "Workspace" with "Row" in display strings |

---

## Verification

After completion:
1. `cargo check` — must compile (will fail until Part 2B done)
2. Continue to Part 2B (Input handlers)

---

*TEAM_011: Phase 1.5.3 Part 2A — Config transformation*
