// TEAM_007: Gesture handling ported from ScrollingSpace
//!
//! This module handles touchpad/mouse gesture-based scrolling within a row.

use std::time::Duration;

use ordered_float::NotNan;

use super::Row;
use crate::animation::Animation;
use crate::input::swipe_tracker::SwipeTracker;
use crate::layout::animated_value::{AnimatedValue, ViewGesture};
use crate::layout::LayoutElement;

/// Amount of touchpad movement to scroll the view for the width of one working area.
const VIEW_GESTURE_WORKING_AREA_MOVEMENT: f64 = 1200.;

impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // Gesture Begin
    // =========================================================================

    /// Begins a view offset gesture (touchpad or mouse scroll).
    pub fn view_offset_gesture_begin(&mut self, is_touchpad: bool) {
        if self.columns.is_empty() {
            return;
        }

        if self.interactive_resize.is_some() {
            return;
        }

        let gesture = ViewGesture {
            current_view_offset: self.view_offset_x.current(),
            animation: None,
            tracker: SwipeTracker::new(),
            delta_from_tracker: self.view_offset_x.current(),
            stationary_view_offset: self.view_offset_x.stationary(),
            is_touchpad,
            dnd_last_event_time: None,
            dnd_nonzero_start_time: None,
        };
        self.view_offset_x = AnimatedValue::Gesture(gesture);
    }

    /// Begins a drag-and-drop scroll gesture.
    pub fn dnd_scroll_gesture_begin(&mut self) {
        if let AnimatedValue::Gesture(ViewGesture {
            dnd_last_event_time: Some(_),
            ..
        }) = &self.view_offset_x
        {
            // Already active.
            return;
        }

        let gesture = ViewGesture {
            current_view_offset: self.view_offset_x.current(),
            animation: None,
            tracker: SwipeTracker::new(),
            delta_from_tracker: self.view_offset_x.current(),
            stationary_view_offset: self.view_offset_x.stationary(),
            is_touchpad: false,
            dnd_last_event_time: Some(self.clock.now_unadjusted()),
            dnd_nonzero_start_time: None,
        };
        self.view_offset_x = AnimatedValue::Gesture(gesture);

        self.interactive_resize = None;
    }

    // =========================================================================
    // Gesture Update
    // =========================================================================

    /// Updates the view offset gesture with new delta.
    pub fn view_offset_gesture_update(
        &mut self,
        delta_x: f64,
        timestamp: Duration,
        is_touchpad: bool,
    ) -> Option<bool> {
        let AnimatedValue::Gesture(gesture) = &mut self.view_offset_x else {
            return None;
        };

        if gesture.is_touchpad != is_touchpad || gesture.dnd_last_event_time.is_some() {
            return None;
        }

        gesture.tracker.push(delta_x, timestamp);

        let norm_factor = if gesture.is_touchpad {
            self.working_area.size.w / VIEW_GESTURE_WORKING_AREA_MOVEMENT
        } else {
            1.
        };
        let pos = gesture.tracker.pos() * norm_factor;
        let view_offset = pos + gesture.delta_from_tracker;
        gesture.current_view_offset = view_offset;

        Some(true)
    }

    /// Updates the DnD scroll gesture with new delta.
    pub fn dnd_scroll_gesture_scroll(&mut self, delta: f64) -> bool {
        let AnimatedValue::Gesture(gesture) = &mut self.view_offset_x else {
            return false;
        };

        let Some(last_time) = gesture.dnd_last_event_time else {
            // Not a DnD scroll.
            return false;
        };

        let config = &self.options.gestures.dnd_edge_view_scroll;

        let now = self.clock.now_unadjusted();
        gesture.dnd_last_event_time = Some(now);

        if delta == 0. {
            // We're outside the scrolling zone.
            gesture.dnd_nonzero_start_time = None;
            return false;
        }

        let nonzero_start = *gesture.dnd_nonzero_start_time.get_or_insert(now);

        // Delay starting the gesture a bit to avoid unwanted movement when dragging across
        // monitors.
        let delay = Duration::from_millis(u64::from(config.delay_ms));
        if now.saturating_sub(nonzero_start) < delay {
            return true;
        }

        let time_delta = now.saturating_sub(last_time).as_secs_f64();

        let delta = delta * time_delta * config.max_speed;

        gesture.tracker.push(delta, now);

        let view_offset = gesture.tracker.pos() + gesture.delta_from_tracker;

        // Clamp it so that it doesn't go too much out of bounds.
        let (leftmost, rightmost) = if self.columns.is_empty() {
            (0., 0.)
        } else {
            let gaps = self.options.layout.gaps;

            let mut leftmost = -self.working_area.size.w;

            let last_col_idx = self.columns.len() - 1;
            let last_col_x = self
                .columns
                .iter()
                .take(last_col_idx)
                .fold(0., |col_x, col| col_x + col.width() + gaps);
            let last_col_width = self.data[last_col_idx].width;
            let mut rightmost = last_col_x + last_col_width - self.working_area.loc.x;

            let active_col_x = self
                .columns
                .iter()
                .take(self.active_column_idx)
                .fold(0., |col_x, col| col_x + col.width() + gaps);
            leftmost -= active_col_x;
            rightmost -= active_col_x;

            (leftmost, rightmost)
        };
        let min_offset = f64::min(leftmost, rightmost);
        let max_offset = f64::max(leftmost, rightmost);
        let clamped_offset = view_offset.clamp(min_offset, max_offset);

        gesture.delta_from_tracker += clamped_offset - view_offset;
        gesture.current_view_offset = clamped_offset;
        true
    }

    // =========================================================================
    // Gesture End
    // =========================================================================

    /// Ends the view offset gesture and snaps to nearest column.
    pub fn view_offset_gesture_end(&mut self, is_touchpad: Option<bool>) -> bool {
        let AnimatedValue::Gesture(gesture) = &mut self.view_offset_x else {
            return false;
        };

        if is_touchpad.is_some_and(|x| gesture.is_touchpad != x) {
            return false;
        }

        // Take into account any idle time between the last event and now.
        let now = self.clock.now_unadjusted();
        gesture.tracker.push(0., now);

        let norm_factor = if gesture.is_touchpad {
            self.working_area.size.w / VIEW_GESTURE_WORKING_AREA_MOVEMENT
        } else {
            1.
        };
        let velocity = gesture.tracker.velocity() * norm_factor;
        let pos = gesture.tracker.pos() * norm_factor;
        let current_view_offset = pos + gesture.delta_from_tracker;

        if self.columns.is_empty() {
            self.view_offset_x = AnimatedValue::Static(current_view_offset);
            return true;
        }

        // Figure out where the gesture would stop after deceleration.
        let end_pos = gesture.tracker.projected_end_pos() * norm_factor;
        let target_view_offset = end_pos + gesture.delta_from_tracker;

        // Compute snapping points and find closest.
        let (new_col_idx, target_snap_view_pos) =
            self.compute_gesture_snap(current_view_offset, target_view_offset);

        let active_col_x = self.column_x(self.active_column_idx);
        let new_col_x = self.column_x(new_col_idx);
        let delta = active_col_x - new_col_x;

        if self.active_column_idx != new_col_idx {
            self.view_offset_to_restore = None;
        }

        self.active_column_idx = new_col_idx;

        let target_view_offset = target_snap_view_pos - new_col_x;

        self.view_offset_x = AnimatedValue::Animation(Animation::new(
            self.clock.clone(),
            current_view_offset + delta,
            target_view_offset,
            velocity,
            self.options.animations.horizontal_view_movement.0,
        ));

        // HACK: deal with things like snapping to the right edge of a larger-than-view window.
        self.animate_view_offset_to_column(None, new_col_idx, None);

        true
    }

    /// Ends the DnD scroll gesture.
    pub fn dnd_scroll_gesture_end(&mut self) {
        let AnimatedValue::Gesture(gesture) = &mut self.view_offset_x else {
            return;
        };

        if gesture.dnd_last_event_time.is_some() && gesture.tracker.pos() == 0. {
            // DnD didn't scroll anything, so preserve the current view position.
            self.view_offset_x = AnimatedValue::Static(gesture.delta_from_tracker);

            if !self.columns.is_empty() {
                // Just in case, make sure the active window remains on screen.
                self.animate_view_offset_to_column(None, self.active_column_idx, None);
            }
            return;
        }

        self.view_offset_gesture_end(None);
    }

    // =========================================================================
    // Snapping Logic
    // =========================================================================

    /// Computes the snap point for a gesture ending at target_view_offset.
    ///
    /// Returns (column_index, snap_view_pos).
    fn compute_gesture_snap(
        &self,
        current_view_offset: f64,
        target_view_offset: f64,
    ) -> (usize, f64) {
        struct Snap {
            view_pos: f64,
            col_idx: usize,
        }

        let mut snapping_points = Vec::new();
        let gaps = self.options.layout.gaps;

        if self.is_centering_focused_column() {
            let mut col_x = 0.;
            for (col_idx, col) in self.columns.iter().enumerate() {
                let col_w = col.width();
                let mode = col.sizing_mode();

                let area = if mode.is_maximized() {
                    self.parent_area
                } else {
                    self.working_area
                };

                let left_strut = area.loc.x;

                let view_pos = if mode.is_fullscreen() {
                    col_x
                } else if area.size.w <= col_w {
                    col_x - left_strut
                } else {
                    col_x - (area.size.w - col_w) / 2. - left_strut
                };
                snapping_points.push(Snap { view_pos, col_idx });

                col_x += col_w + gaps;
            }
        } else {
            // Non-centered mode: snap to column boundaries
            let mut col_x = 0.;
            for (col_idx, col) in self.columns.iter().enumerate() {
                let col_w = col.width();
                let mode = col.sizing_mode();

                let area = if mode.is_maximized() {
                    self.parent_area
                } else {
                    self.working_area
                };

                let left_strut = area.loc.x;
                let padding = if mode.is_maximized() || mode.is_fullscreen() {
                    0.
                } else {
                    ((area.size.w - col_w) / 2.).clamp(0., gaps)
                };

                // Left edge snap
                let view_pos = if mode.is_fullscreen() {
                    col_x
                } else {
                    col_x - padding - left_strut
                };

                snapping_points.push(Snap { view_pos, col_idx });

                col_x += col_w + gaps;
            }
        }

        // Find the closest snapping point.
        snapping_points.sort_by_key(|snap| NotNan::new(snap.view_pos).unwrap());

        let active_col_x = self.column_x(self.active_column_idx);
        let target_view_pos = active_col_x + target_view_offset;
        let target_snap = snapping_points
            .iter()
            .min_by_key(|snap| NotNan::new((snap.view_pos - target_view_pos).abs()).unwrap())
            .unwrap();

        let mut new_col_idx = target_snap.col_idx;

        // Focus the furthest window towards the direction of the gesture.
        if !self.is_centering_focused_column() {
            new_col_idx =
                self.adjust_snap_for_direction(new_col_idx, target_snap.view_pos, current_view_offset, target_view_offset);
        }

        (new_col_idx, target_snap.view_pos)
    }

    /// Adjusts the snap column based on gesture direction.
    fn adjust_snap_for_direction(
        &self,
        mut new_col_idx: usize,
        snap_view_pos: f64,
        current_view_offset: f64,
        target_view_offset: f64,
    ) -> usize {
        let gaps = self.options.layout.gaps;

        if target_view_offset >= current_view_offset {
            // Swiping right - focus furthest visible column to the right
            for col_idx in (new_col_idx + 1)..self.columns.len() {
                let col = &self.columns[col_idx];
                let col_x = self.column_x(col_idx);
                let col_w = col.width();
                let mode = col.sizing_mode();

                let area = if mode.is_maximized() {
                    self.parent_area
                } else {
                    self.working_area
                };

                let left_strut = area.loc.x;
                let padding = if mode.is_maximized() || mode.is_fullscreen() {
                    0.
                } else {
                    ((area.size.w - col_w) / 2.).clamp(0., gaps)
                };

                if mode.is_fullscreen() {
                    if snap_view_pos + self.view_size.w < col_x + col_w {
                        break;
                    }
                } else {
                    if snap_view_pos + left_strut + area.size.w < col_x + col_w + padding {
                        break;
                    }
                }

                new_col_idx = col_idx;
            }
        } else {
            // Swiping left - focus furthest visible column to the left
            for col_idx in (0..new_col_idx).rev() {
                let col = &self.columns[col_idx];
                let col_x = self.column_x(col_idx);
                let col_w = col.width();
                let mode = col.sizing_mode();

                let area = if mode.is_maximized() {
                    self.parent_area
                } else {
                    self.working_area
                };

                let left_strut = area.loc.x;
                let padding = if mode.is_maximized() || mode.is_fullscreen() {
                    0.
                } else {
                    ((area.size.w - col_w) / 2.).clamp(0., gaps)
                };

                if mode.is_fullscreen() {
                    if col_x < snap_view_pos {
                        break;
                    }
                } else {
                    if col_x - padding < snap_view_pos + left_strut {
                        break;
                    }
                }

                new_col_idx = col_idx;
            }
        }

        new_col_idx
    }
}
