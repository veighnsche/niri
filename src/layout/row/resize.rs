// TEAM_007: Interactive resize ported from ScrollingSpace
//!
//! This module handles interactive window resizing within a row.

use niri_ipc::SizeChange;
use smithay::utils::{Logical, Point};

use super::Row;
use crate::layout::types::InteractiveResize;
use crate::layout::{InteractiveResizeData, LayoutElement};
use crate::utils::ResizeEdge;

impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // Interactive Resize
    // =========================================================================

    /// Begins an interactive resize operation.
    pub fn interactive_resize_begin(&mut self, window: W::Id, edges: ResizeEdge) -> bool {
        if self.interactive_resize.is_some() {
            return false;
        }

        let col = self
            .columns
            .iter_mut()
            .find(|col| col.contains(&window));

        let Some(col) = col else {
            return false;
        };

        if !col.pending_sizing_mode().is_normal() {
            return false;
        }

        let tile = col
            .tiles
            .iter_mut()
            .find(|tile| tile.window().id() == &window)
            .unwrap();

        let original_window_size = tile.window_size();

        let resize = InteractiveResize {
            window,
            original_window_size,
            data: InteractiveResizeData { edges },
        };
        self.interactive_resize = Some(resize);

        self.view_offset_x.stop_anim_and_gesture();

        true
    }

    /// Updates an interactive resize operation with new delta.
    pub fn interactive_resize_update(
        &mut self,
        window: &W::Id,
        delta: Point<f64, Logical>,
    ) -> bool {
        let Some(resize) = &self.interactive_resize else {
            return false;
        };

        if window != &resize.window {
            return false;
        }

        let is_centering = self.is_centering_focused_column();

        let col = self
            .columns
            .iter_mut()
            .find(|col| col.contains(window));

        let Some(col) = col else {
            return false;
        };

        let tile_idx = col
            .tiles
            .iter()
            .position(|tile| tile.window().id() == window)
            .unwrap();

        if resize.data.edges.intersects(ResizeEdge::LEFT_RIGHT) {
            let mut dx = delta.x;
            if resize.data.edges.contains(ResizeEdge::LEFT) {
                dx = -dx;
            };

            if is_centering {
                dx *= 2.;
            }

            let window_width = (resize.original_window_size.w + dx).round() as i32;
            col.set_column_width(SizeChange::SetFixed(window_width), Some(tile_idx), false);
        }

        if resize.data.edges.intersects(ResizeEdge::TOP_BOTTOM) {
            // Prevent the simplest case of weird resizing (top edge when this is the topmost
            // window).
            if !(resize.data.edges.contains(ResizeEdge::TOP) && tile_idx == 0) {
                let mut dy = delta.y;
                if resize.data.edges.contains(ResizeEdge::TOP) {
                    dy = -dy;
                };

                // FIXME: some smarter height distribution would be nice here so that vertical
                // resizes work as expected in more cases.

                let window_height = (resize.original_window_size.h + dy).round() as i32;
                col.set_window_height(SizeChange::SetFixed(window_height), Some(tile_idx), false);
            }
        }

        true
    }

    /// Ends an interactive resize operation.
    pub fn interactive_resize_end(&mut self, window: Option<&W::Id>) {
        let Some(resize) = &self.interactive_resize else {
            return;
        };

        if let Some(window) = window {
            if window != &resize.window {
                return;
            }

            // Animate the active window into view right away.
            if self.columns[self.active_column_idx].contains(window) {
                self.animate_view_offset_to_column(None, self.active_column_idx, None);
            }
        }

        self.interactive_resize = None;
    }

    /// Returns the current interactive resize data, if any.
    pub fn interactive_resize_data(&self) -> Option<InteractiveResizeData> {
        self.interactive_resize.as_ref().map(|r| r.data)
    }

    /// Returns the window being interactively resized, if any.
    pub fn interactive_resize_window(&self) -> Option<&W::Id> {
        self.interactive_resize.as_ref().map(|r| &r.window)
    }
}
