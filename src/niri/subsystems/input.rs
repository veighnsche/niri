//! Input tracking subsystem.
//!
//! Tracks scroll gestures, wheel movement, and modifier state for bindings.
//!
//! # Responsibilities
//!
//! - **Gesture tracking**: 3-finger swipe gestures and overview scroll gestures
//! - **Scroll tracking**: Wheel and finger scroll accumulators for binding detection
//! - **Modifier tracking**: Which modifier combinations have active bindings
//!
//! # Example
//!
//! ```ignore
//! // Check if current modifiers have mouse bindings
//! let mods = niri.seat.get_keyboard().unwrap().modifier_state();
//! let modifiers = modifiers_from_state(mods);
//! if niri.input.mods_with_mouse_binds().contains(&modifiers) {
//!     // Handle mouse binding
//! }
//!
//! // Accumulate wheel scroll for binding detection
//! let ticks = niri.input.vertical_wheel_mut().accumulate(delta);
//! if ticks != 0 {
//!     // Trigger scroll binding
//! }
//!
//! // Update modifier sets from config
//! niri.input.update_from_config(&config);
//! ```

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
        use crate::input::{
            mods_with_finger_scroll_binds, mods_with_mouse_binds, mods_with_wheel_binds,
        };

        // Default scroll tick value (15 is commonly used)
        const SCROLL_TICK: i8 = 15;

        let mod_key = config.input.mod_key.unwrap_or(niri_config::ModKey::Super);
        let binds = &config.binds;

        Self {
            gesture_swipe_3f: None,
            overview_swipe: ScrollSwipeGesture::new(),
            vertical_wheel: ScrollTracker::new(SCROLL_TICK),
            horizontal_wheel: ScrollTracker::new(SCROLL_TICK),
            mods_with_mouse_binds: mods_with_mouse_binds(mod_key, binds),
            mods_with_wheel_binds: mods_with_wheel_binds(mod_key, binds),
            vertical_finger: ScrollTracker::new(SCROLL_TICK),
            horizontal_finger: ScrollTracker::new(SCROLL_TICK),
            mods_with_finger_scroll_binds: mods_with_finger_scroll_binds(mod_key, binds),
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
        use crate::input::{
            mods_with_finger_scroll_binds, mods_with_mouse_binds, mods_with_wheel_binds,
        };

        let mod_key = config.input.mod_key.unwrap_or(niri_config::ModKey::Super);
        let binds = &config.binds;

        self.mods_with_mouse_binds = mods_with_mouse_binds(mod_key, binds);
        self.mods_with_wheel_binds = mods_with_wheel_binds(mod_key, binds);
        self.mods_with_finger_scroll_binds = mods_with_finger_scroll_binds(mod_key, binds);
    }
}
