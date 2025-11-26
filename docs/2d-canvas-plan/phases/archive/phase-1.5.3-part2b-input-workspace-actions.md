# Phase 1.5.3 Part 2B: Replace Workspace Action Handlers with Row Handlers

> **Status**: PENDING
> **Type**: BREAKING CHANGE — Workspace → Row transformation
> **Prerequisite**: Part 2A complete (Config Actions replaced)

---

## Overview

Replace workspace-related Action handlers with row-based equivalents in `src/input/mod.rs`.

**Strategy**: Match the Config transformations from Part 2A.

---

## Action Handlers: Replace

### Focus: Replace Workspace → Row
| Old Handler | New Handler | New Call |
|-------------|-------------|----------|
| `Action::FocusWorkspaceDown` | `Action::FocusRowDown` | `layout.focus_row_down()` |
| `Action::FocusWorkspaceUp` | `Action::FocusRowUp` | `layout.focus_row_up()` |
| `Action::FocusWorkspaceDownUnderMouse` | `Action::FocusRowDownUnderMouse` | `mon.focus_row_down()` |
| `Action::FocusWorkspaceUpUnderMouse` | `Action::FocusRowUpUnderMouse` | `mon.focus_row_up()` |
| `Action::FocusWorkspace(ref)` | **REMOVE** | — |
| `Action::FocusWorkspacePrevious` | `Action::FocusPreviousPosition` | `layout.focus_previous_position()` |

### Move Window: Replace Workspace → Row
| Old Handler | New Handler | New Call |
|-------------|-------------|----------|
| `Action::MoveWindowToWorkspaceDown(focus)` | `Action::MoveWindowToRowDown(focus)` | `layout.move_window_to_row_down(focus)` |
| `Action::MoveWindowToWorkspaceUp(focus)` | `Action::MoveWindowToRowUp(focus)` | `layout.move_window_to_row_up(focus)` |
| `Action::MoveWindowToWorkspace(ref, focus)` | **REMOVE** | — |
| `Action::MoveWindowToWorkspaceById { ... }` | **REMOVE** | — |

### Move Column: Replace Workspace → Row
| Old Handler | New Handler | New Call |
|-------------|-------------|----------|
| `Action::MoveColumnToWorkspaceDown(focus)` | `Action::MoveColumnToRowDown(focus)` | `layout.move_column_to_row_down(focus)` |
| `Action::MoveColumnToWorkspaceUp(focus)` | `Action::MoveColumnToRowUp(focus)` | `layout.move_column_to_row_up(focus)` |
| `Action::MoveColumnToWorkspace(ref, focus)` | **REMOVE** | — |

### Move Workspace → Row
| Old Handler | New Handler | New Call |
|-------------|-------------|----------|
| `Action::MoveWorkspaceDown` | `Action::MoveRowDown` | `layout.move_row_down()` |
| `Action::MoveWorkspaceUp` | `Action::MoveRowUp` | `layout.move_row_up()` |
| `Action::MoveWorkspaceToIndex(idx)` | `Action::MoveRowToIndex(idx)` | `layout.move_row_to_index(idx)` |
| `Action::MoveWorkspaceToMonitor*` | `Action::MoveRowToMonitor*` | `layout.move_row_to_monitor_*()` |
| `Action::MoveWorkspaceToOutput(out)` | `Action::MoveRowToOutput(out)` | `layout.move_row_to_output(out)` |

### Combined Actions: Update
| Old Handler | New Handler | Notes |
|-------------|-------------|-------|
| `Action::FocusWindowOrWorkspaceDown` | `Action::FocusWindowOrRowDown` | At edge → row |
| `Action::FocusWindowOrWorkspaceUp` | `Action::FocusWindowOrRowUp` | At edge → row |
| `Action::MoveWindowDownOrToWorkspaceDown` | `Action::MoveWindowDownOrToRowDown` | At edge → row |
| `Action::MoveWindowUpOrToWorkspaceUp` | `Action::MoveWindowUpOrToRowUp` | At edge → row |

---

## Gesture Handlers: Replace with Row/Camera Pan

### Touchpad Gestures
| Old Handler | New Handler | Notes |
|-------------|-------------|-------|
| `workspace_switch_gesture_begin` | `row_pan_gesture_begin` | Vertical camera pan |
| `workspace_switch_gesture_update` | `row_pan_gesture_update` | |
| `workspace_switch_gesture_end` | `row_pan_gesture_end` | |

### Touch Gestures (Overview)
| Old Handler | Notes |
|-------------|-------|
| `touch_overview_grab.rs` | **REMOVE** — Overview mode removed |

### DnD Edge Scrolling
| Old Handler | New Handler | Notes |
|-------------|-------------|-------|
| `dnd_scroll_gesture_*` | `dnd_row_scroll_gesture_*` | Vertical pan to rows |

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/input/mod.rs` | Replace Action handlers |
| `src/input/spatial_movement_grab.rs` | Update for row navigation |
| `src/input/touch_overview_grab.rs` | **DELETE** — Overview removed |

---

## Verification

After completion:
1. `cargo check` — must compile (will fail until Part 2C done)
2. Continue to Part 2C (Layout methods)

---

*TEAM_011: Phase 1.5.3 Part 2B — Input transformation*
