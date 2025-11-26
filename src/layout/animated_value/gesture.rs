//! Gesture tracking for animated values.
//!
//! TEAM_005: Extracted from scrolling.rs ViewGesture.

use std::time::Duration;

use crate::animation::{Animation, Clock};
use crate::input::swipe_tracker::SwipeTracker;

/// State for a gesture-controlled value.
///
/// This tracks both touchpad gestures and drag-and-drop scrolling.
#[derive(Debug)]
pub struct ViewGesture {
    /// The current view offset as modified by the gesture.
    pub current_view_offset: f64,
    /// Animation for extra offset to the current position.
    ///
    /// For example, when we need to activate a specific window during a DnD scroll.
    pub animation: Option<Animation>,
    /// Swipe tracker for gesture momentum.
    pub tracker: SwipeTracker,
    /// Delta accumulated from the tracker.
    pub delta_from_tracker: f64,
    /// The view offset to use for `activate_prev_column_on_removal`.
    pub stationary_view_offset: f64,
    /// Whether the gesture is controlled by the touchpad.
    pub is_touchpad: bool,

    /// If this gesture is for drag-and-drop scrolling, this is the last event's unadjusted
    /// timestamp.
    pub dnd_last_event_time: Option<Duration>,
    /// Time when the drag-and-drop scroll delta became non-zero, used for debouncing.
    ///
    /// If `None` then the scroll delta is currently zero.
    pub dnd_nonzero_start_time: Option<Duration>,
}

impl ViewGesture {
    /// Creates a new gesture starting at the given value.
    pub fn new(value: f64, is_touchpad: bool) -> Self {
        Self {
            current_view_offset: value,
            animation: None,
            tracker: SwipeTracker::new(),
            delta_from_tracker: 0.,
            stationary_view_offset: value,
            is_touchpad,
            dnd_last_event_time: None,
            dnd_nonzero_start_time: None,
        }
    }

    /// Creates a new DnD scroll gesture.
    pub fn new_dnd(value: f64, now: Duration) -> Self {
        Self {
            current_view_offset: value,
            animation: None,
            tracker: SwipeTracker::new(),
            delta_from_tracker: 0.,
            stationary_view_offset: value,
            is_touchpad: false,
            dnd_last_event_time: Some(now),
            dnd_nonzero_start_time: None,
        }
    }

    /// Starts an animation from the given offset.
    pub fn animate_from(&mut self, from: f64, clock: Clock, config: niri_config::Animation) {
        let current = self.animation.as_ref().map_or(0., Animation::value);
        self.animation = Some(Animation::new(clock, from + current, 0., 0., config));
    }
}
