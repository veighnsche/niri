# TEAM_005: AnimatedValue Extraction

## Status: COMPLETE ✅

## Objective
Implement Step 0.2: Extract ViewOffset into reusable AnimatedValue abstraction.

## Progress
- [x] 0.2.1: Create `src/layout/animated_value/mod.rs` with `AnimatedValue` enum
- [x] 0.2.2: Move `ViewGesture` to `animated_value/gesture.rs`
- [x] 0.2.3: Create `AnimatedPoint` for 2D (x, y) — ready for Camera in Phase 3
- [x] 0.2.4: Replace `ViewOffset` in scrolling.rs with `AnimatedValue`
- [x] 0.2.5: Verify all gesture/animation behavior unchanged (251 tests pass)

## Changes Made

### New Files
- `src/layout/animated_value/mod.rs` — `AnimatedValue` enum with Static/Animation/Gesture variants, plus `AnimatedPoint` for 2D
- `src/layout/animated_value/gesture.rs` — `ViewGesture` struct with touchpad and DnD scroll support

### Modified Files
- `src/layout/mod.rs` — Added `pub mod animated_value`
- `src/layout/scrolling.rs`:
  - Replaced `ViewOffset` enum with `AnimatedValue` import
  - Removed `ViewOffset` and `ViewGesture` type definitions
  - Removed `impl ViewOffset` and `impl ViewGesture` blocks
  - Updated all usages: `ViewOffset::Static` → `AnimatedValue::Static`, etc.
  - Updated test helper return type

### API
```rust
// AnimatedValue — 1D animated value
pub enum AnimatedValue {
    Static(f64),
    Animation(Animation),
    Gesture(ViewGesture),
}

impl AnimatedValue {
    pub fn new(value: f64) -> Self;
    pub fn current(&self) -> f64;
    pub fn target(&self) -> f64;
    pub fn stationary(&self) -> f64;
    pub fn is_static(&self) -> bool;
    pub fn is_gesture(&self) -> bool;
    pub fn is_dnd_scroll(&self) -> bool;
    pub fn is_animation_ongoing(&self) -> bool;
    pub fn offset(&mut self, delta: f64);
    pub fn cancel_gesture(&mut self);
    pub fn stop_anim_and_gesture(&mut self);
}

// AnimatedPoint — 2D animated point (for Camera)
pub struct AnimatedPoint {
    pub x: AnimatedValue,
    pub y: AnimatedValue,
}
```

## Handoff
- [x] Code compiles (`cargo check`)
- [x] Tests pass (`cargo test`) — 251 tests
- [x] Golden tests pass (`cargo insta test`) — 58 golden tests
- [x] Team file complete

## Next Steps
Phase 0 is now **COMPLETE**. Next team should proceed to:
- **Phase 1**: Row + Canvas2D (`phases/phase-1-row-and-canvas.md`)
