# Global TODO List — 2D Canvas Refactor

> **Check this file first** to see where past teams planned to add features.
> This maintains architectural consistency across teams.

**Last updated**: TEAM_029 (Error Categorization)

---

# 🚨 COMPILATION ERRORS — BATCH FIX GUIDE

> **Total Errors: 142** — Categorized for efficient batch fixing
> Each category can often be fixed in a single pass through the codebase

---

## Category 1: E0026/E0027 — MonitorSet::NoOutputs Pattern (23 errors) ✅ EASY

**Problem**: Pattern uses old `workspaces` field instead of `canvas`

**Fix Pattern**:
```rust
// BEFORE:
MonitorSet::NoOutputs { workspaces } => ...

// AFTER:
MonitorSet::NoOutputs { canvas } => ...
```

**Locations** (src/layout/mod.rs):
- [ ] Line 786 — `workspaces` → `canvas` (also missing `canvas` field)
- [ ] Line 1143 — `workspaces` → `canvas`
- [ ] Line 1206 — `workspaces` → `canvas`
- [ ] Line 1320 — `workspaces` → `canvas` (also missing `canvas` field)
- [ ] Line 1354 — `workspaces` → `canvas` (also missing `canvas` field)
- [ ] Line 1386 — `workspaces` → `canvas` (also missing `canvas` field)
- [ ] Line 1669 — `workspaces` → `canvas` (also missing `canvas` field)
- [ ] Line 1694 — `workspaces` → `canvas` (also missing `canvas` field)
- [ ] Line 2689 — `workspaces` → `canvas`
- [ ] Line 2782 — `workspaces` → `canvas`
- [ ] Line 2948 — `workspaces` → `canvas` (also missing `canvas` field)
- [ ] Line 4370 — `workspaces` → `canvas`
- [ ] Line 4403 — `workspaces` → `canvas`
- [ ] Line 4433 — `workspaces` → `canvas`
- [ ] Line 4563 — `workspaces` → `canvas`
- [ ] Line 4593 — `workspaces` → `canvas`

**Also E0559 (variant has no field named `workspaces`):**
- [ ] Line 667 — Construction with `workspaces: vec![]` → `canvas: ...`
- [ ] Line 836 — Construction with `workspaces` field

---

## Category 2: E0615 — Method Call Missing Parens (14 errors) ✅ EASY

**Problem**: Accessing `active_workspace_idx` as field instead of method

**Fix Pattern**:
```rust
// BEFORE:
mon.active_workspace_idx

// AFTER:
mon.active_workspace_idx()
```

**Locations** (src/layout/mod.rs):
- [ ] Line 823 — `active_workspace_idx` → `active_workspace_idx()`
- [ ] Line 1116 — `active_workspace_idx` → `active_workspace_idx()`
- [ ] Line 1122 — `active_workspace_idx` → `active_workspace_idx()`
- [ ] Line 1123 — `active_workspace_idx` → `active_workspace_idx()` (also being assigned to)
- [ ] Line 1136 — `active_workspace_idx` → `active_workspace_idx()` (also being assigned to)
- [ ] Line 3294 — `active_workspace_idx` → `active_workspace_idx()`
- [ ] Line 3297 — `active_workspace_idx` → `active_workspace_idx()`
- [ ] Line 3388 — `active_workspace_idx` → `active_workspace_idx()`
- [ ] Line 3404 — `active_workspace_idx` → `active_workspace_idx()`
- [ ] Line 3454 — `active_workspace_idx` → `active_workspace_idx()`
- [ ] Line 3460 — `active_workspace_idx` → `active_workspace_idx()`
- [ ] Line 3685 — `active_workspace_idx` → `active_workspace_idx()`
- [ ] Line 4487 — `active_workspace_idx` → `active_workspace_idx()`

**Note**: Lines 1123 and 1136 are assignments (`-= 1` and `= 0`). These need a setter method like `set_active_workspace_idx()` or direct field access refactor.

---

## Category 3: E0609 — No Field `workspaces` (11 errors) ✅ EASY

**Problem**: Accessing `mon.workspaces` which no longer exists

**Fix Pattern**:
```rust
// BEFORE:
mon.workspaces[idx]
mon.workspaces.len()

// AFTER:
mon.canvas.workspaces()[idx]  // or appropriate canvas method
mon.canvas.workspaces().len()
```

**Locations** (src/layout/mod.rs):
- [ ] Line 1480 — `workspaces` field access
- [ ] Line 2600 — `workspaces` field access
- [ ] Line 2656 — `workspaces` field access
- [ ] Line 2816 — `workspaces` field access
- [ ] Line 3285 — `workspaces` field access
- [ ] Line 3446 — `workspaces` field access
- [ ] Line 3682 — `workspaces` field access
- [ ] Line 3710 — `workspaces` field access
- [ ] Line 3733 — `workspaces` field access
- [ ] Line 4142 — `workspaces` field access
- [ ] Line 4633 — `workspaces` field access

---

## Category 4: E0599 — Missing Monitor Methods (10 errors) ⚠️ MEDIUM

**Problem**: Methods that need to be implemented on Monitor or delegated to canvas

**Missing Methods**:
- [ ] `Monitor::has_window()` — Line 2119 (layout/mod.rs)
- [ ] `Monitor::advance_animations()` — Line 2686 (layout/mod.rs)
- [ ] `Monitor::are_animations_ongoing()` — Line 2725 (layout/mod.rs)
- [ ] `Monitor::unname_workspace()` — Line 1211 (layout/mod.rs)
- [ ] `Monitor::stop_workspace_switch()` — Line 1395 (layout/mod.rs)
- [ ] `Monitor::remove_workspace_by_idx()` — Line 3456 (layout/mod.rs)
- [ ] `Monitor::insert_workspace()` — Line 3460 (layout/mod.rs)
- [ ] `Monitor::activate_workspace_with_anim_config()` — Line 2666 (layout/mod.rs)
- [ ] `Layout::active_monitor_mut()` — Line 4213 (layout/mod.rs)

**Implementation Strategy**: These likely delegate to `canvas` methods or need to be added to `monitor/mod.rs`.

---

## Category 5: E0599 — Missing Row Methods (5 errors) ⚠️ MEDIUM

**Problem**: Row methods called with wrong signature or on wrong type

**Issues**:
- [ ] `Row::move_column_to_index()` — Line 1822 (layout/mod.rs)
- [ ] `(i32, &Row)::scrolling_insert_position()` — Lines 3934, 3958 (layout/mod.rs)
- [ ] `(i32, &Row)::id()` — Lines 2661, 3307 (layout/mod.rs)

**Fix**: These are called on tuple `(i32, &Row)` instead of just `Row`. Need to extract the row: `(idx, row).1.method()` or pattern match.

---

## Category 6: E0308 — Type Mismatches (39 errors) ⚠️ MEDIUM-HARD

**Common patterns**:

### Return Type Mismatches (Row methods returning wrong types):
- [ ] Line 2985 — Row method returns `()` not expected type
- [ ] Line 3008 — Row method returns `()` not expected type  
- [ ] Line 3045 — Row method returns `()` not expected type
- [ ] Line 3091 — Row method returns `()` not expected type
- [ ] Line 3161 — Row method returns `()` not expected type
- [ ] Line 3187 — Row method returns `()` not expected type
- [ ] Line 3887 — Row method returns `()` not expected type
- [ ] Line 3900 — Row method returns `()` not expected type
- [ ] Line 3912 — Row method returns `()` not expected type

**Fix Strategy**: Check what `layout/mod.rs` expects these methods to return, then update `Row` method signatures in `row/mod.rs`.

### i32/usize Conversions:
- [ ] Various lines — `.try_into().unwrap()` or `as usize` / `as i32`

### Option/Result Mismatches:
- [ ] Line 3849 — `.cloned()` on `Option<LayoutPart>` (not `Option<&T>`)

---

## Category 7: E0277 — Comparison Type Mismatches (4 errors) ✅ EASY

**Problem**: Comparing `i32` with `&i32` or `usize` with `i32`

**Locations** (src/layout/mod.rs):
- [ ] Line 2887 — `i32 == &i32` comparison
- [ ] Line 2898 — `i32 == &i32` comparison
- [ ] Line 1998 — `usize == i32` comparison

**Fix**: Dereference or convert types: `== *other` or `.try_into().unwrap()`

---

## Category 8: E0061 — Wrong Argument Count (8 errors) ⚠️ MANUAL

**Issues**:
- [ ] Line 1200 — `()` takes 0 args but 1 supplied
- [ ] Line 1383 — `()` takes 0 args but 1 supplied  
- [ ] Line 3098 — takes 1 arg but 0 supplied
- [ ] Line 4166 — takes 5 args but 4 supplied

**Fix Strategy**: Check method signatures and adjust call sites.

---

## Category 9: E0432 — Unresolved Imports (2 errors) ✅ EASY

**Locations**:
- [ ] src/layout/workspace_types.rs — `niri_config::Direction`, `niri_config::SetColumnWidth`
- [ ] src/ipc/server.rs — `niri_ipc::WorkspaceId`

**Fix**: Add correct imports or use existing types.

---

## Category 10: E0499/E0596 — Borrow Checker Issues (2 errors) ⚠️ HARD

**Problem**: Multiple mutable borrows or borrowing from immutable reference

**Locations**:
- [ ] src/layout/mod.rs:1522 — Double mutable borrow of `mon` in loop
- [ ] src/layout/row/mod.rs:797 — Cannot borrow as mutable from `&` reference

**Fix Strategy**: Refactor loop structure or use interior mutability patterns.

---

## Category 11: E0282 — Type Annotations Needed (2 errors) ✅ EASY

**Locations**:
- [ ] src/layout/mod.rs:3309 — Add type annotation `&mut _`
- [ ] src/layout/mod.rs:4063 — Add type annotation `Option<_>`

---

## Recommended Fix Order for Future Teams

1. **TEAM_030**: Categories 1, 2, 3 (Easy batch fixes) — ~50 errors
2. **TEAM_031**: Category 9, 11 (Imports and annotations) — ~4 errors  
3. **TEAM_032**: Category 7 (Type comparisons) — ~4 errors
4. **TEAM_033**: Categories 4, 5 (Missing methods) — ~15 errors
5. **TEAM_034**: Category 6 (Type mismatches) — ~39 errors
6. **TEAM_035**: Categories 8, 10 (Complex fixes) — ~10 errors

---

## src/layout/monitor.rs — 🔄 PHASE 1.5.3 IN PROGRESS

Migration from Workspace to Canvas2D. These TODOs will be resolved in Parts 2-4.

### Migration TODOs (TEAM_010)
- [ ] TODO(TEAM_010): Remove canvas field comment after all methods migrated (`monitor.rs:81`)
- [ ] TODO(TEAM_010): Remove workspace checks from `windows()` and `has_window()` (`monitor.rs:454`)
- [ ] TODO(TEAM_010): Remove workspace operations from mutation methods (`monitor.rs:584`)

### Workspace Cleanup (TEAM_020)
- [ ] TODO(TEAM_020): Eventually remove workspace check entirely (`mod.rs:1171`)
- [ ] TODO(TEAM_020): Eventually remove workspace check entirely (`mod.rs:1424`)
- [ ] TODO(TEAM_020): Eventually remove workspace config entirely (`mod.rs:2916`)
- [ ] TODO(TEAM_020): Eventually remove workspace check entirely (`mod.rs:3498`)
- [ ] TODO(TEAM_020): Eventually remove workspace check entirely (`mod.rs:3523`)
- [ ] TODO(TEAM_020): Eventually remove workspace check entirely (`mod.rs:3577`)
- [ ] TODO(TEAM_020): Eventually remove workspace check entirely (`mod.rs:3602`)

### Canvas Operations (TEAM_024)
- [ ] TODO: TEAM_024: Implement canvas cleanup logic (`mod.rs:761`)
- [ ] TODO: TEAM_024: Get workspace ID from canvas row (`monitor/gestures.rs:138`)
- [ ] TODO: TEAM_024: Set active workspace index in canvas (`monitor/gestures.rs:142`)
- [ ] TODO: TEAM_024: Implement row removal in canvas (`monitor/config.rs:28`)

### Layout Integration (TEAM_023)
- [ ] TODO: TEAM_023: Apply workspace config to canvas rows if needed (`mod.rs:699`)
- [ ] TODO: TEAM_023: Implement window height setting on canvas/row (`mod.rs:993`)
- [ ] TODO: TEAM_023: Implement proper workspace ID to row mapping (`mod.rs:1010`)
- [ ] TODO: TEAM_023: Implement proper row geometry calculation (`monitor/hit_test.rs:22`)
- [ ] TODO: TEAM_023: Implement proper row geometry (`monitor/hit_test.rs:41`)

---

## src/layout/row/ — ✅ PHASE 1.5.1 COMPLETE

Row module is now feature-complete for Phase 1.5.1. All core ScrollingSpace methods have been ported.

### Column Operations (DONE)
- [x] `add_tile`, `add_column`, `remove_column` — basic versions done
- [x] `move_left`, `move_right`, `move_column_to` — basic versions done
- [x] `add_tile_to_column` — add tile to existing column (TEAM_008)
- [x] `add_tile_right_of` — add tile as new column right of window (TEAM_008)
- [x] `activate_column` — activate column with animation (TEAM_008)
- [x] `remove_tile` — remove tile by window ID (TEAM_008)
- [x] `remove_tile_by_idx` — remove tile by column/tile index with animations (TEAM_008)
- [x] `remove_active_column` — remove the active column (TEAM_008)
- [x] `remove_column_by_idx` — remove column with full animation support (TEAM_008)
- [x] `consume_or_expel_window_left` — consume into left column or expel as new column (TEAM_008)
- [x] `consume_or_expel_window_right` — consume into right column or expel as new column (TEAM_008)
- [x] `consume_into_column` — consume first tile from right column into active (TEAM_008)

### Remaining — ⚠️ ANIMATION GAP (See TEAM_009 questionnaire)
- [ ] TODO(TEAM_006): Animate column movement (port from ScrollingSpace) (`row/operations/move_col.rs:48`)
- [ ] TODO(TEAM_006): Animate movement of other columns (port from ScrollingSpace) (`row/operations/add.rs:157`)

### FIXMEs (Lower Priority)
- [ ] FIXME: Smarter height distribution (`resize.rs:111`)
- [ ] FIXME: Compute and use current velocity (`view_offset.rs:235`)
- [ ] FIXME: Tiles can move by X too in centered/resizing layout (`operations/remove.rs:54`)
- [ ] FIXME: Preserve activate_prev_column_on_removal (`operations/remove.rs:204`)

### View Offset / Animation
- [x] TODO(TEAM_007): Port full `animate_view_offset_to_column` logic — DONE
- [x] TODO(TEAM_007): Port `compute_new_view_offset_*` methods — DONE
- [x] TODO(TEAM_007): Port `animate_view_offset_with_config` — DONE
- [x] TODO(TEAM_007): Port gesture handling (`view_offset_gesture_begin`, etc.) — DONE

### Rendering
- [x] TODO(TEAM_007): Port `render_elements` from ScrollingSpace — DONE
- [x] TODO(TEAM_007): Port `columns_in_render_order` — DONE
- [x] TODO(TEAM_007): Port `update_render_elements` — DONE
- [x] `render_above_top_layer` — returns true when fullscreen and view stationary (TEAM_008)

### Interactive Resize
- [x] TODO(TEAM_007): Port `interactive_resize_begin` from ScrollingSpace — DONE
- [x] TODO(TEAM_007): Port `interactive_resize_update` from ScrollingSpace — DONE
- [x] TODO(TEAM_007): Port `interactive_resize_end` from ScrollingSpace — DONE

### Navigation & Movement Methods (TEAM_028)
- [ ] TODO(TEAM_028): implement window expulsion to floating (`row/mod.rs:972`)
- [ ] TODO(TEAM_028): implement window swapping (`row/mod.rs:978`)
- [ ] TODO(TEAM_028): implement tabbed display toggle (`row/mod.rs:984`)
- [ ] TODO(TEAM_028): implement column display setting (`row/mod.rs:990`)
- [ ] TODO(TEAM_028): implement column centering (`row/mod.rs:996`)
- [ ] TODO(TEAM_028): implement visible columns centering (`row/mod.rs:1002`)
- [ ] TODO(TEAM_028): implement width toggle (`row/mod.rs:1008`)
- [ ] TODO(TEAM_028): implement window width toggle (`row/mod.rs:1014`)
- [ ] TODO(TEAM_028): implement window height toggle (`row/mod.rs:1020`)
- [ ] TODO(TEAM_028): implement full width toggle (`row/mod.rs:1026`)
- [ ] TODO(TEAM_028): implement column width setting (`row/mod.rs:1032`)
- [ ] TODO(TEAM_028): implement window width setting (`row/mod.rs:1038`)
- [ ] TODO(TEAM_028): implement insert position calculation (`row/mod.rs:1044`)
- [ ] TODO(TEAM_028): implement unmap snapshot storage (`row/mod.rs:1051`)
- [ ] TODO(TEAM_028): implement unmap snapshot clearing (`row/mod.rs:1057`)
- [ ] TODO(TEAM_028): implement close animation (`row/mod.rs:1063`)

### Row Compatibility Methods (TEAM_024)
- [ ] TODO: TEAM_024: Implement column width expansion if needed (`row/mod.rs:776`)
- [ ] TODO: TEAM_024: Implement fullscreen state if needed (`row/mod.rs:813`)
- [ ] TODO: TEAM_024: Implement fullscreen toggle if needed (`row/mod.rs:817`)
- [ ] TODO: TEAM_024: Implement maximized state if needed (`row/mod.rs:821`)
- [ ] TODO: TEAM_024: Implement maximized toggle if needed (`row/mod.rs:825`)
- [ ] TODO: TEAM_024: Implement window activation if needed (`row/mod.rs:835`)
- [ ] TODO: TEAM_024: Implement open animation if needed (`row/mod.rs:840`)

### Row Interface Implementation (TEAM_022)
- [ ] TEAM_022: implement proper column addition to canvas (`monitor/mod.rs:294`)
- [ ] TEAM_022: implement proper window configuration (`row/mod.rs:701`)
- [ ] TEAM_022: rows should get output from monitor/canvas (`row/mod.rs:851`)
- [ ] TEAM_022: implement active window logic (`row/mod.rs:858`)
- [ ] TEAM_022: implement urgency detection (`row/mod.rs:870`)
- [ ] TEAM_022: implement hit testing (`row/mod.rs:877`)
- [ ] TEAM_022: implement resize edge detection (`row/mod.rs:884`)
- [ ] TEAM_022: implement active tile visual rectangle (`row/mod.rs:891`)
- [ ] TEAM_022: implement proper check (`row/mod.rs:900`)
- [ ] TEAM_022: implement window update (`row/mod.rs:907`)
- [ ] TEAM_022: rows don't have individual layout configs (`row/mod.rs:913`)

### Row Surface Handling (TEAM_025)
- [ ] TEAM_025: implement proper row removal with active row adjustment (`canvas/mod.rs:294`)
- [ ] TEAM_025: implement proper scrolling width resolution (`row/mod.rs:919`)
- [ ] TEAM_025: implement tile creation (`row/mod.rs:926`)
- [ ] TEAM_025: implement descendants handling (`row/mod.rs:932`)
- [ ] TEAM_025: implement surface lookup (`row/mod.rs:938`)
- [ ] TEAM_025: implement mutable surface lookup (`row/mod.rs:945`)
- [ ] TEAM_025: implement popup target rect (`row/mod.rs:952`)
- [ ] TEAM_025: implement activation without raising (`row/mod.rs:959`)
- [ ] TEAM_025: implement IPC layout generation (`row/mod.rs:965`)

### Row & Canvas Operations (TEAM_018)
- [ ] TODO(TEAM_018): implement proper duplicate name checking for canvas rows (`mod.rs:4497`)
- [ ] TODO(TEAM_018): Implement back-and-forth logic (`canvas/navigation.rs:324`)
- [ ] TODO(TEAM_018): Implement previous row tracking (`canvas/navigation.rs:331`)

### Monitor Operations (TEAM_022)
- [ ] TODO(TEAM_022): Implement proper insert hint rendering with canvas (`monitor/render.rs:45`)
- [ ] TODO(TEAM_022): Implement previous row tracking if needed (`monitor/navigation.rs:59`)
- [ ] TODO(TEAM_022): Implement previous row tracking (`monitor/navigation.rs:67`)

### Navigation System (No Team Assigned)
- [ ] TODO: Implement move window between rows in Canvas2D (`monitor/navigation.rs:124`, `monitor/navigation.rs:129`, `monitor/navigation.rs:139`)
- [ ] TODO: Implement move column between rows in Canvas2D (`monitor/navigation.rs:148`, `monitor/navigation.rs:153`, `monitor/navigation.rs:158`)

### Workspace Types (No Team Assigned)
- [ ] TODO: Eventually remove when external systems are updated to use canvas concepts (`workspace_types.rs:10`)
- [ ] TODO: Eventually remove when external systems are updated (`workspace_types.rs:46`)
- [ ] TODO: This should eventually be moved to a more appropriate location (`workspace_types.rs:76`)

---

## src/layout/canvas/mod.rs

Canvas2D depends on Row completion. Additional work needed:

### Window Operations ✅ COMPLETE (TEAM_009)
- [x] `add_tile`, `add_tile_to_row` — done
- [x] `contains`, `find_window` — done
- [x] `add_window` — routes to correct layer (floating or tiled)
- [x] `remove_window` — finds window across all layers
- [x] `toggle_floating_window` — move window between layers
- [x] `toggle_floating_focus` — switch focus between layers

### Canvas Operations (TEAM_019)
- [ ] TODO(TEAM_019): Implement proper active window handling for Row (`canvas/operations.rs:99`)
- [ ] TODO(TEAM_019): Implement proper active window handling for Row (`canvas/operations.rs:114`)
- [ ] TODO(TEAM_019): Implement layout_config for Row (`canvas/operations.rs:185`)
- [ ] TODO(TEAM_019): Implement start_open_animation for Row (`canvas/operations.rs:208`)
- [ ] TODO(TEAM_019): Implement proper centering for tiled windows (`canvas/operations.rs:231`)
- [ ] TODO(TEAM_019): Implement actual column reordering if needed (`canvas/operations.rs:315`)
- [ ] TODO(TEAM_019): Implement actual column reordering if needed (`canvas/operations.rs:331`)

### Window Management (TEAM_020)
- [ ] TODO(TEAM_020): Implement proper window update (`canvas/operations.rs:439`)
- [ ] TODO(TEAM_020): Properly activate in row (`canvas/operations.rs:459`)
- [ ] TODO(TEAM_020): Implement fullscreen setting (`canvas/operations.rs:478`)
- [ ] TODO(TEAM_020): Implement fullscreen toggle (`canvas/operations.rs:483`)
- [ ] TODO(TEAM_020): Implement maximized setting (`canvas/operations.rs:488`)
- [ ] TODO(TEAM_020): Implement maximized toggle (`canvas/operations.rs:493`)
- [ ] TODO(TEAM_020): Implement proper scroll calculation if needed (`canvas/operations.rs:504`)

### Popup Management (TEAM_021)
- [ ] TODO(TEAM_021): Implement proper popup positioning with row/column offsets (`canvas/operations.rs:511`)

### Floating Layer ✅ COMPLETE (TEAM_009)
- [x] Integrate FloatingSpace into Canvas2D
- [x] Add floating layer rendering
- [x] Update animations to include floating

### Camera
- [x] TODO(TEAM_007): Animate camera_y when changing rows — DONE
- [ ] TODO(TEAM_007): Add vertical_view_movement config to niri-config (`canvas/navigation.rs:79`) — Phase 3

### Rendering ✅ COMPLETE
- [x] TODO(TEAM_007): Add `render_elements` method — DONE
- [x] TODO(TEAM_007): Add `update_render_elements` method — DONE
- [x] TEAM_009: Floating layer rendering integrated
- [ ] TODO(TEAM_007): Apply camera offset to render elements (`canvas/render.rs:25`) — Phase 3

### Floating Layer
- [ ] TODO(TEAM_009): Add close animation for tiled windows in rows (`canvas/floating.rs:126`)

### Navigation
- [ ] Navigation methods for canvas system (Phase 3)

---

## How to Use This File

1. **Before starting work**: Check if your feature is already planned here
2. **When adding TODOs**: Use format `// TODO(TEAM_XXX): description`
3. **Before finishing**: Run `grep -rn "TODO(TEAM" src/layout/` and update this file
4. **When completing a TODO**: Mark it `[x]` here and remove from code

---

*Created by TEAM_006*
*Comprehensively updated by TEAM_028 - All missing TODOs from previous teams now documented including wrong syntax TODOs (TEAM_023, TEAM_024, TEAM_022, TEAM_025, and generic TODOs)*
