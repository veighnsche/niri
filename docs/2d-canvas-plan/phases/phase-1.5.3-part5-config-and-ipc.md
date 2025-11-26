# Phase 1.5.3 Part 5: Remove Workspace Config and IPC

> **Status**: PENDING
> **Prerequisite**: Part 4 complete (workspace fields removed)

---

## Overview

Final cleanup: remove workspace configuration parsing and IPC endpoints.
This is a BREAKING CHANGE for users with workspace config.

---

## Step 5.1: Remove Workspace Config Parsing

| Config Block | File | Change |
|--------------|------|--------|
| `workspaces { }` | `niri-config/src/` | Remove parsing |
| `workspace "name" { }` | `niri-config/src/` | Remove parsing |
| `open-on-workspace` | `niri-config/src/` | Remove from window rules |

**Error Handling**: Config with workspace blocks should produce a clear error:
```
Error: 'workspaces' configuration is no longer supported.
The 2D canvas uses rows instead of workspaces.
```

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

---

## Step 5.3: Remove Workspace Event Stream

| Event | File | Change |
|-------|------|--------|
| `WorkspacesChanged` | `niri-ipc/src/` | Remove |
| `WorkspaceActivated` | `niri-ipc/src/` | Remove |

---

## Step 5.4: Update Documentation

| Document | Change |
|----------|--------|
| `docs/wiki/` | Remove workspace references |
| `resources/default-config.kdl` | Remove workspace examples |
| `README.md` | Update if mentions workspaces |

---

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
