# Phase 1.5.3 Part 4: Remove Workspace Fields

> **Status**: ⏳ PENDING  
> **Prerequisite**: Phase 1.5.3 Actual Row Implementation complete
> **Critical**: Only after row navigation actually works

---

## Overview

Remove all workspace-related fields, types, and structs from the codebase.
This is only possible AFTER row navigation is actually implemented.

**WARNING**: If you run this phase before implementing actual row navigation,
you will break the codebase because "row" methods still call workspace code.

---

## Step 4.1: Remove Workspace Fields from Monitor

| Field | File | Action |
|-------|------|--------|
| `workspaces: Vec<Workspace<W>>` | `src/layout/monitor/mod.rs` | Delete field |
| `active_workspace_idx: usize` | `src/layout/monitor/mod.rs` | Delete field |
| `previous_workspace_id: Option<WorkspaceId>` | `src/layout/monitor/mod.rs` | Delete field |
| `workspace_switch: Option<WorkspaceSwitch>` | `src/layout/monitor/mod.rs` | Delete field |

**Replace with:**
- Canvas-based row management
- Camera-based view tracking

---

## Step 4.2: Remove Workspace Types

| Type | File | Action |
|------|------|--------|
| `Workspace<W>` | `src/layout/workspace.rs` | Delete entire file |
| `WorkspaceId` | `src/layout/workspace.rs` | Delete entire file |
| `WorkspaceSwitch` | `src/layout/monitor/types.rs` | Delete enum |
| `WorkspaceSwitchGesture` | `src/layout/monitor/types.rs` | Delete struct |
| `OutputId` | `src/layout/workspace.rs` | Delete struct |

---

## Step 4.3: Remove Workspace Methods

| Method | File | Action |
|--------|------|--------|
| `active_workspace()` | `src/layout/monitor/workspace_compat.rs` | Delete entire file |
| `find_named_workspace()` | `src/layout/monitor/workspace_compat.rs` | Delete entire file |
| `into_workspaces()` | `src/layout/monitor/workspace_compat.rs` | Delete entire file |
| `switch_workspace_up()` | `src/layout/monitor/workspace_ops.rs` | Delete entire file |
| `switch_workspace_down()` | `src/layout/monitor/workspace_ops.rs` | Delete entire file |
| All workspace animations | `src/layout/monitor/workspace_compat.rs` | Delete entire file |

---

## Step 4.4: Remove Workspace Files

**Files to delete entirely:**
- `src/layout/workspace.rs` - Entire workspace system
- `src/layout/monitor/workspace_compat.rs` - Legacy workspace accessors
- `src/layout/monitor/workspace_ops.rs` - Workspace operations
- `src/layout/monitor/navigation.rs` - Workspace navigation
- `src/layout/monitor/gestures.rs` - Workspace gestures

---

## Step 4.5: Update Monitor Implementation

**Monitor should only contain:**
```rust
pub struct Monitor<W> {
    // Canvas (2D grid)
    canvas: Canvas<W>,
    
    // Camera (view into canvas)
    camera: Camera,
    
    // Output management
    output: Output,
    scale: Scale,
    transform: Transform,
    view_size: Size<f64, Logical>,
    working_area: Rectangle<f64, Logical>,
    
    // Configuration
    base_options: Rc<Options>,
    options: Rc<Options>,
    layout_config: Option<niri_config::LayoutPart>,
    
    // Animation
    clock: Clock,
}
```

---

## Verification

### Compilation Tests
```bash
cargo check                    # Should compile
cargo test --lib              # All tests pass  
cargo insta test              # Golden tests pass
```

### No Workspace References
```bash
# Should return no results
grep -rn "workspace" src/layout/ | grep -v "row"
grep -rn "Workspace" src/layout/
grep -rn "workspace" src/input/ | grep -v "row"
```

### Row Navigation Works
```bash
# These should all work without workspace code
cargo test focus_row_up
cargo test move_window_to_row_down  
cargo test set_row_name
```

---

## Handoff Criteria

- [ ] All workspace fields deleted from Monitor
- [ ] All workspace types deleted
- [ ] All workspace files deleted
- [ ] Monitor only contains Canvas + Camera
- [ ] Code compiles
- [ ] All tests pass
- [ ] No workspace references remain

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
