# Global TODO List — 2D Canvas Refactor

> **Check this file first** before starting work.
> This is the single source of truth for what needs to be done.

**Last updated**: TEAM_063 (Nov 29, 2025)

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
cargo test layout:
./scripts/verify-golden.sh
```

---

### Phase 5: Create `layout_impl/` Module (8 hours, HIGH RISK)

> ⚠️ **HIGH RISK**: This phase touches the core Layout struct with 178 methods.
> Split into 10 sub-phases, each independently verifiable.

**The Big One**: Extract 178 methods from `mod.rs` (5351 LOC → ~400 LOC)

**Prerequisites (must be complete before starting):**
- [x] All tests passing (270/270 ✅)
- [x] Golden tests passing (88/88 ✅)
- [x] Phase 2 complete (FloatingSpace consolidated)
- [x] Phase 3 complete (tile.rs split)
- [x] Phase 4 analyzed (row/mod.rs deferred)

**Target Structure:**
```
layout_impl/
├── mod.rs              — Re-exports only
├── queries.rs          (~15 methods) — is_*, has_*, should_*, getters
├── fullscreen.rs       (~8 methods) — fullscreen/maximize operations
├── resize.rs           (~12 methods) — width/height operations
├── row_management.rs   (~15 methods) — find_row_*, ensure_*, unname_*
├── focus.rs            (~20 methods) — activate_*, active_*, windows_for_*
├── output_ops.rs       (~10 methods) — add/remove output, update_output_size
├── window_ops.rs       (~15 methods) — add/remove/update window, find_*
├── navigation.rs       (~40 methods) — move_*, focus_*, scroll_*
├── interactive_move.rs (~20 methods) — interactive_move_*, DnD
└── render.rs           (~15 methods) — render_*, refresh, with_windows*
```

---

#### Phase 5.0: Setup (15 min, LOW RISK)
**Goal:** Create module structure without moving any code.

```bash
mkdir -p src/layout/layout_impl
touch src/layout/layout_impl/mod.rs
```

Add to `src/layout/mod.rs`:
```rust
mod layout_impl;
```

**Verification:**
```bash
cargo check  # Should compile with empty module
```

---

#### Phase 5.1: Extract `queries.rs` (1 hour, LOW RISK)
**Goal:** Move simple read-only query methods.

**Methods to extract (~15):**
- `is_empty()` — Check if layout has no windows
- `has_window()` — Check if window exists
- `should_trigger_focus_follows_mouse_on()` — FFM check
- `popup_target_rect()` — Get popup rect
- `scroll_amount_to_activate()` — Scroll calculation
- `canvas_snapshot()` — Get snapshot for tests
- All `is_*` and `has_*` methods

**Pattern:**
```rust
// src/layout/layout_impl/queries.rs
use super::*;

impl<W: LayoutElement> Layout<W> {
    pub fn is_empty(&self) -> bool { ... }
    // Move method body here
}
```

**Verification:**
```bash
cargo check && cargo test layout:: && cargo xtask test-all golden
```

---

#### Phase 5.2: Extract `fullscreen.rs` (45 min, LOW RISK)
**Goal:** Move fullscreen/maximize operations.

**Methods to extract (~8):**
- `set_fullscreen()`
- `toggle_fullscreen()`
- `set_maximized()`
- `toggle_maximized()`
- Related helper methods

**Verification:**
```bash
cargo check && cargo test layout:: && cargo xtask test-all golden
```

---

#### Phase 5.3: Extract `resize.rs` (1 hour, MEDIUM RISK)
**Goal:** Move resize operations.

**Methods to extract (~12):**
- `set_column_width()`
- `set_window_width()`
- `set_window_height()`
- `reset_window_height()`
- `expand_column_to_available_width()`
- `toggle_width()`
- `toggle_full_width()`
- `interactive_resize_*()` methods

**Verification:**
```bash
cargo check && cargo test layout:: && cargo xtask test-all golden
```

---

#### Phase 5.4: Extract `row_management.rs` (1 hour, MEDIUM RISK)
**Goal:** Move row/workspace management methods.

**Methods to extract (~15):**
- `find_workspace_by_id()`
- `find_workspace_by_name()`
- `find_row_by_name()`
- `find_workspace_by_ref()`
- `ensure_named_row()`
- `unname_workspace()`
- `unname_workspace_by_ref()`
- `unname_workspace_by_id()`

**Verification:**
```bash
cargo check && cargo test layout:: && cargo xtask test-all golden
```

---

#### Phase 5.5: Extract `focus.rs` (1 hour, MEDIUM RISK)
**Goal:** Move focus and activation methods.

**Methods to extract (~20):**
- `activate_window()`
- `activate_window_without_raising()`
- `active_output()`
- `active_row()`
- `active_row_mut()`
- `active_monitor_mut()`
- `active_monitor_ref()`
- `windows_for_output()`
- `windows_for_output_mut()`
- `with_windows()`
- `with_windows_mut()`

**Verification:**
```bash
cargo check && cargo test layout:: && cargo xtask test-all golden
```

---

#### Phase 5.6: Extract `output_ops.rs` (45 min, MEDIUM RISK)
**Goal:** Move output management methods.

**Methods to extract (~10):**
- `add_output()`
- `remove_output()`
- `update_output_size()`
- `add_column_by_idx()`
- `monitors()`

**Verification:**
```bash
cargo check && cargo test layout:: && cargo xtask test-all golden
```

---

#### Phase 5.7: Extract `window_ops.rs` (1 hour, HIGH RISK)
**Goal:** Move window lifecycle methods.

**Methods to extract (~15):**
- `add_window()`
- `remove_window()`
- `update_window()`
- `descendants_added()`
- `find_window_and_output()`
- `find_window_and_output_mut()`
- `find_wl_surface_*()` methods

**Verification:**
```bash
cargo check && cargo test layout:: && cargo xtask test-all golden
```

---

#### Phase 5.8: Extract `navigation.rs` (1.5 hours, HIGH RISK)
**Goal:** Move navigation methods (largest extraction).

**Methods to extract (~40):**
- `move_left()`, `move_right()`, `move_up()`, `move_down()`
- `move_column_left()`, `move_column_right()`
- `move_to_row()`, `move_to_row_up()`, `move_to_row_down()`
- `move_column_to_row()`
- `focus_left()`, `focus_right()`, `focus_up()`, `focus_down()`
- `focus_row()`, `focus_row_up()`, `focus_row_down()`
- `focus_column_*()` methods
- `scroll_*()` methods
- `consume_*()`, `expel_*()` methods

**Verification:**
```bash
cargo check && cargo test layout:: && cargo xtask test-all golden
```

---

#### Phase 5.9: Extract `interactive_move.rs` (45 min, HIGH RISK)
**Goal:** Move interactive move/DnD methods.

**Methods to extract (~20):**
- `interactive_move_begin()`
- `interactive_move_update()`
- `interactive_move_end()`
- `interactive_move_*()` helpers
- DnD-related methods

**Verification:**
```bash
cargo check && cargo test layout:: && cargo xtask test-all golden
```

---

#### Phase 5.10: Extract `render.rs` (45 min, MEDIUM RISK)
**Goal:** Move render-related methods.

**Methods to extract (~15):**
- `render_*()` methods
- `refresh()`
- `advance_animations()`
- `are_animations_ongoing()`
- `update_render_elements()`

**Verification:**
```bash
cargo check && cargo test layout:: && cargo xtask test-all golden
```

---

### Phase 5 Summary

| Sub-Phase | Effort | Risk | Methods | Checkpoint |
|-----------|--------|------|---------|------------|
| 5.0 Setup | 15min | Low | 0 | cargo check |
| 5.1 queries.rs | 1h | Low | ~15 | Full test suite |
| 5.2 fullscreen.rs | 45min | Low | ~8 | Full test suite |
| 5.3 resize.rs | 1h | Medium | ~12 | Full test suite |
| 5.4 row_management.rs | 1h | Medium | ~15 | Full test suite |
| 5.5 focus.rs | 1h | Medium | ~20 | Full test suite |
| 5.6 output_ops.rs | 45min | Medium | ~10 | Full test suite |
| 5.7 window_ops.rs | 1h | High | ~15 | Full test suite |
| 5.8 navigation.rs | 1.5h | High | ~40 | Full test suite |
| 5.9 interactive_move.rs | 45min | High | ~20 | Full test suite |
| 5.10 render.rs | 45min | Medium | ~15 | Full test suite |
| **Total** | **~10h** | | **~170** | |

**Key Safety Rules:**
1. ✅ Run full test suite after EACH sub-phase
2. ✅ Commit after each successful sub-phase
3. ✅ If tests fail, revert and try smaller extraction
4. ✅ Never move more than 20 methods at once
5. ✅ Keep `mod.rs` compilable at all times

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

| Phase | Effort | Risk | Status | Notes |
|-------|--------|------|--------|-------|
| 0. Cleanup | 1h | None | ✅ **DONE** | TEAM_062 |
| 1. Create elements/ | 2h | Low | ✅ **DONE** | TEAM_062 |
| 2. Consolidate floating | 4h | Medium | ✅ **DONE** | TEAM_063 |
| 3. Split tile | 3h | Medium | ✅ **DONE** | TEAM_063 |
| 4. Split row/mod.rs | 4h | Medium | 📋 **PLANNED** | See Deferred Item 1 |
| 5. Create layout_impl/ | 10h | High | ✅ **DONE** | TEAM_063 + TEAM_064 |
| 6. Split canvas/ops | 3h | Medium | 📋 **PLANNED** | See Deferred Item 2 |
| **Total** | **~27h** | | | |

### Phase 5 Sub-phases

| Sub-Phase | Effort | Risk | Status |
|-----------|--------|------|--------|
| 5.0 Setup | 15min | Low | ✅ DONE |
| 5.1 queries.rs | 1h | Low | ✅ DONE |
| 5.2 fullscreen.rs | 45min | Low | ✅ DONE |
| 5.3 resize.rs | 1h | Medium | ✅ DONE |
| 5.4 row_management.rs | 1h | Medium | ✅ DONE |
| 5.5 focus.rs | 1h | Medium | ✅ DONE |
| 5.6 output_ops.rs | 45min | Medium | ✅ DONE |
| 5.7 window_ops.rs | 1h | High | ✅ DONE |
| 5.8 navigation.rs | 1.5h | High | ✅ DONE |
| 5.9 interactive_move.rs | 45min | High | ✅ DONE (TEAM_064) |
| 5.10 render.rs | 45min | Medium | ✅ DONE (TEAM_064) |

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

> **Last Updated**: TEAM_063 (Nov 29, 2025)

| Metric | Value |
|--------|-------|
| **Build** | ✅ Compiles |
| **Tests** | ✅ 270 passed, 0 failed (100%) |
| **Golden Tests** | ✅ 88 passed (100%) |
| **TODOs in codebase** | ~84 total |

## Recent Completions (TEAM_063)
- ✅ Phase 2: FloatingSpace consolidated into `canvas/floating/`
- ✅ Phase 3: `tile.rs` split into `tile/` module
- 🔄 Phase 4: `row/mod.rs` analyzed, splitting DEFERRED (too risky)

## Ready for Phase 5
All prerequisites met:
- [x] All tests passing
- [x] Golden tests passing
- [x] Phase 2-3 complete
- [x] Phase 4 analyzed

---

# ✅ TEST STATUS

> **Status**: ALL PASSING  
> **Date**: Nov 29, 2025  
> **Latest Update**: TEAM_063 - All 270 tests passing, 88 golden tests passing

## Recently Fixed (TEAM_059, TEAM_063)

1. ✅ `move_window_to_workspace_maximize_and_fullscreen` - Fixed maximize state preservation
2. ✅ `move_to_workspace_by_idx_does_not_leave_empty_workspaces` - Fixed row cleanup/renumbering
3. ✅ **Compilation errors** - Fixed all `workspaces()`/`rows()` method mismatches on Canvas2D vs Layout
4. ✅ **Test compilation** - Fixed `active_workspace()` → `active_row()` calls in tests
5. ✅ **Method naming** - Fixed `move_to_workspace()` → `move_to_row()` in tests

## ✅ All Tests Now Passing (Nov 29, 2025)

All 270 tests and 88 golden tests are now passing. Previous issues have been resolved:

- ✅ Floating window state issues - Fixed by TEAM_059
- ✅ Output/Row management issues - Fixed by TEAM_059
- ✅ Golden snapshot regressions - Fixed by TEAM_045
- ✅ Floating configure events - Fixed by TEAM_054

---

# 🔄 CONTINUOUS TEST ITERATION (TEAM_043 → TEAM_063)

> **Goal**: Run all tests iteratively until 100% pass rate
> **Status**: ✅ COMPLETE - 100% pass rate achieved

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

## ✅ All Test Categories Resolved

- ✅ **Floating tests**: All passing (TEAM_054, TEAM_059)
- ✅ **Animation tests**: All passing (TEAM_045)
- ✅ **Fullscreen tests**: All passing (TEAM_059)
- ✅ **Window opening tests**: All passing (TEAM_059)
- ✅ **Interactive move tests**: All passing (TEAM_059)

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

*Last Updated: TEAM_064 on Nov 29, 2025*

---

# 🔴 DEFERRED ITEMS BREAKDOWN (TEAM_064)

> **Philosophy**: Nothing is "too risky" if broken down enough.
> Each item below is split into atomic, independently-verifiable steps.

---

## 🏗️ DEFERRED ITEM 1: Split `row/mod.rs` (2161 LOC)

> **Previous Status**: "HIGH RISK - DEFERRED"
> **New Status**: Ready for incremental extraction

### Why It Was Deferred
- 80+ methods with complex interdependencies
- Many methods access private fields
- Previous split attempt caused file corruption
- Workspace compatibility layer adds complexity

### The Safe Approach: Extract One Category at a Time

#### Step 1.1: Extract `ColumnData` struct (15 min, NO RISK)
**Goal**: Move the internal `ColumnData` struct to its own file.

```rust
// row/column_data.rs
pub(super) struct ColumnData {
    pub(super) column: Column<W>,
    pub(super) width: ColumnWidth,
    pub(super) is_full_width: bool,
}
```

**Verification**: `cargo check`

#### Step 1.2: Extract State Query Methods (30 min, LOW RISK)
**Goal**: Move read-only `is_*`, `has_*`, `count_*` methods.

**Methods to extract → `row/state.rs`** (~20 methods):
- `is_empty()`, `is_floating()`, `has_window()`
- `column_count()`, `tile_count()`, `active_column_idx()`
- `active_window()`, `active_tile()`, `active_tile_mut()`
- `is_fullscreen()`, `is_maximized()`
- `has_fullscreen()`, `has_maximized()`

**Pattern**:
```rust
// row/state.rs
use super::*;

impl<W: LayoutElement> Row<W> {
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
    // ... move method bodies
}
```

**Verification**: `cargo check && cargo test layout::`

#### Step 1.3: Extract Tile Accessors (30 min, LOW RISK)
**Goal**: Move tile iteration methods.

**Methods to extract → `row/tiles.rs`** (~15 methods):
- `tiles()`, `tiles_mut()`
- `tiles_with_offsets()`, `tiles_with_render_positions()`
- `find_tile()`, `find_tile_mut()`
- `find_wl_surface()`, `find_wl_surface_mut()`
- `window_under()`, `resize_edges_under()`

**Verification**: `cargo check && cargo test layout::`

#### Step 1.4: Extract Column Accessors (20 min, LOW RISK)
**Goal**: Move column iteration methods.

**Methods to extract → `row/columns.rs`** (~10 methods):
- `columns()`, `columns_mut()`
- `column_at()`, `column_at_mut()`
- `active_column()`, `active_column_mut()`
- `column_x()`, `column_width()`

**Verification**: `cargo check && cargo test layout::`

#### Step 1.5: Extract Fullscreen/Maximize (30 min, MEDIUM RISK)
**Goal**: Move fullscreen/maximize operations.

**Methods to extract → `row/fullscreen.rs`** (~10 methods):
- `set_fullscreen()`, `toggle_fullscreen()`
- `set_maximized()`, `toggle_maximized()`
- `unset_fullscreen()`, `unset_maximized()`

**Verification**: `cargo check && cargo test layout:: && cargo xtask test-all golden`

#### Step 1.6: Extract Width/Height Operations (30 min, MEDIUM RISK)
**Goal**: Move resize operations.

**Methods to extract → `row/sizing.rs`** (~15 methods):
- `set_column_width()`, `toggle_width()`, `toggle_full_width()`
- `set_window_width()`, `set_window_height()`
- `toggle_window_width()`, `toggle_window_height()`
- `reset_window_height()`
- `expand_column_to_available_width()`

**Verification**: `cargo check && cargo test layout:: && cargo xtask test-all golden`

#### Step 1.7: Extract Activation Methods (20 min, LOW RISK)
**Goal**: Move focus/activation methods.

**Methods to extract → `row/activation.rs`** (~8 methods):
- `activate_window()`, `activate_column()`
- `activate_prev_column()`, `activate_next_column()`
- `set_active_column_idx()`

**Verification**: `cargo check && cargo test layout::`

#### Step 1.8: Extract DnD/Gesture Methods (20 min, LOW RISK)
**Goal**: Move DnD scroll gesture methods.

**Methods to extract → `row/dnd.rs`** (~5 methods):
- `dnd_scroll_gesture_begin()`, `dnd_scroll_gesture_end()`
- `dnd_scroll_gesture_scroll()`
- `view_offset_gesture_end()`

**Verification**: `cargo check && cargo test layout::`

### Summary: row/mod.rs Split

| Step | File | Methods | Risk | Time |
|------|------|---------|------|------|
| 1.1 | column_data.rs | 1 struct | None | 15min |
| 1.2 | state.rs | ~20 | Low | 30min |
| 1.3 | tiles.rs | ~15 | Low | 30min |
| 1.4 | columns.rs | ~10 | Low | 20min |
| 1.5 | fullscreen.rs | ~10 | Medium | 30min |
| 1.6 | sizing.rs | ~15 | Medium | 30min |
| 1.7 | activation.rs | ~8 | Low | 20min |
| 1.8 | dnd.rs | ~5 | Low | 20min |
| **Total** | | **~84** | | **~3h** |

**Target**: `row/mod.rs` from 2161 LOC → ~600 LOC

---

## 🏗️ DEFERRED ITEM 2: Split `canvas/operations.rs` (869 LOC)

> **Previous Status**: "Phase 6 - After Phase 5"
> **New Status**: Ready for incremental extraction

### Step 2.1: Extract Row Management (30 min, LOW RISK)
**Methods → `canvas/operations/row.rs`** (~200 LOC):
- `ensure_row()`, `row()`, `row_mut()`
- `rows()`, `rows_mut()`
- `cleanup_empty_rows()`, `renumber_rows()`
- `move_row_up()`, `move_row_down()`

**Verification**: `cargo check && cargo test layout::`

### Step 2.2: Extract Window Operations (30 min, LOW RISK)
**Methods → `canvas/operations/window.rs`** (~200 LOC):
- `add_window()`, `remove_window()`
- `find_window()`, `find_window_mut()`
- `has_window()`, `window_count()`

**Verification**: `cargo check && cargo test layout::`

### Step 2.3: Extract Tile Operations (30 min, LOW RISK)
**Methods → `canvas/operations/tile.rs`** (~200 LOC):
- `add_tile()`, `remove_tile()`
- `move_tile_to_row()`

**Verification**: `cargo check && cargo test layout::`

### Step 2.4: Extract State Updates (20 min, LOW RISK)
**Methods → `canvas/operations/state.rs`** (~150 LOC):
- `update_config()`, `update_shaders()`
- `advance_animations()`, `are_animations_ongoing()`

**Verification**: `cargo check && cargo test layout::`

### Summary: canvas/operations.rs Split

| Step | File | LOC | Risk | Time |
|------|------|-----|------|------|
| 2.1 | row.rs | ~200 | Low | 30min |
| 2.2 | window.rs | ~200 | Low | 30min |
| 2.3 | tile.rs | ~200 | Low | 30min |
| 2.4 | state.rs | ~150 | Low | 20min |
| **Total** | | **~750** | | **~2h** |

**Target**: `canvas/operations.rs` from 869 LOC → ~120 LOC (re-exports only)

---

## 🎥 DEFERRED ITEM 3: Camera Zoom System

> **Previous Status**: "Phase 4 - THE DIFFERENTIATOR"
> **New Status**: Ready for incremental implementation

### Why It's Critical
Without zoom, Canvas2D is functionally identical to workspaces!

### Step 3.1: Add Camera Struct (30 min, LOW RISK)
**Goal**: Create Camera struct with x, y, zoom.

```rust
// canvas/camera.rs
pub struct Camera {
    x: AnimatedValue,
    y: AnimatedValue,
    zoom: AnimatedValue,  // 1.0 = normal, 0.5 = see 2x area
}

impl Camera {
    pub fn new(clock: Clock) -> Self {
        Self {
            x: AnimatedValue::new(clock.clone(), 0.0, 0.0, None),
            y: AnimatedValue::new(clock.clone(), 0.0, 0.0, None),
            zoom: AnimatedValue::new(clock, 1.0, 1.0, None),
        }
    }
    
    pub fn x(&self) -> f64 { self.x.value() }
    pub fn y(&self) -> f64 { self.y.value() }
    pub fn zoom(&self) -> f64 { self.zoom.value() }
    
    pub fn set_zoom(&mut self, zoom: f64, config: Option<Animation>) {
        self.zoom.set_target(zoom.clamp(0.1, 2.0), config);
    }
}
```

**Verification**: `cargo check`

### Step 3.2: Integrate Camera into Canvas2D (30 min, LOW RISK)
**Goal**: Replace separate `camera_x`, `camera_y` with Camera struct.

```rust
// canvas/mod.rs
pub struct Canvas2D<W> {
    // Remove:
    // camera_x: AnimatedValue,
    // camera_y: AnimatedValue,
    
    // Add:
    camera: Camera,
    // ...
}
```

**Verification**: `cargo check && cargo test layout::`

### Step 3.3: Add visible_area() Method (20 min, LOW RISK)
**Goal**: Calculate what area of canvas is visible at current zoom.

```rust
impl<W: LayoutElement> Canvas2D<W> {
    pub fn visible_area(&self) -> Rectangle<f64, Logical> {
        let zoom = self.camera.zoom();
        let w = self.view_size.w / zoom;
        let h = self.view_size.h / zoom;
        Rectangle::from_loc_and_size(
            (self.camera.x() - w / 2.0, self.camera.y() - h / 2.0),
            (w, h)
        )
    }
}
```

**Verification**: `cargo check`

### Step 3.4: Add visible_rows() Method (30 min, LOW RISK)
**Goal**: Determine which rows are visible at current zoom.

```rust
impl<W: LayoutElement> Canvas2D<W> {
    pub fn visible_rows(&self) -> impl Iterator<Item = (i32, &Row<W>)> {
        let visible = self.visible_area();
        self.rows().filter(move |(idx, row)| {
            let row_y = row.y_offset();
            let row_h = row.row_height();
            // Row intersects visible area
            row_y < visible.loc.y + visible.size.h &&
            row_y + row_h > visible.loc.y
        })
    }
}
```

**Verification**: `cargo check && cargo test layout::`

### Step 3.5: Add Zoom Rendering (1 hour, MEDIUM RISK)
**Goal**: Scale all render elements by zoom factor.

**Changes needed in `canvas/render.rs`**:
```rust
impl<W: LayoutElement> Canvas2D<W> {
    pub fn render_elements<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        // ...
    ) -> impl Iterator<Item = ...> {
        let zoom = self.camera.zoom();
        let scale = Scale::from(zoom);
        
        // Only render visible rows
        self.visible_rows().flat_map(|(_, row)| {
            row.render_elements(renderer, ...)
                .map(move |elem| {
                    // Scale element by zoom
                    RescaleRenderElement::from_element(elem, ..., zoom)
                })
        })
    }
}
```

**Verification**: `cargo check && cargo test layout:: && cargo xtask test-all golden`

### Step 3.6: Add Input Transform (30 min, MEDIUM RISK)
**Goal**: Convert screen coordinates to canvas coordinates at any zoom.

```rust
impl<W: LayoutElement> Canvas2D<W> {
    /// Convert screen position to canvas position
    pub fn screen_to_canvas(&self, screen_pos: Point<f64, Logical>) -> Point<f64, Logical> {
        let zoom = self.camera.zoom();
        let visible = self.visible_area();
        Point::from((
            visible.loc.x + screen_pos.x / zoom,
            visible.loc.y + screen_pos.y / zoom,
        ))
    }
    
    /// Convert canvas position to screen position
    pub fn canvas_to_screen(&self, canvas_pos: Point<f64, Logical>) -> Point<f64, Logical> {
        let zoom = self.camera.zoom();
        let visible = self.visible_area();
        Point::from((
            (canvas_pos.x - visible.loc.x) * zoom,
            (canvas_pos.y - visible.loc.y) * zoom,
        ))
    }
}
```

**Verification**: `cargo check && cargo test layout::`

### Step 3.7: Add Zoom Actions (30 min, LOW RISK)
**Goal**: Add zoom control methods.

```rust
impl<W: LayoutElement> Canvas2D<W> {
    pub fn zoom_in(&mut self, config: Option<Animation>) {
        let current = self.camera.zoom();
        self.camera.set_zoom(current * 1.25, config);
    }
    
    pub fn zoom_out(&mut self, config: Option<Animation>) {
        let current = self.camera.zoom();
        self.camera.set_zoom(current / 1.25, config);
    }
    
    pub fn zoom_reset(&mut self, config: Option<Animation>) {
        self.camera.set_zoom(1.0, config);
    }
    
    pub fn zoom_to_fit_row(&mut self, row_idx: i32, config: Option<Animation>) {
        if let Some(row) = self.row(row_idx) {
            let row_height = row.row_height();
            let zoom = self.view_size.h / row_height;
            self.camera.set_zoom(zoom.min(1.0), config);
        }
    }
}
```

**Verification**: `cargo check && cargo test layout::`

### Step 3.8: Add Layout Methods (20 min, LOW RISK)
**Goal**: Expose zoom controls through Layout.

```rust
// layout_impl/navigation.rs (add)
impl<W: LayoutElement> Layout<W> {
    pub fn zoom_in(&mut self) {
        if let Some(mon) = self.active_monitor() {
            mon.canvas_mut().zoom_in(self.options.animations.camera_zoom.0);
        }
    }
    
    pub fn zoom_out(&mut self) { ... }
    pub fn zoom_reset(&mut self) { ... }
}
```

**Verification**: `cargo check && cargo test layout::`

### Step 3.9: Add Config Options (20 min, LOW RISK)
**Goal**: Add animation config for camera zoom.

```rust
// niri-config/src/animations.rs
pub struct Animations {
    // ... existing
    pub camera_zoom: Animation,
    pub camera_movement: Animation,
}
```

**Verification**: `cargo check`

### Step 3.10: Add Keybindings (30 min, LOW RISK)
**Goal**: Wire up zoom keybindings.

```rust
// src/input/mod.rs (add to handle_action)
Action::ZoomIn => self.layout.zoom_in(),
Action::ZoomOut => self.layout.zoom_out(),
Action::ZoomReset => self.layout.zoom_reset(),
```

**Verification**: Manual testing

### Summary: Camera Zoom System

| Step | Description | Risk | Time |
|------|-------------|------|------|
| 3.1 | Camera struct | Low | 30min |
| 3.2 | Integrate into Canvas2D | Low | 30min |
| 3.3 | visible_area() | Low | 20min |
| 3.4 | visible_rows() | Low | 30min |
| 3.5 | Zoom rendering | Medium | 1h |
| 3.6 | Input transform | Medium | 30min |
| 3.7 | Zoom actions | Low | 30min |
| 3.8 | Layout methods | Low | 20min |
| 3.9 | Config options | Low | 20min |
| 3.10 | Keybindings | Low | 30min |
| **Total** | | | **~5h** |

---

## 📚 DEFERRED ITEM 4: Camera Bookmarks

> **Previous Status**: "Phase 5 - Replaces workspace switching"
> **New Status**: Ready after Camera Zoom (Item 3)

### Step 4.1: Create CameraBookmark Struct (15 min, LOW RISK)
```rust
// canvas/bookmark.rs
#[derive(Debug, Clone)]
pub struct CameraBookmark {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
    pub name: Option<String>,
}

impl CameraBookmark {
    pub fn from_camera(camera: &Camera, name: Option<String>) -> Self {
        Self {
            x: camera.x(),
            y: camera.y(),
            zoom: camera.zoom(),
            name,
        }
    }
}
```

**Verification**: `cargo check`

### Step 4.2: Add Bookmark Storage to Canvas2D (20 min, LOW RISK)
```rust
// canvas/mod.rs
pub struct Canvas2D<W> {
    // ... existing
    bookmarks: [Option<CameraBookmark>; 10],  // 10 bookmark slots
}
```

**Verification**: `cargo check`

### Step 4.3: Implement save_bookmark() (20 min, LOW RISK)
```rust
impl<W: LayoutElement> Canvas2D<W> {
    pub fn save_bookmark(&mut self, slot: usize) {
        if slot < 10 {
            self.bookmarks[slot] = Some(CameraBookmark::from_camera(&self.camera, None));
        }
    }
}
```

**Verification**: `cargo check`

### Step 4.4: Implement goto_bookmark() (30 min, LOW RISK)
```rust
impl<W: LayoutElement> Canvas2D<W> {
    pub fn goto_bookmark(&mut self, slot: usize, config: Option<Animation>) {
        if let Some(bookmark) = &self.bookmarks[slot] {
            self.camera.set_x(bookmark.x, config.clone());
            self.camera.set_y(bookmark.y, config.clone());
            self.camera.set_zoom(bookmark.zoom, config);
        }
    }
}
```

**Verification**: `cargo check && cargo test layout::`

### Step 4.5: Add Layout Methods (20 min, LOW RISK)
```rust
// layout_impl/navigation.rs
impl<W: LayoutElement> Layout<W> {
    pub fn save_bookmark(&mut self, slot: usize) {
        if let Some(mon) = self.active_monitor() {
            mon.canvas_mut().save_bookmark(slot);
        }
    }
    
    pub fn goto_bookmark(&mut self, slot: usize) {
        if let Some(mon) = self.active_monitor() {
            mon.canvas_mut().goto_bookmark(slot, self.options.animations.camera_movement.0);
        }
    }
}
```

**Verification**: `cargo check`

### Step 4.6: Add Keybindings (30 min, LOW RISK)
```rust
// src/input/mod.rs
Action::SaveBookmark(slot) => self.layout.save_bookmark(slot),
Action::GotoBookmark(slot) => self.layout.goto_bookmark(slot),
```

**Verification**: Manual testing

### Step 4.7: Add IPC Commands (30 min, LOW RISK)
```rust
// niri-ipc/src/lib.rs
pub enum Request {
    // ... existing
    SaveBookmark { slot: u8 },
    GotoBookmark { slot: u8 },
    ListBookmarks,
}
```

**Verification**: `cargo check`

### Summary: Camera Bookmarks

| Step | Description | Risk | Time |
|------|-------------|------|------|
| 4.1 | CameraBookmark struct | Low | 15min |
| 4.2 | Storage in Canvas2D | Low | 20min |
| 4.3 | save_bookmark() | Low | 20min |
| 4.4 | goto_bookmark() | Low | 30min |
| 4.5 | Layout methods | Low | 20min |
| 4.6 | Keybindings | Low | 30min |
| 4.7 | IPC commands | Low | 30min |
| **Total** | | | **~3h** |

---

## 📐 DEFERRED ITEM 5: Row Spanning

> **Previous Status**: "Phase 3 - Advanced feature"
> **New Status**: Ready after Camera Zoom (Item 3)

### Step 5.1: Add row_span Field to Tile (15 min, LOW RISK)
```rust
// tile/mod.rs
pub struct Tile<W> {
    // ... existing
    pub row_span: u8,  // 1 = normal, 2+ = spans multiple rows
}
```

**Verification**: `cargo check`

### Step 5.2: Update Tile Height Calculation (30 min, MEDIUM RISK)
```rust
impl<W: LayoutElement> Tile<W> {
    pub fn effective_height(&self, row_height: f64) -> f64 {
        row_height * self.row_span as f64
    }
}
```

**Verification**: `cargo check && cargo test layout::`

### Step 5.3: Track Cross-Row Occupancy (45 min, MEDIUM RISK)
```rust
// canvas/mod.rs
impl<W: LayoutElement> Canvas2D<W> {
    /// Returns rows that a spanning tile occupies
    fn rows_occupied_by_tile(&self, row_idx: i32, tile: &Tile<W>) -> Range<i32> {
        row_idx..(row_idx + tile.row_span as i32)
    }
    
    /// Check if position is blocked by a spanning tile from above
    fn is_blocked_by_spanning_tile(&self, row_idx: i32, col_idx: usize) -> bool {
        // Check rows above for tiles that span into this row
        for check_row in (row_idx - 10)..row_idx {
            if let Some(row) = self.row(check_row) {
                // Check if any tile in that row spans into row_idx
                // ...
            }
        }
        false
    }
}
```

**Verification**: `cargo check && cargo test layout::`

### Step 5.4: Update Navigation for Spanning (30 min, MEDIUM RISK)
```rust
// Navigation should skip over spanned positions
impl<W: LayoutElement> Canvas2D<W> {
    pub fn focus_down(&mut self) {
        let current_row = self.active_row_idx();
        let mut target_row = current_row + 1;
        
        // Skip rows that are occupied by spanning tiles
        while self.is_blocked_by_spanning_tile(target_row, self.active_column_idx()) {
            target_row += 1;
        }
        
        self.focus_row(target_row);
    }
}
```

**Verification**: `cargo check && cargo test layout::`

### Step 5.5: Add Row Span Actions (20 min, LOW RISK)
```rust
impl<W: LayoutElement> Layout<W> {
    pub fn increase_row_span(&mut self) {
        // Increase row_span of active tile
    }
    
    pub fn decrease_row_span(&mut self) {
        // Decrease row_span of active tile (min 1)
    }
    
    pub fn set_row_span(&mut self, span: u8) {
        // Set specific row_span
    }
}
```

**Verification**: `cargo check`

### Summary: Row Spanning

| Step | Description | Risk | Time |
|------|-------------|------|------|
| 5.1 | row_span field | Low | 15min |
| 5.2 | Height calculation | Medium | 30min |
| 5.3 | Cross-row occupancy | Medium | 45min |
| 5.4 | Navigation update | Medium | 30min |
| 5.5 | Row span actions | Low | 20min |
| **Total** | | | **~2.5h** |

---

## 📡 DEFERRED ITEM 6: IPC/Protocol Migration

> **Previous Status**: "DEFERRED until zoom/bookmarks exist"
> **New Status**: Ready after Items 3 & 4

### Step 6.1: Rename IPC Events (30 min, LOW RISK)
```rust
// niri-ipc/src/lib.rs
pub enum Event {
    // Old:
    // WorkspacesChanged,
    // WorkspaceActivated { ... },
    
    // New:
    RowsChanged,
    RowActivated { output: String, row_idx: i32 },
    CameraChanged { output: String, x: f64, y: f64, zoom: f64 },
    BookmarkSaved { output: String, slot: u8 },
}
```

**Verification**: `cargo check`

### Step 6.2: Rename IPC State Structures (30 min, LOW RISK)
```rust
// niri-ipc/src/state.rs
pub struct RowState {
    pub idx: i32,
    pub name: Option<String>,
    pub is_active: bool,
    pub window_count: usize,
}

pub struct CameraState {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

pub struct OutputState {
    pub name: String,
    pub rows: Vec<RowState>,
    pub camera: CameraState,
    pub bookmarks: Vec<Option<CameraBookmark>>,
}
```

**Verification**: `cargo check`

### Step 6.3: Update IPC Handlers (45 min, MEDIUM RISK)
Update `src/niri.rs` to emit new events.

**Verification**: `cargo check && cargo test`

### Step 6.4: Update niri-msg (30 min, LOW RISK)
Update CLI to use new terminology.

**Verification**: Manual testing

### Summary: IPC/Protocol Migration

| Step | Description | Risk | Time |
|------|-------------|------|------|
| 6.1 | Rename events | Low | 30min |
| 6.2 | Rename state structs | Low | 30min |
| 6.3 | Update handlers | Medium | 45min |
| 6.4 | Update niri-msg | Low | 30min |
| **Total** | | | **~2.5h** |

---

## 📊 COMPLETE DEFERRED ITEMS SUMMARY

| Item | Description | Steps | Total Time | Dependencies |
|------|-------------|-------|------------|--------------|
| **1** | Split row/mod.rs | 8 | ~3h | None |
| **2** | Split canvas/operations.rs | 4 | ~2h | None |
| **3** | Camera Zoom System | 10 | ~5h | None |
| **4** | Camera Bookmarks | 7 | ~3h | Item 3 |
| **5** | Row Spanning | 5 | ~2.5h | Item 3 |
| **6** | IPC Migration | 4 | ~2.5h | Items 3 & 4 |
| **TOTAL** | | **38 steps** | **~18h** | |

### Recommended Execution Order

1. **Items 1 & 2** (parallel, no dependencies) - Code organization
2. **Item 3** (Camera Zoom) - THE differentiator
3. **Item 4** (Bookmarks) - Replaces workspace switching
4. **Item 5** (Row Spanning) - Advanced feature
5. **Item 6** (IPC) - External API update

### Success Criteria

After all items complete:
- [ ] `row/mod.rs` < 700 LOC
- [ ] `canvas/operations.rs` < 200 LOC
- [ ] Camera zoom works (0.1x to 2.0x)
- [ ] 10 camera bookmarks per output
- [ ] Row spanning (1-4 rows)
- [ ] IPC uses row/camera terminology
- [ ] All tests passing
- [ ] Golden tests passing

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
