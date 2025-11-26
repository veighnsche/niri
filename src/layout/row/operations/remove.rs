// TEAM_008: Remove operations split from operations.rs
//!
//! This module handles removing tiles and columns from a row.

use std::cmp::min;

use niri_ipc::ColumnDisplay;

use crate::layout::animated_value::AnimatedValue;
use crate::layout::column::Column;
use crate::layout::column::WindowHeight;
use crate::layout::row::Row;
use crate::layout::{LayoutElement, RemovedTile};
use crate::utils::transaction::Transaction;

impl<W: LayoutElement> Row<W> {
    /// Removes a tile by window ID.
    pub fn remove_tile(&mut self, window: &W::Id, transaction: Transaction) -> RemovedTile<W> {
        let column_idx = self
            .columns
            .iter()
            .position(|col| col.contains(window))
            .unwrap();
        let column = &self.columns[column_idx];

        let tile_idx = column.position(window).unwrap();
        self.remove_tile_by_idx(column_idx, tile_idx, transaction)
    }

    /// Removes a tile by column and tile index.
    pub fn remove_tile_by_idx(
        &mut self,
        column_idx: usize,
        tile_idx: usize,
        transaction: Transaction,
    ) -> RemovedTile<W> {
        // If this is the only tile in the column, remove the whole column.
        if self.columns[column_idx].tiles.len() == 1 {
            let mut column = self.remove_column_by_idx(column_idx);
            return RemovedTile::new(
                column.tiles.remove(tile_idx),
                column.width,
                column.is_full_width,
                false,
            );
        }

        let column = &mut self.columns[column_idx];
        let prev_width = self.data[column_idx].width;

        let movement_config = self.options.animations.window_movement.0;

        // Animate movement of other tiles.
        // FIXME: tiles can move by X too, in a centered or resizing layout with one window smaller
        // than the others.
        let offset_y = column.tile_offset(tile_idx + 1).y - column.tile_offset(tile_idx).y;
        for tile in &mut column.tiles[tile_idx + 1..] {
            tile.animate_move_y_from(offset_y);
        }

        if column.display_mode == ColumnDisplay::Tabbed && tile_idx != column.active_tile_idx {
            // Fade in when removing background tab from a tabbed column.
            let tile = &mut column.tiles[tile_idx];
            tile.animate_alpha(0., 1., movement_config);
        }

        let was_normal = column.sizing_mode().is_normal();

        let tile = column.tiles.remove(tile_idx);
        column.data.remove(tile_idx);

        // If an active column became non-fullscreen after removing the tile, clear the stored
        // unfullscreen offset.
        if column_idx == self.active_column_idx && !was_normal && column.sizing_mode().is_normal() {
            self.view_offset_to_restore = None;
        }

        // If one window is left, reset its weight to 1.
        if column.data.len() == 1 {
            if let WindowHeight::Auto { weight } = &mut column.data[0].height {
                *weight = 1.;
            }
        }

        // Stop interactive resize.
        if let Some(resize) = &self.interactive_resize {
            if tile.window().id() == &resize.window {
                self.interactive_resize = None;
            }
        }

        let removed = RemovedTile::new(
            tile,
            column.width,
            column.is_full_width,
            false,
        );

        #[allow(clippy::comparison_chain)]
        if tile_idx < column.active_tile_idx {
            // A tile above was removed; preserve the current position.
            column.active_tile_idx -= 1;
        } else if tile_idx == column.active_tile_idx {
            // The active tile was removed, so the active tile index shifted to the next tile.
            if tile_idx == column.tiles.len() {
                // The bottom tile was removed and it was active, update active idx to remain valid.
                column.activate_idx(tile_idx - 1);
            } else {
                // Ensure the newly active tile animates to opaque.
                column.tiles[tile_idx].ensure_alpha_animates_to_1();
            }
        }

        column.update_tile_sizes_with_transaction(true, transaction);
        self.data[column_idx].update(column);
        let offset = prev_width - column.width();

        // Animate movement of the other columns.
        if self.active_column_idx <= column_idx {
            for col in &mut self.columns[column_idx + 1..] {
                col.animate_move_from_with_config(offset, movement_config);
            }
        } else {
            for col in &mut self.columns[..=column_idx] {
                col.animate_move_from_with_config(-offset, movement_config);
            }
        }

        removed
    }

    /// Removes the active tile (from the active column).
    ///
    /// Returns the removed tile, or None if the row is empty.
    pub fn remove_active_tile(&mut self, transaction: Transaction) -> Option<RemovedTile<W>> {
        if self.columns.is_empty() {
            return None;
        }
        
        let column_idx = self.active_column_idx;
        let column = &self.columns[column_idx];
        let tile_idx = column.active_tile_idx;
        
        Some(self.remove_tile_by_idx(column_idx, tile_idx, transaction))
    }

    /// Removes the active column.
    pub fn remove_active_column(&mut self) -> Option<Column<W>> {
        if self.columns.is_empty() {
            return None;
        }

        Some(self.remove_column_by_idx(self.active_column_idx))
    }

    /// Removes a column by index with full animation support.
    pub fn remove_column_by_idx(&mut self, column_idx: usize) -> Column<W> {
        // Animate movement of the other columns.
        let movement_config = self.options.animations.window_movement.0;
        let offset = self.column_x(column_idx + 1) - self.column_x(column_idx);
        if self.active_column_idx <= column_idx {
            for col in &mut self.columns[column_idx + 1..] {
                col.animate_move_from_with_config(offset, movement_config);
            }
        } else {
            for col in &mut self.columns[..column_idx] {
                col.animate_move_from_with_config(-offset, movement_config);
            }
        }

        let column = self.columns.remove(column_idx);
        self.data.remove(column_idx);

        // Stop interactive resize.
        if let Some(resize) = &self.interactive_resize {
            if column
                .tiles
                .iter()
                .any(|tile| tile.window().id() == &resize.window)
            {
                self.interactive_resize = None;
            }
        }

        if column_idx + 1 == self.active_column_idx {
            // The previous column, that we were going to activate upon removal of the active
            // column, has just been itself removed.
            self.activate_prev_column_on_removal = None;
        }

        if column_idx == self.active_column_idx {
            self.view_offset_to_restore = None;
        }

        if self.columns.is_empty() {
            self.active_column_idx = 0;
            return column;
        }

        let view_config = self.options.animations.horizontal_view_movement.0;

        if column_idx < self.active_column_idx {
            // A column to the left was removed; preserve the current position.
            // FIXME: preserve activate_prev_column_on_removal.
            self.active_column_idx -= 1;
            self.activate_prev_column_on_removal = None;
        } else if column_idx == self.active_column_idx
            && self.activate_prev_column_on_removal.is_some()
        {
            // The active column was removed, and we needed to activate the previous column.
            if 0 < column_idx {
                let prev_offset = self.activate_prev_column_on_removal.unwrap();

                self.activate_column(self.active_column_idx - 1);

                // Restore the view offset but make sure to scroll the view in case the
                // previous window had resized.
                self.animate_view_offset_with_config(
                    self.active_column_idx,
                    prev_offset,
                    view_config,
                );
                self.animate_view_offset_to_column(
                    None,
                    self.active_column_idx,
                    None,
                );
            }
        } else {
            self.activate_column(min(self.active_column_idx, self.columns.len() - 1));
        }

        column
    }

    /// Removes the column at the specified index (simple version without animations).
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

        column
    }
}
