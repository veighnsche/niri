# Phase 1.5.3 Part 2E: Remove Workspace Tests

> **Status**: PENDING
> **Type**: BREAKING CHANGE — Complete removal, no backwards compatibility
> **Prerequisite**: Part 2D complete (Config Actions removed)

---

## Overview

Remove all tests that test workspace functionality from `src/layout/tests.rs`.

---

## Test Operations to Remove

These `Op` variants should be removed from the test infrastructure:

| Op Variant | Notes |
|------------|-------|
| `Op::FocusWorkspaceDown` | |
| `Op::FocusWorkspaceUp` | |
| `Op::FocusWorkspace(idx)` | |
| `Op::MoveWindowToWorkspaceDown` | |
| `Op::MoveWindowToWorkspaceUp` | |
| `Op::MoveColumnToWorkspaceDown` | |
| `Op::MoveColumnToWorkspaceUp` | |
| `Op::MoveWorkspaceDown` | |
| `Op::MoveWorkspaceUp` | |
| `Op::MoveWorkspaceToOutput(output)` | |
| `Op::SetWorkspaceName { ... }` | |

---

## Tests to Remove

### Workspace Switching Tests
| Test | Notes |
|------|-------|
| `workspaces_update_original_output_on_moving_to_same_output` | |
| `workspaces_update_original_output_on_moving_to_same_monitor` | |
| `open_right_of_on_different_workspace` | |
| `open_right_of_on_different_workspace_ewaf` | |
| `output_active_workspace_is_preserved` | |
| `output_active_workspace_is_preserved_with_other_outputs` | |
| `move_workspace_to_same_monitor_doesnt_reorder` | |
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

## Golden Tests to Remove

| Test | Notes |
|------|-------|
| `golden_w3_focus_window_or_workspace_down` | Tests workspace switching |

---

## Op Enum Updates

### Rename (not remove)
| Current Op | New Op | Notes |
|------------|--------|-------|
| `Op::FocusWindowOrWorkspaceDown` | `Op::FocusWindowDown` | |
| `Op::FocusWindowOrWorkspaceUp` | `Op::FocusWindowUp` | |
| `Op::MoveWindowDownOrToWorkspaceDown` | `Op::MoveWindowDown` | |
| `Op::MoveWindowUpOrToWorkspaceUp` | `Op::MoveWindowUp` | |

---

## Verification

After completion:
1. `cargo test --lib` — all remaining tests must pass
2. `./scripts/verify-golden.sh` — golden tests must pass

---

*TEAM_011: Phase 1.5.3 Part 2E*
