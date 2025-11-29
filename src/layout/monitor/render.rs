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

        // TEAM_057: Compute insert hint render location from insert_hint
        self.insert_hint_render_loc = None;
        if let Some(ref hint) = self.insert_hint {
            // Find the row for this hint and compute the hint area
            let hint_area = match hint.workspace {
                InsertWorkspace::Existing(ws_id) => {
                    // Find the row with this workspace ID
                    if let Some(row_idx) = self.canvas.find_row_by_id(ws_id) {
                        self.canvas.rows().find_map(|(idx, row)| {
                            if idx == row_idx {
                                row.insert_hint_area(hint.position)
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    }
                }
                InsertWorkspace::NewAt(_) => {
                    // For new workspaces, use the active row's hint area
                    if let Some(row) = self.canvas.active_row() {
                        row.insert_hint_area(hint.position)
                    } else {
                        None
                    }
                }
            };

            if let Some(area) = hint_area {
                self.insert_hint_render_loc = Some(InsertHintRenderLoc {
                    workspace: hint.workspace,
                    location: area.loc,
                });

                // Update the insert hint element size
                let view_rect = Rectangle::from_loc_and_size((0., 0.), self.view_size);
                self.insert_hint_element.update_render_elements(
                    area.size,
                    view_rect,
                    hint.corner_radius,
                    self.scale.fractional_scale(),
                );
            }
        }
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

    // =========================================================================
    // TEAM_023: Canvas2D workspace render geometry methods
    // =========================================================================

    /// Returns the render geometry for all rows in the canvas.
    pub fn workspaces_render_geo(&self) -> impl Iterator<Item = Rectangle<f64, Logical>> + '_ {
        self.canvas.rows().map(|(_row_idx, row)| {
            // For Canvas2D, each row occupies the full width and has its own height
            let row_height = row.row_height();
            let y_offset = row.y_offset();

            Rectangle::new(
                Point::from((0.0, y_offset)),
                Size::from((self.view_size.w, row_height)),
            )
        })
    }

    /// Returns an iterator over rows with their render geometry.
    pub fn workspaces_with_render_geo(
        &self,
    ) -> impl Iterator<Item = (&crate::layout::row::Row<W>, Rectangle<f64, Logical>)> + '_ {
        let output_geo = Rectangle::from_size(self.view_size);

        self.canvas
            .rows()
            .map(|(utils::scale::MIN_LOGICAL_AREA, row)| {
                let row_height = row.row_height();
                let y_offset = row.y_offset();
                let geo = Rectangle::new(
                    Point::from((0.0, y_offset)),
                    Size::from((self.view_size.w, row_height)),
                );
                (row, geo)
            })
            // Cull out rows outside the output.
            .filter(move |(_row, geo)| geo.intersection(output_geo).is_some())
    }

    /// Returns an iterator over rows with their render geometry and indices.
    pub fn workspaces_with_render_geo_idx(
        &self,
    ) -> impl Iterator<Item = ((i32, &crate::layout::row::Row<W>), Rectangle<f64, Logical>)> + '_
    {
        let output_geo = Rectangle::from_size(self.view_size);

        self.canvas
            .rows()
            .map(|(row_idx, row)| {
                let row_height = row.row_height();
                let y_offset = row.y_offset();
                let geo = Rectangle::new(
                    Point::from((0.0, y_offset)),
                    Size::from((self.view_size.w, row_height)),
                );
                ((row_idx, row), geo)
            })
            // Cull out rows outside the output.
            .filter(move |(_row, geo)| geo.intersection(output_geo).is_some())
    }

    /// Returns a mutable iterator over rows with their render geometry.
    pub fn workspaces_with_render_geo_mut(
        &mut self,
        cull: bool,
    ) -> impl Iterator<Item = (&mut crate::layout::row::Row<W>, Rectangle<f64, Logical>)> + '_ {
        let output_geo = Rectangle::from_size(self.view_size);
        let view_size = self.view_size;

        self.canvas
            .rows_mut()
            // TEAM_035: Extract row from (i32, &mut Row) tuple
            .map(move |(_, row)| {
                let row_height = row.row_height();
                let y_offset = row.y_offset();
                let geo = Rectangle::new(
                    Point::from((0.0, y_offset)),
                    Size::from((view_size.w, row_height)),
                );
                (row, geo)
            })
            // Cull out rows outside the output.
            .filter(move |(_row, geo)| !cull || geo.intersection(output_geo).is_some())
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
