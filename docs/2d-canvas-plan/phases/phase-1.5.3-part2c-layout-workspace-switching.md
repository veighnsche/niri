# Phase 1.5.3 Part 2C: Replace Layout Workspace Methods with Row Methods

> **Status**: PENDING
> **Type**: BREAKING CHANGE — Workspace → Row transformation
> **Prerequisite**: Part 2B complete (Input handlers replaced)

---

## Overview

Replace workspace methods in `src/layout/mod.rs` with row-based equivalents that delegate to Monitor/Canvas2D.

**Strategy**: Match the transformations from Parts 2A and 2B.

---

## Methods: Replace Workspace → Row

### Focus: Replace
| Old Method | New Method | Delegates To |
|------------|------------|--------------|
| `switch_workspace_up()` | `focus_row_up()` | `monitor.focus_row_up()` |
| `switch_workspace_down()` | `focus_row_down()` | `monitor.focus_row_down()` |
| `switch_workspace(idx)` | **REMOVE** | — |
| `switch_workspace_auto_back_and_forth(idx)` | **REMOVE** | — |
| `switch_workspace_previous()` | `focus_previous_position()` | `monitor.focus_previous_position()` |

### Move Window: Replace
| Old Method | New Method | Delegates To |
|------------|------------|--------------|
| `move_to_workspace_up(focus)` | `move_window_to_row_up(focus)` | `monitor.move_window_to_row_up(focus)` |
| `move_to_workspace_down(focus)` | `move_window_to_row_down(focus)` | `monitor.move_window_to_row_down(focus)` |
| `move_to_workspace(win, idx, act)` | **REMOVE** | — |

### Move Column: Replace
| Old Method | New Method | Delegates To |
|------------|------------|--------------|
| `move_column_to_workspace_up(focus)` | `move_column_to_row_up(focus)` | `monitor.move_column_to_row_up(focus)` |
| `move_column_to_workspace_down(focus)` | `move_column_to_row_down(focus)` | `monitor.move_column_to_row_down(focus)` |
| `move_column_to_workspace(idx, focus)` | **REMOVE** | — |

### Move Row (reorder): Replace
| Old Method | New Method | Delegates To |
|------------|------------|--------------|
| `move_workspace_down()` | `move_row_down()` | `monitor.move_row_down()` |
| `move_workspace_up()` | `move_row_up()` | `monitor.move_row_up()` |
| `move_workspace_to_idx(idx)` | `move_row_to_index(idx)` | `monitor.move_row_to_index(idx)` |

### Combined Actions: Update
| Old Method | New Method | New Behavior |
|------------|------------|--------------|
| `move_down_or_to_workspace_down()` | `move_down_or_to_row_down()` | At edge → move to row |
| `move_up_or_to_workspace_up()` | `move_up_or_to_row_up()` | At edge → move to row |
| `focus_window_or_workspace_down()` | `focus_window_or_row_down()` | At edge → focus row |
| `focus_window_or_workspace_up()` | `focus_window_or_row_up()` | At edge → focus row |

---

## Gesture Methods: Replace

| Old Method | New Method | Notes |
|------------|------------|-------|
| `workspace_switch_gesture_begin(out, is_tp)` | `row_pan_gesture_begin(out, is_tp)` | Vertical pan |
| `workspace_switch_gesture_update(dy, ts, is_tp)` | `row_pan_gesture_update(dy, ts, is_tp)` | |
| `workspace_switch_gesture_end(is_tp, out)` | `row_pan_gesture_end(is_tp, out)` | |

---

## Helper Methods: Remove (workspace-specific)

| Method | Notes |
|--------|-------|
| `find_output_and_workspace_index()` | Workspace-specific lookup |
| `workspace_for_each_*()` | Workspace iteration |

---

## Verification

After completion:
1. `cargo check` — must compile (will fail until Part 2D done)
2. Continue to Part 2D (Monitor refactor)

---

*TEAM_011: Phase 1.5.3 Part 2C — Layout transformation*
