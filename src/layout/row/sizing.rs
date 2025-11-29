// TEAM_064: Width/height sizing operations extracted from mod.rs
//!
//! Column and window sizing operations for Row.

use niri_ipc::SizeChange;

use super::super::LayoutElement;
use super::Row;

impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // Column width operations
    // =========================================================================

    /// Toggle width configuration for the active column.
    /// TEAM_028: Implemented based on ScrollingSpace::toggle_width
    pub fn toggle_width(&mut self, forwards: bool) {
        if self.columns.is_empty() {
            return;
        }

        let col = &mut self.columns[self.active_column_idx];
        col.toggle_width(None, forwards);

        // Cancel any ongoing resize for this column
        if let Some(resize) = &mut self.interactive_resize {
            if &resize.window == col.tiles[col.active_tile_idx].window().id() {
                self.interactive_resize = None;
            }
        }
    }

    /// Toggle full width for active column.
    /// TEAM_028: Implemented based on ScrollingSpace::toggle_full_width
    pub fn toggle_full_width(&mut self) {
        if self.columns.is_empty() {
            return;
        }

        let col = &mut self.columns[self.active_column_idx];
        col.toggle_full_width();

        // Cancel any ongoing resize for this column
        if let Some(resize) = &mut self.interactive_resize {
            if &resize.window == col.tiles[col.active_tile_idx].window().id() {
                self.interactive_resize = None;
            }
        }
    }

    /// Set column width.
    /// TEAM_028: Implemented based on ScrollingSpace::set_window_width
    pub fn set_column_width(&mut self, change: SizeChange) {
        if self.columns.is_empty() {
            return;
        }

        let col_idx = self.active_column_idx;
        let col = &mut self.columns[col_idx];
        col.set_column_width(change, None, true);

        // TEAM_043: Update cached column data after width change
        self.data[col_idx].update(col);

        // Cancel any ongoing resize for this column
        if let Some(resize) = &mut self.interactive_resize {
            if &resize.window == col.tiles[col.active_tile_idx].window().id() {
                self.interactive_resize = None;
            }
        }
    }

    /// Expand the active column to fill available width.
    /// TEAM_035: Updated signature to take no arguments (uses active column)
    pub fn expand_column_to_available_width(&mut self) {
        if self.columns.is_empty() {
            return;
        }

        let col_idx = self.active_column_idx;
        let num_columns = self.columns.len();

        // Don't expand if column is already full width or in special sizing mode
        let col = &mut self.columns[col_idx];
        if !col.pending_sizing_mode().is_normal() || col.is_full_width {
            return;
        }

        // Store extra_size before we need to modify the column
        let extra_size_w = col.extra_size().w;

        // Calculate total width taken by all columns except the active one
        let gap = self.options.layout.gaps;
        let mut total_other_width = 0.0;

        for (idx, data) in self.data.iter().enumerate() {
            if idx != col_idx {
                total_other_width += data.width;
            }
        }

        // Add gaps between columns (num_columns - 1 gaps total, but exclude gap after active if
        // it's last)
        let gaps_between_columns = (num_columns - 1) as f64 * gap;
        total_other_width += gaps_between_columns;

        // Calculate available width (assuming row width equals view width)
        let view_width = self.view_size.w;
        let active_col_current_width = self.data[col_idx].width;
        let available_width =
            view_width - total_other_width - active_col_current_width - extra_size_w;

        if available_width <= 0.0 {
            // No space to expand
            return;
        }

        // If this is the only column, use toggle_full_width for better UX
        if num_columns == 1 {
            col.toggle_full_width();
            return;
        }

        // Expand the active column by the available width
        let new_width = active_col_current_width + available_width;
        col.width = crate::layout::types::ColumnWidth::Fixed(new_width);
        col.preset_width_idx = None;
        col.is_full_width = false;
        col.update_tile_sizes(true);
        // Note: Don't update self.data[col_idx].width here - it updates when tiles respond
    }

    // =========================================================================
    // Window width operations
    // =========================================================================

    /// Toggle window width.
    /// TEAM_028: Implemented based on ScrollingSpace::set_window_width
    /// TEAM_035: Updated signature to accept Option<&W::Id>
    pub fn toggle_window_width(&mut self, window: Option<&W::Id>, forwards: bool) {
        if self.columns.is_empty() {
            return;
        }

        let (col, _tile_idx) = if let Some(window) = window {
            self.columns
                .iter_mut()
                .find_map(|col| {
                    col.tiles
                        .iter()
                        .position(|tile| tile.window().id() == window)
                        .map(|tile_idx| (col, Some(tile_idx)))
                })
                .unwrap()
        } else {
            (&mut self.columns[self.active_column_idx], None)
        };

        col.toggle_width(None, forwards);

        // Cancel any ongoing resize for this column
        if let Some(resize) = &mut self.interactive_resize {
            let target_window =
                window.unwrap_or_else(|| col.tiles[col.active_tile_idx].window().id());
            if &resize.window == target_window {
                self.interactive_resize = None;
            }
        }
    }

    /// Set window width.
    /// TEAM_028: Implemented based on ScrollingSpace::set_window_width
    /// TEAM_035: Updated signature to accept Option<&W::Id>
    pub fn set_window_width(&mut self, window: Option<&W::Id>, change: SizeChange) {
        if self.columns.is_empty() {
            return;
        }

        let (col_idx, tile_idx) = if let Some(window) = window {
            self.columns
                .iter()
                .enumerate()
                .find_map(|(col_idx, col)| {
                    col.tiles
                        .iter()
                        .position(|tile| tile.window().id() == window)
                        .map(|tile_idx| (col_idx, Some(tile_idx)))
                })
                .unwrap()
        } else {
            (self.active_column_idx, None)
        };

        let col = &mut self.columns[col_idx];
        col.set_column_width(change, tile_idx, true);

        // TEAM_043: Update cached column data after width change
        self.data[col_idx].update(col);

        // Cancel any ongoing resize for this column
        if let Some(resize) = &mut self.interactive_resize {
            let target_window =
                window.unwrap_or_else(|| col.tiles[col.active_tile_idx].window().id());
            if &resize.window == target_window {
                self.interactive_resize = None;
            }
        }
    }

    // =========================================================================
    // Window height operations
    // =========================================================================

    /// Toggle window height.
    /// TEAM_028: Implemented based on ScrollingSpace::set_window_height
    /// TEAM_035: Updated signature to accept Option<&W::Id>
    pub fn toggle_window_height(&mut self, window: Option<&W::Id>, forwards: bool) {
        if self.columns.is_empty() {
            return;
        }

        let (col, tile_idx) = if let Some(window) = window {
            self.columns
                .iter_mut()
                .find_map(|col| {
                    col.tiles
                        .iter()
                        .position(|tile| tile.window().id() == window)
                        .map(|tile_idx| (col, Some(tile_idx)))
                })
                .unwrap()
        } else {
            (&mut self.columns[self.active_column_idx], None)
        };

        // Convert forwards boolean to SizeChange
        let change = if forwards {
            SizeChange::AdjustProportion(0.1)
        } else {
            SizeChange::AdjustProportion(-0.1)
        };

        col.set_window_height(change, tile_idx, true);

        // Cancel any ongoing resize for this column
        if let Some(resize) = &mut self.interactive_resize {
            let target_window =
                window.unwrap_or_else(|| col.tiles[col.active_tile_idx].window().id());
            if &resize.window == target_window {
                self.interactive_resize = None;
            }
        }
    }

    /// Reset window height to default.
    /// TEAM_035: Updated signature to accept Option<&W::Id>
    pub fn reset_window_height(&mut self, _window: Option<&W::Id>) {
        // Rows don't control individual window heights - this is a no-op
    }
}
