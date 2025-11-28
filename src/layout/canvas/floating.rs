// TEAM_008: Floating operations split from mod.rs
//!
//! This module handles floating window operations in the canvas.

use niri_config::CenterFocusedColumn;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::utils::{Logical, Point, Size};

use crate::layout::canvas::Canvas2D;
use crate::layout::tile::{Tile, TileRenderSnapshot};
use crate::layout::types::ColumnWidth;
use crate::layout::{LayoutElement, RemovedTile};
use crate::utils::transaction::{Transaction, TransactionBlocker};

impl<W: LayoutElement> Canvas2D<W> {
    // =========================================================================
    // Toggle Floating (TEAM_009)
    // =========================================================================

    /// Toggles the active window between floating and tiled mode.
    ///
    /// If floating is active, moves the window to the tiled layer.
    /// If tiled is active, moves the window to the floating layer.
    pub fn toggle_floating_window(&mut self) {
        self.toggle_floating_window_by_id(None);
    }

    /// Toggles a specific window (or active window if None) between floating and tiled mode.
    // TEAM_040: Added to support toggling specific windows
    pub fn toggle_floating_window_by_id(&mut self, window: Option<&W::Id>) {
        // If a specific window is provided, check if it's floating
        if let Some(id) = window {
            if self.floating.has_window(id) {
                // Move from floating to tiled
                let removed = self.floating.remove_tile(id);
                let mut tile = removed.tile;
                // TEAM_045: Start move animation when returning from floating to tiled
                // so golden snapshots capture tile edge animations like the original
                // Workspace-based implementation.
                tile.animate_move_from(Point::from((0., 0.)));
                let width = removed.width;
                let is_full_width = removed.is_full_width;
                self.add_tile(tile, true, width, is_full_width);

                // Switch to tiled mode if floating is now empty
                if self.floating.is_empty() {
                    self.floating_is_active = false;
                }
                return;
            }

            // Check if it's in a row
            for row in self.rows.values_mut() {
                if row.contains(id) {
                    // Get render position before removing
                    let render_pos = row.tile_render_location(id);
                    let mut removed = row.remove_tile(id, Transaction::new());
                    removed.tile.stop_move_animations();
                    
                    // TEAM_044: Set floating position based on render position (like original Workspace)
                    self.set_floating_position_from_render_pos(&mut removed.tile, render_pos);
                    
                    self.floating.add_tile(removed.tile, true);
                    self.floating_is_active = true;
                    return;
                }
            }
            return;
        }

        // No specific window - toggle the active window
        if self.floating_is_active {
            // Move window from floating to tiled
            if let Some(removed) = self.floating.remove_active_tile() {
                let mut tile = removed.tile;
                // TEAM_045: Start move animation when returning from floating to tiled
                // so golden snapshots capture tile edge animations like the original
                // Workspace-based implementation.
                tile.animate_move_from(Point::from((0., 0.)));
                let width = removed.width;
                let is_full_width = removed.is_full_width;
                self.add_tile(tile, true, width, is_full_width);

                // Switch to tiled mode if floating is now empty
                if self.floating.is_empty() {
                    self.floating_is_active = false;
                }
            }
        } else {
            // Move window from tiled to floating
            if let Some(row) = self.active_row_mut() {
                // Get render position before removing
                let render_pos = row.active_tile_render_location();
                if let Some(mut removed) = row.remove_active_tile(Transaction::new()) {
                    removed.tile.stop_move_animations();
                    
                    // TEAM_044: Set floating position based on render position (like original Workspace)
                    self.set_floating_position_from_render_pos(&mut removed.tile, render_pos);
                    
                    self.floating.add_tile(removed.tile, true);
                    self.floating_is_active = true;
                }
            }
        }
    }
    
    /// Sets the floating position for a tile based on its render position.
    ///
    /// This matches the original Workspace::toggle_window_floating behavior.
    fn set_floating_position_from_render_pos(&self, tile: &mut Tile<W>, render_pos: Option<Point<f64, Logical>>) {
        // Only set position if there's no stored position
        if self.floating.stored_or_default_tile_pos(tile).is_some() {
            return;
        }
        
        let Some(render_pos) = render_pos else {
            return;
        };
        
        // Calculate offset based on center_focused_column setting
        let offset = if self.options.layout.center_focused_column == CenterFocusedColumn::Always {
            Point::from((0., 0.))
        } else {
            Point::from((50., 50.))
        };
        
        let pos = render_pos + offset;
        let size = tile.tile_size();
        let pos = self.floating.clamp_within_working_area(pos, size);
        let pos = self.floating.logical_to_size_frac(pos);
        tile.floating_pos = Some(pos);
    }

    /// Switches focus between floating and tiled layers.
    pub fn toggle_floating_focus(&mut self) {
        if self.floating_is_active {
            if self.has_tiled_windows() {
                self.floating_is_active = false;
            }
        } else if self.has_floating_windows() {
            self.floating_is_active = true;
        }
    }

    /// Focuses the tiled layer (if there are tiled windows).
    ///
    /// This is equivalent to the original Workspace::focus_tiling().
    pub fn focus_tiling(&mut self) {
        if self.floating_is_active && self.has_tiled_windows() {
            self.floating_is_active = false;
        }
    }

    /// Focuses the floating layer (if there are floating windows).
    ///
    /// This is equivalent to the original Workspace::focus_floating().
    pub fn focus_floating(&mut self) {
        if !self.floating_is_active && self.has_floating_windows() {
            self.floating_is_active = true;
        }
    }

    /// Switches focus between floating and tiled layers.
    ///
    /// This is equivalent to the original Workspace::switch_focus_floating_tiling().
    pub fn switch_focus_floating_tiling(&mut self) {
        if self.floating.is_empty() {
            // If floating is empty, keep focus on tiled.
            return;
        } else if !self.has_tiled_windows() {
            // If floating isn't empty but tiled is, keep focus on floating.
            return;
        }

        self.floating_is_active = !self.floating_is_active;
    }

    // =========================================================================
    // Window Operations (TEAM_009)
    // =========================================================================

    /// Adds a window to the canvas.
    ///
    /// Routes to floating space if floating, otherwise to the active row.
    pub fn add_window(
        &mut self,
        tile: Tile<W>,
        activate: bool,
        is_floating: bool,
        width: ColumnWidth,
        is_full_width: bool,
    ) {
        if is_floating {
            self.floating.add_tile(tile, activate);
            if activate {
                self.floating_is_active = true;
            }
        } else {
            self.add_tile(tile, activate, width, is_full_width);
            if activate {
                self.floating_is_active = false;
            }
        }
    }

    /// Removes a window from the canvas.
    ///
    /// Searches both tiled rows and floating space.
    pub fn remove_window(&mut self, id: &W::Id) -> Option<RemovedTile<W>> {
        // Check floating first
        if self.floating.has_window(id) {
            return Some(self.floating.remove_tile(id));
        }

        // Check rows
        for row in self.rows.values_mut() {
            if row.contains(id) {
                return Some(row.remove_tile(id, Transaction::new()));
            }
        }

        None
    }

    /// Returns whether the canvas contains the given window (tiled or floating).
    pub fn contains_any(&self, id: &W::Id) -> bool {
        self.floating.has_window(id) || self.contains(id)
    }

    /// Starts a close animation for a window.
    pub fn start_close_animation_for_window(
        &mut self,
        renderer: &mut GlesRenderer,
        id: &W::Id,
        blocker: TransactionBlocker,
    ) {
        // Try floating first
        if self.floating.has_window(id) {
            self.floating
                .start_close_animation_for_window(renderer, id, blocker);
            return;
        }

        // TODO(TEAM_009): Add close animation for tiled windows in rows
        // For now, tiled windows don't have close animations at the canvas level
    }

    /// Starts a close animation from a tile snapshot.
    pub fn start_close_animation_for_tile(
        &mut self,
        renderer: &mut GlesRenderer,
        snapshot: TileRenderSnapshot,
        tile_size: Size<f64, Logical>,
        tile_pos: Point<f64, Logical>,
        blocker: TransactionBlocker,
    ) {
        self.floating
            .start_close_animation_for_tile(renderer, snapshot, tile_size, tile_pos, blocker);
    }
}
