# Phase P7.5: Extract InputTracking

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1 hour  
> **Risk Level**: 🟡 Medium (scroll state)  
> **Prerequisite**: Phase P7 complete  
> **Creates**: `InputTracking` struct

---

## Goal

Extract scroll/gesture tracking state into an `InputTracking` subsystem that:
- **Owns** scroll tracker state
- **Owns** gesture state
- **Owns** modifier tracking for bindings

---

## Fields to Move from Niri

```rust
// Input tracking (mod.rs)
pub gesture_swipe_3f_cumulative: Option<(f64, f64)>,
pub overview_scroll_swipe_gesture: ScrollSwipeGesture,
pub vertical_wheel_tracker: ScrollTracker,
pub horizontal_wheel_tracker: ScrollTracker,
pub mods_with_mouse_binds: HashSet<Modifiers>,
pub mods_with_wheel_binds: HashSet<Modifiers>,
pub vertical_finger_scroll_tracker: ScrollTracker,
pub horizontal_finger_scroll_tracker: ScrollTracker,
pub mods_with_finger_scroll_binds: HashSet<Modifiers>,
```

---

## Target Architecture

### New File: `src/niri/subsystems/input.rs`

```rust
//! Input tracking subsystem.
//!
//! Tracks scroll gestures, wheel movement, and modifier state for bindings.

use std::collections::HashSet;

use niri_config::Modifiers;

use crate::input::scroll_swipe_gesture::ScrollSwipeGesture;
use crate::input::scroll_tracker::ScrollTracker;

/// Input tracking subsystem.
pub struct InputTracking {
    /// Cumulative 3-finger swipe.
    gesture_swipe_3f: Option<(f64, f64)>,
    
    /// Overview scroll swipe gesture.
    overview_swipe: ScrollSwipeGesture,
    
    /// Vertical wheel tracker.
    vertical_wheel: ScrollTracker,
    
    /// Horizontal wheel tracker.
    horizontal_wheel: ScrollTracker,
    
    /// Modifiers with mouse binds.
    mods_with_mouse_binds: HashSet<Modifiers>,
    
    /// Modifiers with wheel binds.
    mods_with_wheel_binds: HashSet<Modifiers>,
    
    /// Vertical finger scroll tracker.
    vertical_finger: ScrollTracker,
    
    /// Horizontal finger scroll tracker.
    horizontal_finger: ScrollTracker,
    
    /// Modifiers with finger scroll binds.
    mods_with_finger_scroll_binds: HashSet<Modifiers>,
}

impl InputTracking {
    /// Creates a new input tracking subsystem.
    pub fn new(config: &niri_config::Config) -> Self {
        use crate::input::{mods_with_mouse_binds, mods_with_wheel_binds, mods_with_finger_scroll_binds};
        
        Self {
            gesture_swipe_3f: None,
            overview_swipe: ScrollSwipeGesture::new(),
            vertical_wheel: ScrollTracker::new(),
            horizontal_wheel: ScrollTracker::new(),
            mods_with_mouse_binds: mods_with_mouse_binds(config),
            mods_with_wheel_binds: mods_with_wheel_binds(config),
            vertical_finger: ScrollTracker::new(),
            horizontal_finger: ScrollTracker::new(),
            mods_with_finger_scroll_binds: mods_with_finger_scroll_binds(config),
        }
    }
    
    // =========================================================================
    // 3-Finger Swipe Gesture
    // =========================================================================
    
    pub fn swipe_3f(&self) -> Option<(f64, f64)> {
        self.gesture_swipe_3f
    }
    
    pub fn set_swipe_3f(&mut self, value: Option<(f64, f64)>) {
        self.gesture_swipe_3f = value;
    }
    
    pub fn add_swipe_3f(&mut self, dx: f64, dy: f64) {
        match &mut self.gesture_swipe_3f {
            Some((x, y)) => {
                *x += dx;
                *y += dy;
            }
            None => {
                self.gesture_swipe_3f = Some((dx, dy));
            }
        }
    }
    
    // =========================================================================
    // Overview Swipe
    // =========================================================================
    
    pub fn overview_swipe(&self) -> &ScrollSwipeGesture {
        &self.overview_swipe
    }
    
    pub fn overview_swipe_mut(&mut self) -> &mut ScrollSwipeGesture {
        &mut self.overview_swipe
    }
    
    // =========================================================================
    // Wheel Trackers
    // =========================================================================
    
    pub fn vertical_wheel(&self) -> &ScrollTracker {
        &self.vertical_wheel
    }
    
    pub fn vertical_wheel_mut(&mut self) -> &mut ScrollTracker {
        &mut self.vertical_wheel
    }
    
    pub fn horizontal_wheel(&self) -> &ScrollTracker {
        &self.horizontal_wheel
    }
    
    pub fn horizontal_wheel_mut(&mut self) -> &mut ScrollTracker {
        &mut self.horizontal_wheel
    }
    
    // =========================================================================
    // Finger Scroll Trackers
    // =========================================================================
    
    pub fn vertical_finger(&self) -> &ScrollTracker {
        &self.vertical_finger
    }
    
    pub fn vertical_finger_mut(&mut self) -> &mut ScrollTracker {
        &mut self.vertical_finger
    }
    
    pub fn horizontal_finger(&self) -> &ScrollTracker {
        &self.horizontal_finger
    }
    
    pub fn horizontal_finger_mut(&mut self) -> &mut ScrollTracker {
        &mut self.horizontal_finger
    }
    
    // =========================================================================
    // Modifier Sets
    // =========================================================================
    
    pub fn mods_with_mouse_binds(&self) -> &HashSet<Modifiers> {
        &self.mods_with_mouse_binds
    }
    
    pub fn mods_with_wheel_binds(&self) -> &HashSet<Modifiers> {
        &self.mods_with_wheel_binds
    }
    
    pub fn mods_with_finger_scroll_binds(&self) -> &HashSet<Modifiers> {
        &self.mods_with_finger_scroll_binds
    }
    
    /// Updates modifier sets from config.
    pub fn update_from_config(&mut self, config: &niri_config::Config) {
        use crate::input::{mods_with_mouse_binds, mods_with_wheel_binds, mods_with_finger_scroll_binds};
        
        self.mods_with_mouse_binds = mods_with_mouse_binds(config);
        self.mods_with_wheel_binds = mods_with_wheel_binds(config);
        self.mods_with_finger_scroll_binds = mods_with_finger_scroll_binds(config);
    }
}
```

---

## Work Units

### Unit 1: Add to subsystems/mod.rs

```rust
mod input;
pub use input::InputTracking;
```

### Unit 2: Create InputTracking struct

### Unit 3: Move fields from Niri

### Unit 4: Update access patterns

```rust
// Before
self.vertical_wheel_tracker.accumulate(delta)
self.mods_with_mouse_binds.contains(&mods)

// After
self.input.vertical_wheel_mut().accumulate(delta)
self.input.mods_with_mouse_binds().contains(&mods)
```

### Unit 5: Update input handling code

### Unit 6: Verify

---

## Verification Checklist

- [ ] `InputTracking` struct created
- [ ] All input tracking fields removed from Niri
- [ ] Input handling code updated
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/input.rs` | **NEW** ~200 lines |
| `src/niri/mod.rs` | -12 fields |
| `src/input/mod.rs` | Updated |

---

## Next Phase

After completing this phase, proceed to [Phase P9: Final Cleanup](phase-P9-cleanup.md).
