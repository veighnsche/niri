// TEAM_007: Column/tile operations extracted from mod.rs
// TEAM_008: Added add/remove/consume/expel operations ported from ScrollingSpace
//!
//! This module handles adding, removing, and moving columns within a row.

use std::cmp::min;

use niri_ipc::ColumnDisplay;
use smithay::utils::Point;

use super::{ColumnData, Row};
use crate::layout::animated_value::AnimatedValue;
use crate::layout::column::WindowHeight;
use crate::layout::column::Column;
use crate::layout::tile::Tile;
use crate::layout::types::ColumnWidth;
use crate::layout::{LayoutElement, RemovedTile};
use crate::utils::transaction::Transaction;

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

    /// Adds a tile to an existing column.
    ///
    /// If `tile_idx` is None, appends to the end of the column.
    pub fn add_tile_to_column(
        &mut self,
        col_idx: usize,
        tile_idx: Option<usize>,
        tile: Tile<W>,
        activate: bool,
    ) {
        let prev_next_x = self.column_x(col_idx + 1);

        let target_column = &mut self.columns[col_idx];
        let tile_idx = tile_idx.unwrap_or(target_column.tiles.len());
        let mut prev_active_tile_idx = target_column.active_tile_idx;

        target_column.add_tile_at(tile_idx, tile);
        self.data[col_idx].update(target_column);

        if tile_idx <= prev_active_tile_idx {
            target_column.active_tile_idx += 1;
            prev_active_tile_idx += 1;
        }

        if activate {
            target_column.activate_idx(tile_idx);
            if self.active_column_idx != col_idx {
                self.activate_column(col_idx);
            }
        }

        let target_column = &mut self.columns[col_idx];
        if target_column.display_mode == ColumnDisplay::Tabbed {
            if target_column.active_tile_idx == tile_idx {
                // Fade out the previously active tile.
                let tile = &mut target_column.tiles[prev_active_tile_idx];
                tile.animate_alpha(1., 0., self.options.animations.window_movement.0);
            } else {
                // Fade out when adding into a tabbed column into the background.
                let tile = &mut target_column.tiles[tile_idx];
                tile.animate_alpha(1., 0., self.options.animations.window_movement.0);
            }
        }

        // Adding a wider window into a column increases its width now (even if the window will
        // shrink later). Move the columns to account for this.
        let offset = self.column_x(col_idx + 1) - prev_next_x;
        if self.active_column_idx <= col_idx {
            for col in &mut self.columns[col_idx + 1..] {
                col.animate_move_from(-offset);
            }
        } else {
            for col in &mut self.columns[..=col_idx] {
                col.animate_move_from(offset);
            }
        }
    }

    /// Adds a tile as a new column to the right of a specific window.
    pub fn add_tile_right_of(
        &mut self,
        right_of: &W::Id,
        tile: Tile<W>,
        activate: bool,
        width: ColumnWidth,
        is_full_width: bool,
    ) {
        let right_of_idx = self
            .columns
            .iter()
            .position(|col| col.contains(right_of))
            .unwrap();
        let col_idx = right_of_idx + 1;

        self.add_tile(Some(col_idx), tile, activate, width, is_full_width);
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

    // =========================================================================
    // Consume/Expel operations
    // =========================================================================

    /// Consumes a window into the column to the left, or expels it as a new column to the left.
    ///
    /// If the window is the only tile in its column, it moves into the adjacent column.
    /// Otherwise, it moves out of its column as a new column to the left.
    // TEAM_008: Ported from ScrollingSpace
    pub fn consume_or_expel_window_left(&mut self, window: Option<&W::Id>) {
        if self.columns.is_empty() {
            return;
        }

        let (source_col_idx, source_tile_idx) = if let Some(window) = window {
            self.columns
                .iter_mut()
                .enumerate()
                .find_map(|(col_idx, col)| {
                    col.tiles
                        .iter()
                        .position(|tile| tile.window().id() == window)
                        .map(|tile_idx| (col_idx, tile_idx))
                })
                .unwrap()
        } else {
            let source_col_idx = self.active_column_idx;
            let source_tile_idx = self.columns[self.active_column_idx].active_tile_idx;
            (source_col_idx, source_tile_idx)
        };

        let source_column = &self.columns[source_col_idx];
        let prev_off = source_column.tile_offset(source_tile_idx);

        let source_tile_was_active = self.active_column_idx == source_col_idx
            && source_column.active_tile_idx == source_tile_idx;

        if source_column.tiles.len() == 1 {
            if source_col_idx == 0 {
                return;
            }

            // Move into adjacent column.
            let target_column_idx = source_col_idx - 1;

            let offset = if self.active_column_idx <= source_col_idx {
                // Tiles to the right animate from the following column.
                self.column_x(source_col_idx) - self.column_x(target_column_idx)
            } else {
                // Tiles to the left animate to preserve their right edge position.
                f64::max(
                    0.,
                    self.data[target_column_idx].width - self.data[source_col_idx].width,
                )
            };
            let mut offset = Point::from((offset, 0.));

            if source_tile_was_active {
                // Make sure the previous (target) column is activated so the animation looks right.
                //
                // However, if it was already going to be activated, leave the offset as is. This
                // improves the workflow that has become common with tabbed columns: open a new
                // window, then immediately consume it left as a new tab.
                self.activate_prev_column_on_removal
                    .get_or_insert(self.view_offset_x.stationary() + offset.x);
            }

            offset.x += self.columns[source_col_idx].render_offset().x;
            let removed = self.remove_tile_by_idx(
                source_col_idx,
                0,
                Transaction::new(),
            );
            self.add_tile_to_column(target_column_idx, None, removed.tile(), source_tile_was_active);

            let target_column = &mut self.columns[target_column_idx];
            offset.x -= target_column.render_offset().x;
            offset += prev_off - target_column.tile_offset(target_column.tiles.len() - 1);

            let new_tile = target_column.tiles.last_mut().unwrap();
            new_tile.animate_move_from(offset);
        } else {
            // Move out of column.
            let mut offset = Point::from((source_column.render_offset().x, 0.));

            let removed =
                self.remove_tile_by_idx(source_col_idx, source_tile_idx, Transaction::new());

            // We're inserting into the source column position.
            let target_column_idx = source_col_idx;

            let (tile, width, is_full_width, _) = removed.into_parts();
            self.add_tile(
                Some(target_column_idx),
                tile,
                source_tile_was_active,
                width,
                is_full_width,
            );

            if source_tile_was_active {
                // We added to the left, don't activate even further left on removal.
                self.activate_prev_column_on_removal = None;
            }

            if target_column_idx < self.active_column_idx {
                // Tiles to the left animate from the following column.
                offset.x += self.column_x(target_column_idx + 1) - self.column_x(target_column_idx);
            }

            let new_col = &mut self.columns[target_column_idx];
            offset += prev_off - new_col.tile_offset(0);
            new_col.tiles[0].animate_move_from(offset);
        }
    }

    /// Consumes a window into the column to the right, or expels it as a new column to the right.
    ///
    /// If the window is the only tile in its column, it moves into the adjacent column.
    /// Otherwise, it moves out of its column as a new column to the right.
    // TEAM_008: Ported from ScrollingSpace
    pub fn consume_or_expel_window_right(&mut self, window: Option<&W::Id>) {
        if self.columns.is_empty() {
            return;
        }

        let (source_col_idx, source_tile_idx) = if let Some(window) = window {
            self.columns
                .iter_mut()
                .enumerate()
                .find_map(|(col_idx, col)| {
                    col.tiles
                        .iter()
                        .position(|tile| tile.window().id() == window)
                        .map(|tile_idx| (col_idx, tile_idx))
                })
                .unwrap()
        } else {
            let source_col_idx = self.active_column_idx;
            let source_tile_idx = self.columns[self.active_column_idx].active_tile_idx;
            (source_col_idx, source_tile_idx)
        };

        let cur_x = self.column_x(source_col_idx);

        let source_column = &self.columns[source_col_idx];
        let mut offset = Point::from((source_column.render_offset().x, 0.));
        let prev_off = source_column.tile_offset(source_tile_idx);

        let source_tile_was_active = self.active_column_idx == source_col_idx
            && source_column.active_tile_idx == source_tile_idx;

        if source_column.tiles.len() == 1 {
            if source_col_idx + 1 == self.columns.len() {
                return;
            }

            // Move into adjacent column.
            let target_column_idx = source_col_idx;

            offset.x += cur_x - self.column_x(source_col_idx + 1);
            offset.x -= self.columns[source_col_idx + 1].render_offset().x;

            if source_tile_was_active {
                // Make sure the target column gets activated.
                self.activate_prev_column_on_removal = None;
            }

            let removed = self.remove_tile_by_idx(
                source_col_idx,
                0,
                Transaction::new(),
            );
            self.add_tile_to_column(target_column_idx, None, removed.tile(), source_tile_was_active);

            let target_column = &mut self.columns[target_column_idx];
            offset += prev_off - target_column.tile_offset(target_column.tiles.len() - 1);

            let new_tile = target_column.tiles.last_mut().unwrap();
            new_tile.animate_move_from(offset);
        } else {
            // Move out of column.
            let prev_width = self.data[source_col_idx].width;

            let removed =
                self.remove_tile_by_idx(source_col_idx, source_tile_idx, Transaction::new());

            let target_column_idx = source_col_idx + 1;

            let (tile, width, is_full_width, _) = removed.into_parts();
            self.add_tile(
                Some(target_column_idx),
                tile,
                source_tile_was_active,
                width,
                is_full_width,
            );

            offset.x += if self.active_column_idx <= target_column_idx {
                // Tiles to the right animate to the following column.
                cur_x - self.column_x(target_column_idx)
            } else {
                // Tiles to the left animate for a change in width.
                -f64::max(0., prev_width - self.data[target_column_idx].width)
            };

            let new_col = &mut self.columns[target_column_idx];
            offset += prev_off - new_col.tile_offset(0);
            new_col.tiles[0].animate_move_from(offset);
        }
    }

    /// Consumes the first tile from the column to the right into the active column.
    // TEAM_008: Ported from ScrollingSpace
    pub fn consume_into_column(&mut self) {
        if self.columns.len() < 2 {
            return;
        }

        if self.active_column_idx == self.columns.len() - 1 {
            return;
        }

        let target_column_idx = self.active_column_idx;
        let source_column_idx = self.active_column_idx + 1;

        let offset = self.column_x(source_column_idx)
            + self.columns[source_column_idx].render_offset().x
            - self.column_x(target_column_idx);
        let mut offset = Point::from((offset, 0.));
        let prev_off = self.columns[source_column_idx].tile_offset(0);

        let removed = self.remove_tile_by_idx(source_column_idx, 0, Transaction::new());
        self.add_tile_to_column(target_column_idx, None, removed.tile(), false);

        let target_column = &mut self.columns[target_column_idx];
        offset += prev_off - target_column.tile_offset(target_column.tiles.len() - 1);
        offset.x -= target_column.render_offset().x;

        let new_tile = target_column.tiles.last_mut().unwrap();
        new_tile.animate_move_from(offset);
    }
}
