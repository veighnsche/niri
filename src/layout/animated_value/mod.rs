//! Animated value abstraction for smooth transitions.
//!
//! TEAM_005: Extracted from scrolling.rs ViewOffset for reuse in Camera and other components.
//!
//! This module provides [`AnimatedValue<f64>`] for 1D animations and [`AnimatedPoint`] for 2D.
//! Values can be static, animating toward a target, or controlled by a gesture.

use smithay::utils::{Logical, Point};

use crate::animation::Animation;

mod gesture;
pub use gesture::ViewGesture;

/// A value that can be static, animating, or gesture-controlled.
///
/// This is the foundation for smooth camera movement, view scrolling, and zoom transitions.
#[derive(Debug)]
pub enum AnimatedValue {
    /// The value is static.
    Static(f64),
    /// The value is animating toward a target.
    Animation(Animation),
    /// The value is controlled by an ongoing gesture.
    Gesture(ViewGesture),
}

impl AnimatedValue {
    /// Creates a new static value.
    pub fn new(value: f64) -> Self {
        Self::Static(value)
    }

    /// Returns the current value.
    pub fn current(&self) -> f64 {
        match self {
            AnimatedValue::Static(offset) => *offset,
            AnimatedValue::Animation(anim) => anim.value(),
            AnimatedValue::Gesture(gesture) => {
                gesture.current_view_offset
                    + gesture.animation.as_ref().map_or(0., |anim| anim.value())
            }
        }
    }

    /// Returns the target value suitable for computing a new value.
    pub fn target(&self) -> f64 {
        match self {
            AnimatedValue::Static(offset) => *offset,
            AnimatedValue::Animation(anim) => anim.to(),
            // This can be used for example if a gesture is interrupted.
            AnimatedValue::Gesture(gesture) => gesture.current_view_offset,
        }
    }

    /// Returns a value suitable for saving and later restoration.
    ///
    /// This means that it shouldn't return an in-progress animation or gesture value.
    pub fn stationary(&self) -> f64 {
        match self {
            AnimatedValue::Static(offset) => *offset,
            // For animations we can return the final value.
            AnimatedValue::Animation(anim) => anim.to(),
            AnimatedValue::Gesture(gesture) => gesture.stationary_view_offset,
        }
    }

    /// Returns `true` if the value is static.
    pub fn is_static(&self) -> bool {
        matches!(self, Self::Static(_))
    }

    /// Returns `true` if the value is controlled by a gesture.
    pub fn is_gesture(&self) -> bool {
        matches!(self, Self::Gesture(_))
    }

    /// Returns `true` if this is a DnD (drag-and-drop) scroll gesture.
    pub fn is_dnd_scroll(&self) -> bool {
        matches!(&self, AnimatedValue::Gesture(gesture) if gesture.dnd_last_event_time.is_some())
    }

    /// Returns `true` if an animation is currently ongoing.
    pub fn is_animation_ongoing(&self) -> bool {
        match self {
            AnimatedValue::Static(_) => false,
            AnimatedValue::Animation(_) => true,
            AnimatedValue::Gesture(gesture) => gesture.animation.is_some(),
        }
    }

    /// Offsets the value by the given delta.
    pub fn offset(&mut self, delta: f64) {
        match self {
            AnimatedValue::Static(offset) => *offset += delta,
            AnimatedValue::Animation(anim) => anim.offset(delta),
            AnimatedValue::Gesture(gesture) => {
                gesture.stationary_view_offset += delta;
                gesture.delta_from_tracker += delta;
                gesture.current_view_offset += delta;
            }
        }
    }

    /// Cancels the ongoing gesture, keeping the current value.
    pub fn cancel_gesture(&mut self) {
        if let AnimatedValue::Gesture(gesture) = self {
            *self = AnimatedValue::Static(gesture.current_view_offset);
        }
    }

    /// Stops any animation or gesture, setting to the current value.
    pub fn stop_anim_and_gesture(&mut self) {
        *self = AnimatedValue::Static(self.current());
    }
}

/// A 2D point with animated X and Y components.
///
/// Used by Camera for smooth panning and position changes.
#[derive(Debug)]
pub struct AnimatedPoint {
    /// The X component.
    pub x: AnimatedValue,
    /// The Y component.
    pub y: AnimatedValue,
}

impl AnimatedPoint {
    /// Creates a new static point.
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x: AnimatedValue::new(x),
            y: AnimatedValue::new(y),
        }
    }

    /// Returns the current position.
    pub fn current(&self) -> Point<f64, Logical> {
        Point::from((self.x.current(), self.y.current()))
    }

    /// Returns the target position.
    pub fn target(&self) -> Point<f64, Logical> {
        Point::from((self.x.target(), self.y.target()))
    }

    /// Returns a stationary position suitable for saving.
    pub fn stationary(&self) -> Point<f64, Logical> {
        Point::from((self.x.stationary(), self.y.stationary()))
    }

    /// Returns `true` if both components are static.
    pub fn is_static(&self) -> bool {
        self.x.is_static() && self.y.is_static()
    }

    /// Returns `true` if any animation is currently ongoing.
    pub fn is_animation_ongoing(&self) -> bool {
        self.x.is_animation_ongoing() || self.y.is_animation_ongoing()
    }

    /// Offsets the point by the given delta.
    pub fn offset(&mut self, delta: Point<f64, Logical>) {
        self.x.offset(delta.x);
        self.y.offset(delta.y);
    }

    /// Stops any animation or gesture on both components.
    pub fn stop_anim_and_gesture(&mut self) {
        self.x.stop_anim_and_gesture();
        self.y.stop_anim_and_gesture();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_value() {
        let v = AnimatedValue::new(42.0);
        assert!(v.is_static());
        assert!(!v.is_gesture());
        assert!(!v.is_animation_ongoing());
        assert_eq!(v.current(), 42.0);
        assert_eq!(v.target(), 42.0);
        assert_eq!(v.stationary(), 42.0);
    }

    #[test]
    fn offset_static() {
        let mut v = AnimatedValue::new(10.0);
        v.offset(5.0);
        assert_eq!(v.current(), 15.0);
    }

    #[test]
    fn animated_point_basic() {
        let p = AnimatedPoint::new(100.0, 200.0);
        assert!(p.is_static());
        assert!(!p.is_animation_ongoing());
        assert_eq!(p.current(), Point::from((100.0, 200.0)));
    }

    #[test]
    fn animated_point_offset() {
        let mut p = AnimatedPoint::new(10.0, 20.0);
        p.offset(Point::from((5.0, -5.0)));
        assert_eq!(p.current(), Point::from((15.0, 15.0)));
    }
}
