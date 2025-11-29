// TEAM_065: Tile operations extracted from operations.rs
//!
//! This module handles adding, removing, and iterating over tiles in the canvas.

use smithay::utils::{Logical, Point};

use crate::layout::canvas::Canvas2D;
use crate::layout::tile::Tile;
use crate::layout::ColumnWidth;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Canvas2D<W> {
    // =========================================================================
    // Tile Addition
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

    // =========================================================================
    // Tile Iteration
    // =========================================================================

    /// Get all tiles in the canvas (tiled + floating).
    pub fn tiles(&self) -> impl Iterator<Item = &Tile<W>> + '_ {
        let tiled = self.rows.values().flat_map(|row| row.tiles());
        let floating = self.floating.tiles();
        tiled.chain(floating)
    }

    /// Get all tiles in the canvas (mutable).
    pub fn tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile<W>> + '_ {
        let tiled = self.rows.values_mut().flat_map(|row| row.tiles_mut());
        let floating = self.floating.tiles_mut();
        tiled.chain(floating)
    }

    /// Returns all tiles in the canvas (tiled only, not floating).
    /// TEAM_010: Added for Monitor.windows() migration
    pub fn tiled_tiles(&self) -> impl Iterator<Item = &Tile<W>> + '_ {
        self.rows.values().flat_map(|row| row.tiles())
    }

    /// Returns all tiles in the canvas (tiled only, not floating).
    /// TEAM_010: Added for Monitor.windows_mut() migration
    pub fn tiled_tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile<W>> + '_ {
        self.rows.values_mut().flat_map(|row| row.tiles_mut())
    }

    /// Returns all tiles with their render positions.
    pub fn tiles_with_render_positions(
        &self,
    ) -> impl Iterator<Item = (&Tile<W>, Point<f64, Logical>, bool)> + '_
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
