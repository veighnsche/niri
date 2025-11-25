// TEAM_002: Column tile operations (add, remove, focus, move)

use std::cmp::min;
use std::iter::zip;

use niri_ipc::ColumnDisplay;
use smithay::utils::Point;

use super::tile_data::{TileData, WindowHeight};
use super::Column;
use crate::layout::tile::Tile;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Column<W> {
    pub(crate) fn activate_idx(&mut self, idx: usize) -> bool {
        if self.active_tile_idx == idx {
            return false;
        }

        self.active_tile_idx = idx;
        self.tiles[idx].ensure_alpha_animates_to_1();
        true
    }

    pub(crate) fn activate_window(&mut self, window: &W::Id) {
        let idx = self.position(window).unwrap();
        self.activate_idx(idx);
    }

    pub(crate) fn add_tile_at(&mut self, idx: usize, mut tile: Tile<W>) {
        tile.update_config(self.view_size, self.scale, self.options.clone());

        let mut prev_offsets = Vec::with_capacity(self.tiles.len() + 1);
        prev_offsets.extend(self.tile_offsets().take(self.tiles.len()));

        if self.display_mode != ColumnDisplay::Tabbed {
            self.is_pending_fullscreen = false;
            self.is_pending_maximized = false;
        }

        self.data
            .insert(idx, TileData::new(&tile, WindowHeight::auto_1()));
        self.tiles.insert(idx, tile);
        self.update_tile_sizes(true);

        prev_offsets.insert(idx, Point::default());
        for (i, ((tile, offset), prev)) in zip(self.tiles_mut(), prev_offsets).enumerate() {
            if i == idx {
                continue;
            }
            tile.animate_move_from(prev - offset);
        }
    }

    pub(crate) fn update_window(&mut self, window: &W::Id) {
        let (tile_idx, tile) = self
            .tiles
            .iter_mut()
            .enumerate()
            .find(|(_, tile)| tile.window().id() == window)
            .unwrap();

        let prev_height = self.data[tile_idx].size.h;

        tile.update_window();
        self.data[tile_idx].update(tile);

        let offset = prev_height - self.data[tile_idx].size.h;
        let is_tabbed = self.display_mode == ColumnDisplay::Tabbed;

        if !is_tabbed && offset != 0. {
            if tile.resize_animation().is_some() {
                for tile in &mut self.tiles[tile_idx + 1..] {
                    tile.animate_move_y_from_with_config(
                        offset,
                        self.options.animations.window_resize.anim,
                    );
                }
            } else {
                for tile in &mut self.tiles[tile_idx + 1..] {
                    tile.offset_move_y_anim_current(offset);
                }
            }
        }
    }

    pub(crate) fn focus_index(&mut self, index: u8) {
        let idx = min(usize::from(index.saturating_sub(1)), self.tiles.len() - 1);
        self.activate_idx(idx);
    }

    pub(crate) fn focus_up(&mut self) -> bool {
        self.activate_idx(self.active_tile_idx.saturating_sub(1))
    }

    pub(crate) fn focus_down(&mut self) -> bool {
        self.activate_idx(min(self.active_tile_idx + 1, self.tiles.len() - 1))
    }

    pub(crate) fn focus_top(&mut self) {
        self.activate_idx(0);
    }

    pub(crate) fn focus_bottom(&mut self) {
        self.activate_idx(self.tiles.len() - 1);
    }

    pub(crate) fn move_up(&mut self) -> bool {
        let new_idx = self.active_tile_idx.saturating_sub(1);
        if self.active_tile_idx == new_idx {
            return false;
        }

        let mut ys = self.tile_offsets().skip(self.active_tile_idx);
        let active_y = ys.next().unwrap().y;
        let next_y = ys.next().unwrap().y;
        drop(ys);

        self.tiles.swap(self.active_tile_idx, new_idx);
        self.data.swap(self.active_tile_idx, new_idx);
        self.active_tile_idx = new_idx;

        let new_active_y = self.tile_offset(new_idx).y;
        self.tiles[new_idx].animate_move_y_from(active_y - new_active_y);
        self.tiles[new_idx + 1].animate_move_y_from(active_y - next_y);

        true
    }

    pub(crate) fn move_down(&mut self) -> bool {
        let new_idx = min(self.active_tile_idx + 1, self.tiles.len() - 1);
        if self.active_tile_idx == new_idx {
            return false;
        }

        let mut ys = self.tile_offsets().skip(self.active_tile_idx);
        let active_y = ys.next().unwrap().y;
        let next_y = ys.next().unwrap().y;
        drop(ys);

        self.tiles.swap(self.active_tile_idx, new_idx);
        self.data.swap(self.active_tile_idx, new_idx);
        self.active_tile_idx = new_idx;

        let new_active_y = self.tile_offset(new_idx).y;
        self.tiles[new_idx].animate_move_y_from(active_y - new_active_y);
        self.tiles[new_idx - 1].animate_move_y_from(next_y - active_y);

        true
    }
}
