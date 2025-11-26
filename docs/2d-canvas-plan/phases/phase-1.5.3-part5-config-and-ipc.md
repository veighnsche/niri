# Phase 1.5.3 Part 5: Remove Workspace Config and IPC

> **Status**: ⏳ PENDING
> **Prerequisite**: Parts 4 (workspace fields removed) + Actual Row Implementation complete
> **Critical**: Only after workspace system is completely gone

---

## Overview

Final cleanup: remove workspace configuration parsing and IPC endpoints.
This is a BREAKING CHANGE for users with workspace config.

**WARNING**: Only run this after the workspace system is completely removed.
Users will get clear errors when trying to use workspace config.

---

## Step 5.1: Remove Workspace Config Parsing

| Config Block | File | Change |
|--------------|------|--------|
| `workspaces { }` | `niri-config/src/` | Remove parsing, return error |
| `workspace "name" { }` | `niri-config/src/` | Remove parsing, return error |
| `open-on-workspace` | `niri-config/src/` | Remove from window rules |

**Error Handling**: Config with workspace blocks should produce a clear error:
```
Error: 'workspaces' configuration is no longer supported.
The 2D canvas uses rows instead of workspaces.
See migration guide for details.
```

### Files to Modify:
- `niri-config/src/lib.rs` - Remove workspace config structs
- `niri-config/src/workspace.rs` - Delete entire file if exists
- `niri-config/src/binds.rs` - Remove workspace action parsing

---

## Step 5.2: Remove Workspace IPC Endpoints

| Command | File | Change |
|---------|------|--------|
| `niri msg workspaces` | `niri-ipc/src/` | Return error |
| `niri msg focus-workspace` | `niri-ipc/src/` | Return error |
| `niri msg move-window-to-workspace` | `niri-ipc/src/` | Return error |

**Error Response**:
```json
{"error": "Workspaces are not supported. Use row navigation instead."}
```

### Files to Modify:
- `niri-ipc/src/lib.rs` - Remove workspace IPC handlers
- `src/ipc/server.rs` - Remove workspace command handling

---

## Step 5.3: Update Documentation

### Migration Guide
Create clear migration instructions for users:

**Before (workspaces):**
```kdl
workspaces {
    workspace "1" {
        open-on-output = "eDP-1"
    }
    workspace "2" {
        open-on-output = "DP-1" 
    }
}

window-rule {
    match = { app_id = "firefox" }
    open-on-workspace = "2"
}
```

**After (rows):**
```kdl
# Workspaces are no longer supported
# Use row navigation instead:
# Super+Up/Down to focus rows
# Super+Shift+Up/Down to move windows between rows

window-rule {
    match = { app_id = "firefox" }
    # open-on-workspace removed - windows open in focused row
}
```

---

## Step 5.4: Update Error Messages

### Config Parser Errors
```rust
// In workspace config parsing
return Err(ConfigError::Unsupported {
    feature: "workspaces",
    message: "The 2D canvas uses rows instead of workspaces. Use row navigation (Super+Up/Down)."
});
```

### IPC Error Responses
```rust
// In workspace IPC handlers
return Err(IpcError::Unsupported {
    command: "workspaces",
    message: "Use row navigation commands instead."
});
```

---

## Verification

### Config Tests
```bash
# Should return clear error
echo 'workspaces { workspace "1" {} }' | niri --config
# Expected: Error about workspaces not supported
```

### IPC Tests  
```bash
# Should return error
niri msg workspaces
niri msg focus-workspace 1
# Expected: JSON error about using row navigation
```

### Compilation Tests
```bash
cargo check                    # Should compile
cargo test --lib              # All tests pass
cargo insta test              # Golden tests pass
```

### No Workspace Config
```bash
# Should return no results
grep -rn "workspace" niri-config/src/ | grep -v "row"
grep -rn "Workspace" niri-config/src/
grep -rn "workspace" niri-ipc/src/ | grep -v "row"
```

---

## Handoff Criteria

- [ ] Workspace config parsing removed
- [ ] Workspace IPC endpoints removed  
- [ ] Clear error messages for workspace usage
- [ ] Migration guide created
- [ ] Code compiles
- [ ] All tests pass
- [ ] No workspace config/IPC references remain

---

## Notes

This is the final step in removing all workspace support from niri.
After this phase, niri will be fully Canvas2D-based with no workspace functionality.

Users will need to migrate their configs, but the error messages should make it clear what to do instead.

## Verification

1. `cargo check` — must compile
2. `cargo test --lib` — all tests must pass
3. Config with `workspaces { }` should error clearly
4. `niri msg workspaces` should return error

---

## Migration Guide for Users

Create a migration guide explaining:

1. **Workspaces are gone** — replaced by infinite 2D canvas
2. **Row navigation** — `Mod+Up/Down` moves between rows
3. **Camera bookmarks** — `Mod+1/2/3` will save/restore camera positions (Phase 5)
4. **Config changes** — remove `workspaces { }` blocks

---

*TEAM_010: Phase 1.5.3 Part 5*
