# Phase 1.5.3 Part 4: Remove Workspace Fields from Monitor

> **Status**: PENDING
> **Prerequisite**: Parts 1-3 complete

---

## Overview

With all workspace-related functionality removed, we can now remove the workspace
fields from Monitor. This is the final cleanup step.

---

## Step 4.1: Remove Workspace Fields

| Field | File | Change |
|-------|------|--------|
| `workspaces: Vec<Workspace<W>>` | `src/layout/monitor.rs` | Remove |
| `active_workspace_idx: usize` | `src/layout/monitor.rs` | Remove |
| `previous_workspace_id: Option<WorkspaceId>` | `src/layout/monitor.rs` | Remove |
| `workspace_switch: Option<WorkspaceSwitch>` | `src/layout/monitor.rs` | Already removed in Part 2 |

---

## Step 4.2: Remove Workspace Methods

| Method | File | Change |
|--------|------|--------|
| `active_workspace_idx()` | `src/layout/monitor.rs` | Remove |
| `active_workspace_ref()` | `src/layout/monitor.rs` | Remove |
| `active_workspace()` | `src/layout/monitor.rs` | Remove |
| `find_named_workspace()` | `src/layout/monitor.rs` | Remove |
| `find_named_workspace_index()` | `src/layout/monitor.rs` | Remove |
| `add_workspace_at()` | `src/layout/monitor.rs` | Remove |
| `add_workspace_top()` | `src/layout/monitor.rs` | Remove |
| `add_workspace_bottom()` | `src/layout/monitor.rs` | Remove |
| `into_workspaces()` | `src/layout/monitor.rs` | Remove |

---

## Step 4.3: Update Monitor::new()

Current signature:
```rust
pub fn new(
    output: Output,
    workspaces: Vec<Workspace<W>>,  // REMOVE
    ws_id_to_activate: Option<WorkspaceId>,  // REMOVE
    clock: Clock,
    base_options: Rc<Options>,
    layout_config: Option<LayoutPart>,
) -> Self
```

New signature:
```rust
pub fn new(
    output: Output,
    clock: Clock,
    base_options: Rc<Options>,
    layout_config: Option<LayoutPart>,
) -> Self
```

---

## Step 4.4: Fix All Call Sites

The compiler will show all places that use removed fields/methods.
Fix each one:

| Call Site | Current | New |
|-----------|---------|-----|
| `monitor.active_workspace()` | Returns `&mut Workspace` | Use `monitor.canvas_mut()` |
| `monitor.workspaces[idx]` | Direct access | Use canvas methods |
| `Layout::new()` | Passes workspaces | Don't pass workspaces |

---

## Step 4.5: Remove Workspace Types (if unused)

After removing from Monitor, check if these are still used:

| Type | File | Check |
|------|------|-------|
| `Workspace<W>` | `src/layout/workspace.rs` | May still be used by tests |
| `WorkspaceId` | `src/layout/workspace.rs` | Check all usages |
| `WorkspaceAddWindowTarget` | `src/layout/workspace.rs` | Check all usages |

**Note**: Don't remove if still used by tests or other code. Mark as deprecated.

---

## Verification

After each step:
1. `cargo check` — must compile
2. `cargo test --lib` — all 284 tests must pass
3. `./scripts/verify-golden.sh` — all 91 golden tests must pass

---

## Success Criteria

- [ ] No `workspaces` field in Monitor
- [ ] No `active_workspace_idx` field in Monitor
- [ ] No workspace-related methods in Monitor
- [ ] All tests pass
- [ ] All golden tests pass

---

*TEAM_010: Phase 1.5.3 Part 4*
