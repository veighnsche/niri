# TEAM_003: Import Cleanup + Golden Snapshot Planning

## Status: COMPLETED

## Mission
1. Execute Step 0.3: Clean up ScrollingSpace dependencies
2. Design and document Phase 0.5: Golden Snapshot Infrastructure

## Context Read
- `docs/2d-canvas-plan/phases/phase-0-preparation.md` — Phase 0 plan
- `docs/2d-canvas-plan/.teams/TEAM_002_column_extraction.md` — Previous team's work
- `docs/2d-canvas-plan/.questions/TEAM_002_column_module_questions.md` — Resolved questions

## Tasks (from Phase 0.3)
- [x] **0.3.1**: Update all imports to use `column::` directly
- [x] **0.3.2**: Create `src/layout/types.rs` for shared types
- [x] **0.3.3**: Move `InteractiveResize` and `ResolvedSize` to types.rs
- [x] **0.3.4**: Move `InsertPosition` to types.rs
- [x] **0.3.5**: Ensure no `super::workspace` or `super::monitor` imports remain in scrolling
- [x] **0.3.6**: Document the clean public interface for ScrollingSpace

## Changes Made

### Phase 0.5 Documentation Created
- `phases/phase-0.5-golden-snapshots.md` — Index/overview
- `phases/phase-0.5a-golden-infrastructure.md` — Sub-phase A: insta, types, dirs
- `phases/phase-0.5b-golden-code-extraction.md` — Sub-phase B: extract golden code
- `phases/phase-0.5c-core-golden-tests.md` — Sub-phase C: tests A-L (~86 tests)
- `phases/phase-0.5d-advanced-golden-tests.md` — Sub-phase D: tests M-W (~71 tests)
- `phases/golden-test-scenarios.md` — Comprehensive test scenarios (~150 total)

### Other Documentation Updated
- `ai-teams-rules.md` — Restructured rules 1-9, added Rule 4 (Golden Testing)
- `README.md` — Added Phase 0.5 sub-phases, emphasized golden testing
- `.questions/TEAM_003_golden_snapshots.md` — Design decisions with USER answers

### New Files Created
- `src/layout/types.rs` — Shared types module containing:
  - `ColumnWidth` — Width specification for columns
  - `ScrollDirection` — Left/Right direction enum
  - `ResolvedSize` — Tile or Window size in pixels
  - `InteractiveResize<W>` — State for resize operations
  - `InsertPosition` — Where to insert a window

### Modified Files
- `src/layout/mod.rs` — Added `pub mod types;`, updated imports
- `src/layout/scrolling.rs` — Removed re-exports, imports from types module
- `src/layout/floating.rs` — Imports from types module
- `src/layout/workspace.rs` — Re-exports from types module
- `src/layout/monitor.rs` — Re-exports InsertPosition from types module
- `src/layout/column/mod.rs` — Re-exports ColumnWidth from types module
- `src/layout/column/layout.rs` — Imports ResolvedSize from types module
- `src/layout/column/sizing.rs` — Imports ResolvedSize from types module

## Decisions
- **Created types.rs for truly shared types**: Rather than moving all types mentioned in the plan, focused on types that are actually used across multiple modules (ColumnWidth, ScrollDirection, ResolvedSize, InteractiveResize, InsertPosition).
- **Left TileData and WindowHeight in column module**: These are internal to column and not used elsewhere, so keeping them in column/tile_data.rs makes sense.
- **Re-exports for backwards compatibility**: Some modules (workspace.rs, monitor.rs, column/mod.rs) re-export types from types.rs to maintain existing public interfaces.

## Dependency Graph After Cleanup
```
types.rs (no dependencies on other layout modules)
    ↑
column/ (imports from types)
    ↑
scrolling.rs (imports from column, types)
floating.rs (imports from types)
    ↑
workspace.rs (imports from scrolling, floating, types)
    ↑
monitor.rs (imports from workspace, types)
    ↑
mod.rs (imports from all)
```

## For Next Team

### ⚠️ PRIORITY: Phase 0.5.A (Golden Infrastructure) is next

**Start with**: `phases/phase-0.5a-golden-infrastructure.md`

Phase 0.5 is split into 4 sub-phases:
1. **0.5.A** (2-4 hrs): Add insta, create snapshot types, create golden/ dir
2. **0.5.B** (4-6 hrs): Extract golden code, add snapshot() methods
3. **0.5.C** (4-6 hrs): Core golden tests (Groups A-L, ~86 tests)
4. **0.5.D** (4-6 hrs): Advanced golden tests (Groups M-W, ~71 tests)

Each sub-phase can be done by a separate team.

### After Phase 0.5 is complete:
- Step 0.2 (AnimatedValue extraction) can proceed
- All layout refactors must pass `cargo insta test`

### Code State:
- scrolling.rs has no upward dependencies on workspace or monitor
- Shared types are in `src/layout/types.rs`

## Handoff
- [x] Code compiles (`cargo check` passes)
- [x] Tests pass (189 tests, 0 failures)
- [x] Golden tests pass — N/A (not yet implemented)
- [x] Team file complete
