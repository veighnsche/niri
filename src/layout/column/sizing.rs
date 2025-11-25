// TEAM_002: Column sizing (width, height, fullscreen, maximize)

use std::iter::zip;

use niri_ipc::{ColumnDisplay, SizeChange};
use ordered_float::NotNan;
use smithay::utils::Size;

use super::tile_data::WindowHeight;
use super::{Column, ColumnWidth};
// TEAM_003: Import ResolvedSize from types module
use crate::layout::types::ResolvedSize;
use crate::layout::{LayoutElement, SizingMode};
use crate::utils::transaction::Transaction;

impl<W: LayoutElement> Column<W> {
    pub(crate) fn update_tile_sizes(&mut self, animate: bool) {
        self.update_tile_sizes_with_transaction(animate, Transaction::new());
    }

    pub(crate) fn update_tile_sizes_with_transaction(&mut self, animate: bool, transaction: Transaction) {
        let sizing_mode = self.pending_sizing_mode();
        if matches!(sizing_mode, SizingMode::Fullscreen | SizingMode::Maximized) {
            for (tile_idx, tile) in self.tiles.iter_mut().enumerate() {
                let is_active = tile_idx == self.active_tile_idx;
                let transaction = if self.display_mode == ColumnDisplay::Tabbed && !is_active {
                    None
                } else {
                    Some(transaction.clone())
                };

                if matches!(sizing_mode, SizingMode::Fullscreen) {
                    tile.request_fullscreen(animate, transaction);
                } else {
                    tile.request_maximized(self.parent_area.size, animate, transaction);
                }
            }
            return;
        }

        let is_tabbed = self.display_mode == ColumnDisplay::Tabbed;

        let min_size: Vec<_> = self
            .tiles
            .iter()
            .map(|t| t.min_size_nonfullscreen())
            .map(|mut size| {
                size.w = size.w.max(1.);
                size.h = size.h.max(1.);
                size
            })
            .collect();
        let max_size: Vec<_> = self.tiles.iter().map(|t| t.max_size_nonfullscreen()).collect();

        let min_width = min_size
            .iter()
            .map(|size| NotNan::new(size.w).unwrap())
            .max()
            .map(NotNan::into_inner)
            .unwrap();
        let max_width = max_size
            .iter()
            .filter_map(|size| {
                let w = size.w;
                if w == 0. { None } else { Some(NotNan::new(w).unwrap()) }
            })
            .min()
            .map(NotNan::into_inner)
            .unwrap_or(f64::from(i32::MAX));
        let max_width = f64::max(max_width, min_width);

        let width = if self.is_full_width {
            ColumnWidth::Proportion(1.)
        } else {
            self.width
        };

        let working_size = self.working_area.size;
        let extra_size = self.extra_size();

        let width = self.resolve_column_width(width);
        let width = f64::max(f64::min(width, max_width), min_width);
        let max_tile_height = working_size.h - self.options.layout.gaps * 2. - extra_size.h;

        let mut max_non_auto_window_height = None;
        if self.tiles.len() > 1 && !is_tabbed {
            if let Some(non_auto_idx) = self
                .data
                .iter()
                .position(|data| !matches!(data.height, WindowHeight::Auto { .. }))
            {
                let min_height_taken = min_size
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| *idx != non_auto_idx)
                    .map(|(_, min_size)| min_size.h + self.options.layout.gaps)
                    .sum::<f64>();

                let tile = &self.tiles[non_auto_idx];
                let height_left = max_tile_height - min_height_taken;
                max_non_auto_window_height = Some(f64::max(
                    1.,
                    tile.window_height_for_tile_height(height_left).round(),
                ));
            }
        }

        let mut heights = zip(&self.tiles, &self.data)
            .map(|(tile, data)| match data.height {
                auto @ WindowHeight::Auto { .. } => auto,
                WindowHeight::Fixed(height) => {
                    let mut window_height = height.round().max(1.);
                    if let Some(max) = max_non_auto_window_height {
                        window_height = f64::min(window_height, max);
                    } else {
                        let max = tile.window_height_for_tile_height(max_tile_height).round();
                        window_height = f64::min(window_height, max);
                    }
                    WindowHeight::Fixed(tile.tile_height_for_window_height(window_height))
                }
                WindowHeight::Preset(idx) => {
                    let preset = self.options.layout.preset_window_heights[idx];
                    let window_height = match self.resolve_preset_height(preset) {
                        ResolvedSize::Tile(h) => tile.window_height_for_tile_height(h),
                        ResolvedSize::Window(h) => h,
                    };

                    let mut window_height = window_height.round().clamp(1., 100000.);
                    if let Some(max) = max_non_auto_window_height {
                        window_height = f64::min(window_height, max);
                    }

                    let tile_height = tile.tile_height_for_window_height(window_height);
                    WindowHeight::Fixed(tile_height)
                }
            })
            .collect::<Vec<_>>();

        if is_tabbed {
            let tabbed_height = heights
                .iter()
                .find_map(|h| if let WindowHeight::Fixed(h) = h { Some(*h) } else { None })
                .unwrap_or(max_tile_height);

            let min_height = min_size
                .iter()
                .map(|size| NotNan::new(size.h).unwrap())
                .max()
                .map(NotNan::into_inner)
                .unwrap();
            let min_height = f64::min(max_tile_height, min_height);
            let tabbed_height = f64::max(tabbed_height, min_height);

            heights.fill(WindowHeight::Fixed(tabbed_height));
        }

        let gaps_left = self.options.layout.gaps * (self.tiles.len() + 1) as f64;
        let mut height_left = working_size.h - gaps_left;
        let mut auto_tiles_left = self.tiles.len();

        for (h, (min_size, max_size)) in zip(&mut heights, zip(&min_size, &max_size)) {
            if min_size.h == max_size.h {
                *h = WindowHeight::Fixed(min_size.h);
            }

            if let WindowHeight::Fixed(h) = h {
                if max_size.h > 0. {
                    *h = f64::min(*h, max_size.h);
                }
                *h = f64::max(*h, min_size.h);

                height_left -= *h;
                auto_tiles_left -= 1;
            }
        }

        let mut total_weight: f64 = heights
            .iter()
            .filter_map(|h| if let WindowHeight::Auto { weight } = *h { Some(weight) } else { None })
            .sum();

        'outer: while auto_tiles_left > 0 {
            let mut height_left_2 = height_left;
            let mut total_weight_2 = total_weight;
            for ((h, tile), min_size) in zip(zip(&mut heights, &self.tiles), &min_size) {
                let weight = match *h {
                    WindowHeight::Auto { weight } => weight,
                    WindowHeight::Fixed(_) => continue,
                    WindowHeight::Preset(_) => unreachable!(),
                };
                let factor = weight / total_weight_2;

                let mut auto = height_left_2 * factor;

                if min_size.h > auto {
                    auto = min_size.h;
                    *h = WindowHeight::Fixed(auto);
                    height_left -= auto;
                    total_weight -= weight;
                    auto_tiles_left -= 1;
                    continue 'outer;
                }

                auto = tile.tile_height_for_window_height(
                    tile.window_height_for_tile_height(auto).round().max(1.),
                );

                height_left_2 -= auto;
                total_weight_2 -= weight;
            }

            for (h, tile) in zip(&mut heights, &self.tiles) {
                let weight = match *h {
                    WindowHeight::Auto { weight } => weight,
                    WindowHeight::Fixed(_) => continue,
                    WindowHeight::Preset(_) => unreachable!(),
                };
                let factor = weight / total_weight;

                let auto = height_left * factor;
                let auto = tile.tile_height_for_window_height(
                    tile.window_height_for_tile_height(auto).round().max(1.),
                );

                *h = WindowHeight::Fixed(auto);
                height_left -= auto;
                total_weight -= weight;
                auto_tiles_left -= 1;
            }

            assert_eq!(auto_tiles_left, 0);
        }

        for (tile_idx, (tile, h)) in zip(&mut self.tiles, heights).enumerate() {
            let WindowHeight::Fixed(height) = h else { unreachable!() };

            let size = Size::from((width, height));

            let is_active = tile_idx == self.active_tile_idx;
            let transaction = if self.display_mode == ColumnDisplay::Tabbed && !is_active {
                None
            } else {
                Some(transaction.clone())
            };

            tile.request_tile_size(size, animate, transaction);
        }
    }

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
                it.position(|resolved| {
                    match resolved {
                        ResolvedSize::Tile(resolved) => current_tile + 1. < resolved,
                        ResolvedSize::Window(resolved) => current_window + 1. < resolved,
                    }
                })
                .unwrap_or(0)
            } else {
                it.rposition(|resolved| {
                    match resolved {
                        ResolvedSize::Tile(resolved) => resolved + 1. < current_tile,
                        ResolvedSize::Window(resolved) => resolved + 1. < current_window,
                    }
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

    pub(crate) fn set_column_width(&mut self, change: SizeChange, tile_idx: Option<usize>, animate: bool) {
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
                    tile.tile_width_for_window_width(f64::from(fixed)).clamp(1., MAX_PX),
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

    pub(crate) fn set_window_height(&mut self, change: SizeChange, tile_idx: Option<usize>, animate: bool) {
        let tile_idx = tile_idx.unwrap_or(self.active_tile_idx);

        if matches!(self.data[tile_idx].height, WindowHeight::Auto { .. }) {
            self.convert_heights_to_auto();
        }

        let current = self.data[tile_idx].height;
        let tile = &self.tiles[tile_idx];
        let current_window_px = match current {
            WindowHeight::Auto { .. } | WindowHeight::Preset(_) => tile.window_size().h,
            WindowHeight::Fixed(height) => height,
        };
        let current_tile_px = tile.tile_height_for_window_height(current_window_px);

        let working_size = self.working_area.size.h;
        let gaps = self.options.layout.gaps;
        let extra_size = self.extra_size().h;
        let full = working_size - gaps;
        let current_prop = if full == 0. { 1. } else { (current_tile_px + gaps) / full };

        const MAX_PX: f64 = 100000.;

        let mut window_height = match change {
            SizeChange::SetFixed(fixed) => f64::from(fixed),
            SizeChange::SetProportion(proportion) => {
                let tile_height = (working_size - gaps) * (proportion / 100.) - gaps - extra_size;
                tile.window_height_for_tile_height(tile_height)
            }
            SizeChange::AdjustFixed(delta) => current_window_px + f64::from(delta),
            SizeChange::AdjustProportion(delta) => {
                let proportion = current_prop + delta / 100.;
                let tile_height = (working_size - gaps) * proportion - gaps - extra_size;
                tile.window_height_for_tile_height(tile_height)
            }
        };

        let min_height_taken = if self.display_mode == ColumnDisplay::Tabbed {
            0.
        } else {
            self.tiles
                .iter()
                .enumerate()
                .filter(|(idx, _)| *idx != tile_idx)
                .map(|(_, tile)| f64::max(1., tile.min_size_nonfullscreen().h) + gaps)
                .sum::<f64>()
        };
        let height_left = working_size - extra_size - gaps - min_height_taken - gaps;
        let height_left = f64::max(1., tile.window_height_for_tile_height(height_left));
        window_height = f64::min(height_left, window_height);

        let win = &self.tiles[tile_idx].window();
        let min_h = win.min_size().h;
        let max_h = win.max_size().h;

        if max_h > 0 {
            window_height = f64::min(window_height, f64::from(max_h));
        }
        if min_h > 0 {
            window_height = f64::max(window_height, f64::from(min_h));
        }

        self.data[tile_idx].height = WindowHeight::Fixed(window_height.clamp(1., MAX_PX));
        self.is_pending_maximized = false;
        self.update_tile_sizes(animate);
    }

    pub(crate) fn reset_window_height(&mut self, tile_idx: Option<usize>) {
        if self.display_mode == ColumnDisplay::Tabbed {
            for data in &mut self.data {
                data.height = WindowHeight::auto_1();
            }
        } else {
            let tile_idx = tile_idx.unwrap_or(self.active_tile_idx);
            self.data[tile_idx].height = WindowHeight::auto_1();
        }

        self.update_tile_sizes(true);
    }

    pub(crate) fn toggle_window_height(&mut self, tile_idx: Option<usize>, forwards: bool) {
        let tile_idx = tile_idx.unwrap_or(self.active_tile_idx);

        if matches!(self.data[tile_idx].height, WindowHeight::Auto { .. }) {
            self.convert_heights_to_auto();
        }

        let len = self.options.layout.preset_window_heights.len();
        let preset_idx = match self.data[tile_idx].height {
            WindowHeight::Preset(idx) if !self.is_pending_maximized => {
                (idx + if forwards { 1 } else { len - 1 }) % len
            }
            _ => {
                let current = self.data[tile_idx].size.h;
                let tile = &self.tiles[tile_idx];

                let mut it = self
                    .options
                    .layout
                    .preset_window_heights
                    .iter()
                    .copied()
                    .map(|preset| {
                        let window_height = match self.resolve_preset_height(preset) {
                            ResolvedSize::Tile(h) => tile.window_height_for_tile_height(h),
                            ResolvedSize::Window(h) => h,
                        };
                        tile.tile_height_for_window_height(window_height.round().clamp(1., 100000.))
                    });

                if forwards {
                    it.position(|resolved| current + 1. < resolved).unwrap_or(0)
                } else {
                    it.rposition(|resolved| resolved + 1. < current).unwrap_or(len - 1)
                }
            }
        };
        self.data[tile_idx].height = WindowHeight::Preset(preset_idx);
        self.is_pending_maximized = false;
        self.update_tile_sizes(true);
    }

    pub(crate) fn convert_heights_to_auto(&mut self) {
        let heights: Vec<_> = self.tiles.iter().map(|tile| tile.tile_size().h).collect();

        let mut sorted = heights.clone();
        sorted.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        let median = sorted[sorted.len() / 2];

        for (data, height) in zip(&mut self.data, heights) {
            let weight = height / median;
            data.height = WindowHeight::Auto { weight };
        }
    }

    pub(crate) fn set_fullscreen(&mut self, is_fullscreen: bool) {
        if self.is_pending_fullscreen == is_fullscreen {
            return;
        }

        if is_fullscreen {
            assert!(self.tiles.len() == 1 || self.display_mode == ColumnDisplay::Tabbed);
        }

        self.is_pending_fullscreen = is_fullscreen;
        self.update_tile_sizes(true);
    }

    pub(crate) fn set_maximized(&mut self, maximize: bool) {
        if self.is_pending_maximized == maximize {
            return;
        }

        if maximize {
            assert!(self.tiles.len() == 1 || self.display_mode == ColumnDisplay::Tabbed);
        }

        self.is_pending_maximized = maximize;
        self.update_tile_sizes(true);
    }

    pub(crate) fn set_column_display(&mut self, display: ColumnDisplay) {
        if self.display_mode == display {
            return;
        }

        let prev_origin = self.tiles_origin();
        self.display_mode = display;
        let new_origin = self.tiles_origin();
        let origin_delta = prev_origin - new_origin;

        self.display_mode = ColumnDisplay::Normal;
        for (tile, pos) in self.tiles_mut() {
            let mut y_delta = pos.y - prev_origin.y;

            if display == ColumnDisplay::Normal {
                y_delta *= -1.;
            }

            let mut delta = origin_delta;
            delta.y += y_delta;
            tile.animate_move_from(delta);
        }

        for (idx, tile) in self.tiles.iter_mut().enumerate() {
            let is_active = idx == self.active_tile_idx;
            if !is_active {
                let (from, to) = if display == ColumnDisplay::Tabbed {
                    (1., 0.)
                } else {
                    (0., 1.)
                };
                tile.animate_alpha(from, to, self.options.animations.window_movement.0);
            }
        }

        if display == ColumnDisplay::Tabbed {
            self.tab_indicator.start_open_animation(
                self.clock.clone(),
                self.options.animations.window_movement.0,
            );
        }

        self.display_mode = display;
        self.update_tile_sizes(true);
    }
}
