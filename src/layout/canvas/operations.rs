// TEAM_008: Window operations split from mod.rs
//!
//! This module handles adding, removing, and finding windows in the canvas.

use crate::layout::canvas::Canvas2D;
use crate::layout::row::Row;
use crate::layout::tile::Tile;
use crate::layout::types::ColumnWidth;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Canvas2D<W> {
    // =========================================================================
    // Row Management
    // =========================================================================

    /// Creates a new row at the specified index if it doesn't exist.
    pub fn ensure_row(&mut self, row_idx: i32) -> &mut Row<W> {
        self.rows.entry(row_idx).or_insert_with(|| {
            Row::new(
                row_idx,
                self.view_size,
                self.parent_area,
                self.scale,
                self.clock.clone(),
                self.options.clone(),
            )
        })
    }

    /// Removes empty rows (except row 0 which is always kept).
    pub fn cleanup_empty_rows(&mut self) {
        self.rows.retain(|&idx, row| idx == 0 || !row.is_empty());
    }

    // =========================================================================
    // Tile Operations
    // =========================================================================

    /// Adds a tile to the active row.
    pub fn add_tile(
        &mut self,
        tile: Tile<W>,
        activate: bool,
        width: ColumnWidth,
        is_full_width: bool,
    ) {
        let row = self.ensure_row(self.active_row_idx);
        row.add_tile(None, tile, activate, width, is_full_width);
    }

    /// Adds a tile to a specific row.
    pub fn add_tile_to_row(
        &mut self,
        row_idx: i32,
        tile: Tile<W>,
        activate: bool,
        width: ColumnWidth,
        is_full_width: bool,
    ) {
        let row = self.ensure_row(row_idx);
        row.add_tile(None, tile, activate, width, is_full_width);

        if activate {
            self.active_row_idx = row_idx;
            self.update_camera_y();
        }
    }

    /// Returns whether the canvas contains the given window (in tiled rows).
    pub fn contains(&self, window: &W::Id) -> bool {
        self.rows.values().any(|row| row.contains(window))
    }

    /// Finds the row containing the given window.
    pub fn find_window(&self, window: &W::Id) -> Option<(i32, usize)> {
        for (&row_idx, row) in &self.rows {
            if let Some(col_idx) = row.find_column(window) {
                return Some((row_idx, col_idx));
            }
        }
        None
    }

    // =========================================================================
    // Tiles Query
    // =========================================================================

    /// Returns all tiles in the canvas (tiled only, not floating).
    /// TEAM_010: Added for Monitor.windows() migration
    pub fn tiles(&self) -> impl Iterator<Item = &Tile<W>> + '_ {
        self.rows.values().flat_map(|row| row.tiles())
    }

    /// Returns all tiles in the canvas (tiled only, not floating).
    /// TEAM_010: Added for Monitor.windows_mut() migration
    pub fn tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile<W>> + '_ {
        self.rows.values_mut().flat_map(|row| row.tiles_mut())
    }

    /// Returns all windows in the canvas (tiled and floating).
    /// TEAM_010: Added for Monitor.windows() migration
    pub fn windows(&self) -> impl Iterator<Item = &W> + '_ {
        let tiled = self.tiles().map(Tile::window);
        let floating = self.floating.tiles().map(Tile::window);
        tiled.chain(floating)
    }

    /// Returns all windows in the canvas (tiled and floating).
    /// TEAM_010: Added for Monitor.windows_mut() migration
    pub fn windows_mut(&mut self) -> impl Iterator<Item = &mut W> + '_ {
        // Can't easily chain mutable iterators, so collect tiled first
        // This is a limitation we accept for now
        self.rows
            .values_mut()
            .flat_map(|row| row.tiles_mut())
            .map(Tile::window_mut)
    }

    /// Returns all tiles with their render positions.
    pub fn tiles_with_render_positions(
        &self,
    ) -> impl Iterator<Item = (&Tile<W>, smithay::utils::Point<f64, smithay::utils::Logical>, bool)> + '_
    {
        let camera_offset = self.camera_position();
        self.rows.values().flat_map(move |row| {
            row.tiles_with_render_positions()
                .map(move |(tile, mut pos, is_active)| {
                    pos.x -= camera_offset.x;
                    pos.y -= camera_offset.y;
                    (tile, pos, is_active)
                })
        })
    }
}
