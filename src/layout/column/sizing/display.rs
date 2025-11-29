// TEAM_008: Display mode operations split from sizing.rs
//!
//! This module handles fullscreen, maximize, and display mode operations.

use niri_ipc::ColumnDisplay;

use crate::layout::column::Column;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Column<W> {
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
        // Removed early return check to allow setting maximize state even when already set
        // This ensures the maximize state is properly preserved across workspace moves

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
