# Global TODO List — 2D Canvas Refactor

> **Check this file first** before starting work.
> This is the single source of truth for what needs to be done.

**Last updated**: TEAM_062

---

# 🚨 WORKSPACE → CANVAS2D TERMINOLOGY MIGRATION

> **CRITICAL**: Workspaces are **COMPLETELY REMOVED** from Canvas2D.  
> Rows are NOT workspaces. See `README.md` for full explanation.

## Migration Status Overview

| Area | Status | Notes |
|------|--------|-------|
| **Internal Layout Code** | ✅ Complete | TEAM_060 - WorkspaceId→RowId, method renames |
| **Config (niri-config)** | ✅ Complete | TEAM_055 |
| **Test Operations (Op::)** | ✅ Complete | TEAM_014 |
| **Test Function Names** | ⏳ Pending | Still use "workspace" |
| **IPC Commands** | ⏳ Pending | Will be redesigned |
| **User Documentation** | ⏳ Pending | After code migration |

---

## 📋 TERMINOLOGY MIGRATION CHECKLIST

### Legend
- ✅ = Complete
- 🔄 = In Progress  
- ⏳ = Pending
- ❌ = Blocked

---

### 1. Type/Struct Renames

| Old Name | New Name | File | Status |
|----------|----------|------|--------|
| `WorkspaceId` | `RowId` | `src/layout/row_types.rs` | ✅ Done |
| `WorkspaceAddWindowTarget` | `RowAddWindowTarget` | `src/layout/row_types.rs` | ✅ Done |
| `Workspace` (config) | `RowConfig` | `niri-config/src/` | ✅ Done |
| `WorkspaceName` | `RowName` | `niri-config/src/` | ✅ Done |

### 2. Method Renames (src/layout/mod.rs)

| Old Name | New Name | Status |
|----------|----------|--------|
| `move_to_workspace()` | `move_to_row()` | ✅ Done |
| `move_to_workspace_up()` | `move_to_row_up()` | ✅ Done |
| `move_to_workspace_down()` | `move_to_row_down()` | ✅ Done |
| `move_column_to_workspace()` | `move_column_to_row()` | ✅ Done |
| `focus_workspace()` | `focus_row()` | ✅ Done |
| `focus_workspace_up()` | `focus_row_up()` | ✅ Done |
| `focus_workspace_down()` | `focus_row_down()` | ✅ Done |
| `active_workspace()` | `active_row()` | ✅ Done |
| `active_workspace_idx()` | `active_row_idx()` | ✅ Done |
| `find_workspace_by_name()` | `find_row_by_name()` | ✅ Done |
| `ensure_named_workspace()` | `ensure_named_row()` | ✅ Done |

### 3. Method Renames (src/layout/monitor/)

| Old Name | New Name | File | Status |
|----------|----------|------|--------|
| `active_workspace_idx()` | `active_row_idx()` | `mod.rs` | ✅ Done |
| `workspaces()` | `rows()` | `mod.rs` | ✅ Done |
| `workspaces_mut()` | `rows_mut()` | `mod.rs` | ✅ Done |

### 4. Method Renames (src/layout/canvas/)

| Old Name | New Name | File | Status |
|----------|----------|------|--------|
| `workspaces()` | `rows()` | `operations.rs` | ✅ Done |
| `workspaces_mut()` | `rows_mut()` | `operations.rs` | ✅ Done |

### 5. Field Renames

| Old Name | New Name | File | Status |
|----------|----------|------|--------|
| `last_active_workspace_id` | `last_active_row_id` | `src/layout/mod.rs` | ✅ Done |
| `workspace_id_counter` | `row_id_counter` | `src/layout/mod.rs` | ✅ Done |
| `workspace_id` | `row_id` | Various | ⏳ Pending |

### 6. Test Operation Renames (src/layout/tests.rs)

| Old Name | New Name | Status |
|----------|----------|--------|
| `Op::MoveWindowToWorkspace*` | `Op::MoveWindowToRow*` | ✅ Done |
| `Op::MoveColumnToWorkspace*` | `Op::MoveColumnToRow*` | ✅ Done |
| `Op::FocusWorkspace*` | `Op::FocusRow*` | ✅ Done |
| `Op::MoveWorkspace*` | `Op::MoveRow*` | ✅ Done |
| `Op::SetWorkspaceName` | `Op::SetRowName` | ✅ Done |

### 7. Test Function Renames (src/layout/tests.rs)

> **Note**: These test user-facing behavior, so "workspace" in names may be intentional.
> Decide: Keep for user-facing semantics OR rename to "row"?

| Current Name | Proposed Name | Status |
|--------------|---------------|--------|
| `move_to_workspace_by_idx_*` | `move_to_row_by_idx_*` | ⏳ Pending |
| `move_workspace_to_output` | `move_row_to_output` | ⏳ Pending |
| `removing_all_outputs_preserves_empty_named_workspaces` | `..._named_rows` | ⏳ Pending |
| `removing_output_must_keep_empty_focus_on_primary` | Keep? | ⏳ Decide |

### 8. IPC Commands (src/niri.rs, niri-ipc/)

| Old Command | New Command | Status |
|-------------|-------------|--------|
| `focus-workspace` | `focus-row` | ✅ Done (already migrated) |
| `move-window-to-workspace` | `move-window-to-row` | ✅ Done (already migrated) |
| `move-column-to-workspace` | `move-column-to-row` | ✅ Done (already migrated) |

**⚠️ ADDITIONAL FINDINGS from IPC Audit:**

#### IPC Events (NEEDS MIGRATION):
- `WorkspacesChanged` → `RowsChanged` ⏳ Pending
- `WorkspaceUrgencyChanged` → `RowUrgencyChanged` ⏳ Pending  
- `WorkspaceActivated` → `RowActivated` ⏳ Pending
- `WorkspaceActiveWindowChanged` → `RowActiveWindowChanged` ⏳ Pending

#### IPC State Structures (NEEDS MIGRATION):
- `Workspace` struct → `Row` struct ⏳ Pending
- `WorkspacesState` → `RowsState` ⏳ Pending
- `Request::Workspaces` → `Request::Rows` ⏳ Pending

#### Protocol Implementation (🛑 DEFERRED - See Critical Analysis):
- ✅ **Protocol Specification**: `ext_row_v1.xml` - designed for TARGET Canvas2D
- ✅ **Core Trait Definitions**: `ExtRowHandler` trait and core types
- ✅ **Manager Implementation**: Basic protocol state management
- 🛑 **DEFERRED**: Full implementation blocked until zoom/bookmarks exist

**Why Deferred?**: The ext-row protocol is designed for Canvas2D with zoom and bookmarks.
Current Canvas2D behaves like workspaces (one row visible at a time). The protocol should
wait until the compositor actually supports what the protocol exposes.

**Current Plan**: Keep ext-workspace protocol (rows ≈ workspaces) until Phase 3-4 complete.

**See**: `.teams/TEAM_060_ext_row_protocol_design.md` for complete design
**See**: Critical Analysis section above for rationale

### 9. Config Options (niri-config/)

| Old Option | New Option | Status |
|------------|------------|--------|
| `workspace { }` | `row { }` | ✅ Done |
| `open-on-workspace` | `open-on-row` | ✅ Done |
| `workspace-switch` animation | `row-switch` animation | ✅ Done |
| `empty-workspace-above-first` | `empty-row-above-first` | ✅ Done |

### 10. NEW: Camera Bookmark System

> **These are NEW features**, not renames

| Feature | Description | Status |
|---------|-------------|--------|
| `CameraBookmark` struct | Stores `(x, y, zoom)` | ⏳ Pending |
| `camera-bookmark-save` IPC | Save current position | ⏳ Pending |
| `camera-bookmark-goto` IPC | Jump to bookmark | ⏳ Pending |
| `Mod+1/2/3` bindings | Jump to bookmark | ⏳ Pending |
| `Mod+Shift+1/2/3` bindings | Save bookmark | ⏳ Pending |

---

# 🔍 CRITICAL ANALYSIS: Canvas2D vs Workspaces (TEAM_060)

> **Key Insight**: Canvas2D is NOT a renamed workspace system!
> It's a fundamentally different architecture.

## 🏗️ Architectural Difference (THE CORE INSIGHT)

```
WORKSPACES (Old Architecture)
============================
┌──────────────────────────────────────────────────────────────┐
│                        OUTPUT                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐           │
│  │ WORKSPACE 1 │  │ WORKSPACE 2 │  │ WORKSPACE 3 │   ...     │
│  │             │  │             │  │             │           │
│  │ [isolated]  │  │ [isolated]  │  │ [isolated]  │           │
│  │ [separate]  │  │ [separate]  │  │ [separate]  │           │
│  │ [container] │  │ [container] │  │ [container] │           │
│  └─────────────┘  └─────────────┘  └─────────────┘           │
│        ↑                                                      │
│    VISIBLE                                                    │
│  (only ONE at a time)                                         │
│                                                               │
│  User "switches" between workspaces: Mod+1, Mod+2, Mod+3      │
└──────────────────────────────────────────────────────────────┘

CANVAS2D (New Architecture)  
===========================
┌──────────────────────────────────────────────────────────────┐
│                        OUTPUT                                 │
│  ┌──────────────────────────────────────────────────────────┐│
│  │                    ONE INFINITE CANVAS                    ││
│  │                                                           ││
│  │  ROW 0 ─────────────────────────────────────────────────  ││
│  │  │ Col A │ Col B │ Col C │ ...     (ScrollingSpace)       ││
│  │  ─────────────────────────────────────────────────────────││
│  │  ROW 1 ─────────────────────────────────────────────────  ││
│  │  │ Col D │ Col E │ ... │           (ScrollingSpace)       ││
│  │  ─────────────────────────────────────────────────────────││
│  │  ROW 2 ─────────────────────────────────────────────────  ││
│  │  │ Col F │ ... │                   (ScrollingSpace)       ││
│  │  ─────────────────────────────────────────────────────────││
│  │           ↑                                               ││
│  │      CAMERA VIEWPORT                                      ││
│  │      (can see MULTIPLE rows at once via zoom)             ││
│  │                                                           ││
│  └──────────────────────────────────────────────────────────┘│
│                                                               │
│  User PANS and ZOOMS the camera: no "switching"               │
│  Mod+1/2/3 = Jump to saved camera BOOKMARK (x, y, zoom)       │
└──────────────────────────────────────────────────────────────┘
```

## 📐 What Each Component Actually Is

| Component | Definition | Equivalent |
|-----------|------------|------------|
| **Row** | A horizontal ScrollingSpace with columns | `Row = ScrollingSpace` |
| **Canvas2D** | Multiple stacked Rows on one surface | `Canvas2D = Stack of Rows` |
| **Camera** | Viewport with (x, y, zoom) into Canvas | New concept |
| **Bookmark** | Saved camera position (x, y, zoom) | Replaces workspace numbers |

### The Key Equation

```
Canvas2D = Row₀ + Row₁ + Row₂ + ... + Rowₙ   (stacked vertically)

Where:
  Row = ScrollingSpace (horizontal layout of columns)
  Camera = (x, y, zoom) viewport into the canvas
  
User Experience:
  - Zoom OUT → see multiple rows simultaneously
  - Zoom IN → see one row (like current behavior)
  - Pan → move camera across the infinite canvas
  - Bookmark → save (x, y, zoom) for quick jumps
```

## ✅ What's Already Implemented

| Feature | Status | Notes |
|---------|--------|-------|
| **Row struct** | ✅ Done | Equivalent to ScrollingSpace |
| **Canvas2D with multiple rows** | ✅ Done | BTreeMap<i32, Row> storage |
| **Row navigation** | ✅ Done | focus_up/down between rows |
| **Camera X, Y** | ✅ Done | AnimatedValue for position |
| **Terminology** | ✅ Done | workspace → row renames |

## ❌ What's NOT Yet Implemented (CRITICAL!)

### 1. Camera Zoom (Phase 4) - **THE DIFFERENTIATOR**

Without zoom, Canvas2D is functionally identical to workspaces!

```rust
// Currently in Canvas2D:
pub struct Canvas2D<W> {
    camera_x: AnimatedValue,  // ✅ Exists
    camera_y: AnimatedValue,  // ✅ Exists
    // camera_zoom: AnimatedValue,  // ❌ MISSING!
}

// Required:
pub struct Canvas2D<W> {
    camera: Camera,  // x, y, AND zoom
}

pub struct Camera {
    x: AnimatedValue,
    y: AnimatedValue,
    zoom: AnimatedValue,  // 1.0 = normal, 0.5 = see 2 rows
}
```

**Why zoom matters**: 
- At zoom 1.0: See 1 row (like workspaces)
- At zoom 0.5: See 2 rows at once
- At zoom 0.25: See 4 rows at once
- This is the FUNDAMENTAL difference from workspaces!

### 2. Zoom-Based Visibility

```rust
impl Canvas2D {
    /// Which rows are currently visible in the viewport?
    fn visible_rows(&self) -> Vec<i32> {
        let viewport_height = self.view_size.h / self.camera.zoom();
        // Calculate which rows intersect the viewport...
    }
    
    /// What area of the canvas is visible?
    fn visible_area(&self) -> Rectangle {
        let w = self.view_size.w / self.camera.zoom();
        let h = self.view_size.h / self.camera.zoom();
        Rectangle::from_loc_and_size(
            (self.camera.x() - w/2, self.camera.y() - h/2),
            (w, h)
        )
    }
}
```

### 3. Zoom Controls (Phase 4)

| Shortcut | Action |
|----------|--------|
| `Mod+Scroll` | Zoom in/out |
| `Mod+0` | Reset zoom to 100% |
| `Mod+=` | Zoom to fit focused window |
| `Mod+Shift+=` | Zoom to fit all windows |

### 4. Camera Bookmarks (Phase 5)

```rust
pub struct CameraBookmark {
    x: f64,           // Camera X position
    y: f64,           // Camera Y position  
    zoom: f64,        // Zoom level
    row_name: Option<String>,  // Optional row reference
    name: Option<String>,      // User label
}

// User actions:
// Mod+Shift+1 → save_bookmark(1) - save current (x, y, zoom)
// Mod+1 → goto_bookmark(1) - animate camera to saved position
```

### 5. Row Spanning (Phase 3)

```rust
pub struct Tile<W> {
    row_span: u8,  // 1 = normal, 2+ = spans multiple rows
}

// A window can span multiple rows vertically:
// ┌─────────────────────────────────────┐
// │ ROW 0:  [App A] [App B] [ BIG APP ] │
// │ ROW 1:  [App C] [App D] [   ↑↑↑   ] │  ← BIG APP spans 2 rows
// │ ROW 2:  [App E] ...                 │
// └─────────────────────────────────────┘
```

### 6. Zoom-Based Rendering

```rust
impl Canvas2D {
    fn render_elements(&self) -> Vec<RenderElement> {
        let zoom = self.camera.zoom();
        let visible = self.visible_area();
        
        // Only render rows that are visible
        for row in self.rows_in_area(visible) {
            // Scale all elements by zoom factor
            let elements = row.render_elements()
                .map(|e| e.scaled(zoom));
            // Transform positions relative to camera
            // ...
        }
    }
}
```

## 📋 Complete Canvas2D Requirements Checklist

### Phase 3: Row Spanning
- [ ] Add `row_span: u8` to Tile
- [ ] Compute tile height as `row_span * row_height`
- [ ] Handle occupied positions across rows
- [ ] Navigation respects spanning tiles
- [ ] Actions: `increase-row-span`, `decrease-row-span`, `set-row-span N`

### Phase 4: Camera System  
- [ ] Add `camera_zoom: AnimatedValue` to Camera
- [ ] Implement `visible_rows()` based on zoom
- [ ] Implement `visible_area()` based on zoom
- [ ] Zoom rendering: scale all elements by zoom factor
- [ ] Input transform: convert screen coords to canvas coords at any zoom
- [ ] Actions: `zoom-in`, `zoom-out`, `zoom-reset`, `zoom-to-fit`
- [ ] Keybinds: `Mod+Scroll`, `Mod+0`, `Mod+=`
- [ ] Auto-zoom: focus spanning tile → zoom to fit its span
- [ ] Config: `camera-movement` and `camera-zoom` animation settings

### Phase 5: Camera Bookmarks
- [ ] Add `CameraBookmark` struct with (x, y, zoom, row_name?, name?)
- [ ] Add `bookmarks: Vec<CameraBookmark>` to Canvas2D (10 slots)
- [ ] Implement `save_bookmark(slot)` - save current camera state
- [ ] Implement `goto_bookmark(slot)` - animate camera to saved position
- [ ] Actions: `save-bookmark N`, `jump-to-bookmark N`, `delete-bookmark N`
- [ ] Keybinds: `Mod+1/2/3...` = jump, `Mod+Shift+1/2/3...` = save
- [ ] IPC: `niri msg bookmarks`, `niri msg jump-to-bookmark N`
- [ ] Optional: persist bookmarks to state file

### Phase 6: Protocol (AFTER above phases)
- [ ] Update ext-workspace to expose camera state
- [ ] Add camera movement events
- [ ] Add bookmark events
- [ ] Eventually: full ext-row protocol migration

---

## Current Implementation Status

**What's Done**: Terminology, Row struct, basic navigation
**What's Missing**: Zoom, bookmarks, row spanning - the features that MAKE Canvas2D different!

---

# 🎯 UPDATED PRIORITY ORDER (TEAM_060)

## Phase 1: Terminology Cleanup ✅ MOSTLY COMPLETE
- ✅ Internal type renames
- ✅ Internal method renames  
- 🔄 Remaining cleanup (see sections above)

## Phase 2: Camera System (CRITICAL PATH)
> **This is what makes Canvas2D actually different from workspaces!**

| Task | Status | Priority |
|------|--------|----------|
| Add `camera_zoom: AnimatedValue` to Canvas2D | ⏳ Pending | 🔴 HIGH |
| Implement `visible_rows()` based on zoom | ⏳ Pending | 🔴 HIGH |
| Add zoom rendering (scale all elements) | ⏳ Pending | 🔴 HIGH |
| Add `Mod+Scroll` zoom gesture | ⏳ Pending | 🟡 MEDIUM |
| Add `Mod+0` reset zoom | ⏳ Pending | 🟡 MEDIUM |
| Add `Mod+=` zoom to fit focused | ⏳ Pending | 🟡 MEDIUM |

## Phase 3: Camera Bookmarks
> **This replaces workspace switching entirely!**

| Task | Status | Priority |
|------|--------|----------|
| Create `CameraBookmark` struct | ⏳ Pending | 🔴 HIGH |
| Add bookmark storage to Canvas2D | ⏳ Pending | 🔴 HIGH |
| Implement `save_bookmark(slot)` | ⏳ Pending | 🔴 HIGH |
| Implement `goto_bookmark(slot)` | ⏳ Pending | 🔴 HIGH |
| Add `Mod+1/2/3` goto bindings | ⏳ Pending | 🔴 HIGH |
| Add `Mod+Shift+1/2/3` save bindings | ⏳ Pending | 🔴 HIGH |
| IPC: `camera-bookmark-save` | ⏳ Pending | 🟡 MEDIUM |
| IPC: `camera-bookmark-goto` | ⏳ Pending | 🟡 MEDIUM |

## Phase 4: Protocol Migration  
> **Only AFTER zoom and bookmarks work!**

| Task | Status | Priority |
|------|--------|----------|
| Update ext-workspace to expose zoom | ⏳ Pending | 🟡 MEDIUM |
| Add camera movement events | ⏳ Pending | 🟡 MEDIUM |
| Add bookmark events | ⏳ Pending | 🟡 MEDIUM |
| Full ext-row protocol migration | ⏳ Pending | 🟢 LOW |

## Phase 5: Row Spanning (FUTURE)
> **Advanced feature, can wait**

| Task | Status | Priority |
|------|--------|----------|
| Add `row_span` to Tile | ⏳ Pending | 🟢 LOW |
| Cross-row rendering | ⏳ Pending | 🟢 LOW |
| Row span commands | ⏳ Pending | 🟢 LOW |

---

## 🎯 MIGRATION PRIORITY ORDER

1. **Phase A**: Internal type renames (`WorkspaceId` → `RowId`)
2. **Phase B**: Internal method renames (all `*workspace*` → `*row*`)
3. **Phase C**: Test function renames (if decided)
4. **Phase D**: IPC command redesign
5. **Phase E**: Camera bookmark implementation
6. **Phase F**: User documentation update

---

## ⚠️ MIGRATION RULES

1. **Never use "workspace" in new code** - use "row" or "canvas"
2. **Rows are NOT workspaces** - they're horizontal layout strips
3. **Camera bookmarks replace workspace switching** - different concept entirely
4. **One Canvas2D per output** - no discrete containers
5. **Update imports** when renaming files/types

---

# 🏗️ COMPREHENSIVE MODULE ARCHITECTURE REFACTOR (TEAM_062)

> **Goal**: Restructure the entire `src/layout/` module for proper hierarchy and single responsibility
> **Principle**: Hierarchy should match: `Layout → Monitor → Canvas2D → Row → Column → Tile`

## Current Problems

### 1. Monolithic Files
| File | LOC | Problem |
|------|-----|---------|
| `src/layout/mod.rs` | 5353 | Layout struct + MonitorSet + ALL 229 methods |
| `src/layout/row/mod.rs` | 2161 | Row struct + methods despite having submodules |
| `src/layout/tile.rs` | 1469 | Flat file, should be module |
| `src/layout/floating.rs` | 1449 | Confusingly separate from canvas/ |

### 2. Misplaced Files
| File | LOC | Problem |
|------|-----|---------|
| `closing_window.rs` | 275 | Render element scattered at root |
| `opening_window.rs` | 143 | Render element scattered at root |
| `focus_ring.rs` | 280 | Render element scattered at root |
| `shadow.rs` | 184 | Render element scattered at root |
| `tab_indicator.rs` | 412 | Render element scattered at root |
| `insert_hint_element.rs` | 65 | Render element scattered at root |

### 3. Conceptual Confusion
- `floating.rs` (1449 LOC) is SIBLING to `canvas/` but FloatingSpace is PART OF Canvas2D
- `canvas/floating.rs` (292 LOC) exists separately — delegation layer adds confusion
- No clear hierarchy visible in file structure

### 4. Dead Code
| File | LOC | Status |
|------|-----|--------|
| `scrolling.rs` | 3990 | Being replaced by Row |
| `workspace.rs` | 0 | Empty placeholder |

---

## Target Architecture

```
src/layout/  (~50 focused files instead of ~20 bloated ones)
│
├── mod.rs (~400 LOC)
│   - Layout struct definition (fields only)
│   - MonitorSet enum
│   - LayoutElement trait
│   - pub mod declarations
│   - Re-exports
│
├── types.rs (~150 LOC)
│   - ColumnWidth, SizingMode, ConfigureIntent
│   - HitType, ActivateWindow, AddWindowTarget
│   - All shared type definitions
│
├── options.rs (~100 LOC)  ← NEW
│   - Options struct
│   - Default implementations
│
├── layout_impl/  ← NEW (extract 229 methods from mod.rs)
│   ├── mod.rs — Re-exports
│   ├── window_ops.rs (~600 LOC)
│   │   - add_window, remove_window, update_window
│   │   - find_window_*, find_wl_surface_*
│   │   - descendants_added
│   ├── output_ops.rs (~400 LOC)
│   │   - add_output, remove_output
│   │   - update_output_size
│   ├── focus.rs (~500 LOC)
│   │   - activate_window, activate_window_without_raising
│   │   - active_output, active_row, active_monitor
│   ├── navigation.rs (~800 LOC)
│   │   - move_*, focus_* direction methods
│   │   - move_to_row_*, focus_row_*
│   ├── resize.rs (~500 LOC)
│   │   - set_window_width/height
│   │   - interactive_resize_*
│   ├── fullscreen.rs (~400 LOC)
│   │   - set_fullscreen, toggle_fullscreen
│   │   - set_maximized, toggle_maximized
│   ├── row_management.rs (~500 LOC)
│   │   - find_row_*, ensure_named_row
│   │   - unname_*, row lifecycle
│   ├── queries.rs (~400 LOC)
│   │   - is_*, has_*, should_*
│   │   - All state inspection methods
│   ├── interactive_move.rs (~400 LOC)
│   │   - interactive_move_*
│   │   - DnD handling, InteractiveMoveData
│   └── render.rs (~500 LOC)
│       - render_*, refresh
│       - Render element generation
│
├── elements/  ← NEW (group ALL render elements)
│   ├── mod.rs — Re-exports
│   ├── closing_window.rs ← from ../closing_window.rs
│   ├── opening_window.rs ← from ../opening_window.rs
│   ├── focus_ring.rs ← from ../focus_ring.rs
│   ├── shadow.rs ← from ../shadow.rs
│   ├── tab_indicator.rs ← from ../tab_indicator.rs
│   └── insert_hint.rs ← from ../insert_hint_element.rs
│
├── monitor/  — KEEP (already well-structured)
│   ├── mod.rs (~400 LOC)
│   ├── types.rs — InsertHint, WorkspaceSwitch
│   ├── navigation.rs
│   ├── render.rs
│   ├── hit_test.rs
│   ├── config.rs
│   └── gestures.rs
│
├── canvas/  — ENHANCED (absorb floating)
│   ├── mod.rs (~400 LOC) — Canvas2D struct
│   ├── floating/  ← NEW (move from ../floating.rs)
│   │   ├── mod.rs (~400 LOC) — FloatingSpace struct
│   │   ├── operations.rs — add/remove tile
│   │   ├── render.rs — FloatingSpaceRenderElement
│   │   └── resize.rs — resize handling
│   ├── navigation.rs (520 LOC)
│   ├── operations/  ← SPLIT from operations.rs (869 LOC)
│   │   ├── mod.rs — Re-exports
│   │   ├── window.rs — add/remove window
│   │   ├── tile.rs — tile manipulation
│   │   ├── row.rs — row creation/deletion
│   │   └── state.rs — state updates
│   └── render.rs
│
├── row/  — ENHANCED (split mod.rs further)
│   ├── mod.rs (~400 LOC) — Row struct, exports
│   ├── core.rs ← NEW — ColumnData struct
│   ├── tile_ops.rs ← NEW — add_tile, remove_tile
│   ├── columns.rs ← NEW — Column iteration
│   ├── state.rs ← NEW — is_*, has_*, count_* queries
│   ├── gesture.rs (445 LOC)
│   ├── layout.rs (100 LOC)
│   ├── navigation.rs (213 LOC)
│   ├── render.rs (199 LOC)
│   ├── resize.rs (151 LOC)
│   ├── view_offset.rs (321 LOC)
│   └── operations/
│       ├── mod.rs
│       ├── add.rs
│       ├── remove.rs
│       ├── move_col.rs
│       └── consume.rs
│
├── tile/  ← NEW (split from tile.rs)
│   ├── mod.rs (~400 LOC) — Tile struct, core
│   ├── state.rs — State flags, getters
│   ├── resize.rs — Interactive resize
│   └── render.rs — TileRenderElement
│
├── column/  — KEEP (already well-structured)
│   ├── mod.rs, core.rs, layout.rs, operations.rs, render.rs
│   ├── tests.rs, tile_data.rs
│   └── sizing/
│
├── animated_value/  — KEEP
│   ├── mod.rs
│   └── gesture.rs
│
├── row_types.rs — KEEP
├── snapshot.rs — KEEP (testing infrastructure)
│
├── tests/  — KEEP
│   ├── mod.rs, tests.rs
│   ├── animations.rs, fullscreen.rs, golden.rs
│   └── snapshots/
│
└── DELETE:
    ✗ scrolling.rs (deprecated, replaced by Row)
    ✗ workspace.rs (empty placeholder)
    ✗ floating.rs (moved to canvas/floating/)
    ✗ closing_window.rs (moved to elements/)
    ✗ opening_window.rs (moved to elements/)
    ✗ focus_ring.rs (moved to elements/)
    ✗ shadow.rs (moved to elements/)
    ✗ tab_indicator.rs (moved to elements/)
    ✗ insert_hint_element.rs (moved to elements/)
```

---

## Execution Phases

### Phase 0: Cleanup (1 hour, NO RISK) ✅ COMPLETE (TEAM_062)
**Delete dead code:**
- [x] Delete `workspace.rs` (empty file)
- [x] Move `scrolling.rs` to `deprecated/scrolling.rs` (keep for reference)
- [x] Fix `ScrollDirection` import in `src/input/mod.rs` (was using scrolling.rs, now uses types.rs)
- [x] Remove `pub mod scrolling;` from `src/layout/mod.rs`

**Verification:**
```bash
cargo check    # ✅ Passes (warnings only)
cargo test layout::  # ✅ 187 passed
```

---

### Phase 1: Create `elements/` Module (2 hours, LOW RISK) ✅ COMPLETE (TEAM_062)

**Why first**: Just file moves + import updates. No method changes.

**Steps:**
1. [x] Create `src/layout/elements/mod.rs`
2. [x] Move files:
   - [x] `closing_window.rs` → `elements/closing_window.rs`
   - [x] `opening_window.rs` → `elements/opening_window.rs`
   - [x] `focus_ring.rs` → `elements/focus_ring.rs`
   - [x] `shadow.rs` → `elements/shadow.rs`
   - [x] `tab_indicator.rs` → `elements/tab_indicator.rs`
   - [x] `insert_hint_element.rs` → `elements/insert_hint.rs`
3. [x] Update `mod.rs` to declare `pub mod elements;`
4. [x] Add re-exports in `elements/mod.rs`
5. [x] Update all imports across codebase:
   - `src/layout/tile.rs`
   - `src/layout/row/mod.rs`
   - `src/layout/row/render.rs`
   - `src/layout/column/mod.rs`
   - `src/layout/column/core.rs`
   - `src/layout/column/render.rs`
   - `src/layout/floating.rs`
   - `src/layout/monitor/mod.rs` (including render element macro)
   - `src/ui/mru.rs`
   - `src/layer/mapped.rs`
   - `src/layout/elements/tab_indicator.rs` (internal super::super:: fix)

**Verification:**
```bash
cargo check    # ✅ Passes (warnings only)
cargo test     # ✅ 270 passed
```

---

### Phase 2: Consolidate FloatingSpace into `canvas/` (4 hours, MEDIUM RISK)

**Why**: FloatingSpace is PART OF Canvas2D. Current structure is confusing.

**Current:**
- `src/layout/floating.rs` (1449 LOC) — FloatingSpace struct + impl
- `src/layout/canvas/floating.rs` (292 LOC) — Canvas2D floating methods

**Target:**
```
canvas/floating/
├── mod.rs (~400 LOC) — FloatingSpace struct, core impl
├── operations.rs (~400 LOC) — add/remove tile
├── render.rs (~300 LOC) — FloatingSpaceRenderElement
└── resize.rs (~300 LOC) — resize handling
```

**Steps:**
1. [ ] Create `canvas/floating/` directory
2. [ ] Split `floating.rs` into:
   - [ ] `canvas/floating/mod.rs` — Struct, basic methods
   - [ ] `canvas/floating/operations.rs` — add_tile, remove_tile
   - [ ] `canvas/floating/render.rs` — render elements
   - [ ] `canvas/floating/resize.rs` — resize handling
3. [ ] Merge `canvas/floating.rs` methods into appropriate files
4. [ ] Delete old `floating.rs`
5. [ ] Update imports

**Verification:**
```bash
cargo check
cargo test tests::floating
cargo test layout::
```

---

### Phase 3: Split `tile.rs` into `tile/` Module (3 hours, MEDIUM RISK)

**Current:** 1469 LOC flat file

**Target:**
```
tile/
├── mod.rs (~400 LOC) — Tile struct, core impl
├── state.rs (~300 LOC) — State flags, is_*, has_*
├── resize.rs (~400 LOC) — Interactive resize, resize_edges_under
└── render.rs (~400 LOC) — TileRenderElement, rendering
```

**Steps:**
1. [ ] Create `tile/` directory
2. [ ] Move `tile.rs` → `tile/mod.rs`
3. [ ] Extract into submodules:
   - [ ] `tile/state.rs` — State getters/setters
   - [ ] `tile/resize.rs` — Resize methods
   - [ ] `tile/render.rs` — Render element
4. [ ] Update imports

**Verification:**
```bash
cargo check
cargo test
```

---

### Phase 4: Split `row/mod.rs` Further (4 hours, MEDIUM RISK)

**Current:** 2161 LOC despite existing submodules

**Target additions:**
```
row/
├── core.rs ← NEW — ColumnData struct, internal state
├── tile_ops.rs ← NEW — add_tile, remove_tile
├── columns.rs ← NEW — Column iteration, management
└── state.rs ← NEW — is_*, has_*, count_* queries
```

**Steps:**
1. [ ] Extract `ColumnData` struct → `row/core.rs`
2. [ ] Extract tile operations → `row/tile_ops.rs`
3. [ ] Extract column iteration → `row/columns.rs`
4. [ ] Extract state queries → `row/state.rs`
5. [ ] Update `row/mod.rs` to use submodules

**Verification:**
```bash
cargo check
cargo test layout::
./scripts/verify-golden.sh
```

---

### Phase 5: Create `layout_impl/` Module (8 hours, HIGH RISK)

**The Big One**: Extract 229 methods from `mod.rs` (5353 LOC → ~400 LOC)

**Target:**
```
layout_impl/
├── mod.rs
├── window_ops.rs (~600 LOC)
├── output_ops.rs (~400 LOC)
├── focus.rs (~500 LOC)
├── navigation.rs (~800 LOC)
├── resize.rs (~500 LOC)
├── fullscreen.rs (~400 LOC)
├── row_management.rs (~500 LOC)
├── queries.rs (~400 LOC)
├── interactive_move.rs (~400 LOC)
└── render.rs (~500 LOC)
```

**Method Distribution:**

| File | Methods |
|------|---------|
| `window_ops.rs` | add_window, remove_window, update_window, find_window_*, find_wl_surface_*, descendants_added |
| `output_ops.rs` | add_output, remove_output, update_output_size, add_column_by_idx |
| `focus.rs` | activate_window*, active_output, active_row*, active_monitor*, windows_for_output* |
| `navigation.rs` | move_*, focus_*, scroll_*, all direction-based methods |
| `resize.rs` | set_*_width, set_*_height, interactive_resize_*, reset_* |
| `fullscreen.rs` | set_fullscreen, toggle_fullscreen, set_maximized, toggle_maximized |
| `row_management.rs` | find_row_by_*, ensure_named_row, unname_*, find_workspace_by_* |
| `queries.rs` | is_*, has_*, should_*, popup_target_rect, scroll_amount_to_activate |
| `interactive_move.rs` | interactive_move_*, DnD methods |
| `render.rs` | render_*, refresh, with_windows* |

**Steps:**
1. [ ] Create `layout_impl/mod.rs` with re-exports
2. [ ] Extract one category at a time:
   - [ ] Start with `queries.rs` (safest, just getters)
   - [ ] Then `fullscreen.rs` (self-contained)
   - [ ] Then `resize.rs`
   - [ ] Then `row_management.rs`
   - [ ] Then `focus.rs`
   - [ ] Then `output_ops.rs`
   - [ ] Then `window_ops.rs`
   - [ ] Then `navigation.rs` (largest)
   - [ ] Then `interactive_move.rs`
   - [ ] Finally `render.rs`
3. [ ] Use `impl Layout<W>` blocks in each file

**Pattern:**
```rust
// src/layout/layout_impl/queries.rs
use super::*;

impl<W: LayoutElement> Layout<W> {
    pub fn is_empty(&self) -> bool { ... }
    pub fn has_window(&self, window: &W::Id) -> bool { ... }
    // etc.
}
```

```rust
// src/layout/mod.rs
mod layout_impl;  // Just add this line - impl blocks auto-merge
```

**Verification after EACH file:**
```bash
cargo check
cargo test layout::
./scripts/verify-golden.sh
```

---

### Phase 6: Split `canvas/operations.rs` (3 hours, MEDIUM RISK)

**Current:** 869 LOC

**Target:**
```
canvas/operations/
├── mod.rs — Re-exports
├── window.rs — add/remove window
├── tile.rs — tile manipulation
├── row.rs — row creation/deletion, ensure_row
└── state.rs — state updates
```

**Verification:**
```bash
cargo check
cargo test
```

---

## Summary Table

| Phase | Effort | Risk | Files Changed | LOC Moved |
|-------|--------|------|---------------|-----------|
| 0. Cleanup | 1h | None | Delete 2 | -4000 |
| 1. Create elements/ | 2h | Low | Move 6 | ~1300 |
| 2. Consolidate floating | 4h | Medium | Split 1, delete 1 | ~1450 |
| 3. Split tile | 3h | Medium | Split 1 | ~1470 |
| 4. Split row/mod.rs | 4h | Medium | Extract 4 | ~1200 |
| 5. Create layout_impl/ | 8h | High | Extract 10 | ~5000 |
| 6. Split canvas/ops | 3h | Medium | Split 1 | ~870 |
| **Total** | **~25h** | | **~50 files** | |

## Success Metrics

After all phases:
- [ ] No file > 500 LOC (except tests.rs)
- [ ] Each module has ONE responsibility
- [ ] Hierarchy matches: Layout → Monitor → Canvas → Row → Column → Tile
- [ ] All render elements in `elements/`
- [ ] All Layout methods in `layout_impl/`
- [ ] All tests passing
- [ ] Golden tests passing

---

# 📊 CURRENT STATUS

| Metric | Value |
|--------|-------|
| **Build** | ✅ Compiles |
| **Tests** | 256 passed, 16 failed (94.1%) |
| **Golden Tests** | ❌ Snapshot regressions detected |
| **TODOs in codebase** | 84 total |

---

# 🚨 TEST FAILURES (TEAM_059)

> **Status**: IN PROGRESS  
> **Date**: Nov 28, 2025  
> **Latest Update**: Compilation fixes completed, tests now running

## Recently Fixed (TEAM_059)

1. ✅ `move_window_to_workspace_maximize_and_fullscreen` - Fixed maximize state preservation
2. ✅ `move_to_workspace_by_idx_does_not_leave_empty_workspaces` - Fixed row cleanup/renumbering
3. ✅ **Compilation errors** - Fixed all `workspaces()`/`rows()` method mismatches on Canvas2D vs Layout
4. ✅ **Test compilation** - Fixed `active_workspace()` → `active_row()` calls in tests
5. ✅ **Method naming** - Fixed `move_to_workspace()` → `move_to_row()` in tests

## Current Failing Tests (16 total as of Nov 28, 2025)

### Pattern 1: Floating Window State Issues (6 tests)
**Tests**: 
- `restore_to_floating_persists_across_fullscreen_maximize`
- `unmaximize_during_fullscreen_does_not_float`
- `interactive_move_unfullscreen_to_floating_restores_size`
- `interactive_move_unmaximize_to_floating_restores_size`
- `resize_during_interactive_move_propagates_to_floating`
- `interactive_move_restores_floating_size_when_set_to_floating`

**Issue**: Floating windows incorrectly appearing in tiled space after fullscreen/maximize operations, size preservation issues during interactive move.

### Pattern 2: Output/Row Management Issues (3 tests)
**Tests**: 
- `move_workspace_to_output`
- `removing_all_outputs_preserves_empty_named_workspaces`
- `removing_output_must_keep_empty_focus_on_primary`

**Issue**: Row creation, deletion, and index tracking not working correctly when outputs change.

### Pattern 3: Golden Snapshot Regressions (2 tests)
**Tests**: 
- `golden_anim_expand_to_available`
- `golden_y1_expand_column_to_available_width`

**Issue**: Animation capture and column width expansion issues in golden snapshots.

### Pattern 4: Floating Configure Events (5 tests)
**Tests**: 
- `unfullscreen_to_floating_doesnt_send_extra_configure`
- `unfullscreen_to_same_size_windowed_fullscreen_floating`
- `unmaximize_to_floating_doesnt_send_extra_configure`
- `unmaximize_to_same_size_windowed_fullscreen_floating`
- `target_output_and_rows`

**Issue**: Extra configure events being sent to floating windows during state transitions.

---

# 🔄 CONTINUOUS TEST ITERATION (TEAM_043 → TEAM_044)

> **Goal**: Run all tests iteratively until 100% pass rate
> **Status**: IN PROGRESS

## Fixes Applied (TEAM_043)

1. ✅ **Fixed refresh not calling Row::refresh()** - Windows weren't receiving configure events
2. ✅ **Fixed width parameter ignored in Monitor::add_window()** - Was hardcoded to 1.0
3. ✅ **Added floating space refresh** - Floating windows now get refreshed
4. ✅ **Fixed set_column_width for floating** - Now routes to FloatingSpace
5. ✅ **Fixed floating set_window_width/height** - Uses size() instead of expected_size()

## Fixes Applied (TEAM_044)

6. ✅ **Fixed Layout::update_window missing floating check** - Floating windows now get on_commit called
7. ✅ **Fixed Row::update_window missing serial parameter** - on_commit now called for tiled windows
8. ✅ **Fixed floating window toggle position** - Now sets floating position based on render position like original Workspace
9. ✅ **Fixed floating focus state management** - Added focus_tiling(), focus_floating(), switch_focus_floating_tiling() to Canvas2D

## Known Issues (TEAM_044 → TEAM_045)

### ✅ Floating Animation Regression (Resolved by TEAM_045)
- **Test**: `golden_u4_toggle_floating_back_to_tiled`
- **Previous Issue**: Missing animations when toggling window from floating back to tiled (animations list empty in snapshot).
- **Fix (TEAM_045)**: Start a tile move animation when re-inserting a window from floating back to tiled in `Canvas2D::toggle_floating_window_by_id`, so `Row::snapshot()` records tile edge animations that match the golden baseline.
- **Status**: **Resolved – all golden tests now pass (88/88)**.

## Remaining Test Categories

- **Floating tests**: ~22 failing (size preservation issues - complex expected_size() interactions)
  - ✅ **RESOLVED (TEAM_054)**: `unfocus_preserves_current_size` - Root cause was Canvas2D's `find_wl_surface()` not searching floating space, so `window.on_commit()` was never called for floating windows.
- **Animation tests**: ~10 failing (move animations)
- **Fullscreen tests**: ~5 failing (view offset preservation)
- **Window opening tests**: ~10 failing (workspace targeting)
- **Interactive move tests**: ~8 failing

---

# 🎯 PHASE 1: Config Migration (CURRENT PRIORITY)

> **Goal**: Replace all `workspace` terminology with `row`
> **Decision**: Remove immediately, no deprecation period

## Config Changes Needed

### niri-config/src/ (TEAM_055 - COMPLETE ✅)

- [x] **workspace.rs** → rename to `row.rs`
  - [x] Rename `Workspace` struct to `RowConfig`
  - [x] Rename `WorkspaceName` to `RowName`
  - [x] Update all references

- [x] **lib.rs**
  - [x] Change `workspaces: Vec<Workspace>` to `rows: Vec<RowConfig>`
  - [x] Update `pub use` statements

- [x] **window_rule.rs** (or wherever window rules are)
  - [x] Rename `open-on-workspace` to `open-on-row`

- [x] **animations.rs**
  - [x] Rename `workspace_switch` to `row_switch` (or remove if not needed)

- [x] **layout.rs**
  - [x] Rename `empty_workspace_above_first` to `empty_row_above_first`

### src/layout/ (TEAM_055 - COMPLETE ✅)

- [x] **workspace_types.rs** → rename to `row_types.rs`
  - [x] Rename `WorkspaceId` to `RowId`
  - [x] Rename `WorkspaceAddWindowTarget` to `RowAddWindowTarget`
  - [x] Update all imports across codebase

- [x] **mod.rs**
  - [x] Rename `find_workspace_by_name` to `find_row_by_name`
  - [x] Rename `ensure_named_workspace` to `ensure_named_row`
  - [x] Rename `last_active_workspace_id` to `last_active_row_id`
  - [x] Rename `workspace_id_counter` to `row_id_counter`

### src/handlers/ (TEAM_055 - COMPLETE ✅)

- [x] **xdg_shell.rs**
  - [x] Update `workspace_name` variable to `row_name`
  - [x] Update `InitialConfigureState::Configured` fields

- [x] **compositor.rs**
  - [x] Update `workspace_id` to `row_id`

### Tests (TEAM_055 - COMPLETE ✅)

- [x] **src/tests/window_opening.rs**
  - [x] Update test configs to use `row` syntax
  - [x] Rename test functions if needed

---

# ✅ RESOLVED: Animation System Bug

> **Status**: FIXED by TEAM_056
> **Result**: All 12 animation tests passing

## Root Causes Found

1. **Missing column render offset** in `Row::tiles_with_render_positions()` - Column's move animation offset wasn't included in position calculation
2. **Asymmetric resize handling** in `Row::update_window()` - Only animated columns to the right, not columns to the left

## Fixes Applied

### Bug 1: `src/layout/row/layout.rs`
Added `col.render_offset()` to tile position calculation:
```rust
let col_render_off = col.render_offset();
let tile_pos = Point::from((
    view_off_x + col_x + col_render_off.x + tile_offset.x + tile.render_offset().x,
    y_offset + col_render_off.y + tile_offset.y + tile.render_offset().y,
));
```

### Bug 2: `src/layout/row/mod.rs`
Added symmetric animation for left-side column resize:
```rust
} else {
    // Resizing a column to the left of active
    for col in &mut self.columns[..=col_idx] {
        col.animate_move_from_with_config(-offset, ...);
    }
}
```

## Test Results
- Animation tests: 12/12 passing ✅
- Golden tests: 86/88 passing (remaining 2 unrelated to animation)

---

# 📋 REMAINING TODOs FROM CODEBASE

## Analysis by TEAM_057

**Status**: Easy TODOs completed, complex items documented below  
**Date**: Nov 28, 2025

---

## 🔴 HIGH PRIORITY (Causing Test Failures)

### src/layout/mod.rs - Line 4752
**TODO**: `TEAM_018: implement proper duplicate name checking for canvas rows`

**Status**: ✅ **FIXED by TEAM_057**

**Root Cause Analysis**:
The test failure was caused by TWO separate issues:
1. **Duplicate row names**: Names weren't checked for duplicates across rows
2. **Duplicate row IDs**: Row IDs were colliding across canvases (different monitors)

**Fixes Implemented**:
1. **canvas/navigation.rs**: Added duplicate name checking in `set_row_name()` - if another row has the same name, clear it first (move the name to the new row)
2. **canvas/operations.rs**: Changed row ID stride from +1 to +1000 in `ensure_row()` to prevent ID collisions between canvases

**Test Result**: `move_window_to_workspace_with_different_active_output` now passes

---

## ✅ RESOLVED - Floating Window State Preservation

### src/layout/row/operations/fullscreen.rs & src/layout/floating/operations.rs
**TODO**: `TEAM_059: Fix floating window state preservation during fullscreen/maximize operations`

**Status**: ✅ **COMPLETED - All 28 floating tests passing**

**Root Cause Analysis**:
1. **floating.rs `remove_tile_by_idx`**: Was using `window().size()` instead of `expected_size()`, causing incorrect floating size storage
2. **canvas/operations.rs**: Redundant manual `set_floating_window_size` calls were overwriting the correct value
3. **mod.rs `with_windows_mut`**: Wasn't including floating windows in the iterator
4. **mod.rs `interactive_move_begin/update`**: Didn't check floating space, only rows
5. **mod.rs `remove_window`**: Didn't check floating space before rows

**Fixes Implemented**:
1. **floating.rs**: Restored main branch behavior - use `expected_size()` conditionally
2. **canvas/operations.rs**: Removed redundant `set_floating_window_size` calls in set_fullscreen, toggle_fullscreen, set_maximized, toggle_maximized
3. **mod.rs `with_windows_mut`**: Added floating window iteration for both MonitorSet variants
4. **mod.rs `interactive_move_begin`**: Added floating space check with proper borrow checker handling
5. **mod.rs `interactive_move_update`**: Added floating space support for Starting → Moving transition
6. **mod.rs `remove_window`**: Added floating space check before row checks

**All Tests Now Passing**:
- ✅ All 28 `tests::floating::*` tests
- ✅ `unfullscreen_to_floating_doesnt_send_extra_configure`
- ✅ `unmaximize_to_floating_doesnt_send_extra_configure`
- ✅ `unfullscreen_to_same_size_windowed_fullscreen_floating`
- ✅ `unmaximize_to_same_size_windowed_fullscreen_floating`
- ✅ `resize_during_interactive_move_propagates_to_floating`

**Implemented Features**:
- ✅ `restore_to_floating` flag on Tile for state tracking
- ✅ `floating_window_size` field on Tile for size preservation
- ✅ Row methods return boolean indicating restore-to-floating intent
- ✅ Canvas2D methods handle restore-to-floating flag
- ✅ Interactive move logic preserves and restores floating sizes
- ✅ Monitor::add_tile properly routes to floating space when is_floating=true

---

## �🟡 MEDIUM PRIORITY (Functional Enhancements)

### src/layout/mod.rs - Line 798
**TODO**: `TEAM_024: Implement canvas cleanup logic`

**Status**: ✅ **FIXED by TEAM_057**

**Issue**: When `empty_row_above_first` is enabled and there are exactly 2 empty rows, one needs to be removed.

**Fix**: Implemented logic to find and remove the non-origin row (row != 0) when both rows are empty. The origin row (row 0) is always preserved.

**Tests**: All `ewaf` (empty_row_above_first) tests pass.

### src/layout/mod.rs - Line 1052
**TODO**: `TEAM_023: Implement window height setting on canvas/row`

**Status**: ✅ **FIXED by TEAM_057**

**Fix**: Added `set_window_height()` method to Canvas2D that finds the row containing the window and delegates to Row's existing `set_window_height()` method.

### src/layout/mod.rs - Line 1069
**TODO**: `TEAM_023: Implement proper workspace ID to row mapping`

**Status**: ✅ **FIXED by TEAM_057**

**Fix**: Added `find_row_by_id()` method to Canvas2D that searches all rows for matching workspace ID. Used in `AddWindowTarget::Workspace` handling.

### src/layout/row/operations/move_col.rs - Line 52
**TODO**: `TEAM_006: Animate column movement (port from ScrollingSpace)`

**Status**: ✅ **FIXED by TEAM_057**

**Fix**: Ported animation logic from ScrollingSpace:
- Animate the moved column from its old position
- Animate all columns in between (they shift by the moved column's width)
- Uses `animate_move_from()` on each affected column

### src/layout/row/mod.rs - Line 2002
**TODO**: `Implement proper conversion using working area`

**Status**: ✅ **FIXED by TEAM_057**

**Fix**: Implemented proper coordinate conversion:
- Subtracts working area location from logical position
- Divides by working area size to get 0.0-1.0 fractions
- Handles edge cases with max(size, 1.0)

### src/layout/monitor/render.rs - Line 45
**TODO**: `TEAM_022: Implement proper insert hint rendering with canvas`

**Status**: ✅ **FIXED by TEAM_057**

**Fix**: 
1. Added `insert_hint_area()` method to Row (ported from ScrollingSpace)
2. Updated `update_render_elements()` in monitor/render.rs to:
   - Look up the row by workspace ID
   - Call `insert_hint_area()` to compute the hint rectangle
   - Update `insert_hint_render_loc` and `insert_hint_element`

---

## 🟢 LOW PRIORITY (Documentation)

### src/layout/row_types.rs - Various lines
**TODO**: Documentation comments about removing WorkspaceId

**Status**: ✅ **COMPLETED by TEAM_057**
- These were just documentation notes, not actionable items
- Comments cleaned up to be purely informational

---

## 📊 SUMMARY

**Total TODOs Analyzed**: 9
- ✅ **Completed**: 9 (ALL DONE!)
- 🔴 **High Priority**: 0 
- 🟡 **Medium Priority**: 0

**TEAM_057 completed ALL remaining TODOs!**

**Implementation Summary**:
1. ~~Fix duplicate name checking (test failure)~~ ✅ DONE
2. ~~Implement canvas cleanup logic~~ ✅ DONE
3. ~~Implement workspace ID to row mapping~~ ✅ DONE
4. ~~Design Canvas2D window height API~~ ✅ DONE
5. ~~Port column movement animations~~ ✅ DONE
6. ~~Fix coordinate conversion~~ ✅ DONE
7. ~~Implement insert hint rendering~~ ✅ DONE

---

*Last Updated: TEAM_062 on Nov 29, 2025*

---

# 🔮 FUTURE PHASES (After Phase 1)

## Phase 2: Row System
- Row naming (any row can be named)
- Row lifecycle (creation/deletion rules)
- Global row ID counter
- Active row tracking

## Phase 3: Row Spanning
- `row_span` field on Tile
- Cross-row occupancy tracking
- `increase-row-span` / `decrease-row-span` actions

## Phase 4: Camera System
- Camera struct with (x, y, zoom)
- Auto-zoom for row span
- Zoom gestures (Mod+Scroll)
- Render transform pipeline

## Phase 5: Camera Bookmarks
- Save/restore camera positions
- `Mod+1/2/3` for bookmarks
- Optional row name reference

## Phase 6: Navigation & Polish
- Geometric navigation (find nearest tile)
- Origin-based leading edges
- Spawn direction based on quadrant
- Documentation

---

# 📝 FOLLOW-UP QUESTIONS

## From TEAM_042 Questionnaire

1. **Row 0 naming**: Can row 0 be renamed, or is "origin" special?
   - Decision: Any row can be named ✅

2. **Negative rows**: Rows can go negative (above origin)?
   - Decision: Yes, rows are indexed ..., -2, -1, 0, 1, 2, ... ✅

3. **Window spanning**: How does a window's row assignment work when spanning?
   - Decision: Top-left corner (0,0 point) determines the row ✅

4. **Zoom behavior**: When zoomed out, how does focus work?
   - Open question: Need to define focus behavior at different zoom levels

5. **Config migration**: How to handle users with old `workspace` configs?
   - Decision: Remove immediately, no deprecation ✅

---

# 🗄️ ARCHIVED (Completed Work)

<details>
<summary>Click to expand completed work history</summary>

## Compilation Fixes (TEAM_030-040)
- All MonitorSet::NoOutputs patterns updated
- All method call parens fixed
- All workspace field accesses migrated to canvas
- All Monitor/Row methods implemented
- All type mismatches resolved
- All borrow checker issues fixed

## Core Migration (TEAM_021)
- workspace.rs (1,997 lines) DELETED
- workspace_compat.rs (302 lines) DELETED
- workspace_ops.rs DELETED
- Canvas2D is sole layout system

## Row Implementation (TEAM_036-040)
- `window_under()`, `resize_edges_under()` implemented
- `activate_window()`, `is_urgent()` implemented
- `set_fullscreen()`, `toggle_fullscreen()` implemented
- `set_maximized()`, `toggle_maximized()` implemented
- `configure_new_window()`, `update_window()` implemented
- `toggle_width()`, `toggle_window_width/height()` implemented
- `find_wl_surface()`, `find_wl_surface_mut()` implemented

## Animation System (TEAM_039)
- Move animation creation logic implemented
- Old position calculation fixed
- Delta calculation working
- Animation parameters fixed (0,1,0 → 1,0,0)
- Rendering integration confirmed

## Floating Window System (TEAM_044)
- ✅ Floating toggle position calculation fixed (based on render position)
- ✅ Floating focus state management implemented
- ✅ Golden snapshot system expanded for floating windows
- ❌ **Missing**: Floating-to-tiled animation in toggle_floating_window_by_id
- ❌ **Missing**: Animation capture for golden tests when returning from floating

</details>

---

*Check `phases/` for detailed phase documentation.*
*Check `.questions/` for architecture decisions.*
*Check `.teams/` for team handoff notes.*
*Check `.teams/TEAM_062_monolithic_refactor_plan.md` for detailed refactoring plan.*

---

# 🚀 RECOMMENDED NEXT STEPS (TEAM_062)

## START HERE: Phase 0 - Cleanup

**Immediate action (1 hour, NO RISK):**

```bash
# Delete dead files
rm src/layout/workspace.rs                    # Empty placeholder
mkdir -p src/layout/deprecated
mv src/layout/scrolling.rs src/layout/deprecated/  # Keep for reference

# Verify
cargo check
cargo test layout::
```

---

## Then: Phase 1 - Create `elements/` Module

**Low risk (2 hours):** Group all render elements together.

```bash
# Create elements module
mkdir -p src/layout/elements

# Move files
mv src/layout/closing_window.rs src/layout/elements/
mv src/layout/opening_window.rs src/layout/elements/
mv src/layout/focus_ring.rs src/layout/elements/
mv src/layout/shadow.rs src/layout/elements/
mv src/layout/tab_indicator.rs src/layout/elements/
mv src/layout/insert_hint_element.rs src/layout/elements/insert_hint.rs

# Then create src/layout/elements/mod.rs with re-exports
# Then update all imports across codebase
```

---

## Full Refactoring Roadmap

See the **COMPREHENSIVE MODULE ARCHITECTURE REFACTOR** section above for:
- Complete target architecture diagram
- 6 execution phases with detailed steps
- Verification commands for each phase
- Method distribution tables

| Phase | Effort | Risk | Result |
|-------|--------|------|--------|
| 0. Cleanup | 1h | None | Delete dead code |
| 1. Create elements/ | 2h | Low | Group render elements |
| 2. Consolidate floating | 4h | Medium | Fix conceptual model |
| 3. Split tile | 3h | Medium | tile.rs → tile/ |
| 4. Split row/mod.rs | 4h | Medium | Better organization |
| 5. Create layout_impl/ | 8h | High | mod.rs: 5353 → ~400 LOC |
| 6. Split canvas/ops | 3h | Medium | Better organization |

**Total: ~25 hours of focused work**

---

## Success Criteria

After all phases:
- [ ] No file > 500 LOC (except tests.rs)
- [ ] Hierarchy matches: Layout → Monitor → Canvas → Row → Column → Tile
- [ ] All render elements in `elements/`
- [ ] All Layout methods in `layout_impl/`
- [ ] All tests passing
