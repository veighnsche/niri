// TEAM_007: Layout computation extracted from mod.rs
//!
//! This module handles tile position queries and configuration updates.

use std::iter::zip;
use std::rc::Rc;

use smithay::utils::{Logical, Point, Size};

use super::{compute_working_area, Row};
use crate::layout::tile::Tile;
use crate::layout::{LayoutElement, Options};

impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // Configuration
    // =========================================================================

    /// Updates configuration when output changes.
    pub fn update_config(
        &mut self,
        view_size: Size<f64, Logical>,
        parent_area: smithay::utils::Rectangle<f64, Logical>,
        scale: f64,
        options: Rc<Options>,
    ) {
        let working_area = compute_working_area(parent_area, scale, options.layout.struts);

        for (column, data) in zip(&mut self.columns, &mut self.data) {
            column.update_config(view_size, working_area, parent_area, scale, options.clone());
            data.update(column);
        }

        self.view_size = view_size;
        self.working_area = working_area;
        self.parent_area = parent_area;
        self.scale = scale;
        self.options = options;
        self.y_offset = self.row_index as f64 * view_size.h;

        // Apply always-center and such right away.
        if !self.columns.is_empty() && !self.view_offset_x.is_gesture() {
            self.animate_view_offset_to_column(None, self.active_column_idx, None);
        }
    }

    // =========================================================================
    // Tile position queries
    // =========================================================================

    /// Returns tiles with their render positions, offset by the row's Y position.
    pub fn tiles_with_render_positions(
        &self,
    ) -> impl Iterator<Item = (&Tile<W>, Point<f64, Logical>, bool)> + '_ {
        let view_offset = self.view_offset_x.current();
        let y_offset = self.y_offset;
        let active_col_idx = self.active_column_idx;

        self.columns
            .iter()
            .enumerate()
            .flat_map(move |(col_idx, col)| {
                let col_x = self.column_x(col_idx) + view_offset;
                let is_active_col = col_idx == active_col_idx;

                // tiles() returns (tile, tile_offset) pairs
                col.tiles().enumerate().map(move |(tile_idx, (tile, tile_offset))| {
                    let tile_pos = Point::from((
                        col_x + tile_offset.x,
                        y_offset + tile_offset.y,
                    ));
                    let is_active = is_active_col && tile_idx == col.active_tile_idx;
                    (tile, tile_pos, is_active)
                })
            })
    }
}
