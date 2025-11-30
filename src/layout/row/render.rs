// TEAM_007: Row rendering extracted from ScrollingSpace
//!
//! This module handles rendering of columns and tiles within a row.

use std::iter::zip;

use smithay::utils::{Point, Rectangle, Scale};

use super::Row;
use crate::layout::elements::closing_window::ClosingWindowRenderElement;
use crate::layout::elements::tab_indicator::TabIndicatorRenderElement;
use crate::layout::tile::TileRenderElement;
use crate::layout::LayoutElement;
use crate::niri_render_elements;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::RenderTarget;

niri_render_elements! {
    RowRenderElement<R> => {
        Tile = TileRenderElement<R>,
        ClosingWindow = ClosingWindowRenderElement,
        TabIndicator = TabIndicatorRenderElement,
    }
}

impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // Render order iteration
    // =========================================================================

    /// Returns columns in render order (active first, then rest).
    ///
    /// The active column is rendered first so it appears on top.
    pub(crate) fn columns_in_render_order(
        &self,
    ) -> impl Iterator<Item = (&crate::layout::column::Column<W>, f64)> + '_ {
        let offsets = self.column_xs_in_render_order();

        let (first, active, rest) = if self.columns.is_empty() {
            (&[][..], &[][..], &[][..])
        } else {
            let (first, rest) = self.columns.split_at(self.active_column_idx);
            let (active, rest) = rest.split_at(1);
            (first, active, rest)
        };

        let columns = active.iter().chain(first).chain(rest);
        zip(columns, offsets)
    }

    /// Returns column X positions in render order.
    fn column_xs_in_render_order(&self) -> impl Iterator<Item = f64> + '_ {
        let active_idx = self.active_column_idx;
        let active_pos = self.column_x(active_idx);
        let offsets = self
            .column_xs()
            .enumerate()
            .filter_map(move |(idx, pos)| (idx != active_idx).then_some(pos));
        std::iter::once(active_pos).chain(offsets)
    }

    /// Returns column X positions in order.
    fn column_xs(&self) -> impl Iterator<Item = f64> + '_ {
        let gaps = self.options.layout.gaps;
        let mut x = 0.;
        self.data.iter().map(move |data| {
            let col_x = x;
            x += data.width + gaps;
            col_x
        })
    }

    // =========================================================================
    // Render queries
    // =========================================================================

    /// Returns whether this row should render above the top layer.
    ///
    /// This is true when the active column is fullscreen and the view is stationary.
    // TEAM_008: Ported from ScrollingSpace
    pub fn render_above_top_layer(&self) -> bool {
        if self.columns.is_empty() {
            return false;
        }

        if !self.view_offset_x.is_static() {
            return false;
        }

        self.columns[self.active_column_idx]
            .sizing_mode()
            .is_fullscreen()
    }

    // =========================================================================
    // Render elements
    // =========================================================================

    /// Renders all elements in this row.
    pub fn render_elements<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        target: RenderTarget,
        focus_ring: bool,
    ) -> Vec<RowRenderElement<R>> {
        let mut rv = vec![];

        let scale = Scale::from(self.scale);

        // Draw the closing windows on top of the other windows.
        let view_rect = Rectangle::new(
            Point::from((self.view_pos(), self.y_offset)),
            self.view_size,
        );
        for closing in self.closing_windows.iter().rev() {
            let elem = closing.render(renderer.as_gles_renderer(), view_rect, scale, target);
            rv.push(elem.into());
        }

        if self.columns.is_empty() {
            return rv;
        }

        let mut first = true;

        // Render columns in order (active first).
        let view_off = Point::from((-self.view_pos(), 0.));
        for (col, col_x) in self.columns_in_render_order() {
            let col_off = Point::from((col_x, 0.));
            let col_render_off = col.render_offset();

            // Draw the tab indicator on top.
            {
                let pos = view_off + col_off + col_render_off;
                let pos = pos.to_physical_precise_round(scale).to_logical(scale);
                rv.extend(col.tab_indicator.render(renderer, pos).map(Into::into));
            }

            for (tile, tile_off, visible) in col.tiles_in_render_order() {
                let tile_pos =
                    view_off + col_off + col_render_off + tile_off + tile.render_offset();
                // Round to physical pixels.
                let tile_pos = tile_pos.to_physical_precise_round(scale).to_logical(scale);

                // For the active tile (which comes first), draw the focus ring.
                let draw_focus_ring = focus_ring && first;
                first = false;

                // Handle visibility for tabbed mode.
                // We want to animate opacity when going in and out of tabbed mode,
                // so we selectively ignore "visible" here when animating alpha.
                let visible = visible || tile.has_alpha_animation();
                if !visible {
                    continue;
                }

                rv.extend(
                    tile.render(renderer, tile_pos, draw_focus_ring, target)
                        .map(Into::into),
                );
            }
        }

        rv
    }

    // =========================================================================
    // Render state updates
    // =========================================================================

    /// Updates render elements for all columns in this row.
    pub fn update_render_elements(&mut self, is_active_row: bool) {
        let view_rect = Rectangle::new(Point::from((self.view_pos(), 0.)), self.view_size);

        let active_col_idx = self.active_column_idx;
        for (col_idx, (col, col_x)) in self.columns_mut_with_offsets().enumerate() {
            let is_active_col = is_active_row && col_idx == active_col_idx;
            let mut col_view_rect = view_rect;
            col_view_rect.loc.x -= col_x + col.render_offset().x;
            col.update_render_elements(is_active_col, col_view_rect);
        }
    }

    /// Returns mutable columns with their X offsets.
    fn columns_mut_with_offsets(
        &mut self,
    ) -> impl Iterator<Item = (&mut crate::layout::column::Column<W>, f64)> {
        let gaps = self.options.layout.gaps;
        let mut x = 0.;
        self.columns.iter_mut().map(move |col| {
            let col_x = x;
            x += col.width() + gaps;
            (col, col_x)
        })
    }
}
