# Golden Snapshot Wiring TODO

> **Purpose**: Complete checklist for porting snapshot infrastructure to main branch.
> **Created by**: TEAM_011 (fixing TEAM_004's mistake)

---

## Problem Summary

TEAM_004 created the snapshot infrastructure **on the 2d-canvas branch**, not from main.
This means we've been comparing refactored code to itself — no actual regression detection.

---

## Current Snapshot Infrastructure (on 2d-canvas)

### Files to Port

| File | Purpose | Lines | Dependencies |
|------|---------|-------|--------------|
| `src/layout/snapshot.rs` | Snapshot types | 285 | `serde`, `smithay::utils` |
| `src/layout/column/mod.rs` (lines 90-131) | `Column::snapshot()` | 42 | `snapshot.rs` types |
| `src/layout/scrolling.rs` (lines 3589-3782) | `ScrollingSpace::snapshot()` + helpers | 194 | `snapshot.rs` types, `Animation` |
| `src/layout/tests/golden.rs` | Test file | 993 | `snapshot.rs`, test infrastructure |

### Method Dependencies

#### `Column::snapshot(column_x: f64) -> ColumnSnapshot`
Calls on `self`:
- `self.tiles()` — returns iterator of `(Tile, Point)` ✅ EXISTS ON MAIN (line 5245)
- `self.width()` — returns `f64` ✅ EXISTS ON MAIN (line 4695)
- `self.active_tile_idx` — field ✅ EXISTS ON MAIN
- `self.is_full_width` — field ✅ EXISTS ON MAIN
- `self.sizing_mode().is_fullscreen()` — method ✅ EXISTS ON MAIN (line 4212)

Calls on `Tile`:
- `tile.tile_size()` — returns `Size<f64, Logical>` ✅ EXISTS ON MAIN (line 751)

#### `ScrollingSpace::snapshot() -> ScrollingSnapshot`
Calls on `self`:
- `self.columns.iter()` — field ✅ EXISTS ON MAIN
- `self.column_x(idx)` — method ✅ EXISTS ON MAIN (line 2316)
- `self.view_offset` — field ⚠️ DIFFERENT TYPE (main: `ViewOffset`, 2d-canvas: `AnimatedValue`)
- `self.working_area` — field ✅ EXISTS ON MAIN
- `self.view_size` — field ✅ EXISTS ON MAIN

Animation capture (TEAM_010 additions):
- `self.view_offset` animation extraction — ⚠️ NEEDS ADAPTATION for `ViewOffset` enum
- `column.move_animation()` — ⚠️ ADDED BY 2D-CANVAS
- `tile.resize_animation()` — ✅ EXISTS ON MAIN (line 557)
- `tile.move_x_animation_with_from()` — ⚠️ ADDED BY 2D-CANVAS
- `tile.move_y_animation_with_from()` — ⚠️ ADDED BY 2D-CANVAS

---

## Wiring Checklist for Main Branch

### Phase 1: Core Types (No Code Changes to Main)

- [ ] **1.1** Copy `src/layout/snapshot.rs` to main
- [ ] **1.2** Add `pub mod snapshot;` to `src/layout/mod.rs`
- [ ] **1.3** Add `yaml` feature to `insta` in `Cargo.toml`
- [ ] **1.4** Verify `cargo check` passes

### Phase 2: Column Snapshot Method

- [ ] **2.1** Add `#[cfg(test)] impl<W: LayoutElement> Column<W>` block to main's `scrolling.rs`
- [ ] **2.2** Implement `Column::snapshot(column_x: f64) -> ColumnSnapshot`
  - Uses: `self.tiles()`, `self.width()`, `self.active_tile_idx`, `self.is_full_width`, `self.sizing_mode()`
  - All methods exist on main ✅

### Phase 3: ScrollingSpace Snapshot Method (Position Only)

- [ ] **3.1** Add `#[cfg(test)]` block to main's `ScrollingSpace`
- [ ] **3.2** Implement `ScrollingSpace::snapshot() -> ScrollingSnapshot`
  - **CRITICAL**: Main uses `ViewOffset` enum, not `AnimatedValue`
  - For `view_offset.current()`, use:
    ```rust
    match &self.view_offset {
        ViewOffset::Static(v) => *v,
        ViewOffset::Animation(anim) => anim.value(),
        ViewOffset::Gesture(gesture) => gesture.current_view_offset,
    }
    ```
- [ ] **3.3** Leave `animations` field empty for now (main doesn't have all animation accessors)

### Phase 4: Animation Snapshot Methods (Full Parity)

These methods don't exist on main and need to be added:

- [ ] **4.1** Add `Column::move_animation() -> Option<(&Animation, f64)>`
  - Main has `move_animation: Option<MoveAnimation>` field
  - Need to expose it

- [ ] **4.2** Add `Column::tiles_with_animations()` iterator
  - Simple wrapper around `self.tiles.iter().enumerate()`

- [ ] **4.3** Add `Tile::move_x_animation_with_from() -> Option<(&Animation, f64)>`
  - Main's Tile has `move_x_animation: Option<Animation>` but no `from` tracking
  - ⚠️ May need to add `move_x_from` field to Tile

- [ ] **4.4** Add `Tile::move_y_animation_with_from() -> Option<(&Animation, f64)>`
  - Same issue as 4.3

- [ ] **4.5** Add `Tile::resize_animation_from_sizes() -> Option<(Size, Size)>`
  - Main has `resize_animation` but may not track `from` sizes

- [ ] **4.6** Add `ScrollingSpace::extract_animation_kind()` helper
  - Needs `Animation::easing_curve_name()`, `spring_params()`, `deceleration_params()`
  - Check if these exist on main's Animation type

### Phase 5: Test Infrastructure

- [ ] **5.1** Copy `src/layout/tests/golden.rs` to main
- [ ] **5.2** Add `mod golden;` to `src/layout/tests.rs`
- [ ] **5.3** Ensure `TestWindow`, `TestWindowParams`, `check_ops`, `Op` are accessible
- [ ] **5.4** Run `cargo test golden` — should fail (no snapshots yet)

### Phase 6: Generate Golden Snapshots

- [ ] **6.1** Run `cargo insta test --accept` on main branch
- [ ] **6.2** Verify all 91 tests generate snapshots
- [ ] **6.3** Lock snapshots: `chmod 444 src/layout/tests/snapshots/*.snap`
- [ ] **6.4** Commit snapshots to main (or golden branch)

### Phase 7: Port Back to 2d-canvas

- [ ] **7.1** Copy `src/layout/tests/snapshots/*.snap` from main to 2d-canvas
- [ ] **7.2** Run `cargo insta test` on 2d-canvas
- [ ] **7.3** If tests fail → we found regressions!
- [ ] **7.4** Fix regressions or document intentional changes

---

## Key Differences: Main vs 2d-canvas

| Component | Main Branch | 2d-canvas Branch |
|-----------|-------------|------------------|
| View offset type | `ViewOffset` enum | `AnimatedValue` enum |
| Column location | `scrolling.rs` (inline) | `column/mod.rs` (extracted) |
| Animation tracking | Partial | Full (with `from` values) |
| `tiles()` signature | `Iterator<Item = &Tile>` | `Iterator<Item = (&Tile, Point)>` |

---

## Decision: Animation Snapshots

**Option A**: Port animation tracking to main (more work, full parity)
**Option B**: Generate position-only snapshots from main, animation snapshots from 2d-canvas

**Recommendation**: Option A — we need to verify animations don't regress.

The animation methods added by TEAM_010 are:
1. `Column::move_animation()` — easy to add
2. `Column::tiles_with_animations()` — trivial
3. `Tile::move_x_animation_with_from()` — needs `from` field
4. `Tile::move_y_animation_with_from()` — needs `from` field
5. `Tile::resize_animation_from_sizes()` — needs `from` sizes

If main doesn't track `from` values, we can:
- Add the fields to main temporarily for snapshot generation
- Or accept that animation snapshots will be 2d-canvas only

---

## Files to Centralize

Before porting, consolidate all snapshot logic:

```
src/layout/snapshot/
├── mod.rs              # Re-exports
├── types.rs            # ScrollingSnapshot, ColumnSnapshot, TileSnapshot, etc.
├── scrolling_impl.rs   # ScrollingSpace::snapshot() implementation
├── column_impl.rs      # Column::snapshot() implementation
└── animation.rs        # Animation extraction helpers
```

This makes porting easier — just copy the directory.

---

*Created by TEAM_011 to fix TEAM_004's critical error.*
