// TEAM_063: Tile render elements and rendering methods
//!
//! This module handles rendering for tiles including the render element type,
//! render methods, and snapshot functionality.

use niri_config::{Color, CornerRadius, GradientInterpolation};
use smithay::backend::renderer::element::{Element, Kind};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::utils::{Logical, Point, Rectangle, Scale};

use super::Tile;
use crate::layout::elements::focus_ring::FocusRingRenderElement;
use crate::layout::elements::opening_window::OpeningWindowRenderElement;
use crate::layout::{LayoutElement, LayoutElementRenderElement};
use crate::niri_render_elements;
use crate::render_helpers::border::BorderRenderElement;
use crate::render_helpers::clipped_surface::ClippedSurfaceRenderElement;
use crate::render_helpers::damage::ExtraDamage;
use crate::render_helpers::offscreen::OffscreenRenderElement;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::resize::ResizeRenderElement;
use crate::render_helpers::shadow::ShadowRenderElement;
use crate::render_helpers::snapshot::RenderSnapshot;
use crate::render_helpers::solid_color::{SolidColorBuffer, SolidColorRenderElement};
use crate::render_helpers::RenderTarget;

// TEAM_063: TileRenderElement type moved from mod.rs
niri_render_elements! {
    TileRenderElement<R> => {
        LayoutElement = LayoutElementRenderElement<R>,
        FocusRing = FocusRingRenderElement,
        SolidColor = SolidColorRenderElement,
        Opening = OpeningWindowRenderElement,
        Resize = ResizeRenderElement,
        Border = BorderRenderElement,
        Shadow = ShadowRenderElement,
        ClippedSurface = ClippedSurfaceRenderElement<R>,
        Offscreen = OffscreenRenderElement,
        ExtraDamage = ExtraDamage,
    }
}

pub type TileRenderSnapshot =
    RenderSnapshot<TileRenderElement<GlesRenderer>, TileRenderElement<GlesRenderer>>;

impl<W: LayoutElement> Tile<W> {
    pub fn update_render_elements(&mut self, is_active: bool, view_rect: Rectangle<f64, Logical>) {
        let rules = self.window.rules();
        let animated_tile_size = self.animated_tile_size();
        let expanded_progress = self.expanded_progress();

        let draw_border_with_background = rules
            .draw_border_with_background
            .unwrap_or_else(|| !self.window.has_ssd());
        let border_width = self.visual_border_width().unwrap_or(0.);

        // Do the inverse of tile_size() in order to handle the unfullscreen animation for windows
        // that were smaller than the fullscreen size, and therefore their animated_window_size() is
        // currently much smaller than the tile size.
        let mut border_window_size = animated_tile_size;
        border_window_size.w -= border_width * 2.;
        border_window_size.h -= border_width * 2.;

        let radius = rules
            .geometry_corner_radius
            .map_or(CornerRadius::default(), |radius| {
                radius.expanded_by(border_width as f32)
            })
            .scaled_by(1. - expanded_progress as f32);
        self.border.update_render_elements(
            border_window_size,
            is_active,
            !draw_border_with_background,
            self.window.is_urgent(),
            Rectangle::new(
                view_rect.loc - Point::from((border_width, border_width)),
                view_rect.size,
            ),
            radius,
            self.scale,
            1. - expanded_progress as f32,
        );

        let radius = if self.visual_border_width().is_some() {
            radius
        } else {
            rules
                .geometry_corner_radius
                .unwrap_or_default()
                .scaled_by(1. - expanded_progress as f32)
        };
        self.shadow.update_render_elements(
            animated_tile_size,
            is_active,
            radius,
            self.scale,
            1. - expanded_progress as f32,
        );

        let draw_focus_ring_with_background = if self.border.is_off() {
            draw_border_with_background
        } else {
            false
        };
        let radius = radius.expanded_by(self.focus_ring.width() as f32);
        self.focus_ring.update_render_elements(
            animated_tile_size,
            is_active,
            !draw_focus_ring_with_background,
            self.window.is_urgent(),
            view_rect,
            radius,
            self.scale,
            1. - expanded_progress as f32,
        );

        self.fullscreen_backdrop.resize(animated_tile_size);
    }

    pub(super) fn render_inner<'a, R: NiriRenderer + 'a>(
        &'a self,
        renderer: &mut R,
        location: Point<f64, Logical>,
        focus_ring: bool,
        target: RenderTarget,
    ) -> impl Iterator<Item = TileRenderElement<R>> + 'a {
        let _span = tracy_client::span!("Tile::render_inner");

        let scale = Scale::from(self.scale);
        let fullscreen_progress = self.fullscreen_progress();
        let expanded_progress = self.expanded_progress();

        let win_alpha = if self.window.is_ignoring_opacity_window_rule() {
            1.
        } else {
            let alpha = self.window.rules().opacity.unwrap_or(1.).clamp(0., 1.);

            // Interpolate towards alpha = 1. at fullscreen.
            let p = fullscreen_progress as f32;
            alpha * (1. - p) + 1. * p
        };

        // This is here rather than in render_offset() because render_offset() is currently assumed
        // by the code to be temporary. So, for example, interactive move will try to "grab" the
        // tile at its current render offset and reset the render offset to zero by cancelling the
        // tile move animations. On the other hand, bob_offset() is not resettable, so adding it in
        // render_offset() would cause obvious animation glitches.
        //
        // This isn't to say that adding it here is perfect; indeed, it kind of breaks view_rect
        // passed to update_render_elements(). But, it works well enough for what it is.
        let location = location + self.bob_offset();

        let window_loc = self.window_loc();
        let window_size = self.window_size().to_f64();
        let animated_window_size = self.animated_window_size();
        let window_render_loc = location + window_loc;
        let area = Rectangle::new(window_render_loc, animated_window_size);

        let rules = self.window.rules();

        // Clip to geometry including during the fullscreen animation to help with buggy clients
        // that submit a full-sized buffer before acking the fullscreen state (Firefox).
        let clip_to_geometry = fullscreen_progress < 1. && rules.clip_to_geometry == Some(true);
        let radius = rules
            .geometry_corner_radius
            .unwrap_or_default()
            .scaled_by(1. - expanded_progress as f32);

        // If we're resizing, try to render a shader, or a fallback.
        let mut resize_shader = None;
        let mut resize_popups = None;
        let mut resize_fallback = None;

        if let Some(resize) = &self.resize_animation {
            resize_popups = Some(
                self.window
                    .render_popups(renderer, window_render_loc, scale, win_alpha, target)
                    .into_iter()
                    .map(Into::into),
            );

            if ResizeRenderElement::has_shader(renderer) {
                let gles_renderer = renderer.as_gles_renderer();

                if let Some(texture_from) = resize.snapshot.texture(gles_renderer, scale, target) {
                    let window_elements = self.window.render_normal(
                        gles_renderer,
                        Point::from((0., 0.)),
                        scale,
                        1.,
                        target,
                    );

                    let current = resize
                        .offscreen
                        .render(gles_renderer, scale, &window_elements)
                        .map_err(|err| warn!("error rendering window to texture: {err:?}"))
                        .ok();

                    // Clip blocked-out resizes unconditionally because they use solid color render
                    // elements.
                    let clip_to_geometry = if target
                        .should_block_out(resize.snapshot.block_out_from)
                        && target.should_block_out(rules.block_out_from)
                    {
                        true
                    } else {
                        clip_to_geometry
                    };

                    if let Some((elem_current, _sync_point, mut data)) = current {
                        let texture_current = elem_current.texture().clone();
                        // The offset and size are computed in physical pixels and converted to
                        // logical with the same `scale`, so converting them back with rounding
                        // inside the geometry() call gives us the same physical result back.
                        let texture_current_geo = elem_current.geometry(scale);

                        let elem = ResizeRenderElement::new(
                            area,
                            scale,
                            texture_from.clone(),
                            resize.snapshot.size,
                            (texture_current, texture_current_geo),
                            window_size,
                            resize.anim.value() as f32,
                            resize.anim.clamped_value().clamp(0., 1.) as f32,
                            radius,
                            clip_to_geometry,
                            win_alpha,
                        );

                        // We're drawing the resize shader, not the offscreen directly.
                        data.id = elem.id().clone();

                        // This is not a problem for split popups as the code will look for them by
                        // original id when it doesn't find them on the offscreen.
                        self.window.set_offscreen_data(Some(data));
                        resize_shader = Some(elem.into());
                    }
                }
            }

            if resize_shader.is_none() {
                let fallback_buffer = SolidColorBuffer::new(area.size, [1., 0., 0., 1.]);
                resize_fallback = Some(
                    SolidColorRenderElement::from_buffer(
                        &fallback_buffer,
                        area.loc,
                        win_alpha,
                        Kind::Unspecified,
                    )
                    .into(),
                );
            }
        }

        // If we're not resizing, render the window itself.
        let mut window_surface = None;
        let mut window_popups = None;
        let mut rounded_corner_damage = None;
        let has_border_shader = BorderRenderElement::has_shader(renderer);
        if resize_shader.is_none() && resize_fallback.is_none() {
            let window = self
                .window
                .render(renderer, window_render_loc, scale, win_alpha, target);

            let geo = Rectangle::new(window_render_loc, window_size);
            let radius = radius.fit_to(window_size.w as f32, window_size.h as f32);

            let clip_shader = ClippedSurfaceRenderElement::shader(renderer).cloned();

            if clip_to_geometry && clip_shader.is_some() {
                let damage = self.rounded_corner_damage.element();
                rounded_corner_damage = Some(damage.with_location(window_render_loc).into());
            }

            window_surface = Some(window.normal.into_iter().map(move |elem| match elem {
                LayoutElementRenderElement::Wayland(elem) => {
                    // If we should clip to geometry, render a clipped window.
                    if clip_to_geometry {
                        if let Some(shader) = clip_shader.clone() {
                            if ClippedSurfaceRenderElement::will_clip(&elem, scale, geo, radius) {
                                return ClippedSurfaceRenderElement::new(
                                    elem,
                                    scale,
                                    geo,
                                    shader.clone(),
                                    radius,
                                )
                                .into();
                            }
                        }
                    }

                    // Otherwise, render it normally.
                    LayoutElementRenderElement::Wayland(elem).into()
                }
                LayoutElementRenderElement::SolidColor(elem) => {
                    // In this branch we're rendering a blocked-out window with a solid
                    // color. We need to render it with a rounded corner shader even if
                    // clip_to_geometry is false, because in this case we're assuming that
                    // the unclipped window CSD already has corners rounded to the
                    // user-provided radius, so our blocked-out rendering should match that
                    // radius.
                    if radius != CornerRadius::default() && has_border_shader {
                        return BorderRenderElement::new(
                            geo.size,
                            Rectangle::from_size(geo.size),
                            GradientInterpolation::default(),
                            Color::from_color32f(elem.color()),
                            Color::from_color32f(elem.color()),
                            0.,
                            Rectangle::from_size(geo.size),
                            0.,
                            radius,
                            scale.x as f32,
                            1.,
                        )
                        .with_location(geo.loc)
                        .into();
                    }

                    // Otherwise, render the solid color as is.
                    LayoutElementRenderElement::SolidColor(elem).into()
                }
            }));

            window_popups = Some(window.popups.into_iter().map(Into::into));
        }

        let rv = resize_popups
            .into_iter()
            .flatten()
            .chain(resize_shader)
            .chain(resize_fallback)
            .chain(window_popups.into_iter().flatten())
            .chain(rounded_corner_damage)
            .chain(window_surface.into_iter().flatten());

        let elem = (fullscreen_progress > 0.).then(|| {
            let alpha = fullscreen_progress as f32;

            // During the un/fullscreen animation, render a border element in order to use the
            // animated corner radius.
            if fullscreen_progress < 1. && has_border_shader {
                let border_width = self.visual_border_width().unwrap_or(0.);
                let radius = rules
                    .geometry_corner_radius
                    .map_or(CornerRadius::default(), |radius| {
                        radius.expanded_by(border_width as f32)
                    })
                    .scaled_by(1. - expanded_progress as f32);

                let size = self.fullscreen_backdrop.size();
                let color = self.fullscreen_backdrop.color();
                BorderRenderElement::new(
                    size,
                    Rectangle::from_size(size),
                    GradientInterpolation::default(),
                    Color::from_color32f(color),
                    Color::from_color32f(color),
                    0.,
                    Rectangle::from_size(size),
                    0.,
                    radius,
                    scale.x as f32,
                    alpha,
                )
                .with_location(location)
                .into()
            } else {
                SolidColorRenderElement::from_buffer(
                    &self.fullscreen_backdrop,
                    location,
                    alpha,
                    Kind::Unspecified,
                )
                .into()
            }
        });
        let rv = rv.chain(elem);

        let elem = self.visual_border_width().map(|width| {
            self.border
                .render(renderer, location + Point::from((width, width)))
                .map(Into::into)
        });
        let rv = rv.chain(elem.into_iter().flatten());

        // Hide the focus ring when maximized/fullscreened. It's not normally visible anyway due to
        // being outside the monitor or obscured by a solid colored bar, but it is visible under
        // semitransparent bars in maximized state (which is a bit weird) and in the overview (also
        // a bit weird).
        let elem = (focus_ring && expanded_progress < 1.)
            .then(|| self.focus_ring.render(renderer, location).map(Into::into));
        let rv = rv.chain(elem.into_iter().flatten());

        let elem = (expanded_progress < 1.)
            .then(|| self.shadow.render(renderer, location).map(Into::into));
        rv.chain(elem.into_iter().flatten())
    }

    pub fn render<'a, R: NiriRenderer + 'a>(
        &'a self,
        renderer: &mut R,
        location: Point<f64, Logical>,
        focus_ring: bool,
        target: RenderTarget,
    ) -> impl Iterator<Item = TileRenderElement<R>> + 'a {
        let _span = tracy_client::span!("Tile::render");

        let scale = Scale::from(self.scale);

        let tile_alpha = self
            .alpha_animation
            .as_ref()
            .map_or(1., |alpha| alpha.anim.clamped_value()) as f32;

        let mut open_anim_elem = None;
        let mut alpha_anim_elem = None;
        let mut window_elems = None;

        self.window().set_offscreen_data(None);

        if let Some(open) = &self.open_animation {
            let renderer = renderer.as_gles_renderer();
            let elements = self.render_inner(renderer, Point::from((0., 0.)), focus_ring, target);
            let elements = elements.collect::<Vec<TileRenderElement<_>>>();
            match open.render(
                renderer,
                &elements,
                self.animated_tile_size(),
                location,
                scale,
                tile_alpha,
            ) {
                Ok((elem, data)) => {
                    self.window().set_offscreen_data(Some(data));
                    open_anim_elem = Some(elem.into());
                }
                Err(err) => {
                    warn!("error rendering window opening animation: {err:?}");
                }
            }
        } else if let Some(alpha) = &self.alpha_animation {
            let renderer = renderer.as_gles_renderer();
            let elements = self.render_inner(renderer, Point::from((0., 0.)), focus_ring, target);
            let elements = elements.collect::<Vec<TileRenderElement<_>>>();
            match alpha.offscreen.render(renderer, scale, &elements) {
                Ok((elem, _sync, data)) => {
                    let offset = elem.offset();
                    let elem = elem.with_alpha(tile_alpha).with_offset(location + offset);

                    self.window().set_offscreen_data(Some(data));
                    alpha_anim_elem = Some(elem.into());
                }
                Err(err) => {
                    warn!("error rendering tile to offscreen for alpha animation: {err:?}");
                }
            }
        }

        if open_anim_elem.is_none() && alpha_anim_elem.is_none() {
            window_elems = Some(self.render_inner(renderer, location, focus_ring, target));
        }

        open_anim_elem
            .into_iter()
            .chain(alpha_anim_elem)
            .chain(window_elems.into_iter().flatten())
    }

    pub fn store_unmap_snapshot_if_empty(&mut self, renderer: &mut GlesRenderer) {
        if self.unmap_snapshot.is_some() {
            return;
        }

        self.unmap_snapshot = Some(self.render_snapshot(renderer));
    }

    fn render_snapshot(&self, renderer: &mut GlesRenderer) -> TileRenderSnapshot {
        let _span = tracy_client::span!("Tile::render_snapshot");

        let contents = self.render(renderer, Point::from((0., 0.)), false, RenderTarget::Output);

        // A bit of a hack to render blocked out as for screencast, but I think it's fine here.
        let blocked_out_contents = self.render(
            renderer,
            Point::from((0., 0.)),
            false,
            RenderTarget::Screencast,
        );

        RenderSnapshot {
            contents: contents.collect(),
            blocked_out_contents: blocked_out_contents.collect(),
            block_out_from: self.window.rules().block_out_from,
            size: self.animated_tile_size(),
            texture: Default::default(),
            blocked_out_texture: Default::default(),
        }
    }

    pub fn take_unmap_snapshot(&mut self) -> Option<TileRenderSnapshot> {
        self.unmap_snapshot.take()
    }
}
