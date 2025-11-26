// TEAM_007: Column/tile operations extracted from mod.rs
//!
//! This module handles adding, removing, and moving columns within a row.

use super::{ColumnData, Row};
use crate::layout::animated_value::AnimatedValue;
use crate::layout::column::Column;
use crate::layout::tile::Tile;
use crate::layout::types::ColumnWidth;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // Add operations
    // =========================================================================

    /// Adds a tile as a new column.
    ///
    /// If `col_idx` is None, inserts after the active column.
    pub fn add_tile(
        &mut self,
        col_idx: Option<usize>,
        tile: Tile<W>,
        activate: bool,
        width: ColumnWidth,
        is_full_width: bool,
    ) {
        let column = Column::new_with_tile(
            tile,
            self.view_size,
            self.working_area,
            self.parent_area,
            self.scale,
            width,
            is_full_width,
        );

        self.add_column(col_idx, column, activate);
    }

    /// Adds a column at the specified index.
    ///
    /// If `idx` is None, inserts after the active column.
    pub fn add_column(&mut self, idx: Option<usize>, mut column: Column<W>, activate: bool) {
        let was_empty = self.columns.is_empty();

        let idx = idx.unwrap_or_else(|| {
            if was_empty {
                0
            } else {
                self.active_column_idx + 1
            }
        });

        column.update_config(
            self.view_size,
            self.working_area,
            self.parent_area,
            self.scale,
            self.options.clone(),
        );

        self.data.insert(idx, ColumnData { width: column.width() });
        self.columns.insert(idx, column);

        if activate {
            // If this is the first window on an empty row, skip animation.
            if was_empty {
                self.view_offset_x = AnimatedValue::new(0.);
            }

            let prev_offset = (!was_empty && idx == self.active_column_idx + 1)
                .then(|| self.view_offset_x.stationary());

            self.active_column_idx = idx;
            self.animate_view_offset_to_column(None, idx, None);
            self.activate_prev_column_on_removal = prev_offset;
        } else if !was_empty && idx <= self.active_column_idx {
            self.active_column_idx += 1;
        }

        // TODO(TEAM_006): Animate movement of other columns (port from ScrollingSpace)
    }

    // =========================================================================
    // Remove operations
    // =========================================================================

    /// Removes the column at the specified index.
    pub fn remove_column(&mut self, idx: usize) -> Column<W> {
        let column = self.columns.remove(idx);
        self.data.remove(idx);

        if self.columns.is_empty() {
            self.active_column_idx = 0;
            self.activate_prev_column_on_removal = None;
        } else if idx < self.active_column_idx {
            self.active_column_idx -= 1;
        } else if idx == self.active_column_idx {
            // Activate previous or next column
            if let Some(prev_offset) = self.activate_prev_column_on_removal.take() {
                if self.active_column_idx > 0 {
                    self.active_column_idx -= 1;
                }
                self.view_offset_x = AnimatedValue::new(prev_offset);
            } else {
                self.active_column_idx = self.active_column_idx.min(self.columns.len() - 1);
            }
            self.animate_view_offset_to_column(None, self.active_column_idx, None);
        }

        // TODO(TEAM_006): Animate movement of other columns (port from ScrollingSpace)

        column
    }

    // =========================================================================
    // Move operations
    // =========================================================================

    /// Moves the active column to the left.
    pub fn move_left(&mut self) -> bool {
        if self.active_column_idx == 0 {
            return false;
        }
        self.move_column_to(self.active_column_idx - 1);
        true
    }

    /// Moves the active column to the right.
    pub fn move_right(&mut self) -> bool {
        let new_idx = self.active_column_idx + 1;
        if new_idx >= self.columns.len() {
            return false;
        }
        self.move_column_to(new_idx);
        true
    }

    /// Moves the active column to a specific index.
    pub(crate) fn move_column_to(&mut self, new_idx: usize) {
        if self.active_column_idx == new_idx {
            return;
        }

        let current_col_x = self.column_x(self.active_column_idx);

        let column = self.columns.remove(self.active_column_idx);
        let data = self.data.remove(self.active_column_idx);
        self.columns.insert(new_idx, column);
        self.data.insert(new_idx, data);

        // Preserve the camera position when moving.
        let view_offset_delta = -self.column_x(self.active_column_idx) + current_col_x;
        self.view_offset_x.offset(view_offset_delta);

        self.active_column_idx = new_idx;
        self.animate_view_offset_to_column(None, new_idx, None);

        // TODO(TEAM_006): Animate column movement (port from ScrollingSpace)
    }
}
