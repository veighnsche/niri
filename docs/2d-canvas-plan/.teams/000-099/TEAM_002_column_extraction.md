# TEAM_002: Column Extraction

## Status: COMPLETED

## Mission
Execute Step 0.1: Extract Column from scrolling.rs into layout/column/ module.

## Context Read
- `docs/2d-canvas-plan/phases/phase-0-preparation.md` — detailed extraction plan
- `src/layout/scrolling.rs` — source file (5586 lines before, ~3900 lines after)
  - Column struct: lines 145-216
  - Column impl: lines 3918-5439 (~1520 lines of methods)

## Changes Made

### New Files Created
- `src/layout/column/mod.rs` — Column struct, ColumnWidth enum, MoveAnimation struct, resolve_preset_size function
- `src/layout/column/tile_data.rs` — TileData struct, WindowHeight enum
- `src/layout/column/core.rs` — Constructor (new_with_tile), update_config, getters, animation methods
- `src/layout/column/operations.rs` — Tile add/remove/focus/move operations
- `src/layout/column/layout.rs` — Tile positioning, offset calculations, size resolution
- `src/layout/column/render.rs` — Rendering methods, animation helpers
- `src/layout/column/sizing.rs` — Width/height toggling, fullscreen/maximize, update_tile_sizes
- `src/layout/column/tests.rs` — verify_invariants method (test-only)

### Modified Files
- `src/layout/mod.rs` — Added `pub mod column;` declaration
- `src/layout/scrolling.rs` — Removed Column struct and impl (~1520 lines), now imports from column module
  - Added: `use super::column::{resolve_preset_size, WindowHeight};`
  - Added: `pub use super::column::{Column, ColumnWidth};` (backwards compatibility re-export)

## Decisions
- **Re-export Column and ColumnWidth from scrolling.rs**: Other modules (monitor.rs, workspace.rs, floating.rs) were importing these from scrolling. Re-exporting maintains backwards compatibility while we migrate.
- **Split impl blocks by category**: Rather than one giant impl block, methods are organized into logical submodules (core, operations, layout, render, sizing, tests).
- **Keep fields pub(crate)**: Per Phase 0 plan, extract first, then refactor visibility later.
- **Test-only code in separate module**: verify_invariants is only compiled for tests.

## For Next Team
- The Column module is now self-contained in `src/layout/column/`
- scrolling.rs is reduced from ~5586 lines to ~3900 lines
- Next step per Phase 0: Extract ScrollingSpace from scrolling.rs (Step 0.1.2)
- Consider eventually removing the re-exports from scrolling.rs and updating all import sites

### File Size Summary
| File | Lines |
|------|-------|
| mod.rs | 103 |
| tile_data.rs | 59 |
| core.rs | 219 |
| operations.rs | 151 |
| layout.rs | 191 |
| render.rs | 125 |
| sizing.rs | 565 |
| tests.rs | 114 |
| **Total** | **1527** |

scrolling.rs reduced from 5586 → 3899 lines (saved ~1687 lines)

## Handoff
- [x] Code compiles (`cargo check` passes)
- [x] Tests pass (189 tests, 0 failures)
- [x] Team file complete
- [x] Questions file created and resolved (`.questions/TEAM_002_column_module_questions.md`)
- [x] Phase 0 plan updated with test strategy and resolved decisions
