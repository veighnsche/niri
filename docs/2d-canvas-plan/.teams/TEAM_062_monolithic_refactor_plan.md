# TEAM_062 — Comprehensive Module Architecture Refactor

## Mission
Complete architectural analysis of `src/layout/` module structure and create a comprehensive refactoring plan that:
1. Fixes monolithic files (>1000 LOC)
2. Corrects conceptual hierarchy mismatches
3. Groups related code together (render elements)
4. Removes dead code

## Date
2025-11-29

---

## Key Findings

### LOC Growth Since TEAM_038 Analysis (Nov 27 → Nov 29)

| File | Was | Now | Growth | Status |
|------|-----|-----|--------|--------|
| `src/niri.rs` | 5141 | 6603 | +1462 | 🔴 Growing |
| `src/layout/mod.rs` | 3861 | 5353 | +1492 | 🔴 Growing FAST |
| `src/input/mod.rs` | 4302 | 5109 | +807 | 🔴 Growing |
| `src/layout/scrolling.rs` | 3000 | 3990 | +990 | ⚫ Skip (deprecated) |
| `src/backend/tty.rs` | 2804 | 3465 | +661 | 🟡 Stable |
| `niri-config/src/lib.rs` | 2163 | 2327 | +164 | 🟡 Stable |
| `src/layout/row/mod.rs` | 963 | 2161 | +1198 | 🔴 Growing FAST |

### Critical Insight

**`src/layout/mod.rs` and `src/layout/row/mod.rs` are growing rapidly** because they're at the center of Canvas2D work. Every feature addition dumps code into these files. This is unsustainable.

---

## Analysis Method

1. Counted methods with: `grep -n "^    pub fn\|^    fn" <file> | wc -l`
2. Identified method categories by name patterns
3. Compared against well-structured existing modules (`column/`, `monitor/`, `canvas/`)

### Well-Structured Module Patterns

```
column/
├── core.rs      — Core struct, basic state
├── layout.rs    — Layout calculations
├── operations.rs — State-changing operations
├── render.rs    — Render element generation
├── sizing/      — Size calculation submodule
└── tests.rs     — Unit tests

monitor/
├── mod.rs       — Core Monitor struct
├── config.rs    — Configuration handling
├── gestures.rs  — Gesture processing
├── hit_test.rs  — Hit testing
├── navigation.rs— Navigation methods
├── render.rs    — Rendering
└── types.rs     — Type definitions

canvas/
├── mod.rs       — Core Canvas2D struct
├── floating.rs  — Floating window handling
├── navigation.rs— Navigation methods
├── operations.rs— State operations
└── render.rs    — Rendering
```

**Pattern**: Split by **responsibility/behavior**, not by arbitrary size.

---

## Detailed Refactoring Plans

### Priority 1: src/layout/mod.rs (5353 LOC)

**229 methods on `Layout<W>`** — needs major split.

#### Method Categorization

| Category | Method Count | Target File |
|----------|--------------|-------------|
| Window ops | ~30 | `window_ops.rs` |
| Output ops | ~15 | `output_ops.rs` |
| Focus/activation | ~25 | `focus.rs` |
| Navigation | ~40 | `navigation.rs` |
| Resize | ~20 | `resize.rs` |
| Fullscreen/maximize | ~15 | `fullscreen.rs` |
| Row management | ~20 | `row_management.rs` |
| State queries | ~30 | `queries.rs` |
| Rendering | ~20 | `render.rs` |
| Interactive move | ~15 | `interactive_move.rs` |
| Core/misc | ~19 | `mod.rs` (keep) |

#### Target Structure

```
src/layout/
├── mod.rs (~500 LOC)
│   - Layout struct definition
│   - MonitorSet enum
│   - Options struct
│   - Types (SizeFrac, SizingMode, etc.)
│   - LayoutElement trait
│   - Core impl block (new, with_options)
│   - Module re-exports
│
├── window_ops.rs (~600 LOC)
│   - add_window(), remove_window(), update_window()
│   - find_window_*(), find_wl_surface_*()
│   - descendants_added()
│
├── output_ops.rs (~400 LOC)
│   - add_output(), remove_output()
│   - update_output_size()
│   - Output management
│
├── focus.rs (~500 LOC)
│   - activate_window(), activate_window_without_raising()
│   - active_output(), active_row(), active_row_mut()
│   - active_monitor(), active_monitor_mut()
│
├── navigation.rs (~800 LOC)
│   - move_*(), focus_*() methods
│   - Focus/move direction handling
│
├── resize.rs (~500 LOC)
│   - set_window_width/height()
│   - interactive_resize_*()
│   - Size adjustment methods
│
├── fullscreen.rs (~400 LOC)
│   - set_fullscreen(), toggle_fullscreen()
│   - set_maximized(), toggle_maximized()
│
├── row_management.rs (~500 LOC)
│   - find_row_by_*(), ensure_named_row()
│   - unname_*(), row lifecycle
│
├── queries.rs (~400 LOC)
│   - is_*(), has_*(), should_*()
│   - State inspection methods
│
├── render.rs (~500 LOC)
│   - render_*(), refresh()
│   - Render element generation
│
└── interactive_move.rs (~400 LOC)
    - interactive_move_*()
    - DnD handling
```

### Priority 2: src/layout/row/mod.rs (2161 LOC)

Already partially split, but `mod.rs` is still huge.

#### Current Submodules
- gesture.rs (445 LOC)
- layout.rs (~100 LOC)
- navigation.rs (~200 LOC)
- render.rs (~200 LOC)
- resize.rs (~150 LOC)
- view_offset.rs (~300 LOC)
- operations/ (subdirectory)

#### What's Still in mod.rs
- `ColumnData` struct
- Row struct definition
- ~100+ methods on Row

#### Target Additions

```
src/layout/row/
├── mod.rs (~400 LOC) — Row struct, exports, core impl
├── core.rs (~300 LOC) — ColumnData, internal state
├── tile_ops.rs (~400 LOC) — add_tile, remove_tile, tile manipulation
├── columns.rs (~300 LOC) — Column management, iteration
├── state.rs (~300 LOC) — State queries (has_*, is_*, count_*)
├── (keep existing submodules)
└── operations/
```

---

## Refactoring Strategy

### Approach: Incremental Extraction

1. **Don't refactor all at once** — too risky
2. **Extract ONE category at a time**
3. **Keep `mod.rs` as the entry point** — re-export from submodules
4. **Use `impl` blocks in submodules** — Rust allows `impl Foo` in multiple files

### Example: Extracting window_ops.rs

```rust
// src/layout/window_ops.rs
use super::*;  // Import types from mod.rs

impl<W: LayoutElement> Layout<W> {
    pub fn add_window(&mut self, ...) { ... }
    pub fn remove_window(&mut self, ...) { ... }
    // etc.
}
```

```rust
// src/layout/mod.rs
mod window_ops;  // Pull in the impl block
// No need to re-export since methods are on Layout<W>
```

### Testing Strategy

After each extraction:
1. `cargo check` — Must compile
2. `cargo test layout::` — Layout tests must pass
3. `cargo test` — All tests must pass

---

## Recommended Execution Order

### Week 1: Layout Module

| Task | File | Est. LOC | Effort |
|------|------|----------|--------|
| 1. Extract window_ops.rs | layout/mod.rs | ~600 | Medium |
| 2. Extract output_ops.rs | layout/mod.rs | ~400 | Easy |
| 3. Extract focus.rs | layout/mod.rs | ~500 | Medium |

### Week 2: Layout Module (continued)

| Task | File | Est. LOC | Effort |
|------|------|----------|--------|
| 4. Extract navigation.rs | layout/mod.rs | ~800 | Hard |
| 5. Extract resize.rs | layout/mod.rs | ~500 | Medium |
| 6. Extract row_management.rs | layout/mod.rs | ~500 | Medium |

### Week 3: Row Module

| Task | File | Est. LOC | Effort |
|------|------|----------|--------|
| 7. Extract core.rs | row/mod.rs | ~300 | Easy |
| 8. Extract tile_ops.rs | row/mod.rs | ~400 | Medium |
| 9. Extract state.rs | row/mod.rs | ~300 | Easy |

### Week 4+: Other Files (Lower Priority)

- src/niri.rs
- src/input/mod.rs
- src/backend/tty.rs

---

## Files to SKIP

| File | LOC | Reason |
|------|-----|--------|
| `src/layout/scrolling.rs` | 3990 | Being deprecated |
| `src/layout/tests.rs` | 3592 | Tests can be long |
| `src/tests/*.rs` | Various | Test files |

---

## Handoff

- [x] Code compiles (`cargo check`) — N/A (analysis only)
- [x] Tests pass (`cargo test`) — N/A (analysis only)
- [x] TODO.md updated with refactoring plan
- [x] TEAM_038 analysis updated with current LOC counts
- [x] Team file complete

## Summary

**Created comprehensive refactoring plan for 8 critical files (>2000 LOC)**:
1. Updated LOC counts (significant growth since Nov 27)
2. Identified `src/layout/mod.rs` and `src/layout/row/mod.rs` as highest priority
3. Categorized 229 methods on Layout<W> into 10 target files
4. Documented incremental extraction strategy
5. Provided execution timeline

## Additional Findings: Structural Problems

### 1. Render Elements Scattered
6 render element files are scattered at `src/layout/` root instead of grouped:
- `closing_window.rs` (275 LOC)
- `opening_window.rs` (143 LOC)
- `focus_ring.rs` (280 LOC)
- `shadow.rs` (184 LOC)
- `tab_indicator.rs` (412 LOC)
- `insert_hint_element.rs` (65 LOC)

**Solution**: Create `elements/` module and move all.

### 2. FloatingSpace Conceptual Confusion
- `src/layout/floating.rs` (1449 LOC) — FloatingSpace struct + impl
- `src/layout/canvas/floating.rs` (292 LOC) — Canvas2D methods delegating to FloatingSpace

FloatingSpace is PART OF Canvas2D but structured as a sibling!

**Solution**: Move FloatingSpace INTO `canvas/floating/` module.

### 3. Dead Code
- `workspace.rs` — 0 LOC (empty placeholder)
- `scrolling.rs` — 3990 LOC (being replaced by Row)

**Solution**: Delete workspace.rs, move scrolling.rs to deprecated/.

### 4. Hierarchy Mismatch
Current structure doesn't reflect conceptual hierarchy:
```
SHOULD BE: Layout → Monitor → Canvas2D → Row → Column → Tile
ACTUALLY:  All major types at same level with method dumping in mod.rs
```

---

## Handoff

- [x] Code compiles (`cargo check`) — N/A (analysis only)
- [x] Tests pass (`cargo test`) — N/A (analysis only)
- [x] TODO.md updated with COMPREHENSIVE refactoring plan (6 phases)
- [x] TEAM_038 analysis updated with current LOC counts
- [x] Team file complete

## Summary

**Created comprehensive refactoring plan covering:**
1. **Phase 0**: Cleanup (delete dead code)
2. **Phase 1**: Create `elements/` module (group render elements)
3. **Phase 2**: Consolidate FloatingSpace into `canvas/` (fix hierarchy)
4. **Phase 3**: Split `tile.rs` into `tile/` module
5. **Phase 4**: Split `row/mod.rs` further
6. **Phase 5**: Create `layout_impl/` module (THE BIG ONE: 5353 LOC → ~400)
7. **Phase 6**: Split `canvas/operations.rs`

**Total estimated effort: ~25 hours**

**Next team should start with**: Phase 0 (Cleanup) and Phase 1 (Create elements/)
