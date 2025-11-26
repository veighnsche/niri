// TEAM_008: Navigation split from mod.rs
//!
//! This module handles navigation between rows and columns in the canvas.

use crate::animation::Animation;
use crate::layout::animated_value::AnimatedValue;
use crate::layout::canvas::Canvas2D;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Canvas2D<W> {
    /// Focuses the row above the current row.
    pub fn focus_up(&mut self) -> bool {
        let target_row = self.active_row_idx - 1;
        self.focus_row(target_row)
    }

    /// Focuses the row below the current row.
    pub fn focus_down(&mut self) -> bool {
        let target_row = self.active_row_idx + 1;
        self.focus_row(target_row)
    }

    /// Focuses the column to the left in the active row.
    pub fn focus_left(&mut self) -> bool {
        if let Some(row) = self.active_row_mut() {
            row.focus_left()
        } else {
            false
        }
    }

    /// Focuses the column to the right in the active row.
    pub fn focus_right(&mut self) -> bool {
        if let Some(row) = self.active_row_mut() {
            row.focus_right()
        } else {
            false
        }
    }

    /// Focuses a specific row.
    pub(crate) fn focus_row(&mut self, target_row: i32) -> bool {
        if !self.rows.contains_key(&target_row) {
            return false;
        }

        // Try to maintain the same column index
        let col_idx = self
            .active_row()
            .map(|r| r.active_column_idx())
            .unwrap_or(0);

        self.active_row_idx = target_row;

        // Focus the same column index (or the last one if it doesn't exist)
        if let Some(row) = self.active_row_mut() {
            let max_col = row.column_count().saturating_sub(1);
            row.focus_column(col_idx.min(max_col));
        }

        self.update_camera_y();
        true
    }

    /// Updates the camera Y position to follow the active row.
    // TEAM_007: Animate camera_y when changing rows
    pub(crate) fn update_camera_y(&mut self) {
        if let Some(row) = self.active_row() {
            let target_y = row.y_offset();
            let current_y = self.camera_y.current();

            // If already at target, no need to animate
            let pixel = 1. / self.scale;
            if (current_y - target_y).abs() < pixel {
                self.camera_y = AnimatedValue::Static(target_y);
                return;
            }

            // TODO(TEAM_007): Add vertical_view_movement config to niri-config
            // For now, use horizontal_view_movement since they typically use similar easing
            let config = self.options.animations.horizontal_view_movement.0;
            self.camera_y = AnimatedValue::Animation(Animation::new(
                self.clock.clone(),
                current_y,
                target_y,
                0., // initial velocity
                config,
            ));
        }
    }
}
