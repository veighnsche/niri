// TEAM_013: Hit testing methods extracted from monitor.rs
//!
//! This module contains hit testing and geometry query methods.

use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::layout::monitor::{InsertWorkspace, Monitor};
use crate::layout::{HitType, LayoutElement};
use crate::utils::ResizeEdge;

impl<W: LayoutElement> Monitor<W> {
    // =========================================================================
    // Row hit testing
    // =========================================================================

    pub fn row_under(
        &self,
        pos_within_output: Point<f64, Logical>,
    ) -> Option<(&crate::layout::row::Row<W>, Rectangle<f64, Logical>)> {
        let (row, geo) = self.canvas.rows().find_map(|(_, row)| {
            // TEAM_023: Implement proper row geometry calculation
            // Use same pattern as render geometry but account for camera offset
            let row_height = row.row_height();
            let y_offset = row.y_offset();
            
            // Get camera position to translate world coordinates to screen coordinates
            let camera = self.canvas.camera_position();
            
            // Calculate row geometry in world space
            let world_geo = Rectangle::new(
                Point::from((0.0, y_offset)),
                Size::from((self.view_size.w, row_height)),
            );
            
            // Translate to screen space by subtracting camera offset
            let screen_geo = Rectangle::new(
                Point::from((world_geo.loc.x - camera.x, world_geo.loc.y - camera.y)),
                world_geo.size,
            );

            screen_geo.contains(pos_within_output).then_some((row, world_geo))
        })?;
        Some((row, geo))
    }

    pub fn row_under_narrow(
        &self,
        pos_within_output: Point<f64, Logical>,
    ) -> Option<&crate::layout::row::Row<W>> {
        self.canvas.rows()
            .find_map(|(_, row)| {
                // TEAM_023: Implement proper row geometry calculation
                // Use same pattern as render geometry but account for camera offset
                let row_height = row.row_height();
                let y_offset = row.y_offset();
                
                // Get camera position to translate world coordinates to screen coordinates
                let camera = self.canvas.camera_position();
                
                // Calculate row geometry in world space
                let world_geo = Rectangle::new(
                    Point::from((0.0, y_offset)),
                    Size::from((self.view_size.w, row_height)),
                );
                
                // Translate to screen space by subtracting camera offset
                let screen_geo = Rectangle::new(
                    Point::from((world_geo.loc.x - camera.x, world_geo.loc.y - camera.y)),
                    world_geo.size,
                );

                screen_geo.contains(pos_within_output).then_some(row)
            })
    }

    // =========================================================================
    // Window hit testing
    // =========================================================================

    pub fn window_under(&self, pos_within_output: Point<f64, Logical>) -> Option<(&W, HitType)> {
        let (ws, geo) = self.row_under(pos_within_output)?;

        // DEPRECATED(overview): Removed overview_progress zoom handling
        // TEAM_036: Row's window_under now returns Option<(&W, HitType)>
        let (win, hit) = ws.window_under(pos_within_output - geo.loc)?;
        Some((win, hit.offset_win_pos(geo.loc)))
    }

    pub fn resize_edges_under(&self, pos_within_output: Point<f64, Logical>) -> Option<ResizeEdge> {
        // DEPRECATED(overview): Removed overview_progress check
        let (ws, geo) = self.row_under(pos_within_output)?;
        ws.resize_edges_under(pos_within_output - geo.loc)
    }

    // =========================================================================
    // Insert position
    // =========================================================================

    pub(in crate::layout) fn insert_position(
        &self,
        pos_within_output: Point<f64, Logical>,
    ) -> (InsertWorkspace, Rectangle<f64, Logical>) {
        let mut iter = self.workspaces_with_render_geo_idx();

        let dummy = Rectangle::default();

        // Monitors always have at least one workspace.
        let ((idx, ws), geo) = iter.next().unwrap();

        // Check if above first.
        if pos_within_output.y < geo.loc.y {
            return (InsertWorkspace::NewAt(idx as usize), dummy);
        }

        let contains = move |geo: Rectangle<f64, Logical>| {
            geo.loc.y <= pos_within_output.y && pos_within_output.y < geo.loc.y + geo.size.h
        };

        // Check first.
        // TEAM_050: Use actual workspace ID, not synthetic ID from index
        if contains(geo) {
            return (InsertWorkspace::Existing(ws.id()), geo);
        }

        let mut last_geo = geo;
        let mut last_idx = idx;
        for ((idx, ws), geo) in iter {
            // Check gap above.
            let gap_loc = Point::from((last_geo.loc.x, last_geo.loc.y + last_geo.size.h));
            let gap_size = Size::from((geo.size.w, geo.loc.y - gap_loc.y));
            let gap_geo = Rectangle::new(gap_loc, gap_size);
            if contains(gap_geo) {
                return (InsertWorkspace::NewAt(idx as usize), dummy);
            }

            // Check workspace itself.
            if contains(geo) {
                return (InsertWorkspace::Existing(ws.id()), geo);
            }

            last_geo = geo;
            last_idx = idx;
        }

        // Anything below.
        (InsertWorkspace::NewAt((last_idx + 1) as usize), dummy)
    }

    // =========================================================================
    // Active tile geometry
    // =========================================================================

    /// Returns the geometry of the active tile relative to and clamped to the output.
    ///
    /// During animations, assumes the final view position.
    pub fn active_tile_visual_rectangle(&self) -> Option<Rectangle<f64, Logical>> {
        // DEPRECATED(overview): Removed overview_open check
        self.active_workspace_ref().and_then(|ws| ws.active_tile_visual_rectangle())
    }
}
