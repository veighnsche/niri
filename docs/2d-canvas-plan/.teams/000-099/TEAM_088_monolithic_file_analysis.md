# TEAM_088 — Monolithic File Analysis & Refactoring Plan

> **Updated: Nov 30, 2025**

## Mission
Analyze all Rust files exceeding 500 LOC and create a comprehensive refactoring roadmap to break them into smaller, focused modules.

## Current Status
- **Date**: 2025-11-30
- **Tool Used**: `wc -l` for LOC counting
- **Threshold**: 1000 LOC target, 500 LOC ideal
- **Violating Files**: 5 critical files (>2000 LOC)

---

## Summary Statistics (Nov 30, 2025)

| Category | Count | Total LOC | Notes |
|----------|-------|-----------|-------|
| Critical (>2000 LOC) | 5 | 17,254 | PRIORITY |
| Severe (1000-2000 LOC) | 15 | 21,411 | Can wait |
| Moderate (500-1000 LOC) | 31 | 21,185 | Lower priority |
| **Files to Skip** | 2 | ~9,105 | Deprecated/tests |

---

## Critical Files (>2000 LOC)

These are the highest priority for refactoring.

### 1. `src/layout/deprecated/scrolling.rs` — 5517 LOC
**Description**: Legacy scrolling space implementation
**Status**: ⚫ **SKIP** — Deprecated, being replaced by Row in 2D Canvas refactor
**Action**: Do NOT refactor. Will be removed when Row is complete.

### 2. `src/layout/tests.rs` — 3588 LOC
**Description**: Layout unit tests
**Status**: ⚫ **SKIP** — Test file, acceptable as-is
**Suggested Split** (if needed):
- `src/layout/tests/mod.rs` — Test utilities and imports
- `src/layout/tests/basic.rs` — Basic layout tests
- `src/layout/tests/columns.rs` — Column operation tests
- `src/layout/tests/windows.rs` — Window operation tests
- `src/layout/tests/focus.rs` — Focus tests
- `src/layout/tests/resize.rs` — Resize tests
- `src/layout/tests/navigation.rs` — Navigation tests

### 3. `src/backend/tty.rs` — 3473 LOC
**Description**: TTY/DRM backend for native display
**Suggested Split**:
- `src/backend/tty/mod.rs` — Core TTY backend
- `src/backend/tty/drm.rs` — DRM device handling
- `src/backend/tty/output.rs` — Output management
- `src/backend/tty/input.rs` — Input device handling
- `src/backend/tty/session.rs` — Session management
- `src/backend/tty/render.rs` — Rendering pipeline

### 4. `src/niri/mod.rs` — 2349 LOC
**Description**: Main application state and event handling
**Status**: 🟡 **IMPROVED** — Was 6603 LOC, now 2349 LOC after refactor
**Note**: Already split into `src/niri/` module structure with:
- `render.rs` (1056 LOC)
- `config.rs` (556 LOC)
- Other submodules
**Suggested Further Split** (if needed):
- `src/niri/output.rs` — Output management
- `src/niri/surface.rs` — Surface handling
- `src/niri/popups.rs` — Popup management
- `src/niri/cursor.rs` — Cursor handling
- `src/niri/screenshot.rs` — Screenshot functionality

### 5. `niri-config/src/lib.rs` — 2327 LOC
**Description**: Configuration parsing and validation
**Suggested Split**:
- `niri-config/src/lib.rs` — Core Config struct and parsing
- `niri-config/src/validation.rs` — Config validation
- `niri-config/src/defaults.rs` — Default values
- `niri-config/src/merge.rs` — Config merging logic
- `niri-config/src/parse.rs` — KDL parsing helpers

---

## Severe Files (1000-2000 LOC)

### 6. `src/ui/mru.rs` — 1940 LOC
**Description**: Most Recently Used window switcher UI
**Suggested Split**:
- `src/ui/mru/mod.rs` — Core MRU state
- `src/ui/mru/render.rs` — Rendering
- `src/ui/mru/input.rs` — Input handling
- `src/ui/mru/layout.rs` — Layout calculations

### 7. `src/layout/mod.rs` — 1875 LOC
**Description**: Layout management and window arrangement
**Status**: 🟡 **IMPROVED** — Was 5353 LOC, now 1875 LOC after refactor
**Note**: Already split into `src/layout/layout_impl/` with:
- `interactive_move.rs` (788 LOC)
- `navigation.rs` (592 LOC)
- `render.rs` (500 LOC)
**Suggested Further Split** (if needed):
- `src/layout/actions.rs` — User actions (move, resize, etc.)
- `src/layout/window_ops.rs` — Window operations
- `src/layout/focus.rs` — Focus management

### 8. `niri-ipc/src/lib.rs` — 1874 LOC
**Description**: IPC message types and serialization
**Suggested Split**:
- `niri-ipc/src/lib.rs` — Core types
- `niri-ipc/src/request.rs` — Request types
- `niri-ipc/src/response.rs` — Response types
- `niri-ipc/src/event.rs` — Event types

### 9. `src/input/actions.rs` — 1722 LOC
**Description**: Input action handling
**Status**: 🆕 **NEW** — Not in previous analysis
**Suggested Split**:
- `src/input/actions/mod.rs` — Core action dispatch
- `src/input/actions/window.rs` — Window-related actions
- `src/input/actions/workspace.rs` — Workspace/row actions
- `src/input/actions/output.rs` — Output actions
- `src/input/actions/system.rs` — System actions (quit, reload, etc.)

### 10. `src/handlers/xdg_shell.rs` — 1557 LOC
**Description**: XDG shell protocol handlers
**Suggested Split**:
- `src/handlers/xdg_shell/mod.rs` — Core handlers
- `src/handlers/xdg_shell/toplevel.rs` — Toplevel handling
- `src/handlers/xdg_shell/popup.rs` — Popup handling
- `src/handlers/xdg_shell/positioner.rs` — Positioner logic

### 11. `src/layout/row/mod.rs` — 1513 LOC
**Description**: Row implementation (core of Canvas2D)
**Status**: 🔴 **CRITICAL** — Actively being developed
**Note**: Already has submodules: gesture.rs, layout.rs, navigation.rs, render.rs, resize.rs, view_offset.rs, operations/
**Suggested Further Split**:
- `core.rs` (ColumnData)
- `tile_ops.rs`
- `columns.rs`
- `state.rs`

### 12. `src/tests/floating.rs` — 1368 LOC
**Description**: Floating window tests
**Status**: ⚫ **SKIP** — Test file, acceptable as-is

### 13. `src/window/mapped.rs` — 1367 LOC
**Description**: Mapped window state and operations
**Suggested Split**:
- `src/window/mapped/mod.rs` — Core MappedWindow
- `src/window/mapped/state.rs` — State management
- `src/window/mapped/rules.rs` — Window rule application
- `src/window/mapped/render.rs` — Rendering helpers

### 14. `src/pw_utils.rs` — 1280 LOC
**Description**: PipeWire utilities for screen capture
**Suggested Split**:
- `src/pw_utils/mod.rs` — Core PipeWire state
- `src/pw_utils/stream.rs` — Stream management
- `src/pw_utils/cast.rs` — Screen cast logic
- `src/pw_utils/buffer.rs` — Buffer handling

### 15. `src/layout/tests/golden.rs` — 1242 LOC
**Description**: Golden snapshot tests
**Status**: ⚫ **SKIP** — Test file, acceptable as-is

### 16. `src/ui/screenshot_ui.rs` — 1209 LOC
**Description**: Screenshot UI overlay
**Suggested Split**:
- `src/ui/screenshot_ui/mod.rs` — Core state
- `src/ui/screenshot_ui/render.rs` — Rendering
- `src/ui/screenshot_ui/selection.rs` — Selection handling
- `src/ui/screenshot_ui/input.rs` — Input handling

### 17. `niri-config/src/appearance.rs` — 1189 LOC
**Description**: Appearance configuration (colors, borders, etc.)
**Suggested Split**:
- `niri-config/src/appearance/mod.rs` — Core types
- `niri-config/src/appearance/colors.rs` — Color definitions
- `niri-config/src/appearance/borders.rs` — Border config
- `niri-config/src/appearance/focus_ring.rs` — Focus ring config

### 18. `src/input/pointer.rs` — 1120 LOC
**Description**: Pointer/mouse handling
**Status**: 🆕 **NEW** — Not in previous analysis
**Suggested Split**:
- `src/input/pointer/mod.rs` — Core pointer state
- `src/input/pointer/motion.rs` — Motion handling
- `src/input/pointer/button.rs` — Button handling
- `src/input/pointer/scroll.rs` — Scroll handling

### 19. `src/layout/tests/animations.rs` — 1099 LOC
**Description**: Animation tests
**Status**: ⚫ **SKIP** — Test file, acceptable as-is

### 20. `src/niri/render.rs` — 1056 LOC
**Description**: Niri rendering logic
**Status**: 🆕 **NEW** — Split from src/niri.rs
**Note**: May be acceptable at this size, monitor for growth

---

## Moderate Files (500-1000 LOC)

| LOC | File | Notes |
|-----|------|-------|
| 961 | `niri-config/src/binds.rs` | Keybinding config — split by action type |
| 942 | `src/input/mod.rs` | 🟡 **IMPROVED** — Was 5109 LOC, now 942 LOC |
| 923 | `src/protocols/output_management.rs` | Protocol handler — split by message type |
| 905 | `src/tests/window_opening.rs` | Test file — may be acceptable |
| 836 | `src/layout/tile/mod.rs` | 🟡 **IMPROVED** — Was 1469 LOC, now 836 LOC |
| 836 | `src/handlers/mod.rs` | Handler implementations — split by protocol |
| 804 | `niri-config/src/animations.rs` | Animation config — split by animation type |
| 802 | `src/ipc/server.rs` | IPC server — split by command category |
| 788 | `src/layout/layout_impl/interactive_move.rs` | 🆕 Split from layout/mod.rs |
| 777 | `src/tests/client.rs` | Test client — may be acceptable |
| 747 | `src/utils/watcher.rs` | File watcher — may be acceptable |
| 747 | `src/ipc/client.rs` | IPC client — split by command category |
| 745 | `niri-config/src/input.rs` | Input config — split by device type |
| 727 | `src/protocols/ext_workspace.rs` | Workspace protocol — split by message type |
| 706 | `src/ui/hotkey_overlay.rs` | Hotkey UI — split render/input |
| 646 | `niri-config/src/output.rs` | Output config — may be acceptable |
| 638 | `src/layout/canvas/floating/mod.rs` | 🆕 Canvas floating layout |
| 606 | `src/layout/monitor/mod.rs` | Monitor layout management |
| 600 | `src/layout/tests/fullscreen.rs` | Test file — may be acceptable |
| 592 | `src/layout/layout_impl/navigation.rs` | 🆕 Split from layout/mod.rs |
| 584 | `src/handlers/compositor.rs` | Compositor handlers |
| 566 | `src/utils/mod.rs` | Utility functions |
| 563 | `src/protocols/virtual_pointer.rs` | Virtual pointer protocol |
| 556 | `src/niri/config.rs` | 🆕 Split from niri/mod.rs |
| 543 | `src/render_helpers/shader_element.rs` | Shader rendering |
| 518 | `src/layout/canvas/navigation.rs` | 🆕 Canvas navigation |
| 508 | `src/protocols/screencopy.rs` | Screencopy protocol |
| 507 | `src/layout/tile/render.rs` | Tile rendering |
| 506 | `src/protocols/ext_row/manager.rs` | 🆕 Row protocol manager |
| 506 | `src/input/move_grab.rs` | Move grab handling |
| 500 | `src/layout/layout_impl/render.rs` | 🆕 Split from layout/mod.rs |

---

## Refactoring Progress Since TEAM_038

### Major Improvements ✅
1. **`src/niri.rs`**: 6603 → 2349 LOC (-64%) — Split into `src/niri/` module
2. **`src/input/mod.rs`**: 5109 → 942 LOC (-82%) — Split into submodules
3. **`src/layout/mod.rs`**: 5353 → 1875 LOC (-65%) — Split into `layout_impl/`
4. **`src/layout/tile.rs`**: 1469 → 836 LOC (-43%) — Split into `tile/`

### New Large Files (from refactoring)
- `src/input/actions.rs` — 1722 LOC (extracted from input/mod.rs)
- `src/input/pointer.rs` — 1120 LOC (extracted from input/mod.rs)
- `src/niri/render.rs` — 1056 LOC (extracted from niri.rs)
- `src/layout/layout_impl/interactive_move.rs` — 788 LOC (extracted from layout/mod.rs)

---

## Refactoring Priority

### Phase 1: Critical (Immediate)
1. `src/backend/tty.rs` (3473 LOC) — Highest remaining non-deprecated file
2. `niri-config/src/lib.rs` (2327 LOC) — Configuration core

### Phase 2: High Priority
3. `src/ui/mru.rs` (1940 LOC)
4. `niri-ipc/src/lib.rs` (1874 LOC)
5. `src/input/actions.rs` (1722 LOC)

### Phase 3: Medium Priority
6. `src/handlers/xdg_shell.rs` (1557 LOC)
7. `src/layout/row/mod.rs` (1513 LOC) — Coordinate with 2D canvas work
8. `src/window/mapped.rs` (1367 LOC)
9. `src/pw_utils.rs` (1280 LOC)
10. `src/ui/screenshot_ui.rs` (1209 LOC)

### Phase 4: Lower Priority
- Remaining 1000+ LOC files
- Test files (acceptable as-is)
- Config files (lower complexity)

---

## Notes

### Files That May Be Acceptable As-Is
- **Test files**: `tests.rs`, `golden.rs`, `animations.rs`, `floating.rs` — tests are often long but linear
- **Config files**: Lower complexity, mostly struct definitions
- **Protocol handlers**: Sometimes hard to split due to protocol structure

### Files Being Replaced
- `src/layout/deprecated/scrolling.rs` — Deprecated, will be removed when Row is complete
- Do NOT refactor deprecated files

### Dependencies on 2D Canvas Refactor
- `src/layout/row/mod.rs` — Actively changing
- `src/layout/canvas/` — New module structure
- Coordinate with ongoing refactor work

---

## Handoff

- [x] Code compiles (`cargo check`) — N/A (analysis only)
- [x] Tests pass (`cargo test`) — N/A (analysis only)
- [x] Team file complete
- [x] Analysis documented

## Work Completed

1. **Created updated LOC analysis** — This file
2. **Archived input refactor phases** — Moved `phase-I1*.md` to `archive/`
3. **Created TTY refactor phases** — 8 new phase files:
   - `phase-T1-tty-refactor.md` — Overview
   - `phase-T1.1-extract-types.md` — Types & module structure
   - `phase-T1.2-extract-device.md` — OutputDevice
   - `phase-T1.3-extract-helpers.md` — Helper functions
   - `phase-T1.4-extract-lifecycle.md` — Device lifecycle
   - `phase-T1.5-extract-connectors.md` — Connector handling
   - `phase-T1.6-extract-render.md` — Render pipeline
   - `phase-T1.7-extract-output.md` — Output management
4. **Updated phases/README.md** — Reflects new TTY focus

## Next Steps for Future Teams

1. Start with [Phase T1.1: Extract Types](../phases/phase-T1.1-extract-types.md)
2. Follow phases in order (T1.1 → T1.7)
3. Each phase is self-contained with verification steps
4. Update this document with completion status
