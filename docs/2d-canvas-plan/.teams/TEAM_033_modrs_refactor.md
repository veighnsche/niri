# TEAM_033: Layout mod.rs Refactoring

## Team Number: TEAM_033

### Mission
1. Assess and refine the proposed module structure from TODO.md (created by a "junior")
2. Refactor `/src/layout/mod.rs` (4,907 lines) into focused modules
3. Take compilation errors into account during refactoring

### Progress Summary

**Errors Fixed**: 35 errors fixed (75 → 40)

**Key Fixes Made**:
1. Fixed input/mod.rs type mismatches with `active_workspace_ref()` returning Option
2. Added `row()` and `row_mut()` methods to Canvas2D for direct row access
3. Added `into_canvas()` and `append_canvas()` methods to Monitor
4. Fixed multiple borrow checker issues by restructuring loops (find first, then mutate)
5. Updated `rows_mut()` to return tuples like `rows()` for consistency
6. Fixed `workspaces_mut()` to map tuples to just rows
7. Added missing methods: `has_windows()`, `start_close_animation_for_tile()`
8. Fixed `unname_workspace()` to take WorkspaceId and return bool
9. Fixed multiple destructuring issues with row iteration
10. Fixed `remove_output()` to properly transfer canvas between monitors
11. Fixed NoOutputs `add_window` to use proper tile creation and types
12. Fixed scrolling_width wrapping in ColumnWidth::Proportion

### Current State Analysis

**Compilation Status**: 40 errors remaining
- 31 E0308 (type mismatches) - mostly Row return types and iterator yields
- 3 E0308 (if/else incompatible types) - Option wrapping issues
- 5 E0061 (wrong argument count) - method signature mismatches
- 1 E0283 (type annotations needed)

**mod.rs Structure** (4,907 lines):
- Lines 1-298: Module imports, `LayoutElement` trait, render element types
- Lines 300-500: `Layout` struct, `MonitorSet` enum, support types
- Lines 500-660: Small impl blocks for support types
- Lines 660-4885: Main `impl<W: LayoutElement> Layout<W>` (~4,225 lines!)

**Key Types in mod.rs**:
- `Layout<W>` - Main layout manager
- `MonitorSet<W>` - Enum tracking Normal/NoOutputs state
- `InteractiveMoveState<W>`, `InteractiveMoveData<W>` - Move state
- `DndData<W>`, `DndHold<W>`, `DndHoldTarget` - DnD state
- `InteractiveResizeData` - Resize state
- `RemovedTile<W>` - Removed tile wrapper
- `Options` - Layout options
- `ConfigureIntent`, `ActivateWindow`, `HitType`, `SizingMode` - Enums

### Problems with TODO.md Proposed Structure

The TODO.md proposes:
```
src/layout/core/
src/layout/input/  # keyboard.rs, pointer.rs, focus.rs
```

**Issues**:
1. `input/` is wrong scope - input handling is in `src/input/`, not layout
2. `keyboard.rs`, `pointer.rs` don't belong here - layout doesn't handle raw input
3. `core/` is vague and doesn't provide clear separation
4. Proposed structure ignores existing well-organized modules (monitor/, canvas/, row/)

### Refined Module Structure

Instead of the TODO.md proposal, I recommend extracting by **functionality domains**:

```
src/layout/
├── mod.rs                (~500 lines) Layout struct + MonitorSet + core state
├── types/
│   ├── mod.rs            (~200 lines) Re-exports
│   ├── traits.rs         (~200 lines) LayoutElement trait + SizingMode
│   ├── options.rs        (~100 lines) Options struct
│   ├── window_types.rs   (~200 lines) RemovedTile, ActivateWindow, HitType, AddWindowTarget
│   └── interactive.rs    (~150 lines) InteractiveResizeData, InteractiveMoveState, DndData
├── window_ops.rs         (~500 lines) add/remove/find/update window operations
├── focus.rs              (~400 lines) focus_left/right/up/down/column/row operations
├── movement.rs           (~500 lines) move_to_workspace/output/row operations
├── sizing.rs             (~400 lines) toggle_width/height/fullscreen/maximize
├── interactive/
│   ├── mod.rs            (~100 lines) Re-exports
│   ├── move_grab.rs      (~700 lines) interactive_move_* methods
│   ├── resize.rs         (~200 lines) interactive_resize_* methods
│   └── dnd.rs            (~150 lines) dnd_* methods
├── gesture.rs            (~200 lines) *_gesture_* methods
├── animation.rs          (~300 lines) advance_animations, are_animations_ongoing
├── render.rs             (~200 lines) render_*, update_render_elements
├── config.rs             (~200 lines) update_config, update_options
├── monitor/              (existing - well organized)
├── canvas/               (existing)
├── row/                  (existing)
├── column/               (existing)
└── ... (other existing modules)
```

### Implementation Plan

**Phase 1**: Fix compilation errors first
- Some borrow checker issues may resolve after refactoring
- But need compiling baseline before major restructuring

**Phase 2**: Extract types (low risk)
- Move types to types/ module
- Keep re-exports in mod.rs for compatibility

**Phase 3**: Extract operations by domain
- Each domain gets its own file
- Layout delegates to specialized modules

### Recommended Next Steps for TEAM_034

1. **Continue fixing type mismatches** (~31 remaining E0308 errors)
   - Many are Row method return types needing adjustment
   - Iterator yields needing proper type conversion

2. **Fix method argument count issues** (~5 E0061 errors)
   - Check method signatures vs call sites
   - May need wrapper methods or signature updates

3. **Once compiling, focus on modular refactoring**
   - Start with extracting types to `types/` module
   - Then extract operations by domain

### Files Modified by TEAM_033:
- `src/layout/mod.rs` - Multiple fixes for borrow issues and type mismatches
- `src/layout/canvas/mod.rs` - Added `row()`, `row_mut()` methods
- `src/layout/canvas/operations.rs` - Fixed `workspaces_mut()` and iteration
- `src/layout/monitor/mod.rs` - Added `into_canvas()`, `append_canvas()`, fixed `unname_workspace()`
- `src/layout/monitor/gestures.rs` - Fixed borrow issues
- `src/layout/row/mod.rs` - Added `has_windows()`, `start_close_animation_for_tile()`
- `src/input/mod.rs` - Fixed Option handling for `active_workspace_ref()`
- `src/input/spatial_movement_grab.rs` - Fixed output clone

---

## Handoff Checklist
- [ ] Code compiles (`cargo check`) - 40 errors remaining
- [ ] Tests pass (`cargo test`) - Not yet (need compilation first)
- [x] Team file complete

---
*Created by TEAM_033*
