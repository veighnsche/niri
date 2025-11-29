// TEAM_002: Column layout calculations (tile positioning)

use std::iter::{self, zip};

use niri_config::{CenterFocusedColumn, PresetSize};
use niri_ipc::ColumnDisplay;
use ordered_float::NotNan;
use smithay::utils::{Logical, Point, Size};

use super::tile_data::{TileData, WindowHeight};
use super::{resolve_preset_size, Column, ColumnWidth};
use crate::layout::tile::Tile;
// TEAM_003: Import ResolvedSize from types module
use crate::layout::types::ResolvedSize;
use crate::layout::{LayoutElement, SizingMode};

impl<W: LayoutElement> Column<W> {
    /// Extra size taken up by elements in the column such as the tab indicator.
    pub(crate) fn extra_size(&self) -> Size<f64, Logical> {
        if self.display_mode == ColumnDisplay::Tabbed {
            self.tab_indicator.extra_size(self.tiles.len(), self.scale)
        } else {
            Size::from((0., 0.))
        }
    }

    pub(crate) fn resolve_preset_width(&self, preset: PresetSize) -> ResolvedSize {
        let extra = self.extra_size();
        resolve_preset_size(preset, &self.options, self.working_area.size.w, extra.w)
    }

    pub(crate) fn resolve_preset_height(&self, preset: PresetSize) -> ResolvedSize {
        let extra = self.extra_size();
        resolve_preset_size(preset, &self.options, self.working_area.size.h, extra.h)
    }

    pub(crate) fn resolve_column_width(&self, width: ColumnWidth) -> f64 {
        let working_size = self.working_area.size;
        let gaps = self.options.layout.gaps;
        let extra = self.extra_size();

        match width {
            ColumnWidth::Proportion(proportion) => {
                (working_size.w - gaps) * proportion - gaps - extra.w
            }
            ColumnWidth::Fixed(width) => width,
        }
    }

    pub(crate) fn tiles_origin(&self) -> Point<f64, Logical> {
        let mut origin = Point::from((0., 0.));

        match self.sizing_mode() {
            SizingMode::Normal => (),
            SizingMode::Maximized => {
                origin.y += self.parent_area.loc.y;
                return origin;
            }
            SizingMode::Fullscreen => return origin,
        }

        origin.y += self.working_area.loc.y + self.options.layout.gaps;

        if self.display_mode == ColumnDisplay::Tabbed {
            origin += self
                .tab_indicator
                .content_offset(self.tiles.len(), self.scale);
        }

        origin
    }

    pub(crate) fn tile_offsets_iter(
        &self,
        data: impl Iterator<Item = TileData>,
    ) -> impl Iterator<Item = Point<f64, Logical>> {
        let center = self.options.layout.center_focused_column == CenterFocusedColumn::Always;
        let gaps = self.options.layout.gaps;
        let tabbed = self.display_mode == ColumnDisplay::Tabbed;

        let tiles_width = self
            .data
            .iter()
            .map(|data| NotNan::new(data.size.w).unwrap())
            .max()
            .map(NotNan::into_inner)
            .unwrap_or(0.);

        let mut origin = self.tiles_origin();

        let dummy = TileData {
            height: WindowHeight::auto_1(),
            size: Size::default(),
            interactively_resizing_by_left_edge: false,
        };
        let data = data.chain(iter::once(dummy));

        data.map(move |data| {
            let mut pos = origin;

            if center {
                pos.x += (tiles_width - data.size.w) / 2.;
            } else if data.interactively_resizing_by_left_edge {
                pos.x += tiles_width - data.size.w;
            }

            if !tabbed {
                origin.y += data.size.h + gaps;
            }

            pos
        })
    }

    pub(crate) fn tile_offsets(&self) -> impl Iterator<Item = Point<f64, Logical>> + '_ {
        self.tile_offsets_iter(self.data.iter().copied())
    }

    pub(crate) fn tile_offset(&self, tile_idx: usize) -> Point<f64, Logical> {
        self.tile_offsets().nth(tile_idx).unwrap()
    }

    pub(crate) fn tile_offsets_in_render_order(
        &self,
        data: impl Iterator<Item = TileData>,
    ) -> impl Iterator<Item = Point<f64, Logical>> {
        let active_idx = self.active_tile_idx;
        let active_pos = self.tile_offset(active_idx);
        let offsets = self
            .tile_offsets_iter(data)
            .enumerate()
            .filter_map(move |(idx, pos)| (idx != active_idx).then_some(pos));
        iter::once(active_pos).chain(offsets)
    }

    pub fn tiles(&self) -> impl Iterator<Item = (&Tile<W>, Point<f64, Logical>)> + '_ {
        let offsets = self.tile_offsets_iter(self.data.iter().copied());
        zip(&self.tiles, offsets)
    }

    pub(crate) fn tiles_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut Tile<W>, Point<f64, Logical>)> + '_ {
        let offsets = self.tile_offsets_iter(self.data.iter().copied());
        zip(&mut self.tiles, offsets)
    }

    pub(crate) fn tiles_in_render_order(
        &self,
    ) -> impl Iterator<Item = (&Tile<W>, Point<f64, Logical>, bool)> + '_ {
        let offsets = self.tile_offsets_in_render_order(self.data.iter().copied());

        let (first, rest) = self.tiles.split_at(self.active_tile_idx);
        let (active, rest) = rest.split_at(1);

        let active = active.iter().map(|tile| (tile, true));

        let rest_visible = self.display_mode != ColumnDisplay::Tabbed;
        let rest = first.iter().chain(rest);
        let rest = rest.map(move |tile| (tile, rest_visible));

        let tiles = active.chain(rest);
        zip(tiles, offsets).map(|((tile, visible), pos)| (tile, pos, visible))
    }

    pub(crate) fn width(&self) -> f64 {
        let mut tiles_width = self
            .data
            .iter()
            .map(|data| NotNan::new(data.size.w).unwrap())
            .max()
            .map(NotNan::into_inner)
            .unwrap();

        if self.display_mode == ColumnDisplay::Tabbed && self.sizing_mode().is_normal() {
            let extra_size = self.tab_indicator.extra_size(self.tiles.len(), self.scale);
            tiles_width += extra_size.w;
        }

        tiles_width
    }
}
