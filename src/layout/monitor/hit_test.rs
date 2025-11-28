// TEAM_013: Hit testing methods extracted from monitor.rs
//!
//! This module contains hit testing and geometry query methods.

use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::layout::monitor::{InsertWorkspace, Monitor};
use crate::layout::{HitType, LayoutElement};
use crate::utils::ResizeEdge;

impl<W: LayoutElement> Monitor<W> {
    // =========================================================================
    // Workspace hit testing
    // =========================================================================

    pub fn workspace_under(
        &self,
        pos_within_output: Point<f64, Logical>,
    ) -> Option<(&crate::layout::row::Row<W>, Rectangle<f64, Logical>)> {
        let (row, geo) = self.canvas.workspaces().find_map(|(_, row)| {
            // Get the row geometry - this is a simplified approach
            // TODO: TEAM_023: Implement proper row geometry calculation
            let geo = Rectangle::from_loc_and_size((0., 0.), self.view_size);
            
            // Extend width to entire output.
            let loc = Point::from((0., geo.loc.y));
            let size = Size::from((self.view_size.w, geo.size.h));
            let bounds = Rectangle::new(loc, size);

            bounds.contains(pos_within_output).then_some((row, geo))
        })?;
        Some((row, geo))
    }

    pub fn workspace_under_narrow(
        &self,
        pos_within_output: Point<f64, Logical>,
    ) -> Option<&crate::layout::row::Row<W>> {
        self.canvas.workspaces()
            .find_map(|(_, row)| {
                // Simplified geometry check - TODO: TEAM_023: Implement proper row geometry
                let geo = Rectangle::from_loc_and_size((0., 0.), self.view_size);
                geo.contains(pos_within_output).then_some(row)
            })
    }

    // =========================================================================
    // Window hit testing
    // =========================================================================

    pub fn window_under(&self, pos_within_output: Point<f64, Logical>) -> Option<(&W, HitType)> {
        let (ws, geo) = self.workspace_under(pos_within_output)?;

        // DEPRECATED(overview): Removed overview_progress zoom handling
        // TEAM_036: Row's window_under now returns Option<(&W, HitType)>
        let (win, hit) = ws.window_under(pos_within_output - geo.loc)?;
        Some((win, hit.offset_win_pos(geo.loc)))
    }

    pub fn resize_edges_under(&self, pos_within_output: Point<f64, Logical>) -> Option<ResizeEdge> {
        // DEPRECATED(overview): Removed overview_progress check
        let (ws, geo) = self.workspace_under(pos_within_output)?;
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
