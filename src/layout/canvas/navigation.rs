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

    // =========================================================================
    // TEAM_018: Row navigation methods for layout layer compatibility
    // =========================================================================

    /// Focuses the row above the current row.
    /// 
    /// This is the method that Layout.focus_row_up() should call.
    pub fn focus_row_up(&mut self) -> bool {
        if self.active_row_idx > 0 {
            self.focus_up()
        } else {
            false
        }
    }

    /// Focuses the row below the current row.
    /// 
    /// This is the method that Layout.focus_row_down() should call.
    pub fn focus_row_down(&mut self) -> bool {
        let max_row = self.rows.keys().max().copied().unwrap_or(0);
        if self.active_row_idx < max_row {
            self.focus_down()
        } else {
            false
        }
    }

    /// Moves the active window to the row above.
    pub fn move_window_to_row_up(&mut self) -> bool {
        if self.active_row_idx <= 0 {
            return false;
        }

        let target_row_idx = self.active_row_idx - 1;
        self.move_active_window_to_row(target_row_idx)
    }

    /// Moves the active window to the row below.
    pub fn move_window_to_row_down(&mut self) -> bool {
        let max_row = self.rows.keys().max().copied().unwrap_or(0);
        if self.active_row_idx >= max_row {
            return false;
        }

        let target_row_idx = self.active_row_idx + 1;
        self.move_active_window_to_row(target_row_idx)
    }

    /// Helper method to move the active window to a specific row.
    fn move_active_window_to_row(&mut self, target_row_idx: i32) -> bool {
        // Check if there's an active window in the current row
        let has_active_window = if let Some(row) = self.active_row_mut() {
            if let Some(column) = row.active_column_mut() {
                column.active_tile_idx < column.tiles.len()
            } else {
                false
            }
        } else {
            false
        };

        if !has_active_window {
            return false;
        }

        // Remove the window from current row
        let removed_tile = if let Some(row) = self.active_row_mut() {
            row.remove_active_tile(crate::utils::transaction::Transaction::new())
        } else {
            return false;
        };

        let Some(removed_tile) = removed_tile else {
            return false;
        };

        // Ensure target row exists
        self.ensure_row(target_row_idx);

        // Add window to target row with correct argument order
        if let Some(target_row) = self.rows.get_mut(&target_row_idx) {
            let is_full_width = removed_tile.is_full_width();
            let width = removed_tile.width();
            target_row.add_tile(
                None, 
                removed_tile.tile(), 
                is_full_width,
                width,
                true
            );
            // Switch focus to target row
            self.focus_row(target_row_idx);
            true
        } else {
            false
        }
    }

    /// Moves the active column to the row above.
    pub fn move_column_to_row_up(&mut self, activate: bool) -> bool {
        if self.active_row_idx <= 0 {
            return false;
        }

        let target_row_idx = self.active_row_idx - 1;
        self.move_active_column_to_row(target_row_idx, activate)
    }

    /// Moves the active column to the row below.
    pub fn move_column_to_row_down(&mut self, activate: bool) -> bool {
        let max_row = self.rows.keys().max().copied().unwrap_or(0);
        if self.active_row_idx >= max_row {
            return false;
        }

        let target_row_idx = self.active_row_idx + 1;
        self.move_active_column_to_row(target_row_idx, activate)
    }

    /// Helper method to move the active column to a specific row.
    fn move_active_column_to_row(&mut self, target_row_idx: i32, activate: bool) -> bool {
        // Remove the active column from current row
        let removed_column = if let Some(row) = self.active_row_mut() {
            row.remove_active_column()
        } else {
            return false;
        };

        let Some(column) = removed_column else {
            return false;
        };

        // Ensure target row exists
        self.ensure_row(target_row_idx);

        // Add column to target row
        if let Some(target_row) = self.rows.get_mut(&target_row_idx) {
            target_row.add_column(None, column, activate);
            // Switch focus to target row if activate is true
            if activate {
                self.focus_row(target_row_idx);
            }
            true
        } else {
            false
        }
    }

    /// Moves the current row up in the row order.
    pub fn move_row_up(&mut self) -> bool {
        if self.active_row_idx <= 0 {
            return false;
        }

        let current_idx = self.active_row_idx;
        let target_idx = current_idx - 1;

        self.swap_rows(current_idx, target_idx)
    }

    /// Moves the current row down in the row order.
    pub fn move_row_down(&mut self) -> bool {
        let max_row = self.rows.keys().max().copied().unwrap_or(0);
        if self.active_row_idx >= max_row {
            return false;
        }

        let current_idx = self.active_row_idx;
        let target_idx = current_idx + 1;

        self.swap_rows(current_idx, target_idx)
    }

    /// Helper method to swap two rows and update their indices.
    fn swap_rows(&mut self, idx1: i32, idx2: i32) -> bool {
        // Get both rows
        let Some(mut row1) = self.rows.remove(&idx1) else {
            return false;
        };
        let Some(mut row2) = self.rows.remove(&idx2) else {
            // Put row1 back since we failed
            self.rows.insert(idx1, row1);
            return false;
        };

        // Update their indices using setter methods
        row1.set_row_index(idx2);
        row1.set_y_offset(idx2 as f64 * row1.row_height());
        row2.set_row_index(idx1);
        row2.set_y_offset(idx1 as f64 * row2.row_height());

        // Put them back with swapped indices
        self.rows.insert(idx2, row1);
        self.rows.insert(idx1, row2);

        // Update active row index if it was one of the swapped rows
        if self.active_row_idx == idx1 {
            self.active_row_idx = idx2;
        } else if self.active_row_idx == idx2 {
            self.active_row_idx = idx1;
        }

        // Update camera to follow the active row
        self.update_camera_y();
        true
    }

    /// Sets the name of the current row.
    pub fn set_row_name(&mut self, name: Option<String>) {
        if let Some(row) = self.active_row_mut() {
            row.set_name(name);
        }
    }

    /// Unsets the name of the current row.
    pub fn unset_row_name(&mut self) {
        self.set_row_name(None);
    }

    // =========================================================================
    // TEAM_018: Additional workspace → canvas migration methods
    // =========================================================================

    /// Switches to a specific row by index.
    /// Replaces monitor.switch_workspace(idx)
    pub fn switch_to_row(&mut self, idx: i32) -> bool {
        self.focus_row(idx)
    }

    /// Switches to a specific row with auto-back-and-forth behavior.
    /// Replaces monitor.switch_workspace_auto_back_and_forth(idx)
    pub fn switch_to_row_auto_back_and_forth(&mut self, idx: i32) -> bool {
        // TODO(TEAM_018): Implement back-and-forth logic
        self.switch_to_row(idx)
    }

    /// Switches to the previous row.
    /// Replaces monitor.switch_workspace_previous()
    pub fn switch_to_previous_row(&mut self) -> bool {
        // TODO(TEAM_018): Implement previous row tracking
        false
    }

    /// Moves active window to a specific row.
    /// Replaces monitor.move_to_workspace(window, idx, activate)
    pub fn move_window_to_row(&mut self, window: &W::Id, idx: i32, activate: bool) -> bool {
        // Find which row contains the window
        let (current_row_idx, column_idx, tile_idx) = if let Some((row_idx, row, _tile)) = self.find_window(window) {
            // Find column and tile indices within the row
            let mut found = None;
            for (col_idx, column) in row.columns().enumerate() {
                for (tile_idx, tile_check) in column.tiles_iter().enumerate() {
                    if tile_check.window().id() == window {
                        found = Some((col_idx, tile_idx));
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            
            if let Some((col_idx, tile_idx)) = found {
                (row_idx, col_idx, tile_idx)
            } else {
                return false;
            }
        } else {
            return false;
        };

        // Remove window from current row
        let removed_tile = if let Some(row) = self.rows.get_mut(&current_row_idx) {
            row.remove_tile_by_idx(column_idx, tile_idx, crate::utils::transaction::Transaction::new(), None)
        } else {
            return false;
        };

        // Ensure target row exists
        self.ensure_row(idx);

        // Add window to target row
        if let Some(target_row) = self.rows.get_mut(&idx) {
            let is_full_width = removed_tile.is_full_width();
            let width = removed_tile.width();
            target_row.add_tile(
                None,
                removed_tile.tile(),
                is_full_width,
                width,
                activate
            );
            
            if activate {
                self.focus_row(idx);
            }
            true
        } else {
            false
        }
    }

    /// Moves active window up or to the row above.
    /// Replaces monitor.move_up_or_to_workspace_up()
    pub fn move_up_or_to_row_up(&mut self) -> bool {
        // Try to move up within current row first
        if let Some(row) = self.active_row_mut() {
            if row.active_column_idx() > 0 {
                return row.focus_left();
            }
        }
        
        // If at first column, try to move to row above
        self.move_window_to_row_up()
    }

    /// Moves active window down or to the row below.
    /// Replaces monitor.move_down_or_to_workspace_down()
    pub fn move_down_or_to_row_down(&mut self) -> bool {
        // Try to move down within current row first
        if let Some(row) = self.active_row_mut() {
            let column_count = row.column_count();
            if row.active_column_idx() < column_count.saturating_sub(1) {
                return row.focus_right();
            }
        }
        
        // If at last column, try to move to row below
        self.move_window_to_row_down()
    }

    /// Focuses window or moves to row below.
    /// Replaces monitor.focus_window_or_workspace_down()
    pub fn focus_window_or_row_down(&mut self) -> bool {
        // Try to focus down within current row first
        if let Some(row) = self.active_row_mut() {
            if row.focus_right() {
                return true;
            }
        }
        
        // If no focus change, try to move to row below
        self.focus_row_down()
    }

    /// Focuses window or moves to row above.
    /// Replaces monitor.focus_window_or_workspace_up()
    pub fn focus_window_or_row_up(&mut self) -> bool {
        // Try to focus up within current row first
        if let Some(row) = self.active_row_mut() {
            if row.focus_left() {
                return true;
            }
        }
        
        // If no focus change, try to move to row above
        self.focus_row_up()
    }
}
