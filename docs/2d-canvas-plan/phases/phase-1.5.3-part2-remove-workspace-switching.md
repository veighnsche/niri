# Phase 1.5.3 Part 2: Remove Workspace Switching

> **Status**: PENDING
> **Prerequisite**: Part 1 complete (Monitor methods migrated)

---

## Overview

Once Monitor uses Canvas2D for window operations, we can remove workspace switching.
This is a BREAKING CHANGE — users will lose workspace functionality.

---

## Step 2.1: Disable Workspace Keybinds

| Keybind | Action | File | Change |
|---------|--------|------|--------|
| `Mod+1-9` | `focus-workspace N` | `src/input/` | Return early / no-op |
| `Mod+Shift+1-9` | `move-window-to-workspace N` | `src/input/` | Return early / no-op |
| `Mod+Page_Up` | `focus-workspace-up` | `src/input/` | Return early / no-op |
| `Mod+Page_Down` | `focus-workspace-down` | `src/input/` | Return early / no-op |
| `Mod+Shift+Page_Up` | `move-window-to-workspace-up` | `src/input/` | Return early / no-op |
| `Mod+Shift+Page_Down` | `move-window-to-workspace-down` | `src/input/` | Return early / no-op |

**Note**: `Mod+1/2/3` will be repurposed for camera bookmarks in Phase 5.

---

## Step 2.2: Remove Workspace Switch Animation

| Item | File | Change |
|------|------|--------|
| `WorkspaceSwitch` enum | `src/layout/monitor.rs` | Remove |
| `WorkspaceSwitchGesture` | `src/layout/monitor.rs` | Remove |
| `workspace_switch` field | `src/layout/monitor.rs` | Remove |
| `activate_workspace()` | `src/layout/monitor.rs` | Remove |
| `activate_workspace_with_anim_config()` | `src/layout/monitor.rs` | Remove |

---

## Step 2.3: Remove Workspace Gesture

| Item | File | Change |
|------|------|--------|
| Touchpad workspace gesture | `src/input/` | Remove |
| `WORKSPACE_GESTURE_MOVEMENT` | `src/layout/monitor.rs` | Remove |
| `WORKSPACE_GESTURE_RUBBER_BAND` | `src/layout/monitor.rs` | Remove |

---

## Step 2.4: Remove DnD Workspace Scrolling

| Item | File | Change |
|------|------|--------|
| `WORKSPACE_DND_EDGE_SCROLL_MOVEMENT` | `src/layout/monitor.rs` | Remove |
| DnD edge scrolling logic | `src/layout/monitor.rs` | Remove |

---

## Verification

After each step:
1. `cargo check` — must compile
2. `cargo test --lib` — tests may fail (expected, fix them)
3. Keybinds should do nothing (manual test)

---

## Expected Test Failures

Some tests may reference workspace switching. These need to be:
1. Updated to use Canvas2D
2. Or removed if testing workspace-specific behavior

---

*TEAM_010: Phase 1.5.3 Part 2*
