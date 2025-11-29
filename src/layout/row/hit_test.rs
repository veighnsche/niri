// TEAM_064: Hit testing methods extracted from mod.rs
//!
//! This module handles hit testing for mouse/touch interactions within a row.

use niri_ipc::ColumnDisplay;
use smithay::utils::{Logical, Point};

use super::Row;
use crate::layout::LayoutElement;
use crate::utils::ResizeEdge;

impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // Hit Testing
    // =========================================================================

    /// Find window under the given point.
    /// TEAM_036: Implemented based on ScrollingSpace::window_under
    pub fn window_under(&self, pos: Point<f64, Logical>) -> Option<(&W, super::super::HitType)> {
        let scale = self.scale;
        let view_off = Point::from((-self.view_pos(), 0.));
        
        for (col, col_x) in self.columns_in_render_order() {
            let col_off = Point::from((col_x, 0.));
            let col_render_off = col.render_offset();

            // Hit the tab indicator.
            if col.display_mode == ColumnDisplay::Tabbed && col.sizing_mode().is_normal() {
                let col_pos = view_off + col_off + col_render_off;
                let col_pos = col_pos.to_physical_precise_round(scale).to_logical(scale);

                if let Some(idx) = col.tab_indicator.hit(
                    col.tab_indicator_area(),
                    col.tiles.len(),
                    scale,
                    pos - col_pos,
                ) {
                    let hit = super::super::HitType::Activate {
                        is_tab_indicator: true,
                    };
                    return Some((col.tiles[idx].window(), hit));
                }
            }

            for (tile, tile_off, visible) in col.tiles_in_render_order() {
                if !visible {
                    continue;
                }

                let tile_pos =
                    view_off + col_off + col_render_off + tile_off + tile.render_offset();
                // Round to physical pixels.
                let tile_pos = tile_pos.to_physical_precise_round(scale).to_logical(scale);

                if let Some(rv) = super::super::HitType::hit_tile(tile, tile_pos, pos) {
                    return Some(rv);
                }
            }
        }

        None
    }

    /// Find resize edges under the given point.
    /// TEAM_036: Implemented based on original Workspace::resize_edges_under
    pub fn resize_edges_under(&self, pos: Point<f64, Logical>) -> Option<ResizeEdge> {
        let scale = self.scale;
        let view_off = Point::from((-self.view_pos(), 0.));
        
        for (col, col_x) in self.columns_in_render_order() {
            let col_off = Point::from((col_x, 0.));
            let col_render_off = col.render_offset();

            for (tile, tile_off, visible) in col.tiles_in_render_order() {
                if !visible {
                    continue;
                }

                let tile_pos =
                    view_off + col_off + col_render_off + tile_off + tile.render_offset();
                // Round to physical pixels.
                let tile_pos = tile_pos.to_physical_precise_round(scale).to_logical(scale);

                let pos_within_tile = pos - tile_pos;
                
                // Check if point is within this tile
                if tile.hit(pos_within_tile).is_some() {
                    let size = tile.tile_size().to_f64();
                    
                    // Determine resize edges based on position within tile (thirds)
                    let mut edges = ResizeEdge::empty();
                    if pos_within_tile.x < size.w / 3. {
                        edges |= ResizeEdge::LEFT;
                    } else if 2. * size.w / 3. < pos_within_tile.x {
                        edges |= ResizeEdge::RIGHT;
                    }
                    if pos_within_tile.y < size.h / 3. {
                        edges |= ResizeEdge::TOP;
                    } else if 2. * size.h / 3. < pos_within_tile.y {
                        edges |= ResizeEdge::BOTTOM;
                    }
                    return Some(edges);
                }
            }
        }

        None
    }
}
