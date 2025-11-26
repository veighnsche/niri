# TEAM_006: Row + Canvas2D (Phase 1)

## Status: ✅ COMPLETE

## Objective
Implement Phase 1: Create Row and Canvas2D modules with basic vertical navigation.

## Design Decision

**Chose Option B (Clean Slate)** per Rule 0: Quality > Speed.

Row owns its columns directly rather than wrapping ScrollingSpace.
This avoids indirection and technical debt.

## Progress

### Step 1.1: Row Module
- [x] 1.1.1: Create `row/mod.rs` with `Row<W>` struct (clean slate, not wrapper)
- [x] 1.1.3: Add row-specific fields (`row_index`, `y_offset`)
- [x] Basic queries: `is_empty`, `columns`, `active_column_idx`, `contains`, `find_column`
- [x] Basic navigation: `focus_left`, `focus_right`, `focus_column`
- [x] Animation: `advance_animations`, `are_animations_ongoing`
- [x] Tile queries: `tiles_with_render_positions`
- [x] Column operations: `add_tile`, `add_column`, `remove_column`
- [x] Column movement: `move_left`, `move_right`, `move_column_to`
- [ ] TODO: Column movement animations
- [ ] TODO: Consume/expel operations
- [ ] TODO: Gesture handling
- [ ] TODO: Full view offset animation logic
- [ ] TODO: Rendering

### Step 1.2: Canvas2D Module
- [x] 1.2.1: Create `canvas/mod.rs` with `Canvas2D<W>` struct
- [x] 1.2.2: Use `BTreeMap<i32, Row<W>>` for sparse row storage
- [x] Basic navigation: `focus_up`, `focus_down`, `focus_left`, `focus_right`
- [x] Row management: `ensure_row`, `cleanup_empty_rows`
- [x] Animation: `advance_animations`, `are_animations_ongoing`
- [x] Window operations: `add_tile`, `add_tile_to_row`, `contains`, `find_window`
- [ ] TODO: FloatingSpace integration
- [ ] TODO: Rendering

### Step 1.3: Vertical Navigation
- [x] 1.3.1: Implement `Canvas2D::focus_up()`
- [x] 1.3.2: Implement `Canvas2D::focus_down()`
- [ ] TODO: 1.3.3: Animate camera Y to follow active row

### Step 1.4: Feature Flag
- [ ] TODO: Not started

## Changes Made

### New Files
- `src/layout/row/mod.rs` — Row struct with clean-slate implementation
- `src/layout/canvas/mod.rs` — Canvas2D struct

### Modified Files
- `src/layout/mod.rs` — Added `pub mod row` and `pub mod canvas`
- `src/layout/scrolling.rs` — Added public accessors for Row compatibility
- `docs/2d-canvas-plan/ai-teams-rules.md` — Added Rule 0: Quality > Speed

## Handoff
- [x] Code compiles (`cargo check`)
- [x] Tests pass (`cargo test`) — 251 tests
- [x] Golden tests pass (`./scripts/verify-golden.sh`) — 58 tests
- [x] Team file complete

## Notes for Next Team

Row is a **partial implementation**. Key missing pieces:
1. Window operations (`add_window`, `remove_window`)
2. Column operations (consume, expel)
3. Full view offset animation logic (ported stub only)
4. Rendering
5. Interactive resize

The current implementation provides the structure but needs ScrollingSpace
methods ported incrementally. About ~3000 lines still to port.

Canvas2D is similarly partial — depends on Row completion.
