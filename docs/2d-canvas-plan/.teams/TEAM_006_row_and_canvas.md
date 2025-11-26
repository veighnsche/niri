# TEAM_006: Row + Canvas2D (Phase 1)

## Status: IN PROGRESS

## Objective
Implement Phase 1: Create Row and Canvas2D modules with basic vertical navigation.

## Plan
Per `phases/phase-1-row-and-canvas.md`:

### Step 1.1: Row Module
- [ ] 1.1.1: Create `row/mod.rs` with `Row<W>` struct
- [ ] 1.1.2: Port `ScrollingSpace` logic (composition approach)
- [ ] 1.1.3: Add row-specific fields (`row_index`, `y_offset`)
- [ ] 1.1.4: Create `row/layout.rs`
- [ ] 1.1.5: Create `row/navigation.rs`
- [ ] 1.1.6: Create `row/operations.rs`
- [ ] 1.1.7: Unit tests

### Step 1.2: Canvas2D Module
- [ ] 1.2.1-1.2.7: TBD after Row complete

### Step 1.3: Vertical Navigation
- [ ] 1.3.1-1.3.4: TBD after Canvas2D complete

### Step 1.4: Feature Flag
- [ ] 1.4.1-1.4.4: TBD

## Design Decision

**Option A (Composition)**: `Row` wraps `ScrollingSpace`
- Less code duplication, faster to implement
- Row adds row_index and y_offset on top of ScrollingSpace

Starting with Option A for pragmatic progress.

## Progress
(Will update as work progresses)

## Changes Made
(Will update as work progresses)

## Handoff
- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] Golden tests pass (`./scripts/verify-golden.sh`)
- [ ] Team file complete
