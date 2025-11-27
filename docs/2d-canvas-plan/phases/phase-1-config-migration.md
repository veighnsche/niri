# Phase 1: Config Migration (Workspace → Row)

> **Status**: 🔄 **CURRENT PRIORITY**
> **Goal**: Replace all workspace config/terminology with row semantics
> **Decision**: Remove workspace syntax immediately (no deprecation period)
> **Blocking**: Animation bug (67 test failures) - see TODO.md

---

## Overview

Based on USER decisions in `TEAM_042_workspace_vs_canvas_architecture.md`:
- Replace `workspace` config block with `row` block
- Replace `open-on-workspace` with `open-on-row`
- Remove workspace syntax immediately (no backwards compatibility)

## Related TODOs from Codebase

These TODOs will be resolved by this phase:
- `TODO(TEAM_020): Eventually remove workspace check entirely` (6 locations in mod.rs)
- `TODO(TEAM_020): Eventually remove workspace config entirely` (mod.rs:3021)
- `TODO: Eventually remove when external systems updated` (workspace_types.rs)

---

## Step 1.1: Config Syntax Changes

### niri-config Changes

| Old Syntax | New Syntax | File |
|------------|------------|------|
| `workspace "name" { }` | `row "name" { }` | `niri-config/src/lib.rs` |
| `open-on-workspace "name"` | `open-on-row "name"` | `niri-config/src/window_rule.rs` |

### Tasks

- [ ] **1.1.1**: Rename `Workspace` struct to `RowConfig` in niri-config
- [ ] **1.1.2**: Rename `workspace` KDL block to `row`
- [ ] **1.1.3**: Rename `open-on-workspace` to `open-on-row` in window rules
- [ ] **1.1.4**: Update config parsing to reject old `workspace` syntax
- [ ] **1.1.5**: Update default-config.kdl examples

### Implementation

```kdl
# OLD (remove)
workspace "browser" {
    open-on-output "eDP-1"
}

window-rule {
    match app-id="firefox"
    open-on-workspace "browser"
}

# NEW
row "browser" {
    open-on-output "eDP-1"
}

window-rule {
    match app-id="firefox"
    open-on-row "browser"
}
```

---

## Step 1.2: Internal Terminology Changes

### Layout Module Changes

| Old Term | New Term | Files |
|----------|----------|-------|
| `WorkspaceId` | `RowId` | `workspace_types.rs` → `row_types.rs` |
| `find_workspace_by_name` | `find_row_by_name` | `layout/mod.rs` |
| `ensure_named_workspace` | `ensure_named_row` | `layout/mod.rs` |
| `active_workspace` | `active_row` | `layout/mod.rs`, `monitor/mod.rs` |

### Tasks

- [ ] **1.2.1**: Rename `workspace_types.rs` to `row_types.rs`
- [ ] **1.2.2**: Rename `WorkspaceId` to `RowId`
- [ ] **1.2.3**: Update all method names from `workspace` to `row`
- [ ] **1.2.4**: Update comments and documentation

---

## Step 1.3: Test Updates

### Tasks

- [ ] **1.3.1**: Rename test file `window_opening.rs` test cases
- [ ] **1.3.2**: Update test configs to use `row` syntax
- [ ] **1.3.3**: Update snapshot assertions for row terminology
- [ ] **1.3.4**: Remove workspace-specific test cases

---

## Step 1.4: IPC Updates

### Tasks

- [ ] **1.4.1**: Remove workspace IPC commands (they return errors)
- [ ] **1.4.2**: Add row IPC commands if needed
- [ ] **1.4.3**: Update `niri msg` CLI

---

## Verification

```bash
# Config with old syntax should error
echo 'workspace "test" {}' | niri --config
# Expected: Error about unknown block

# Config with new syntax should work
echo 'row "test" {}' | niri --config
# Expected: Success

# Tests should pass
cargo test
cargo insta test
```

---

## Success Criteria

- [ ] `workspace` config block produces clear error
- [ ] `row` config block works correctly
- [ ] `open-on-row` window rule works
- [ ] All tests pass with new terminology
- [ ] No `workspace` references remain (except error messages)

---

## Files to Modify

| File | Changes |
|------|---------|
| `niri-config/src/lib.rs` | Rename Workspace → RowConfig |
| `niri-config/src/workspace.rs` | Rename to `row.rs` |
| `niri-config/src/window_rule.rs` | open-on-workspace → open-on-row |
| `src/layout/workspace_types.rs` | Rename to `row_types.rs` |
| `src/layout/mod.rs` | Update method names |
| `src/tests/window_opening.rs` | Update test configs |
| `resources/default-config.kdl` | Update examples |

---

*Phase 1 - Config Migration*
