# Phase 1.5.3 Part 2D: Refactor Monitor into Modules + Remove Workspace

> **Status**: PENDING
> **Type**: BREAKING CHANGE + REFACTOR
> **Prerequisite**: Part 2C complete (Layout workspace methods removed)

---

## Overview

**Key insight**: Instead of just removing workspace code from monitor.rs, we **refactor it into modules** following the Row/Column pattern. Workspace code simply doesn't get migrated.

Current: `monitor.rs` (2255 lines monolith)
Target: `monitor/` (modular, ~800 lines total after workspace removal)

---

## Target Structure (following Row pattern)

```
src/layout/monitor/
├── mod.rs          - Core struct, new(), output/canvas accessors (~200 lines)
├── operations.rs   - Window operations: add_window, add_tile (~150 lines)
├── navigation.rs   - Row navigation (NEW: FocusRowDown/Up, MoveToRow) (~100 lines)
├── render.rs       - Rendering: render_elements (~200 lines)
├── hit_test.rs     - Geometry: window_under, resize_edges_under (~100 lines)
├── config.rs       - Config: update_config, update_output_size (~50 lines)
└── insert_hint.rs  - Insert hint types and rendering (~100 lines)
```

---

## What Gets Migrated (KEEP)

### Core Struct Fields
| Field | Module | Notes |
|-------|--------|-------|
| `output: Output` | mod.rs | |
| `output_name: String` | mod.rs | |
| `scale: Scale` | mod.rs | |
| `view_size: Size` | mod.rs | |
| `working_area: Rectangle` | mod.rs | |
| `canvas: Canvas2D<W>` | mod.rs | **Primary layout primitive** |
| `insert_hint` | insert_hint.rs | |
| `insert_hint_element` | insert_hint.rs | |
| `clock: Clock` | mod.rs | |
| `base_options: Rc<Options>` | mod.rs | |
| `options: Rc<Options>` | mod.rs | |
| `layout_config: Option<LayoutPart>` | mod.rs | |

### Methods to Migrate
| Method | Module | Notes |
|--------|--------|-------|
| `new()` | mod.rs | Simplified (no workspaces) |
| `output()`, `output_name()` | mod.rs | |
| `canvas()`, `canvas_mut()` | mod.rs | |
| `scale()`, `view_size()`, `working_area()` | mod.rs | |
| `windows()`, `has_window()` | mod.rs | Delegates to canvas |
| `add_window()`, `add_tile()` | operations.rs | |
| `active_window()` | navigation.rs | Delegates to canvas |
| `render_elements()` | render.rs | Simplified (no workspace switching) |
| `window_under()` | hit_test.rs | |
| `resize_edges_under()` | hit_test.rs | |
| `update_config()` | config.rs | |
| `update_output_size()` | config.rs | |
| `advance_animations()` | mod.rs | Simplified |
| `are_animations_ongoing()` | mod.rs | Simplified |

---

## What Gets REMOVED (NOT migrated)

### Types (entire removal)
| Type | Notes |
|------|-------|
| `WorkspaceSwitch` enum | Workspace animation |
| `WorkspaceSwitchGesture` struct | Gesture tracking |
| `InsertWorkspace` enum | Workspace-based insert |
| `OverviewProgress` enum | Overview mode |

### Constants
| Constant | Notes |
|----------|-------|
| `WORKSPACE_GESTURE_MOVEMENT` | |
| `WORKSPACE_GESTURE_RUBBER_BAND` | |
| `WORKSPACE_DND_EDGE_SCROLL_MOVEMENT` | |

### Fields
| Field | Notes |
|-------|-------|
| `workspaces: Vec<Workspace<W>>` | **Primary removal target** |
| `active_workspace_idx: usize` | |
| `previous_workspace_id: Option<WorkspaceId>` | |
| `workspace_switch: Option<WorkspaceSwitch>` | |
| `overview_open: bool` | |
| `overview_progress: Option<OverviewProgress>` | |

### Methods (not migrated)
| Method | Notes |
|--------|-------|
| `active_workspace_idx()` | Workspace accessor |
| `active_workspace_ref()` | Workspace accessor |
| `active_workspace()` | Workspace accessor |
| `find_named_workspace()` | Workspace query |
| `add_workspace_at()` | Workspace management |
| `clean_up_workspaces()` | Workspace cleanup |
| `activate_workspace()` | Workspace switching |
| `switch_workspace_*()` | Workspace switching |
| `move_to_workspace_*()` | Workspace movement |
| `move_column_to_workspace_*()` | Workspace movement |
| `move_workspace_*()` | Workspace reordering |
| `workspace_switch_gesture_*()` | Workspace gestures |
| `dnd_scroll_gesture_*()` | DnD gestures |
| `overview_zoom()` | Overview mode |
| `set_overview_progress()` | Overview mode |
| `workspace_render_idx()` | Workspace animation |
| `workspaces_render_geo()` | Workspace rendering |
| `workspace_under()` | Workspace hit test |
| `render_workspace_shadows()` | Overview rendering |

---

## New Methods to Add (Row Navigation)

These replace workspace navigation with row navigation:

| New Method | Replaces | Notes |
|------------|----------|-------|
| `focus_row_down()` | `switch_workspace_down()` | Navigate to row below |
| `focus_row_up()` | `switch_workspace_up()` | Navigate to row above |
| `move_window_to_row_down()` | `move_to_workspace_down()` | Move window to row below |
| `move_window_to_row_up()` | `move_to_workspace_up()` | Move window to row above |
| `move_column_to_row_down()` | `move_column_to_workspace_down()` | Move column to row below |
| `move_column_to_row_up()` | `move_column_to_workspace_up()` | Move column to row above |
| `move_row_down()` | `move_workspace_down()` | Reorder row |
| `move_row_up()` | `move_workspace_up()` | Reorder row |

---

## Execution Steps

### Step 1: Create module structure
```bash
mkdir -p src/layout/monitor
touch src/layout/monitor/{mod.rs,operations.rs,navigation.rs,render.rs,hit_test.rs,config.rs,insert_hint.rs}
```

### Step 2: Move core struct (mod.rs)
- Move `Monitor` struct with only kept fields
- Move `new()` without workspace initialization
- Move output/canvas accessors

### Step 3: Move operations (operations.rs)
- Move `add_window()`, `add_tile()`
- Update to use canvas directly (no workspace)

### Step 4: Create navigation (navigation.rs)
- Create new row navigation methods
- Delegate to `Canvas2D`

### Step 5: Move rendering (render.rs)
- Move `render_elements()` simplified for single canvas
- Remove workspace switching visual effects

### Step 6: Move hit testing (hit_test.rs)
- Move `window_under()`, `resize_edges_under()`
- Simplify (no workspace selection)

### Step 7: Move config (config.rs)
- Move `update_config()`, `update_output_size()`

### Step 8: Move insert hint (insert_hint.rs)
- Move `InsertHint` struct (without workspace variant)
- Move insert hint rendering

### Step 9: Delete old monitor.rs
- Verify all needed code migrated
- Delete monolith

---

## Verification

After each step:
1. `cargo check` — must compile
2. Continue to Part 2E (tests)

---

*TEAM_011: Phase 1.5.3 Part 2D — Refactor + Remove*
