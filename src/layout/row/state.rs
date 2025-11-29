// TEAM_064: State query methods extracted from mod.rs
//!
//! Read-only state inspection methods for Row.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle};

use super::super::column::Column;
use super::super::tile::Tile;
use super::super::LayoutElement;
use super::Row;

impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // Basic state queries
    // =========================================================================

    /// Returns whether this row has no columns.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Returns the number of columns.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Returns the active column index.
    pub fn active_column_idx(&self) -> usize {
        self.active_column_idx
    }

    /// Returns whether this row contains the given window id.
    pub fn has_window(&self, window: &W::Id) -> bool {
        self.columns.iter().any(|col| col.contains(window))
    }

    /// Returns whether this row contains the given window id.
    /// Alias for has_window for canvas compatibility.
    pub fn contains(&self, window: &W::Id) -> bool {
        self.has_window(window)
    }

    /// Returns whether the given window is floating in this row.
    /// TEAM_024: Added for workspace compatibility - always false for tiled rows
    pub fn is_floating(&self, _id: &W::Id) -> bool {
        false // Rows only contain tiled windows, floating windows are in Canvas2D.floating
    }

    /// Check if this row has any windows.
    /// TEAM_033: Added for workspace cleanup logic
    pub fn has_windows(&self) -> bool {
        !self.columns.is_empty()
    }

    /// Check if this row has any windows or a name.
    /// TEAM_022: Stub implementation
    pub fn has_windows_or_name(&self) -> bool {
        self.has_windows() || self.name().is_some()
    }

    /// Check if this row is urgent.
    /// TEAM_022: Implemented - checks all windows in the row
    pub fn is_urgent(&self) -> bool {
        for column in &self.columns {
            for tile in &column.tiles {
                if tile.window().is_urgent() {
                    return true;
                }
            }
        }
        false
    }

    // =========================================================================
    // Active element accessors
    // =========================================================================

    /// Returns the active column, if any.
    pub fn active_column(&self) -> Option<&Column<W>> {
        self.columns.get(self.active_column_idx)
    }

    /// Returns a mutable reference to the active column.
    pub fn active_column_mut(&mut self) -> Option<&mut Column<W>> {
        self.columns.get_mut(self.active_column_idx)
    }

    /// Returns the active window.
    pub fn active_window(&self) -> Option<&W> {
        self.active_column()
            .and_then(|col| col.tiles_iter().nth(col.active_tile_idx))
            .map(|tile| tile.window())
    }

    /// Get mutable reference to the active window.
    /// TEAM_022: Stub implementation
    pub fn active_window_mut(&mut self) -> Option<&mut W> {
        if let Some(col) = self.active_column_mut() {
            let active_tile_idx = col.active_tile_idx;
            col.tiles_iter_mut()
                .nth(active_tile_idx)
                .map(|tile| tile.window_mut())
        } else {
            None
        }
    }

    /// Get the visual rectangle of the active tile.
    /// TEAM_022: Stub implementation
    pub fn active_tile_visual_rectangle(&self) -> Option<Rectangle<f64, Logical>> {
        self.active_column()
            .and_then(|col| col.tiles_iter().nth(col.active_tile_idx))
            .map(|tile| Rectangle::from_loc_and_size(Point::new(0.0, 0.0), tile.tile_size()))
    }

    // =========================================================================
    // Column iteration
    // =========================================================================

    /// Returns an iterator over the columns.
    pub fn columns(&self) -> impl Iterator<Item = &Column<W>> {
        self.columns.iter()
    }

    /// Finds the column containing the given window.
    pub fn find_column(&self, window: &W::Id) -> Option<usize> {
        self.columns.iter().position(|col| col.contains(window))
    }

    // =========================================================================
    // Tile iteration
    // =========================================================================

    /// Returns all tiles in this row.
    /// TEAM_010: Added for Canvas2D.windows() migration
    pub fn tiles(&self) -> impl Iterator<Item = &Tile<W>> + '_ {
        self.columns.iter().flat_map(|col| col.tiles_iter())
    }

    /// Returns all tiles in this row (mutable).
    /// TEAM_010: Added for Canvas2D.windows_mut() migration
    pub fn tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile<W>> + '_ {
        self.columns.iter_mut().flat_map(|col| col.tiles_iter_mut())
    }

    /// Returns all windows in this row.
    /// TEAM_024: Added for workspace compatibility
    pub fn windows(&self) -> impl Iterator<Item = &W> + '_ {
        self.tiles().map(|tile| tile.window())
    }

    /// Returns all windows in this row (mutable).
    /// TEAM_024: Added for workspace compatibility
    pub fn windows_mut(&mut self) -> impl Iterator<Item = &mut W> + '_ {
        self.tiles_mut().map(|tile| tile.window_mut())
    }

    // =========================================================================
    // Surface finders
    // =========================================================================

    /// Find a window by Wayland surface.
    /// TEAM_036: Implemented - searches all tiles for matching surface
    pub fn find_wl_surface(&self, wl_surface: &WlSurface) -> Option<&W> {
        self.tiles()
            .find(|tile| tile.window().is_wl_surface(wl_surface))
            .map(|tile| tile.window())
    }

    /// Find a window by Wayland surface (mutable).
    /// TEAM_036: Implemented - searches all tiles for matching surface
    pub fn find_wl_surface_mut(&mut self, wl_surface: &WlSurface) -> Option<&mut W> {
        for column in &mut self.columns {
            for tile in &mut column.tiles {
                if tile.window().is_wl_surface(wl_surface) {
                    return Some(tile.window_mut());
                }
            }
        }
        None
    }
}
