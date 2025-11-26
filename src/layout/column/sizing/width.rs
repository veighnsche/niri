// TEAM_008: Width operations split from sizing.rs
//!
//! This module handles column width operations.

use niri_ipc::SizeChange;

use crate::layout::column::{Column, ColumnWidth};
use crate::layout::types::ResolvedSize;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Column<W> {
    pub(crate) fn toggle_width(&mut self, tile_idx: Option<usize>, forwards: bool) {
        let tile_idx = tile_idx.unwrap_or(self.active_tile_idx);

        let preset_idx = if self.is_full_width || self.is_pending_maximized {
            None
        } else {
            self.preset_width_idx
        };

        let len = self.options.layout.preset_column_widths.len();
        let preset_idx = if let Some(idx) = preset_idx {
            (idx + if forwards { 1 } else { len - 1 }) % len
        } else {
            let tile = &self.tiles[tile_idx];
            let current_window = tile.window_expected_or_current_size().w;
            let current_tile = tile.tile_expected_or_current_size().w;

            let mut it = self
                .options
                .layout
                .preset_column_widths
                .iter()
                .map(|preset| self.resolve_preset_width(*preset));

            if forwards {
                it.position(|resolved| match resolved {
                    ResolvedSize::Tile(resolved) => current_tile + 1. < resolved,
                    ResolvedSize::Window(resolved) => current_window + 1. < resolved,
                })
                .unwrap_or(0)
            } else {
                it.rposition(|resolved| match resolved {
                    ResolvedSize::Tile(resolved) => resolved + 1. < current_tile,
                    ResolvedSize::Window(resolved) => resolved + 1. < current_window,
                })
                .unwrap_or(len - 1)
            }
        };

        let preset = self.options.layout.preset_column_widths[preset_idx];
        self.set_column_width(SizeChange::from(preset), Some(tile_idx), true);

        self.preset_width_idx = Some(preset_idx);
    }

    pub(crate) fn toggle_full_width(&mut self) {
        if self.is_pending_maximized {
            self.is_pending_maximized = false;
            self.is_full_width = false;
        } else {
            self.is_full_width = !self.is_full_width;
        }

        self.update_tile_sizes(true);
    }

    pub(crate) fn set_column_width(
        &mut self,
        change: SizeChange,
        tile_idx: Option<usize>,
        animate: bool,
    ) {
        let current = if self.is_full_width || self.is_pending_maximized {
            ColumnWidth::Proportion(1.)
        } else {
            self.width
        };

        let current_px = self.resolve_column_width(current);

        const MAX_PX: f64 = 100000.;
        const MAX_F: f64 = 10000.;

        let width = match (current, change) {
            (_, SizeChange::SetFixed(fixed)) => {
                let tile_idx = tile_idx.unwrap_or(self.active_tile_idx);
                let tile = &self.tiles[tile_idx];
                ColumnWidth::Fixed(
                    tile.tile_width_for_window_width(f64::from(fixed))
                        .clamp(1., MAX_PX),
                )
            }
            (_, SizeChange::SetProportion(proportion)) => {
                ColumnWidth::Proportion((proportion / 100.).clamp(0., MAX_F))
            }
            (_, SizeChange::AdjustFixed(delta)) => {
                let width = (current_px + f64::from(delta)).clamp(1., MAX_PX);
                ColumnWidth::Fixed(width)
            }
            (ColumnWidth::Proportion(current), SizeChange::AdjustProportion(delta)) => {
                let proportion = (current + delta / 100.).clamp(0., MAX_F);
                ColumnWidth::Proportion(proportion)
            }
            (ColumnWidth::Fixed(_), SizeChange::AdjustProportion(delta)) => {
                let full = self.working_area.size.w - self.options.layout.gaps;
                let current = if full == 0. {
                    1.
                } else {
                    (current_px + self.options.layout.gaps + self.extra_size().w) / full
                };
                let proportion = (current + delta / 100.).clamp(0., MAX_F);
                ColumnWidth::Proportion(proportion)
            }
        };

        self.width = width;
        self.preset_width_idx = None;
        self.is_full_width = false;
        self.is_pending_maximized = false;
        self.update_tile_sizes(animate);
    }
}
