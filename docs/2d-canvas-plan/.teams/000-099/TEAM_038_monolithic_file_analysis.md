# TEAM_038 — Monolithic File Analysis & Refactoring Plan

> **⚠️ UPDATED by TEAM_062 on Nov 29, 2025**

## Mission
Analyze all Rust files exceeding 500 LOC and create a comprehensive refactoring roadmap to break them into smaller, focused modules.

## Current Status
- **Date**: 2025-11-29 (updated from 2025-11-27)
- **Tool Used**: `wc -l` for LOC counting
- **Threshold**: 1000 LOC target, 500 LOC ideal
- **Violating Files**: 8 critical files (>2000 LOC)

---

## Summary Statistics (UPDATED Nov 29, 2025)

| Category | Count | Total LOC | Notes |
|----------|-------|-----------|-------|
| Critical (>2000 LOC) | 8 | 34,505 | PRIORITY |
| Severe (1000-2000 LOC) | 9 | ~12,000 | Can wait |
| Moderate (500-1000 LOC) | ~15 | ~10,000 | Lower priority |
| **Files to Skip** | 4 | ~10,000 | Deprecated/tests |

---

## Critical Files (>2000 LOC)

These are the highest priority for refactoring.

### 1. `src/niri.rs` — 6603 LOC (was 5141)
**Description**: Main application state and event handling
**Suggested Split**:
- `src/niri/mod.rs` — Core Niri struct and state
- `src/niri/render.rs` — Rendering logic
- `src/niri/events.rs` — Event handling
- `src/niri/output.rs` — Output management
- `src/niri/surface.rs` — Surface handling
- `src/niri/popups.rs` — Popup management
- `src/niri/cursor.rs` — Cursor handling
- `src/niri/screenshot.rs` — Screenshot functionality

### 2. `src/input/mod.rs` — 5109 LOC (was 4302)
**Description**: Input handling (keyboard, mouse, touch, gestures)
**Suggested Split**:
- `src/input/mod.rs` — Core input state
- `src/input/keyboard.rs` — Keyboard handling
- `src/input/pointer.rs` — Mouse/pointer handling
- `src/input/touch.rs` — Touch handling
- `src/input/tablet.rs` — Tablet/stylus handling
- `src/input/gestures.rs` — Gesture processing
- `src/input/bindings.rs` — Keybinding resolution
- `src/input/focus.rs` — Focus management

### 3. `src/layout/mod.rs` — 5353 LOC (was 3861) — CRITICAL!
**Description**: Layout management and window arrangement
**Suggested Split**:
- `src/layout/mod.rs` — Core Layout struct
- `src/layout/actions.rs` — User actions (move, resize, etc.)
- `src/layout/window_ops.rs` — Window operations
- `src/layout/focus.rs` — Focus management
- `src/layout/output_ops.rs` — Output-related operations
- `src/layout/state.rs` — State queries and updates
- `src/layout/render.rs` — Render element generation

### 4. `src/layout/tests.rs` — 3223 LOC
**Description**: Layout unit tests
**Suggested Split**:
- `src/layout/tests/mod.rs` — Test utilities and imports
- `src/layout/tests/basic.rs` — Basic layout tests
- `src/layout/tests/columns.rs` — Column operation tests
- `src/layout/tests/windows.rs` — Window operation tests
- `src/layout/tests/focus.rs` — Focus tests
- `src/layout/tests/resize.rs` — Resize tests
- `src/layout/tests/navigation.rs` — Navigation tests

### 5. `src/layout/scrolling.rs` — 3990 LOC (was 3000)
**Description**: Scrolling space implementation (legacy, being replaced by Row)
**Status**: ⚫ **SKIP** — Being deprecated in 2D Canvas refactor
**Action**: Do NOT refactor. Will be removed when Row is complete.

### 6. `src/backend/tty.rs` — 3465 LOC (was 2804)
**Description**: TTY/DRM backend for native display
**Suggested Split**:
- `src/backend/tty/mod.rs` — Core TTY backend
- `src/backend/tty/drm.rs` — DRM device handling
- `src/backend/tty/output.rs` — Output management
- `src/backend/tty/input.rs` — Input device handling
- `src/backend/tty/session.rs` — Session management
- `src/backend/tty/render.rs` — Rendering pipeline

### 7. `niri-config/src/lib.rs` — 2327 LOC (was 2163)
**Description**: Configuration parsing and validation
**Suggested Split**:
- `niri-config/src/lib.rs` — Core Config struct and parsing
- `niri-config/src/validation.rs` — Config validation
- `niri-config/src/defaults.rs` — Default values
- `niri-config/src/merge.rs` — Config merging logic
- `niri-config/src/parse.rs` — KDL parsing helpers

---

### 8. `src/layout/row/mod.rs` — 2161 LOC — NEW CRITICAL!
**Description**: Row implementation (core of Canvas2D)
**Status**: 🔴 **CRITICAL** — Actively being developed
**Suggested Split**:
- Already has: gesture.rs, layout.rs, navigation.rs, render.rs, resize.rs, view_offset.rs, operations/
- Need to add: `core.rs` (ColumnData), `tile_ops.rs`, `columns.rs`, `state.rs`

---

## Severe Files (1000-2000 LOC)

### 9. `src/ui/mru.rs` — 1940 LOC (was 1513)
**Description**: Most Recently Used window switcher UI
**Suggested Split**:
- `src/ui/mru/mod.rs` — Core MRU state
- `src/ui/mru/render.rs` — Rendering
- `src/ui/mru/input.rs` — Input handling
- `src/ui/mru/layout.rs` — Layout calculations

### 10. `src/handlers/xdg_shell.rs` — 1554 LOC (was 1228)
**Description**: XDG shell protocol handlers
**Suggested Split**:
- `src/handlers/xdg_shell/mod.rs` — Core handlers
- `src/handlers/xdg_shell/toplevel.rs` — Toplevel handling
- `src/handlers/xdg_shell/popup.rs` — Popup handling
- `src/handlers/xdg_shell/positioner.rs` — Positioner logic

### 11. `src/layout/floating.rs` — 1449 LOC (was 1113)
**Description**: Floating window layout
**Suggested Split**:
- `src/layout/floating/mod.rs` — Core FloatingSpace
- `src/layout/floating/operations.rs` — Window operations
- `src/layout/floating/render.rs` — Rendering
- `src/layout/floating/resize.rs` — Resize handling

### 12. `niri-ipc/src/lib.rs` — 1877 LOC (was 1106)
**Description**: IPC message types and serialization
**Suggested Split**:
- `niri-ipc/src/lib.rs` — Core types
- `niri-ipc/src/request.rs` — Request types
- `niri-ipc/src/response.rs` — Response types
- `niri-ipc/src/event.rs` — Event types

### 13. `src/layout/tile.rs` — 1469 LOC (was 1096)
**Description**: Tile (window container) implementation
**Suggested Split**:
- `src/layout/tile/mod.rs` — Core Tile struct
- `src/layout/tile/render.rs` — Rendering
- `src/layout/tile/resize.rs` — Resize logic
- `src/layout/tile/state.rs` — State management

### 14. `src/pw_utils.rs` — 1280 LOC (was 1057)
**Description**: PipeWire utilities for screen capture
**Suggested Split**:
- `src/pw_utils/mod.rs` — Core PipeWire state
- `src/pw_utils/stream.rs` — Stream management
- `src/pw_utils/cast.rs` — Screen cast logic
- `src/pw_utils/buffer.rs` — Buffer handling

### 15. `niri-config/src/appearance.rs` — 1189 LOC (was 1047)
**Description**: Appearance configuration (colors, borders, etc.)
**Suggested Split**:
- `niri-config/src/appearance/mod.rs` — Core types
- `niri-config/src/appearance/colors.rs` — Color definitions
- `niri-config/src/appearance/borders.rs` — Border config
- `niri-config/src/appearance/focus_ring.rs` — Focus ring config

### 16. `src/ui/screenshot_ui.rs` — 1209 LOC (was 1002)
**Description**: Screenshot UI overlay
**Suggested Split**:
- `src/ui/screenshot_ui/mod.rs` — Core state
- `src/ui/screenshot_ui/render.rs` — Rendering
- `src/ui/screenshot_ui/selection.rs` — Selection handling
- `src/ui/screenshot_ui/input.rs` — Input handling

### 17. `src/window/mapped.rs` — 1367 LOC (was 961)
**Description**: Mapped window state and operations
**Suggested Split**:
- `src/window/mapped/mod.rs` — Core MappedWindow
- `src/window/mapped/state.rs` — State management
- `src/window/mapped/rules.rs` — Window rule application
- `src/window/mapped/render.rs` — Rendering helpers

---

## Moderate Files (500-1000 LOC)

| LOC | File | Notes |
|-----|------|-------|
| 901 | `src/tests/floating.rs` | Test file — split by test category |
| 877 | `niri-config/src/binds.rs` | Keybinding config — split by action type |
| 858 | `src/protocols/output_management.rs` | Protocol handler — split by message type |
| 798 | `src/layout/tests/animations.rs` | Test file — may be acceptable |
| 727 | `niri-config/src/animations.rs` | Animation config — split by animation type |
| 724 | `src/tests/window_opening.rs` | Test file — split by scenario |
| 686 | `src/tests/client.rs` | Test client — may be acceptable |
| 677 | `niri-config/src/input.rs` | Input config — split by device type |
| 675 | `src/handlers/mod.rs` | Handler implementations — split by protocol |
| 660 | `src/layout/tests/golden.rs` | Golden tests — may be acceptable |
| 657 | `src/ipc/server.rs` | IPC server — split by command category |
| 647 | `src/ipc/client.rs` | IPC client — split by command category |
| 600 | `src/protocols/ext_workspace.rs` | Workspace protocol — split by message type |
| 591 | `src/utils/watcher.rs` | File watcher — may be acceptable |
| 581 | `src/ui/hotkey_overlay.rs` | Hotkey UI — split render/input |
| 565 | `niri-config/src/output.rs` | Output config — may be acceptable |

---

## Refactoring Priority

### Phase 1: Critical (Immediate)
1. `src/niri.rs` (5141 LOC) — Highest impact
2. `src/input/mod.rs` (4302 LOC) — Second highest
3. `src/layout/mod.rs` (3861 LOC) — Core to 2D canvas work

### Phase 2: High Priority
4. `src/backend/tty.rs` (2804 LOC)
5. `niri-config/src/lib.rs` (2163 LOC)
6. `src/ui/mru.rs` (1513 LOC)

### Phase 3: Medium Priority
7. `src/handlers/xdg_shell.rs` (1228 LOC)
8. `src/layout/floating.rs` (1113 LOC)
9. `niri-ipc/src/lib.rs` (1106 LOC)
10. `src/layout/tile.rs` (1096 LOC)

### Phase 4: Lower Priority
- Remaining 1000+ LOC files
- Test files (may be acceptable as-is)
- Config files (lower complexity)

---

## Notes

### Files That May Be Acceptable As-Is
- **Test files**: `tests.rs`, `golden.rs`, `animations.rs` — tests are often long but linear
- **Config files**: Lower complexity, mostly struct definitions
- **Protocol handlers**: Sometimes hard to split due to protocol structure

### Files Being Replaced
- `src/layout/scrolling.rs` — Being replaced by Row in 2D canvas refactor
- May not need refactoring if removal is imminent

### Dependencies on 2D Canvas Refactor
- `src/layout/mod.rs` — Actively changing
- `src/layout/row/mod.rs` — Actively changing
- Coordinate with ongoing refactor work

---

## Handoff

- [ ] Code compiles (`cargo check`) — N/A (analysis only)
- [ ] Tests pass (`cargo test`) — N/A (analysis only)
- [x] Team file complete
- [x] Analysis documented

## Next Steps for Future Teams

1. Pick a file from Phase 1 priority list
2. Create detailed split plan with function inventory
3. Execute refactor following modular principles
4. Update this document with completion status
