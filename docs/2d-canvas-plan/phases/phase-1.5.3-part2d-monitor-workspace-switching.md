# Phase 1.5.3 Part 2A: Remove Workspace Switching from Monitor

> **Status**: PENDING
> **Type**: BREAKING CHANGE — Complete removal, no backwards compatibility
> **Prerequisite**: Part 1 complete (Monitor methods migrated to Canvas2D)

---

## Overview

Remove all workspace switching infrastructure from `src/layout/monitor.rs`.

---

## Types to Remove

| Type | Lines | Notes |
|------|-------|-------|
| `WorkspaceSwitch` enum | ~119-123 | Animation/Gesture variants |
| `WorkspaceSwitchGesture` struct | ~125-153 | Gesture tracking state |
| `InsertWorkspace` enum | ~158-162 | Used for insert hints |

---

## Constants to Remove

| Constant | Line | Notes |
|----------|------|-------|
| `WORKSPACE_GESTURE_MOVEMENT` | ~40 | Touchpad gesture distance |
| `WORKSPACE_GESTURE_RUBBER_BAND` | ~42-45 | Rubber band config |
| `WORKSPACE_DND_EDGE_SCROLL_MOVEMENT` | ~50 | DnD scroll distance |

---

## Fields to Remove from Monitor

| Field | Notes |
|-------|-------|
| `workspace_switch: Option<WorkspaceSwitch>` | In-progress switch state |
| `previous_workspace_id: Option<WorkspaceId>` | For back-and-forth |

---

## Methods to Remove from Monitor

### Workspace Switching
| Method | Notes |
|--------|-------|
| `switch_workspace_up()` | |
| `switch_workspace_down()` | |
| `switch_workspace(idx)` | |
| `switch_workspace_auto_back_and_forth(idx)` | |
| `switch_workspace_previous()` | |
| `previous_workspace_idx()` | Private helper |
| `activate_workspace(idx)` | Core switching logic |
| `activate_workspace_with_anim_config(idx, config)` | |

### Move Window/Column to Workspace
| Method | Notes |
|--------|-------|
| `move_to_workspace_up(focus)` | |
| `move_to_workspace_down(focus)` | |
| `move_to_workspace(window, idx, activate)` | |
| `move_column_to_workspace_up(activate)` | |
| `move_column_to_workspace_down(activate)` | |
| `move_column_to_workspace(idx, activate)` | |

### Move Workspace (reorder)
| Method | Notes |
|--------|-------|
| `move_workspace_down()` | |
| `move_workspace_up()` | |
| `move_workspace_to_idx(old_idx, new_idx)` | |

### Workspace Gestures
| Method | Notes |
|--------|-------|
| `workspace_switch_gesture_begin(is_touchpad)` | |
| `workspace_switch_gesture_update(delta_y, timestamp, is_touchpad)` | |
| `workspace_switch_gesture_end(is_touchpad)` | |
| `dnd_scroll_gesture_begin()` | |
| `dnd_scroll_gesture_scroll(pos, speed)` | |
| `dnd_scroll_gesture_end()` | |

### Combined Actions (need rework, not removal)
| Method | Current Behavior | New Behavior |
|--------|------------------|--------------|
| `move_down_or_to_workspace_down()` | Move in column OR to workspace | Move in column only (Phase 4: move to row) |
| `move_up_or_to_workspace_up()` | Move in column OR to workspace | Move in column only (Phase 4: move to row) |
| `focus_window_or_workspace_down()` | Focus in column OR workspace | Focus in column only (Phase 4: focus row) |
| `focus_window_or_workspace_up()` | Focus in column OR workspace | Focus in column only (Phase 4: focus row) |

---

## Methods to Update

### `advance_animations()`
Remove workspace_switch animation handling.

### `are_animations_ongoing()`
Remove workspace_switch check.

### `are_transitions_ongoing()`
Remove workspace_switch check.

### `workspace_render_idx()`
Remove — only used for workspace switching animation.

### `render_above_top_layer()`
Remove workspace_switch check.

---

## Impl Blocks to Remove

| Impl | Notes |
|------|-------|
| `impl WorkspaceSwitch` | current_idx, target_idx, offset, is_animation_ongoing |
| `impl WorkspaceSwitchGesture` | min_max, animate_from |
| `impl InsertWorkspace` | existing_id |

---

## Verification

After completion:
1. `cargo check` — must compile
2. Fix all call sites in Part 2B before running tests

---

*TEAM_011: Phase 1.5.3 Part 2A*
