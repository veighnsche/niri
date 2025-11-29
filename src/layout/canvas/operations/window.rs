// TEAM_065: Window operations extracted from operations.rs
//!
//! This module handles finding, activating, and updating windows in the canvas.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::layout::canvas::Canvas2D;
use crate::layout::row::Row;
use crate::layout::tile::Tile;
use crate::layout::{LayoutElement, SizeChange};

impl<W: LayoutElement> Canvas2D<W> {
    // =========================================================================
    // Window Finding
    // =========================================================================

    /// Find a window across all rows in the canvas.
    pub fn find_window(&self, id: &W::Id) -> Option<(i32, &Row<W>, &Tile<W>)> {
        for (&row_idx, row) in &self.rows {
            if let Some(tile) = row.tiles().find(|tile| tile.window().id() == id) {
                return Some((row_idx, row, tile));
            }
        }
        None
    }

    /// Find a window across all rows in the canvas (mutable).
    /// Returns the row index and a mutable reference to the tile.
    pub fn find_window_mut(&mut self, id: &W::Id) -> Option<(i32, &mut Tile<W>)> {
        for (&row_idx, row) in &mut self.rows {
            if row.contains(id) {
                // Find the tile index first
                let tile_idx = row.tiles().position(|tile| tile.window().id() == id)?;
                // Then get mutable access to the specific tile
                if let Some(column) = row.active_column_mut() {
                    if let Some(tile) = column.tiles_iter_mut().nth(tile_idx) {
                        return Some((row_idx, tile));
                    }
                }
            }
        }
        None
    }

    /// Check if any row in the canvas contains the window.
    pub fn has_window(&self, id: &W::Id) -> bool {
        self.rows.values().any(|row| row.contains(id)) || self.floating.has_window(id)
    }

    /// Returns whether the canvas contains the given window (in tiled rows).
    pub fn contains(&self, window: &W::Id) -> bool {
        self.rows.values().any(|row| row.contains(window))
    }

    /// Finds the row containing the given window.
    pub fn find_window_row(&self, window: &W::Id) -> Option<(i32, usize)> {
        for (&row_idx, row) in &self.rows {
            if let Some(col_idx) = row.find_column(window) {
                return Some((row_idx, col_idx));
            }
        }
        None
    }

    // =========================================================================
    // Window Access
    // =========================================================================

    /// Get the active window in the canvas (tiled or floating).
    pub fn active_window(&self) -> Option<&W> {
        if self.floating_is_active {
            self.floating.active_window()
        } else if let Some(row) = self.active_row() {
            // TEAM_019: Implemented proper active window handling for Row
            // Use Row's active_window() which properly tracks active_tile_idx
            row.active_window()
        } else {
            None
        }
    }

    /// Get the active window in the canvas (mutable).
    pub fn active_window_mut(&mut self) -> Option<&mut W> {
        if self.floating_is_active {
            self.floating.active_window_mut()
        } else if let Some(row) = self.active_row_mut() {
            // TEAM_019: Implemented proper active window handling for Row
            // Use Row's active_window_mut() which properly tracks active_tile_idx
            row.active_window_mut()
        } else {
            None
        }
    }

    /// Get all windows in the canvas (tiled + floating).
    pub fn windows(&self) -> impl Iterator<Item = &W> + '_ {
        let tiled = self.rows.values().flat_map(|row| row.tiles());
        let floating = self.floating.tiles();
        tiled.chain(floating).map(|tile| tile.window())
    }

    /// Get all windows in the canvas (mutable).
    pub fn windows_mut(&mut self) -> impl Iterator<Item = &mut W> + '_ {
        let tiled = self.rows.values_mut().flat_map(|row| row.tiles_mut());
        let floating = self.floating.tiles_mut();
        tiled.chain(floating).map(|tile| tile.window_mut())
    }

    // =========================================================================
    // Window Height
    // =========================================================================

    /// TEAM_057: Set the height of a window in the canvas.
    /// Finds the row containing the window and calls set_window_height on it.
    pub fn set_window_height(&mut self, window_id: &W::Id, change: SizeChange) {
        // Find which row contains this window
        let row_idx = self
            .rows
            .iter()
            .find(|(_, row)| row.contains(window_id))
            .map(|(&idx, _)| idx);

        if let Some(idx) = row_idx {
            if let Some(row) = self.rows.get_mut(&idx) {
                row.set_window_height(Some(window_id), change);
            }
        }
    }

    // =========================================================================
    // WlSurface Lookup
    // =========================================================================

    /// Workspace equivalent: find window by wl_surface
    // TEAM_054: Fixed to also search floating space
    pub fn find_wl_surface(&self, wl_surface: &WlSurface) -> Option<&W> {
        // Check tiled windows
        if let Some(window) = self
            .tiled_tiles()
            .find(|tile| tile.window().is_wl_surface(wl_surface))
            .map(|tile| tile.window())
        {
            return Some(window);
        }

        // Check floating windows
        self.floating
            .tiles()
            .find(|tile| tile.window().is_wl_surface(wl_surface))
            .map(|tile| tile.window())
    }

    /// Workspace equivalent: find window by wl_surface (mutable)  
    // TEAM_054: Fixed to also search floating space
    pub fn find_wl_surface_mut(&mut self, wl_surface: &WlSurface) -> Option<&mut W> {
        // Check tiled windows
        for row in self.rows.values_mut() {
            for tile in row.tiles_mut() {
                if tile.window().is_wl_surface(wl_surface) {
                    return Some(tile.window_mut());
                }
            }
        }

        // Check floating windows
        for tile in self.floating.tiles_mut() {
            if tile.window().is_wl_surface(wl_surface) {
                return Some(tile.window_mut());
            }
        }

        None
    }

    // =========================================================================
    // Window Activation
    // =========================================================================

    /// Workspace equivalent: update window in all rows
    pub fn update_window(&mut self, window: &W, serial: Option<smithay::utils::Serial>) {
        // TEAM_020: Implemented proper window update
        // Find the row containing this window and delegate to Row's update_window
        for row in self.rows.values_mut() {
            if row.contains(window.id()) {
                row.update_window(window.id(), serial);
                break;
            }
        }
    }

    /// Workspace equivalent: activate window in canvas
    pub fn activate_window(&mut self, window: &W) -> bool {
        // Try floating first
        if self.floating.has_window(window.id()) {
            self.floating.activate_window(window.id());
            self.floating_is_active = true;
            return true;
        }

        // Then try tiled
        for (&row_idx, row) in self.rows.iter_mut() {
            if row.contains(window.id()) {
                // TEAM_020: Properly activate in row
                // Use Row's activate_window method which handles focus and active tile tracking
                if row.activate_window(window.id()) {
                    self.active_row_idx = row_idx;
                    self.floating_is_active = false;
                    return true;
                }
            }
        }
        false
    }

    /// Workspace equivalent: activate window without raising
    pub fn activate_window_without_raising(&mut self, window: &W) -> bool {
        // For now, same as activate_window
        self.activate_window(window)
    }

    // =========================================================================
    // Window Queries
    // =========================================================================

    /// Workspace equivalent: check if any window exists (like workspace.has_windows())
    pub fn has_windows(&self) -> bool {
        self.tiled_tiles().next().is_some() || self.floating.tiles().next().is_some()
    }

    /// Workspace equivalent: check if has windows or name (for workspace cleanup logic)
    pub fn has_windows_or_name(&self) -> bool {
        self.has_windows() || self.rows.values().any(|row| row.name().is_some())
    }

    /// Workspace equivalent: check if any window is urgent
    pub fn has_urgent_window(&self) -> bool {
        self.tiled_tiles().any(|tile| tile.window().is_urgent())
    }

    /// Check if a window is floating.
    pub fn is_window_floating(&self, id: &W::Id) -> bool {
        self.floating.has_window(id)
    }

    /// Workspace equivalent: get scroll amount to activate window
    pub fn scroll_amount_to_activate(&self, window: &W) -> f64 {
        // TEAM_020: Implemented proper scroll calculation
        // In Canvas2D, use actual row y_offset positions instead of calculating from indices
        if let Some((row_idx, row, _tile)) = self.find_window(window.id()) {
            if row_idx == self.active_row_idx {
                // Window is in active row, no scroll needed
                0.0
            } else {
                // Calculate scroll distance using actual row positions
                let active_row_y = self.active_row().map(|row| row.y_offset()).unwrap_or(0.0);
                let target_row_y = row.y_offset();
                target_row_y - active_row_y
            }
        } else {
            0.0 // Window not found, no scroll
        }
    }

    /// Workspace equivalent: descendants added check - TEAM_021
    pub fn descendants_added(&self, id: &W::Id) -> bool {
        // Find the window and check if it has any descendants
        if let Some((_, _, _tile)) = self.find_window(id) {
            // For now, return false - descendants logic can be implemented later
            // This is typically used for popup windows and child surfaces
            false
        } else {
            false
        }
    }
}
