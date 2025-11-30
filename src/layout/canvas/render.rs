// TEAM_008: Rendering split from mod.rs
//!
//! This module handles rendering the canvas and its elements.

use smithay::utils::Scale;

use crate::layout::canvas::{Canvas2D, Canvas2DRenderElement};
use crate::layout::LayoutElement;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::RenderTarget;

impl<W: LayoutElement> Canvas2D<W> {
    // TEAM_007: Added render_elements method

    /// Renders all elements in the canvas.
    ///
    /// Returns render elements for all visible rows, with camera offset applied.
    /// NOTE: In Smithay, elements rendered FIRST appear on TOP (front-to-back order).
    pub fn render_elements<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        target: RenderTarget,
        focus_ring: bool,
    ) -> Vec<Canvas2DRenderElement<R>> {
        let mut rv = vec![];
        // TEAM_049: Apply camera offset to render elements for proper scrolling
        let _camera = self.camera_position();
        let _scale = Scale::from(self.scale);

        let active_row_idx = self.active_row_idx;

        // TEAM_106: Render floating layer FIRST (so it appears on top)
        let floating_focus_ring = focus_ring && self.floating_is_active;
        let view_rect = self.working_area;
        let floating_elements =
            self.floating
                .render_elements(renderer, view_rect, target, floating_focus_ring);
        for elem in floating_elements {
            rv.push(elem.into());
        }

        // Then render active row (with focus ring unless floating is active)
        if let Some(row) = self.rows.get(&active_row_idx) {
            let row_focus_ring = focus_ring && !self.floating_is_active;
            let row_elements = row.render_elements(renderer, target, row_focus_ring);
            for elem in row_elements {
                rv.push(elem.into());
            }
        }

        // Finally render non-active rows (at the back)
        for (&row_idx, row) in &self.rows {
            if row_idx == active_row_idx {
                continue;
            }

            let row_elements = row.render_elements(renderer, target, false);
            for elem in row_elements {
                rv.push(elem.into());
            }
        }

        rv
    }

    /// Updates render elements for all rows.
    pub fn update_render_elements(&mut self) {
        let active_row_idx = self.active_row_idx;
        let floating_is_active = self.floating_is_active;

        for (&row_idx, row) in &mut self.rows {
            // Active row is only truly active if floating is not active
            let is_active_row = row_idx == active_row_idx && !floating_is_active;
            row.update_render_elements(is_active_row);
        }

        // TEAM_009: Update floating render elements
        let view_rect = self.working_area;
        self.floating
            .update_render_elements(floating_is_active, view_rect);
    }
}
