// TEAM_008: Floating operations split from mod.rs
//!
//! This module handles floating window operations in the canvas.

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
        if self.floating_is_active {
            // Move window from floating to tiled
            if let Some(removed) = self.floating.remove_active_tile() {
                let tile = removed.tile;
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
                if let Some(removed) = row.remove_active_tile(Transaction::new()) {
                    let tile = removed.tile;
                    self.floating.add_tile(tile, true);
                    self.floating_is_active = true;
                }
            }
        }
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
