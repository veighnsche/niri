# Phase 1.5.3: Workspace Removal Checklist

> **Purpose**: Detailed checklist of everything to remove when replacing Workspace with Canvas2D.
> **Type**: BREAKING CHANGE — No feature flags, no backwards compatibility.

---

## Removal Order

Remove in this order to minimize intermediate breakage:

```
1. Consumers (keybinds, hot corner, IPC)
      ↓
2. Overview mode
      ↓
3. Monitor workspace fields
      ↓
4. Config parsing
      ↓
5. Workspace struct (if unused)
```

---

## Step 1: Remove Consumers

### 1.1 Keybinds

| Keybind | Action | File | Status |
|---------|--------|------|--------|
| `Mod+1` | `focus-workspace 1` | `src/input/` | ⏳ Disable |
| `Mod+2` | `focus-workspace 2` | `src/input/` | ⏳ Disable |
| `Mod+3` | `focus-workspace 3` | `src/input/` | ⏳ Disable |
| `Mod+4-9` | `focus-workspace N` | `src/input/` | ⏳ Disable |
| `Mod+Shift+1-9` | `move-window-to-workspace N` | `src/input/` | ⏳ Remove |
| `Mod+Page_Up` | `focus-workspace-up` | `src/input/` | ⏳ Remove |
| `Mod+Page_Down` | `focus-workspace-down` | `src/input/` | ⏳ Remove |
| `Mod+Shift+Page_Up` | `move-window-to-workspace-up` | `src/input/` | ⏳ Remove |
| `Mod+Shift+Page_Down` | `move-window-to-workspace-down` | `src/input/` | ⏳ Remove |

**Note**: `Mod+1/2/3` will be repurposed for camera bookmarks in Phase 5.4. For now, disable them (no action).

### 1.2 Hot Corner

| Trigger | Action | File | Status |
|---------|--------|------|--------|
| Mouse to top-right corner | Open overview | `src/input/` | ⏳ Remove |

### 1.3 IPC Endpoints

| Command | File | Status |
|---------|------|--------|
| `niri msg workspaces` | `niri-ipc/` | ⏳ Remove |
| `niri msg focus-workspace` | `niri-ipc/` | ⏳ Remove |
| `niri msg move-window-to-workspace` | `niri-ipc/` | ⏳ Remove |
| Workspace event stream | `niri-ipc/` | ⏳ Remove |

**Handling**: Return error "Workspaces not supported" or remove endpoints entirely.

---

## Step 2: Remove Overview Mode

### 2.1 Overview State

| Item | File | Status |
|------|------|--------|
| `overview_open: bool` | `src/niri.rs` or similar | ⏳ Remove |
| Overview rendering | `src/layout/` | ⏳ Remove |
| Overview input handling | `src/input/` | ⏳ Remove |
| Overview animations | `src/layout/` | ⏳ Remove |

### 2.2 Overview Triggers

| Trigger | File | Status |
|---------|------|--------|
| Overview keybind | `src/input/` | ⏳ Remove |
| Overview gesture | `src/input/` | ⏳ Remove |
| Hot corner (already removed in 1.2) | — | — |

**Note**: Overview will be broken until Phase 5. This is acceptable per USER decision.

---

## Step 3: Remove Monitor Workspace Fields

### 3.1 Monitor Struct Changes

```rust
// BEFORE (src/layout/monitor.rs)
pub struct Monitor<W: LayoutElement> {
    workspaces: Vec<Workspace<W>>,
    active_workspace_idx: usize,
    // ... other fields
}

// AFTER
pub struct Monitor<W: LayoutElement> {
    canvas: Canvas2D<W>,
    // ... other fields
}
```

### 3.2 Fields to Remove

| Field | Status |
|-------|--------|
| `workspaces: Vec<Workspace<W>>` | ⏳ Replace with `canvas: Canvas2D<W>` |
| `active_workspace_idx: usize` | ⏳ Remove |
| `workspace_switch_*` fields | ⏳ Remove |
| `previous_workspace_id` | ⏳ Remove |

### 3.3 Methods to Remove/Update

| Method | Action | Status |
|--------|--------|--------|
| `active_workspace()` | Replace with `canvas()` | ⏳ |
| `active_workspace_mut()` | Replace with `canvas_mut()` | ⏳ |
| `switch_workspace()` | Remove | ⏳ |
| `switch_workspace_up()` | Remove | ⏳ |
| `switch_workspace_down()` | Remove | ⏳ |
| `move_window_to_workspace()` | Remove | ⏳ |
| `add_window()` | Update to use Canvas2D | ⏳ |
| `remove_window()` | Update to use Canvas2D | ⏳ |
| All other workspace methods | Update or remove | ⏳ |

---

## Step 4: Remove Config Parsing

### 4.1 Config Block

```kdl
// REMOVE THIS ENTIRE BLOCK
workspaces {
    // ...
}
```

### 4.2 Config Handling

| Option | Action | Status |
|--------|--------|--------|
| `workspaces { }` block | Error on parse | ⏳ |
| `workspace-*` actions | Error on parse | ⏳ |

**Behavior**: If user has `workspaces { }` in config, niri should:
- Print clear error message explaining workspaces are removed
- Suggest removing the block
- Exit or continue with warning (TBD)

---

## Step 5: Remove Workspace Struct (If Unused)

### 5.1 Check Usage

After steps 1-4, check if `Workspace` struct is used anywhere:

```bash
grep -rn "Workspace" src/layout/
```

### 5.2 If Unused

| File | Action | Status |
|------|--------|--------|
| `src/layout/workspace.rs` | Delete | ⏳ |
| `src/layout/mod.rs` | Remove `mod workspace` | ⏳ |

### 5.3 If Still Used

Document why and plan for removal in later phase.

---

## Verification Checklist

After each step, verify:

- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] Golden tests pass (`cargo insta test`)
- [ ] Animation regression checklist verified

---

## Final Verification

After all steps complete:

- [ ] `niri msg workspaces` returns error
- [ ] `Mod+1/2/3` does nothing (disabled)
- [ ] Hot corner does nothing
- [ ] Overview mode removed
- [ ] Config with `workspaces { }` produces error
- [ ] No references to `Workspace` struct (or documented why)
- [ ] All 251+ tests pass
- [ ] All 58 golden tests pass
- [ ] Animation regression checklist verified

---

## Rollback Plan

If something goes catastrophically wrong:

1. `git stash` current changes
2. Return to last known good state
3. Document what went wrong
4. Create smaller, incremental steps

---

## Files Likely to Change

| File | Changes |
|------|---------|
| `src/layout/monitor.rs` | Major — workspace → canvas |
| `src/layout/mod.rs` | Remove workspace module |
| `src/layout/workspace.rs` | Delete |
| `src/niri.rs` | Overview removal |
| `src/input/*.rs` | Keybind removal |
| `niri-ipc/src/lib.rs` | IPC removal |
| `niri-config/src/*.rs` | Config removal |

---

## Estimated Effort

| Step | Effort |
|------|--------|
| Step 1: Remove consumers | 2-4 hours |
| Step 2: Remove overview | 2-4 hours |
| Step 3: Remove Monitor fields | 4-8 hours |
| Step 4: Remove config | 1-2 hours |
| Step 5: Remove Workspace struct | 1-2 hours |
| **Total** | **10-20 hours** |

---

*Created by TEAM_009 — Workspace Removal Checklist*
