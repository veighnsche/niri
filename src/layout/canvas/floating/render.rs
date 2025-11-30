// TEAM_063: FloatingSpace render elements and close animations
//!
//! This module handles rendering and close animations for floating windows.

use niri_config::utils::MergeWith as _;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::utils::{Logical, Point, Rectangle, Scale, Size};

use super::FloatingSpace;
use crate::animation::Animation;
use crate::layout::elements::closing_window::{ClosingWindow, ClosingWindowRenderElement};
use crate::layout::tile::{TileRenderElement, TileRenderSnapshot};
use crate::layout::{ConfigureIntent, LayoutElement};
use crate::niri_render_elements;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::RenderTarget;
use crate::utils::transaction::TransactionBlocker;

// TEAM_063: FloatingSpace render element type
niri_render_elements! {
    FloatingSpaceRenderElement<R> => {
        Tile = TileRenderElement<R>,
        ClosingWindow = ClosingWindowRenderElement,
    }
}

impl<W: LayoutElement> FloatingSpace<W> {
    // =========================================================================
    // Render Elements
    // =========================================================================

    pub fn update_render_elements(&mut self, is_active: bool, view_rect: Rectangle<f64, Logical>) {
        let active = self.active_window_id.clone();
        for (tile, offset) in self.tiles_with_offsets_mut() {
            let id = tile.window().id();
            let is_active = is_active && Some(id) == active.as_ref();

            let mut tile_view_rect = view_rect;
            tile_view_rect.loc -= offset + tile.render_offset();
            tile.update_render_elements(is_active, tile_view_rect);
        }
    }

    pub fn render_elements<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        view_rect: Rectangle<f64, Logical>,
        target: RenderTarget,
        focus_ring: bool,
    ) -> Vec<FloatingSpaceRenderElement<R>> {
        let mut rv = Vec::new();

        let scale = Scale::from(self.scale);

        // Draw the closing windows on top of the other windows.
        //
        // FIXME: I guess this should rather preserve the stacking order when the window is closed.
        for closing in self.closing_windows.iter().rev() {
            let elem = closing.render(renderer.as_gles_renderer(), view_rect, scale, target);
            rv.push(elem.into());
        }

        let active = self.active_window_id.clone();
        for (tile, tile_pos) in self.tiles_with_render_positions() {
            // For the active tile, draw the focus ring.
            let focus_ring = focus_ring && Some(tile.window().id()) == active.as_ref();

            rv.extend(
                tile.render(renderer, tile_pos, focus_ring, target)
                    .map(Into::into),
            );
        }

        rv
    }

    // =========================================================================
    // Close Animations
    // =========================================================================

    pub fn start_close_animation_for_window(
        &mut self,
        renderer: &mut GlesRenderer,
        id: &W::Id,
        blocker: TransactionBlocker,
    ) {
        let (tile, tile_pos) = self
            .tiles_with_render_positions_mut(false)
            .find(|(tile, _)| tile.window().id() == id)
            .unwrap();

        let Some(snapshot) = tile.take_unmap_snapshot() else {
            return;
        };

        let tile_size = tile.tile_size();

        self.start_close_animation_for_tile(renderer, snapshot, tile_size, tile_pos, blocker);
    }

    // TEAM_107: Store unmap snapshot for floating window
    pub fn store_unmap_snapshot_if_empty(&mut self, renderer: &mut GlesRenderer, window: &W::Id) {
        for tile in &mut self.tiles {
            if tile.window().id() == window {
                tile.store_unmap_snapshot_if_empty(renderer);
                return;
            }
        }
    }

    // TEAM_107: Clear unmap snapshot for floating window
    pub fn clear_unmap_snapshot(&mut self, window: &W::Id) {
        for tile in &mut self.tiles {
            if tile.window().id() == window {
                let _ = tile.take_unmap_snapshot();
                return;
            }
        }
    }

    pub fn start_close_animation_for_tile(
        &mut self,
        renderer: &mut GlesRenderer,
        snapshot: TileRenderSnapshot,
        tile_size: Size<f64, Logical>,
        tile_pos: Point<f64, Logical>,
        blocker: TransactionBlocker,
    ) {
        let anim = Animation::new(
            self.clock.clone(),
            0.,
            1.,
            0.,
            self.options.animations.window_close.anim,
        );

        let blocker = if self.options.disable_transactions {
            TransactionBlocker::completed()
        } else {
            blocker
        };

        let scale = Scale::from(self.scale);
        let res = ClosingWindow::new(
            renderer, snapshot, scale, tile_size, tile_pos, blocker, anim,
        );
        match res {
            Ok(closing) => {
                self.closing_windows.push(closing);
            }
            Err(err) => {
                warn!("error creating a closing window animation: {err:?}");
            }
        }
    }

    // =========================================================================
    // Refresh
    // =========================================================================

    pub fn refresh(&mut self, is_active: bool, is_focused: bool) {
        let active = self.active_window_id.clone();
        for tile in &mut self.tiles {
            let win = tile.window_mut();

            win.set_active_in_column(true);
            win.set_floating(true);

            let mut is_active = is_active && Some(win.id()) == active.as_ref();
            if self.options.deactivate_unfocused_windows {
                is_active &= is_focused;
            }
            win.set_activated(is_active);

            let resize_data = self
                .interactive_resize
                .as_ref()
                .filter(|resize| &resize.window == win.id())
                .map(|resize| resize.data);
            win.set_interactive_resize(resize_data);

            let border_config = self.options.layout.border.merged_with(&win.rules().border);
            let bounds = compute_toplevel_bounds(border_config, self.working_area.size);
            win.set_bounds(bounds);

            // If transactions are disabled, also disable combined throttling, for more
            // intuitive behavior.
            let intent = if self.options.disable_resize_throttling {
                ConfigureIntent::CanSend
            } else {
                win.configure_intent()
            };

            if matches!(
                intent,
                ConfigureIntent::CanSend | ConfigureIntent::ShouldSend
            ) {
                win.send_pending_configure();
            }

            win.refresh();
        }
    }

    // =========================================================================
    // Toplevel Bounds
    // =========================================================================

    pub fn new_window_toplevel_bounds(
        &self,
        rules: &crate::window::ResolvedWindowRules,
    ) -> Size<i32, Logical> {
        let border_config = self.options.layout.border.merged_with(&rules.border);
        compute_toplevel_bounds(border_config, self.working_area.size)
    }
}

/// Compute the toplevel bounds for a floating window.
pub(crate) fn compute_toplevel_bounds(
    border_config: niri_config::Border,
    working_area_size: Size<f64, Logical>,
) -> Size<i32, Logical> {
    let mut border = 0.;
    if !border_config.off {
        border = border_config.width * 2.;
    }

    Size::from((
        f64::max(working_area_size.w - border, 1.),
        f64::max(working_area_size.h - border, 1.),
    ))
    .to_i32_floor()
}
