# Phase 1.5.3 Part 2C: Remove Workspace Actions from Input

> **Status**: PENDING
> **Type**: BREAKING CHANGE — Complete removal, no backwards compatibility
> **Prerequisite**: Part 2B complete (Layout workspace switching removed)

---

## Overview

Remove all workspace-related Action handlers from `src/input/mod.rs`.

---

## Actions to Remove from Input Handler

### Focus Workspace Actions
| Action | Lines (approx) | Notes |
|--------|----------------|-------|
| `Action::FocusWorkspaceDown` | 1419-1425 | |
| `Action::FocusWorkspaceUp` | 1436-1442 | |
| `Action::FocusWorkspaceDownUnderMouse` | 1426-1435 | |
| `Action::FocusWorkspaceUpUnderMouse` | 1443-1452 | |
| `Action::FocusWorkspace(reference)` | 1453-1483 | |
| `Action::FocusWorkspacePrevious` | 1484-1490 | |

### Move Window to Workspace Actions
| Action | Lines (approx) | Notes |
|--------|----------------|-------|
| `Action::MoveWindowToWorkspaceDown(focus)` | 1265-1270 | |
| `Action::MoveWindowToWorkspaceUp(focus)` | 1271-1276 | |
| `Action::MoveWindowToWorkspace(reference, focus)` | 1277-1315 | |
| `Action::MoveWindowToWorkspaceById { ... }` | 1316-1372 | |

### Move Column to Workspace Actions
| Action | Lines (approx) | Notes |
|--------|----------------|-------|
| `Action::MoveColumnToWorkspaceDown(focus)` | 1373-1378 | |
| `Action::MoveColumnToWorkspaceUp(focus)` | 1379-1384 | |
| `Action::MoveColumnToWorkspace(reference, focus)` | 1385-1418 | |

### Move Workspace Actions
| Action | Lines (approx) | Notes |
|--------|----------------|-------|
| `Action::MoveWorkspaceDown` | 1491-1495 | |
| `Action::MoveWorkspaceUp` | 1496-1500 | |
| `Action::MoveWorkspaceToIndex(new_idx)` | 1501-... | |
| `Action::MoveWorkspaceToOutput(output)` | | |
| `Action::MoveWorkspaceToMonitorLeft` | | |
| `Action::MoveWorkspaceToMonitorRight` | | |
| `Action::MoveWorkspaceToMonitorUp` | | |
| `Action::MoveWorkspaceToMonitorDown` | | |

### Combined Actions (need rework, not removal)
| Action | Current Behavior | New Behavior |
|--------|------------------|--------------|
| `Action::MoveWindowDownOrToWorkspaceDown` | Move in column OR workspace | Move in column only |
| `Action::MoveWindowUpOrToWorkspaceUp` | Move in column OR workspace | Move in column only |
| `Action::FocusWindowOrWorkspaceDown` | Focus in column OR workspace | Focus in column only |
| `Action::FocusWindowOrWorkspaceUp` | Focus in column OR workspace | Focus in column only |

---

## Gesture Handlers to Remove

### Touchpad Gestures
| Handler | File | Notes |
|---------|------|-------|
| Workspace switch gesture begin | `on_gesture_swipe_begin` | |
| Workspace switch gesture update | `on_gesture_swipe_update` | |
| Workspace switch gesture end | `on_gesture_swipe_end` | |

### Touch Gestures
| Handler | File | Notes |
|---------|------|-------|
| Touch workspace gesture | `touch_overview_grab.rs` | |

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/input/mod.rs` | Remove Action handlers |
| `src/input/spatial_movement_grab.rs` | Remove workspace switch references |
| `src/input/touch_overview_grab.rs` | Remove workspace gesture |

---

## Verification

After completion:
1. `cargo check` — must compile
2. Continue to Part 2D (config Actions)

---

*TEAM_011: Phase 1.5.3 Part 2C*
