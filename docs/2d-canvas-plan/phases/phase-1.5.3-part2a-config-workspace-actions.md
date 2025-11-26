# Phase 1.5.3 Part 2D: Remove Workspace Actions from Config

> **Status**: PENDING
> **Type**: BREAKING CHANGE — Complete removal, no backwards compatibility
> **Prerequisite**: Part 2C complete (Input handlers removed)

---

## Overview

Remove all workspace-related Action variants from `niri-config/src/binds.rs`.

---

## Action Variants to Remove

### Focus Workspace
| Variant | Notes |
|---------|-------|
| `FocusWorkspace(WorkspaceReference)` | |
| `FocusWorkspaceDown` | |
| `FocusWorkspaceUp` | |
| `FocusWorkspaceDownUnderMouse` | |
| `FocusWorkspaceUpUnderMouse` | |
| `FocusWorkspacePrevious` | |

### Move Window to Workspace
| Variant | Notes |
|---------|-------|
| `MoveWindowToWorkspace(WorkspaceReference, bool)` | |
| `MoveWindowToWorkspaceDown(bool)` | |
| `MoveWindowToWorkspaceUp(bool)` | |
| `MoveWindowToWorkspaceById { ... }` | |

### Move Column to Workspace
| Variant | Notes |
|---------|-------|
| `MoveColumnToWorkspace(WorkspaceReference, bool)` | |
| `MoveColumnToWorkspaceDown(bool)` | |
| `MoveColumnToWorkspaceUp(bool)` | |

### Move Workspace
| Variant | Notes |
|---------|-------|
| `MoveWorkspaceDown` | |
| `MoveWorkspaceUp` | |
| `MoveWorkspaceToIndex(u8)` | |
| `MoveWorkspaceToOutput(String)` | |
| `MoveWorkspaceToMonitorLeft` | |
| `MoveWorkspaceToMonitorRight` | |
| `MoveWorkspaceToMonitorUp` | |
| `MoveWorkspaceToMonitorDown` | |

### Combined Actions (rename, not remove)
| Current Variant | New Variant | Notes |
|-----------------|-------------|-------|
| `MoveWindowDownOrToWorkspaceDown` | `MoveWindowDown` | Phase 4: will move to row |
| `MoveWindowUpOrToWorkspaceUp` | `MoveWindowUp` | Phase 4: will move to row |
| `FocusWindowOrWorkspaceDown` | `FocusWindowDown` | Phase 4: will focus row |
| `FocusWindowOrWorkspaceUp` | `FocusWindowUp` | Phase 4: will focus row |

---

## Types to Remove

| Type | File | Notes |
|------|------|-------|
| `WorkspaceReference` | `niri-config/src/lib.rs` | Index or name reference |

---

## Parsing to Remove

| Parser | File | Notes |
|--------|------|-------|
| `focus-workspace` | `binds.rs` | |
| `move-window-to-workspace` | `binds.rs` | |
| `move-column-to-workspace` | `binds.rs` | |
| `move-workspace-down` | `binds.rs` | |
| `move-workspace-up` | `binds.rs` | |
| `move-workspace-to-index` | `binds.rs` | |
| `move-workspace-to-output` | `binds.rs` | |
| `move-workspace-to-monitor-*` | `binds.rs` | |

---

## Hotkey Overlay Updates

| File | Changes |
|------|---------|
| `src/ui/hotkey_overlay.rs` | Remove workspace actions from display |

---

## Verification

After completion:
1. `cargo check` — must compile
2. Continue to Part 2E (tests)

---

*TEAM_011: Phase 1.5.3 Part 2D*
