# TEAM_004: Golden Snapshot Infrastructure (Phase 0.5.A + 0.5.B partial)

## Status: IN PROGRESS

## Mission
Execute Phase 0.5.A and 0.5.B: Set up golden snapshot testing infrastructure.

## Context Read
- `docs/2d-canvas-plan/README.md` — Master plan
- `docs/2d-canvas-plan/phases/phase-0.5-golden-snapshots.md` — Phase overview
- `docs/2d-canvas-plan/phases/phase-0.5a-golden-infrastructure.md` — Phase 0.5.A
- `docs/2d-canvas-plan/phases/phase-0.5b-golden-code-extraction.md` — Phase 0.5.B
- `docs/2d-canvas-plan/.teams/TEAM_003_import_cleanup.md` — Previous team's handoff

## Phase 0.5.A Tasks (COMPLETED)
- [x] **0.5.A.1**: Verify `insta` dependency (already present in Cargo.toml)
- [x] **0.5.A.2**: Create `src/layout/snapshot.rs` with snapshot types
- [x] **0.5.A.3**: Create `src/layout/golden/mod.rs` directory structure  
- [x] **0.5.A.4**: Update `src/layout/mod.rs` to declare both modules
- [x] **0.5.A.5**: Verify `cargo check` passes

## Phase 0.5.B Tasks (IN PROGRESS)
- [ ] **0.5.B.1**: Extract golden code from main branch — SKIPPED (using insta snapshots instead)
- [x] **0.5.B.2**: Add `snapshot()` method to `ScrollingSpace`
- [x] **0.5.B.3**: Add `snapshot()` method to `Column`
- [x] **0.5.B.4**: Create initial golden tests (Group A: 4 tests)
- [x] **0.5.B.5**: Accept baseline snapshots

## Changes Made

### Files Created
- `src/layout/snapshot.rs` — Snapshot types with `From` impls for smithay types
- `src/layout/golden/mod.rs` — Golden reference code placeholder (chmod -w)
- `src/layout/tests/golden.rs` — Golden snapshot tests (Group A)
- `src/layout/tests/snapshots/` — 4 baseline snapshot files (YAML)

### Files Modified
- `Cargo.toml` — Added `yaml` feature to insta
- `src/layout/mod.rs` — Added snapshot and golden module declarations
- `src/layout/scrolling.rs` — Added `snapshot()` method
- `src/layout/column/mod.rs` — Added `snapshot()` method  
- `src/layout/tests.rs` — Added golden module, made TestWindow/TestWindowParams pub(super)
- `docs/2d-canvas-plan/ai-teams-rules.md` — Added Rule 8 (maximize context), updated project state

## Current Golden Tests (58 tests)

### Groups A-L (30 tests)
- **A**: Basic window management (4)
- **B**: Focus changes (4)
- **C**: Column width presets (3)
- **D**: Centering (2)
- **E**: Multi-tile columns (4)
- **F**: Fullscreen (1)
- **G**: Move column (2)
- **H**: Move window within column (2)
- **I**: Window heights (2)
- **J**: Tabbed display (2)
- **K**: Close window (2)
- **L**: Edge cases (2)

### Groups M-W (28 tests)
- **M**: Insert position (2) — new window placement
- **N**: Close window effects (3) — first/last/only
- **O**: Edge cases (3) — empty workspace, single tile moves
- **P**: Interactive resize (3) — right/left/bottom edges
- **Q**: Swap window (2) — swap left/right
- **R**: Focus wrap-around (4) — right-or-first, left-or-last, down-or-top, up-or-bottom
- **S**: Focus by index (2) — column index, window in column
- **T**: Focus top/bottom (2)
- **U**: Floating windows (3) — toggle, focus tiling, switch
- **V**: View gestures (1) — touchpad scroll
- **W**: Combined focus (3) — down-or-right, up-or-left, window-or-workspace

## Questions Created
- `.questions/TEAM_004_keyboard_shortcuts.md` — Keyboard shortcuts for 2D canvas
- `.questions/TEAM_004_golden_test_gaps.md` — Gap analysis (mostly addressed)

## Code State
- Golden testing infrastructure fully operational
- `snapshot()` methods work correctly
- 58 baseline tests passing (Groups A-W)
- Use `cargo insta test` to verify, `cargo insta review` to inspect changes

## Handoff
- [x] Code compiles (`cargo check` passes)
- [x] Tests pass (`cargo test` — 247 passed, including 58 golden)
- [x] Golden tests pass (`cargo insta test`)
- [x] Team file complete
