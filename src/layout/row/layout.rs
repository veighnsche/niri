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
    ///
    /// TEAM_044: Fixed to use -view_pos() like original ScrollingSpace
    pub fn tiles_with_render_positions(
        &self,
    ) -> impl Iterator<Item = (&Tile<W>, Point<f64, Logical>, bool)> + '_ {
        // Use -view_pos() like the original ScrollingSpace
        let view_off_x = -self.view_pos();
        let y_offset = self.y_offset;
        let active_col_idx = self.active_column_idx;

        self.columns
            .iter()
            .enumerate()
            .flat_map(move |(col_idx, col)| {
                let col_x = self.column_x(col_idx);
                let is_active_col = col_idx == active_col_idx;
                // TEAM_056: Include column's render offset (move animation) in position calculation
                let col_render_off = col.render_offset();

                // tiles() returns (tile, tile_offset) pairs
                col.tiles().enumerate().map(move |(tile_idx, (tile, tile_offset))| {
                    let tile_pos = Point::from((
                        view_off_x + col_x + col_render_off.x + tile_offset.x + tile.render_offset().x,
                        y_offset + col_render_off.y + tile_offset.y + tile.render_offset().y,
                    ));
                    let is_active = is_active_col && tile_idx == col.active_tile_idx;
                    (tile, tile_pos, is_active)
                })
            })
    }
    
    /// Returns the render location of a specific tile by ID.
    ///
    /// TEAM_044: Added for toggle_floating_window_by_id
    pub fn tile_render_location(&self, id: &W::Id) -> Option<Point<f64, Logical>> {
        self.tiles_with_render_positions()
            .find(|(tile, _, _)| tile.window().id() == id)
            .map(|(_, pos, _)| pos)
    }
    
    /// Returns the render location of the active tile.
    ///
    /// TEAM_044: Added for toggle_floating_window_by_id
    pub fn active_tile_render_location(&self) -> Option<Point<f64, Logical>> {
        self.tiles_with_render_positions()
            .find(|(_, _, is_active)| *is_active)
            .map(|(_, pos, _)| pos)
    }
}
