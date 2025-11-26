# Phase 1.5.3 Part 2E: Replace Workspace Tests with Row Tests

> **Status**: PENDING
> **Type**: BREAKING CHANGE — Workspace → Row transformation
> **Prerequisite**: Part 2D complete (Monitor refactored)

---

## Overview

Replace workspace test operations with row-based equivalents in `src/layout/tests.rs`.

**Strategy**: Tests that make sense for rows get transformed. Tests that are workspace-specific get removed.

---

## Test Operations: Replace vs Remove

### Op Variants: REPLACE with Row
| Old Op | New Op | Notes |
|--------|--------|-------|
| `Op::FocusWorkspaceDown` | `Op::FocusRowDown` | |
| `Op::FocusWorkspaceUp` | `Op::FocusRowUp` | |
| `Op::MoveWindowToWorkspaceDown` | `Op::MoveWindowToRowDown` | |
| `Op::MoveWindowToWorkspaceUp` | `Op::MoveWindowToRowUp` | |
| `Op::MoveColumnToWorkspaceDown` | `Op::MoveColumnToRowDown` | |
| `Op::MoveColumnToWorkspaceUp` | `Op::MoveColumnToRowUp` | |
| `Op::MoveWorkspaceDown` | `Op::MoveRowDown` | |
| `Op::MoveWorkspaceUp` | `Op::MoveRowUp` | |
| `Op::SetWorkspaceName { ... }` | `Op::SetRowName { ... }` | |

### Op Variants: REMOVE
| Op | Notes |
|----|-------|
| `Op::FocusWorkspace(idx)` | No jump to specific row |
| `Op::MoveWorkspaceToOutput(output)` | Needs different semantics |

---

## Tests: Transform vs Remove

### Tests to TRANSFORM (Workspace → Row semantics)
| Old Test | New Test | Notes |
|----------|----------|-------|
| `focus_window_or_workspace_down` | `focus_window_or_row_down` | Edge navigation |
| `focus_window_or_workspace_up` | `focus_window_or_row_up` | Edge navigation |
| `move_to_workspace_down` | `move_to_row_down` | |
| `move_to_workspace_up` | `move_to_row_up` | |

### Tests to REMOVE (workspace-specific)
| Test | Notes |
|------|-------|
| `workspaces_update_original_output_on_moving_to_same_output` | Workspace-specific |
| `workspaces_update_original_output_on_moving_to_same_monitor` | Workspace-specific |
| `open_right_of_on_different_workspace` | Workspace-specific |
| `open_right_of_on_different_workspace_ewaf` | Workspace-specific |
| `output_active_workspace_is_preserved` | Workspace-specific |
| `output_active_workspace_is_preserved_with_other_outputs` | Workspace-specific |
| `move_workspace_to_same_monitor_doesnt_reorder` | Workspace-specific |
| `move_column_to_workspace_unfocused_with_multiple_monitors` | |
| `move_window_to_workspace_maximize_and_fullscreen` | |

### Any test using workspace Ops
Search for tests containing:
- `FocusWorkspace`
- `MoveWindowToWorkspace`
- `MoveColumnToWorkspace`
- `MoveWorkspace`
- `SetWorkspaceName`

---

## Golden Tests: Transform vs Remove

### Transform (Workspace → Row)
| Old Test | New Test | Notes |
|----------|----------|-------|
| `golden_w3_focus_window_or_workspace_down` | `golden_focus_window_or_row_down` | Edge navigation |

### Remove (workspace-specific)
Golden tests that only test workspace switching should be removed.

---

## Op Enum Updates

### Rename (Workspace → Row)
| Current Op | New Op | Notes |
|------------|--------|-------|
| `Op::FocusWindowOrWorkspaceDown` | `Op::FocusWindowOrRowDown` | At edge → row |
| `Op::FocusWindowOrWorkspaceUp` | `Op::FocusWindowOrRowUp` | At edge → row |
| `Op::MoveWindowDownOrToWorkspaceDown` | `Op::MoveWindowDownOrToRowDown` | At edge → row |
| `Op::MoveWindowUpOrToWorkspaceUp` | `Op::MoveWindowUpOrToRowUp` | At edge → row |

---

## New Tests to Add (Row-specific)

| Test | Notes |
|------|-------|
| `focus_row_down_creates_row` | Focusing down creates new row if needed |
| `focus_row_up_creates_row` | Focusing up creates new row if needed |
| `move_window_to_row_geometric_placement` | Window placed near geometrically nearest |
| `row_spanning_window_shrinks_on_row_move` | Per USER decision |
| `row_reordering` | `MoveRowDown/Up` |

---

## Verification

After completion:
1. `cargo test --lib` — all remaining tests must pass
2. `./scripts/verify-golden.sh` — golden tests must pass
3. New row tests should pass

---

*TEAM_011: Phase 1.5.3 Part 2E — Test transformation*
