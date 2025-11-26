# Phase 1.5.3 Part 2B: Remove Workspace Switching from Layout

> **Status**: PENDING
> **Type**: BREAKING CHANGE — Complete removal, no backwards compatibility
> **Prerequisite**: Part 2A complete (Monitor workspace switching removed)

---

## Overview

Remove all workspace switching methods from `src/layout/mod.rs` that delegate to Monitor.

---

## Methods to Remove from Layout

### Workspace Switching
| Method | Calls | Notes |
|--------|-------|-------|
| `switch_workspace_up()` | `monitor.switch_workspace_up()` | |
| `switch_workspace_down()` | `monitor.switch_workspace_down()` | |
| `switch_workspace(idx)` | `monitor.switch_workspace(idx)` | |
| `switch_workspace_auto_back_and_forth(idx)` | `monitor.switch_workspace_auto_back_and_forth(idx)` | |
| `switch_workspace_previous()` | `monitor.switch_workspace_previous()` | |

### Move Window/Column to Workspace
| Method | Calls | Notes |
|--------|-------|-------|
| `move_to_workspace_up(focus)` | `monitor.move_to_workspace_up(focus)` | |
| `move_to_workspace_down(focus)` | `monitor.move_to_workspace_down(focus)` | |
| `move_to_workspace(window, idx, activate)` | `monitor.move_to_workspace(...)` | |
| `move_column_to_workspace_up(focus)` | `monitor.move_column_to_workspace_up(focus)` | |
| `move_column_to_workspace_down(focus)` | `monitor.move_column_to_workspace_down(focus)` | |
| `move_column_to_workspace(idx, focus)` | `monitor.move_column_to_workspace(idx, focus)` | |

### Move Workspace (reorder)
| Method | Calls | Notes |
|--------|-------|-------|
| `move_workspace_down()` | `monitor.move_workspace_down()` | |
| `move_workspace_up()` | `monitor.move_workspace_up()` | |
| `move_workspace_to_idx(idx)` | `monitor.move_workspace_to_idx(...)` | |

### Combined Actions (need rework, not removal)
| Method | Current Behavior | New Behavior |
|--------|------------------|--------------|
| `move_down_or_to_workspace_down()` | Delegates to monitor | Call `active_workspace().move_down()` only |
| `move_up_or_to_workspace_up()` | Delegates to monitor | Call `active_workspace().move_up()` only |
| `focus_window_or_workspace_down()` | Delegates to monitor | Call `active_workspace().focus_down()` only |
| `focus_window_or_workspace_up()` | Delegates to monitor | Call `active_workspace().focus_up()` only |

---

## Workspace Gesture Methods to Remove

| Method | Notes |
|--------|-------|
| `workspace_switch_gesture_begin(output, is_touchpad)` | |
| `workspace_switch_gesture_update(delta_y, timestamp, is_touchpad)` | |
| `workspace_switch_gesture_end(is_touchpad, output)` | |

---

## Verification

After completion:
1. `cargo check` — must compile
2. Fix all call sites in Part 2C before running tests

---

*TEAM_011: Phase 1.5.3 Part 2B*
