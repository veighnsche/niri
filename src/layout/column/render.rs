// TEAM_002: Column rendering

use std::iter::zip;

use niri_ipc::ColumnDisplay;
use smithay::utils::{Logical, Point, Rectangle, Size};

use super::{Column, MoveAnimation};
use crate::animation::Animation;
use crate::layout::elements::tab_indicator::TabInfo;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Column<W> {
    pub fn render_offset(&self) -> Point<f64, Logical> {
        let mut offset = Point::from((0., 0.));

        if let Some(move_) = &self.move_animation {
            offset.x += move_.from * move_.anim.value();
        }

        offset
    }

    pub fn animate_move_from(&mut self, from_x_offset: f64) {
        self.animate_move_from_with_config(
            from_x_offset,
            self.options.animations.window_movement.0,
        );
    }

    pub fn animate_move_from_with_config(
        &mut self,
        from_x_offset: f64,
        config: niri_config::Animation,
    ) {
        let current_offset = self
            .move_animation
            .as_ref()
            .map_or(0., |move_| move_.from * move_.anim.value());

        let anim = Animation::new(self.clock.clone(), 1., 0., 0., config);
        self.move_animation = Some(MoveAnimation {
            anim,
            from: from_x_offset + current_offset,
        });
    }

    pub fn offset_move_anim_current(&mut self, offset: f64) {
        if let Some(move_) = self.move_animation.as_mut() {
            let value = move_.anim.value();
            if value > 0.001 {
                move_.from += offset / value;
            }
        }
    }

    pub fn update_render_elements(&mut self, is_active: bool, view_rect: Rectangle<f64, Logical>) {
        let active_idx = self.active_tile_idx;
        for (tile_idx, (tile, tile_off)) in self.tiles_mut().enumerate() {
            let is_active = is_active && tile_idx == active_idx;

            let mut tile_view_rect = view_rect;
            tile_view_rect.loc -= tile_off + tile.render_offset();
            tile.update_render_elements(is_active, tile_view_rect);
        }

        let config = self.tab_indicator.config();
        let offsets = self.tile_offsets_iter(self.data.iter().copied());
        let tabs = zip(&self.tiles, offsets)
            .enumerate()
            .map(|(tile_idx, (tile, tile_off))| {
                let is_active = tile_idx == active_idx;
                let is_urgent = tile.window().is_urgent();
                let tile_pos = tile_off + tile.render_offset();
                TabInfo::from_tile(tile, tile_pos, is_active, is_urgent, &config)
            });

        let enabled = self.display_mode == ColumnDisplay::Tabbed && self.sizing_mode().is_normal();

        self.tab_indicator.update_render_elements(
            enabled,
            self.tab_indicator_area(),
            view_rect,
            self.tiles.len(),
            tabs,
            is_active,
            self.scale,
        );
    }

    pub(crate) fn tab_indicator_area(&self) -> Rectangle<f64, Logical> {
        let mut max_height = 0.;
        for tile in &self.tiles {
            max_height = f64::max(max_height, tile.tile_size().h);
        }

        let tile = &self.tiles[self.active_tile_idx];
        let area_size = Size::from((tile.animated_tile_size().w, max_height));

        Rectangle::new(self.tiles_origin(), area_size)
    }

    pub fn start_open_animation(&mut self, id: &W::Id) -> bool {
        for tile in &mut self.tiles {
            if tile.window().id() == id {
                tile.start_open_animation();

                if self.display_mode == ColumnDisplay::Tabbed
                    && self.sizing_mode().is_normal()
                    && self.tiles.len() == 1
                    && !self.tab_indicator.config().hide_when_single_tab
                {
                    self.tab_indicator.start_open_animation(
                        self.clock.clone(),
                        self.options.animations.window_open.anim,
                    );
                }

                return true;
            }
        }

        false
    }
}
