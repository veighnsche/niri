// TEAM_013: Rendering methods extracted from monitor.rs
//!
//! This module contains rendering-related methods for Monitor.

use std::iter::zip;

use niri_config::CornerRadius;
use smithay::backend::renderer::element::utils::{
    CropRenderElement, Relocate, RelocateRenderElement, RescaleRenderElement,
};
use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::layout::monitor::{
    InsertHintRenderLoc, InsertWorkspace, Monitor, MonitorInnerRenderElement, MonitorRenderElement,
};
use crate::layout::workspace::{Workspace, WorkspaceRenderElement};
use crate::layout::LayoutElement;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::RenderTarget;
use crate::utils::round_logical_in_physical;

impl<W: LayoutElement> Monitor<W> {
    // =========================================================================
    // Workspace geometry
    // =========================================================================

    pub fn workspaces_render_geo(&self) -> impl Iterator<Item = Rectangle<f64, Logical>> {
        let scale = self.scale.fractional_scale();
        let zoom = self.overview_zoom();

        let ws_size = self.workspace_size(zoom);
        let gap = self.workspace_gap(zoom);
        let ws_height_with_gap = ws_size.h + gap;

        let static_offset = (self.view_size.to_point() - ws_size.to_point()).downscale(2.);
        let static_offset = static_offset
            .to_physical_precise_round(scale)
            .to_logical(scale);

        let first_ws_y = -self.workspace_render_idx() * ws_height_with_gap;
        let first_ws_y = round_logical_in_physical(scale, first_ws_y);

        // Return position for one-past-last workspace too.
        (0..=self.workspaces.len()).map(move |idx| {
            let y = first_ws_y + idx as f64 * ws_height_with_gap;
            let loc = Point::from((0., y)) + static_offset;
            Rectangle::new(loc, ws_size)
        })
    }

    pub fn workspaces_with_render_geo(
        &self,
    ) -> impl Iterator<Item = (&Workspace<W>, Rectangle<f64, Logical>)> {
        let output_geo = Rectangle::from_size(self.view_size);

        let geo = self.workspaces_render_geo();
        zip(self.workspaces.iter(), geo)
            // Cull out workspaces outside the output.
            .filter(move |(_ws, geo)| geo.intersection(output_geo).is_some())
    }

    pub fn workspaces_with_render_geo_idx(
        &self,
    ) -> impl Iterator<Item = ((usize, &Workspace<W>), Rectangle<f64, Logical>)> {
        let output_geo = Rectangle::from_size(self.view_size);

        let geo = self.workspaces_render_geo();
        zip(self.workspaces.iter().enumerate(), geo)
            // Cull out workspaces outside the output.
            .filter(move |(_ws, geo)| geo.intersection(output_geo).is_some())
    }

    pub fn workspaces_with_render_geo_mut(
        &mut self,
        cull: bool,
    ) -> impl Iterator<Item = (&mut Workspace<W>, Rectangle<f64, Logical>)> {
        let output_geo = Rectangle::from_size(self.view_size);

        let geo = self.workspaces_render_geo();
        zip(self.workspaces.iter_mut(), geo)
            // Cull out workspaces outside the output.
            .filter(move |(_ws, geo)| !cull || geo.intersection(output_geo).is_some())
    }

    // =========================================================================
    // Update render elements
    // =========================================================================

    pub fn update_render_elements(&mut self, is_active: bool) {
        let mut insert_hint_ws_geo = None;
        let insert_hint_ws_id = self
            .insert_hint
            .as_ref()
            .and_then(|hint| hint.workspace.existing_id());

        for (ws, geo) in self.workspaces_with_render_geo_mut(true) {
            ws.update_render_elements(is_active);

            if Some(ws.id()) == insert_hint_ws_id {
                insert_hint_ws_geo = Some(geo);
            }
        }

        self.insert_hint_render_loc = None;
        if let Some(hint) = &self.insert_hint {
            match hint.workspace {
                InsertWorkspace::Existing(ws_id) => {
                    if let Some(ws) = self.workspaces.iter().find(|ws| ws.id() == ws_id) {
                        if let Some(mut area) = ws.insert_hint_area(hint.position) {
                            let scale = ws.scale().fractional_scale();
                            let view_size = ws.view_size();

                            // Make sure the hint is at least partially visible.
                            if matches!(
                                hint.position,
                                crate::layout::types::InsertPosition::NewColumn(_)
                            ) {
                                let zoom = self.overview_zoom();
                                let geo = insert_hint_ws_geo.unwrap();
                                let geo = geo.downscale(zoom);

                                area.loc.x = area.loc.x.max(-geo.loc.x - area.size.w / 2.);
                                area.loc.x =
                                    area.loc.x.min(geo.loc.x + geo.size.w - area.size.w / 2.);
                            }

                            // Round to physical pixels.
                            area = area.to_physical_precise_round(scale).to_logical(scale);

                            let view_rect = Rectangle::new(area.loc.upscale(-1.), view_size);
                            self.insert_hint_element.update_render_elements(
                                area.size,
                                view_rect,
                                hint.corner_radius,
                                scale,
                            );
                            self.insert_hint_render_loc = Some(InsertHintRenderLoc {
                                workspace: hint.workspace,
                                location: area.loc,
                            });
                        }
                    } else {
                        error!("insert hint workspace missing from monitor");
                    }
                }
                InsertWorkspace::NewAt(ws_idx) => {
                    let scale = self.scale.fractional_scale();
                    let zoom = self.overview_zoom();
                    let gap = self.workspace_gap(zoom);

                    let hint_gap = round_logical_in_physical(scale, gap * 0.1);
                    let hint_height = gap - hint_gap * 2.;

                    let next_ws_geo = self.workspaces_render_geo().nth(ws_idx).unwrap();
                    let hint_width = round_logical_in_physical(scale, next_ws_geo.size.w * 0.75);
                    let hint_x =
                        round_logical_in_physical(scale, (next_ws_geo.size.w - hint_width) / 2.);

                    let hint_loc_diff = Point::from((-hint_x, hint_height + hint_gap));
                    let hint_loc = next_ws_geo.loc - hint_loc_diff;
                    let hint_size = Size::from((hint_width, hint_height));

                    // Sometimes the hint ends up 1 px wider than necessary and/or 1 px
                    // narrower than necessary. The values here seem correct. Might have to do with
                    // how zooming out currently doesn't round to output scale properly.

                    // Compute view rect as if we're above the next workspace (rather than below
                    // the previous one).
                    let view_rect = Rectangle::new(hint_loc_diff, next_ws_geo.size);

                    self.insert_hint_element.update_render_elements(
                        hint_size,
                        view_rect,
                        CornerRadius::default(),
                        scale,
                    );
                    self.insert_hint_render_loc = Some(InsertHintRenderLoc {
                        workspace: hint.workspace,
                        location: hint_loc,
                    });
                }
            }
        }
    }

    // =========================================================================
    // Render methods
    // =========================================================================

    pub fn render_above_top_layer(&self) -> bool {
        // Render above the top layer only if the view is stationary.
        if self.workspace_switch.is_some() || self.overview_progress.is_some() {
            return false;
        }

        let ws = &self.workspaces[self.active_workspace_idx];
        ws.render_above_top_layer()
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

    pub fn render_elements<'a, R: NiriRenderer>(
        &'a self,
        renderer: &'a mut R,
        target: RenderTarget,
        focus_ring: bool,
    ) -> impl Iterator<
        Item = (
            Rectangle<f64, Logical>,
            MonitorRenderElement<R>,
            impl Iterator<Item = MonitorRenderElement<R>> + 'a,
        ),
    > {
        let _span = tracy_client::span!("Monitor::render_elements");

        let scale = self.scale.fractional_scale();
        // Ceil the height in physical pixels.
        let height = (self.view_size.h * scale).ceil() as i32;

        // Crop the elements to prevent them overflowing, currently visible during a workspace
        // switch.
        //
        // HACK: crop to infinite bounds at least horizontally where we
        // know there's no workspace joining or monitor bounds, otherwise
        // it will cut pixel shaders and mess up the coordinate space.
        // There's also a damage tracking bug which causes glitched
        // rendering for maximized GTK windows.
        //
        // FIXME: use proper bounds after fixing the Crop element.
        let crop_bounds = if self.workspace_switch.is_some() || self.overview_progress.is_some() {
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

        let zoom = self.overview_zoom();

        // Draw the insert hint.
        let mut insert_hint = None;
        if !self.options.layout.insert_hint.off {
            if let Some(render_loc) = self.insert_hint_render_loc {
                if let InsertWorkspace::Existing(workspace_id) = render_loc.workspace {
                    insert_hint = Some((
                        workspace_id,
                        self.insert_hint_element
                            .render(renderer, render_loc.location),
                    ));
                }
            }
        }

        self.workspaces_with_render_geo().map(move |(ws, geo)| {
            let map_ws_contents = move |elem: WorkspaceRenderElement<R>| {
                let elem = CropRenderElement::from_element(elem, scale, crop_bounds)?;
                let elem = MonitorInnerRenderElement::Workspace(elem);
                Some(elem)
            };

            let (floating, scrolling) = ws.render_elements(renderer, target, focus_ring);
            let floating = floating.filter_map(map_ws_contents);
            let scrolling = scrolling.filter_map(map_ws_contents);

            let hint = if matches!(insert_hint, Some((hint_ws_id, _)) if hint_ws_id == ws.id()) {
                let iter = insert_hint.take().unwrap().1;
                let iter = iter.filter_map(move |elem| {
                    let elem = CropRenderElement::from_element(elem, scale, crop_bounds)?;
                    let elem = MonitorInnerRenderElement::InsertHint(elem);
                    Some(elem)
                });
                Some(iter)
            } else {
                None
            };
            let hint = hint.into_iter().flatten();

            let iter = floating.chain(hint).chain(scrolling);

            let scale_relocate = move |elem| {
                let elem = RescaleRenderElement::from_element(elem, Point::from((0, 0)), zoom);
                RelocateRenderElement::from_element(
                    elem,
                    // The offset we get from workspaces_with_render_positions() is already
                    // rounded to physical pixels, but it's in the logical coordinate
                    // space, so we need to convert it to physical.
                    geo.loc.to_physical_precise_round(scale),
                    Relocate::Relative,
                )
            };

            let iter = iter.map(scale_relocate);

            let background = ws.render_background();
            let background = scale_relocate(MonitorInnerRenderElement::SolidColor(background));

            (geo, background, iter)
        })
    }

    pub fn render_workspace_shadows<'a, R: NiriRenderer>(
        &'a self,
        renderer: &'a mut R,
    ) -> impl Iterator<Item = MonitorRenderElement<R>> + 'a {
        let _span = tracy_client::span!("Monitor::render_workspace_shadows");

        let scale = self.scale.fractional_scale();
        let zoom = self.overview_zoom();
        let overview_clamped_progress = self.overview_progress.as_ref().map(|p| p.clamped_value());

        self.workspaces_with_render_geo()
            .flat_map(move |(ws, geo)| {
                let shadow = overview_clamped_progress.map(|value| {
                    ws.render_shadow(renderer)
                        .map(move |elem| elem.with_alpha(value.clamp(0., 1.) as f32))
                        .map(MonitorInnerRenderElement::Shadow)
                });
                let iter = shadow.into_iter().flatten();

                iter.map(move |elem| {
                    let elem = RescaleRenderElement::from_element(elem, Point::from((0, 0)), zoom);
                    RelocateRenderElement::from_element(
                        elem,
                        geo.loc.to_physical_precise_round(scale),
                        Relocate::Relative,
                    )
                })
            })
    }
}
