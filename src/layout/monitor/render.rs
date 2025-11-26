// TEAM_013: Rendering methods extracted from monitor.rs
// TEAM_022: Updated to use Canvas2D instead of workspaces
//!
//! This module contains rendering-related methods for Monitor.

use smithay::backend::renderer::element::utils::{
    CropRenderElement, Relocate, RelocateRenderElement, RescaleRenderElement,
};
use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::layout::monitor::{
    InsertHintRenderLoc, InsertWorkspace, Monitor, MonitorInnerRenderElement, MonitorRenderElement,
};
use crate::layout::LayoutElement;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::RenderTarget;

impl<W: LayoutElement> Monitor<W> {
    // =========================================================================
    // TEAM_022: Canvas geometry helpers
    // =========================================================================

    /// Returns the canvas view size.
    pub fn workspace_size(&self, _zoom: f64) -> Size<f64, Logical> {
        self.view_size
    }

    /// Returns the gap between rows (0 for canvas).
    pub fn workspace_gap(&self, _zoom: f64) -> f64 {
        0.0
    }

    // =========================================================================
    // Update render elements
    // =========================================================================

    pub fn update_render_elements(&mut self, is_active: bool) {
        // TEAM_022: Delegate to canvas
        if is_active {
            self.canvas.update_render_elements();
        }

        // TEAM_022: Insert hint handling simplified
        self.insert_hint_render_loc = None;
        // TODO(TEAM_022): Implement proper insert hint rendering with canvas
    }

    // =========================================================================
    // Render methods
    // =========================================================================

    pub fn render_above_top_layer(&self) -> bool {
        // Render above the top layer only if the view is stationary.
        if self.workspace_switch.is_some() {
            return false;
        }

        // TEAM_022: Check canvas for render above top layer
        if let Some(row) = self.canvas.active_row() {
            row.render_above_top_layer()
        } else {
            false
        }
    }

    pub fn render_insert_hint_between_workspaces<R: NiriRenderer>(
        &self,
        renderer: &mut R,
    ) -> impl Iterator<Item = MonitorRenderElement<R>> {
        let mut rv = None;

        if !self.options.layout.insert_hint.off {
            if let Some(render_loc) = self.insert_hint_render_loc {
                if let InsertWorkspace::NewAt(_) = render_loc.workspace {
                    let iter = self
                        .insert_hint_element
                        .render(renderer, render_loc.location)
                        .map(MonitorInnerRenderElement::UncroppedInsertHint);
                    rv = Some(iter);
                }
            }
        }

        rv.into_iter().flatten().map(|elem| {
            let elem = RescaleRenderElement::from_element(elem, Point::default(), 1.);
            RelocateRenderElement::from_element(elem, Point::default(), Relocate::Relative)
        })
    }

    /// TEAM_022: Simplified render_elements using Canvas2D
    pub fn render_elements<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        target: RenderTarget,
        focus_ring: bool,
    ) -> Vec<MonitorRenderElement<R>> {
        let _span = tracy_client::span!("Monitor::render_elements");

        let scale = self.scale.fractional_scale();
        let height = (self.view_size.h * scale).ceil() as i32;

        // Crop bounds for overflow prevention
        let crop_bounds = if self.workspace_switch.is_some() {
            Rectangle::new(
                Point::from((-i32::MAX / 2, 0)),
                Size::from((i32::MAX, height)),
            )
        } else {
            Rectangle::new(
                Point::from((-i32::MAX / 2, -i32::MAX / 2)),
                Size::from((i32::MAX, i32::MAX)),
            )
        };

        // Get elements from canvas
        let canvas_elements = self.canvas.render_elements(renderer, target, focus_ring);

        // Convert canvas elements to monitor elements
        canvas_elements
            .into_iter()
            .filter_map(|elem| {
                let elem = CropRenderElement::from_element(elem, scale, crop_bounds)?;
                let elem = MonitorInnerRenderElement::Canvas(elem);
                let elem = RescaleRenderElement::from_element(elem, Point::from((0, 0)), 1.0);
                Some(RelocateRenderElement::from_element(
                    elem,
                    Point::from((0, 0)),
                    Relocate::Relative,
                ))
            })
            .collect()
    }
}
