// TEAM_008: Window operations split from mod.rs
//!
//! This module handles adding, removing, and finding windows in the canvas.

use std::rc::Rc;

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use crate::layout::canvas::Canvas2D;
use crate::layout::row::Row;
use crate::layout::tile::Tile;
use crate::layout::types::ColumnWidth;
use crate::layout::{LayoutElement, Options};
use niri_ipc::PositionChange;

impl<W: LayoutElement> Canvas2D<W> {
    // =========================================================================
    // Row Management
    // =========================================================================

    /// Creates a new row at the specified index if it doesn't exist.
    pub fn ensure_row(&mut self, row_idx: i32) -> &mut Row<W> {
        self.rows.entry(row_idx).or_insert_with(|| {
            Row::new(
                row_idx,
                self.view_size,
                self.parent_area,
                self.scale,
                self.clock.clone(),
                self.options.clone(),
            )
        })
    }

    /// Removes empty rows (except row 0 which is always kept).
    pub fn cleanup_empty_rows(&mut self) {
        self.rows.retain(|&idx, row| idx == 0 || !row.is_empty());
    }

    // =========================================================================
    // Tile Operations
    // =========================================================================

    /// Adds a tile to the active row.
    pub fn add_tile(
        &mut self,
        tile: Tile<W>,
        activate: bool,
        width: ColumnWidth,
        is_full_width: bool,
    ) {
        let row = self.ensure_row(self.active_row_idx);
        row.add_tile(None, tile, activate, width, is_full_width);
    }

    // =========================================================================
    // Workspace Replacement Methods
    // TEAM_019: Replace workspace iteration patterns
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

    /// Get the active window in the canvas (tiled or floating).
    pub fn active_window(&self) -> Option<&W> {
        if self.floating_is_active {
            self.floating.active_window()
        } else if let Some(row) = self.active_row() {
            // For now, just get the first tile from the active column
            // TODO(TEAM_019): Implement proper active window handling for Row
            row.active_column()
                .and_then(|col| col.tiles_iter().next())
                .map(|tile| tile.window())
        } else {
            None
        }
    }

    /// Get the active window in the canvas (mutable).
    pub fn active_window_mut(&mut self) -> Option<&mut W> {
        if self.floating_is_active {
            self.floating.active_window_mut()
        } else if let Some(row) = self.active_row_mut() {
            // For now, just get the first tile from the active column
            // TODO(TEAM_019): Implement proper active window handling for Row
            row.active_column_mut()
                .and_then(|col| col.tiles_iter_mut().next())
                .map(|tile| tile.window_mut())
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

    /// Get all tiles in the canvas (tiled + floating).
    pub fn tiles(&self) -> impl Iterator<Item = &Tile<W>> + '_ {
        let tiled = self.rows.values().flat_map(|row| row.tiles());
        let floating = self.floating.tiles();
        tiled.chain(floating)
    }

    /// Get all tiles in the canvas (mutable).
    pub fn tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile<W>> + '_ {
        let tiled = self.rows.values_mut().flat_map(|row| row.tiles_mut());
        let floating = self.floating.tiles_mut();
        tiled.chain(floating)
    }

    // =========================================================================
    // Workspace Management Methods
    // TEAM_019: Replace workspace lifecycle operations
    // =========================================================================

    /// Update canvas configuration for all rows.
    pub fn update_config(&mut self, options: Rc<Options>) {
        self.options = options.clone();
        
        // Update all rows with correct parameters
        for row in self.rows.values_mut() {
            row.update_config(
                self.view_size,
                self.parent_area,
                self.scale,
                options.clone(),
            );
        }
        
        // Update floating space
        self.floating.update_config(
            self.view_size,
            self.working_area,
            self.scale,
            options,
        );
    }

    /// Update layout configuration for a specific row (by name).
    pub fn update_row_layout_config(&mut self, row_name: &str, _layout_config: Option<niri_config::LayoutPart>) {
        // Find row by name and update its config
        for row in self.rows.values_mut() {
            if let Some(name) = row.name() {
                if name == row_name {
                    // TODO(TEAM_019): Implement layout_config for Row
                    // For now, just update with current options
                    row.update_config(
                        self.view_size,
                        self.parent_area,
                        self.scale,
                        self.options.clone(),
                    );
                    break;
                }
            }
        }
    }

    /// Start open animation for a window.
    pub fn start_window_open_animation(&mut self, id: &W::Id) -> bool {
        // Check floating windows first
        if self.floating.start_open_animation(id) {
            return true;
        }
        
        // Check tiled windows
        for row in self.rows.values_mut() {
            // TODO(TEAM_019): Implement start_open_animation for Row
            // For now, just check if window exists
            if row.contains(id) {
                return true;
            }
        }
        
        false
    }

    /// Check if a window is floating.
    pub fn is_window_floating(&self, id: &W::Id) -> bool {
        self.floating.has_window(id)
    }

    // =========================================================================
    // Workspace Operation Replacements
    // TEAM_019: Replace workspace method calls
    // =========================================================================

    /// Center a window (replaces workspace.center_window).
    pub fn center_window(&mut self, id: Option<&W::Id>) {
        // For now, delegate to floating space since centering is primarily a floating operation
        // TODO(TEAM_019): Implement proper centering for tiled windows
        if let Some(id) = id {
            if self.floating.has_window(id) {
                self.floating.center_window(Some(id));
            }
        } else {
            // Center active window
            if self.floating_is_active {
                self.floating.center_window(None);
            }
        }
    }

    /// Move a floating window (replaces workspace.move_floating_window).
    pub fn move_floating_window(
        &mut self,
        id: Option<&W::Id>,
        x: PositionChange,
        y: PositionChange,
        animate: bool,
    ) {
        // Only floating windows can be moved this way
        if let Some(id) = id {
            if self.floating.has_window(id) {
                self.floating.move_window(Some(id), x, y, animate);
            }
        } else {
            // Move active floating window
            if self.floating_is_active {
                self.floating.move_window(None, x, y, animate);
            }
        }
    }

    /// Switch focus between floating and tiling (replaces workspace.switch_focus_floating_tiling).
    pub fn switch_focus_floating_tiling(&mut self) {
        if self.floating_is_active {
            // Switch to tiled
            self.floating_is_active = false;
            if let Some(row) = self.active_row_mut() {
                row.focus_column(0);
            }
        } else {
            // Switch to floating if there are floating windows
            if self.floating.tiles().next().is_some() {
                self.floating_is_active = true;
                self.floating.focus_leftmost();
            }
        }
    }

    /// Move focus left within current context (replaces workspace.move_left).
    pub fn move_left(&mut self) -> bool {
        if self.floating_is_active {
            self.floating.move_left();
            true
        } else if let Some(row) = self.active_row_mut() {
            row.focus_left()
        } else {
            false
        }
    }

    /// Move focus right within current context (replaces workspace.move_right).
    pub fn move_right(&mut self) -> bool {
        if self.floating_is_active {
            self.floating.move_right();
            true
        } else if let Some(row) = self.active_row_mut() {
            row.focus_right()
        } else {
            false
        }
    }

    /// Move active column to first position (replaces workspace.move_column_to_first).
    pub fn move_column_to_first(&mut self) {
        if self.floating_is_active {
            return; // No effect on floating windows
        }
        
        if let Some(row) = self.active_row_mut() {
            if row.active_column_idx() > 0 {
                // For now, just focus the first column
                // TODO(TEAM_019): Implement actual column reordering if needed
                row.focus_column(0);
            }
        }
    }

    /// Move active column to last position (replaces workspace.move_column_to_last).
    pub fn move_column_to_last(&mut self) {
        if self.floating_is_active {
            return; // No effect on floating windows
        }
        
        if let Some(row) = self.active_row_mut() {
            let last_idx = row.column_count().saturating_sub(1);
            if row.active_column_idx() < last_idx {
                // For now, just focus the last column
                // TODO(TEAM_019): Implement actual column reordering if needed
                row.focus_column(last_idx);
            }
        }
    }

    /// Adds a tile to a specific row.
    pub fn add_tile_to_row(
        &mut self,
        row_idx: i32,
        tile: Tile<W>,
        activate: bool,
        width: ColumnWidth,
        is_full_width: bool,
    ) {
        let row = self.ensure_row(row_idx);
        row.add_tile(None, tile, activate, width, is_full_width);

        if activate {
            self.active_row_idx = row_idx;
            self.update_camera_y();
        }
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
    // Tiles Query
    // =========================================================================

    /// Returns all tiles in the canvas (tiled only, not floating).
    /// TEAM_010: Added for Monitor.windows() migration
    pub fn tiled_tiles(&self) -> impl Iterator<Item = &Tile<W>> + '_ {
        self.rows.values().flat_map(|row| row.tiles())
    }

    /// Returns all tiles in the canvas (tiled only, not floating).
    /// TEAM_010: Added for Monitor.windows_mut() migration
    pub fn tiled_tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile<W>> + '_ {
        self.rows.values_mut().flat_map(|row| row.tiles_mut())
    }

    /// Returns all tiles with their render positions.
    pub fn tiles_with_render_positions(
        &self,
    ) -> impl Iterator<Item = (&Tile<W>, smithay::utils::Point<f64, smithay::utils::Logical>, bool)> + '_
    {
        let camera_offset = self.camera_position();
        self.rows.values().flat_map(move |row| {
            row.tiles_with_render_positions()
                .map(move |(tile, mut pos, is_active)| {
                    pos.x -= camera_offset.x;
                    pos.y -= camera_offset.y;
                    (tile, pos, is_active)
                })
        })
    }

    // =========================================================================
    // TEAM_020: Workspace Replacement Methods
    // Methods to replace workspace functionality for migration
    // =========================================================================

    /// Workspace equivalent: check if any window exists (like workspace.has_windows())
    pub fn has_windows(&self) -> bool {
        self.tiled_tiles().next().is_some() || self.floating.tiles().next().is_some()
    }

    /// Workspace equivalent: check if has windows or name (for workspace cleanup logic)
    pub fn has_windows_or_name(&self) -> bool {
        self.has_windows() || self.rows.values().any(|row| row.name().is_some())
    }

    /// Workspace equivalent: find window by wl_surface
    pub fn find_wl_surface(&self, wl_surface: &WlSurface) -> Option<&W> {
        self.tiled_tiles()
            .find(|tile| tile.window().is_wl_surface(wl_surface))
            .map(|tile| tile.window())
    }

    /// Workspace equivalent: find window by wl_surface (mutable)  
    pub fn find_wl_surface_mut(&mut self, wl_surface: &WlSurface) -> Option<&mut W> {
        // Check tiled only for now
        for row in self.rows.values_mut() {
            for tile in row.tiles_mut() {
                if tile.window().is_wl_surface(wl_surface) {
                    return Some(tile.window_mut());
                }
            }
        }
        None
    }

    /// Workspace equivalent: update window in all rows
    pub fn update_window(&mut self, window: &W, serial: Option<smithay::utils::Serial>) {
        // TODO(TEAM_020): Implement proper window update
        // For now, just ensure window exists in canvas
        let _ = self.find_window(window.id());
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
                // Find and activate the window in the row
                for tile in row.tiles_mut() {
                    if tile.window().id() == window.id() {
                        // TODO(TEAM_020): Properly activate in row
                        self.active_row_idx = row_idx;
                        self.floating_is_active = false;
                        return true;
                    }
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

    /// Workspace equivalent: set fullscreen for window
    pub fn set_fullscreen(&mut self, id: &W::Id, is_fullscreen: bool) {
        // TODO(TEAM_020): Implement fullscreen setting
    }

    /// Workspace equivalent: toggle fullscreen for window
    pub fn toggle_fullscreen(&mut self, id: &W::Id) {
        // TODO(TEAM_020): Implement fullscreen toggle
    }

    /// Workspace equivalent: set maximized for window
    pub fn set_maximized(&mut self, id: &W::Id, maximize: bool) {
        // TODO(TEAM_020): Implement maximized setting
    }

    /// Workspace equivalent: toggle maximized for window
    pub fn toggle_maximized(&mut self, id: &W::Id) {
        // TODO(TEAM_020): Implement maximized toggle
    }

    /// Workspace equivalent: check if any window is urgent
    pub fn has_urgent_window(&self) -> bool {
        self.tiled_tiles().any(|tile| tile.window().is_urgent())
    }

    /// Workspace equivalent: get scroll amount to activate window
    pub fn scroll_amount_to_activate(&self, window: &W) -> f64 {
        // For now, return 0.0 (no scroll needed in canvas)
        // TODO(TEAM_020): Implement proper scroll calculation if needed
        0.0
    }

    /// Workspace equivalent: descendants added check
    pub fn descendants_added(&self, id: &W::Id) -> bool {
        // TODO(TEAM_020): Implement descendants check
        false
    }

    /// Workspace equivalent: get popup target rect
    pub fn popup_target_rect(&self, window: &W) -> smithay::utils::Rectangle<f64, smithay::utils::Logical> {
        // TODO(TEAM_020): Implement popup target rect
        // Return empty rect for now
        smithay::utils::Rectangle::from_loc_and_size((0.0, 0.0), (0.0, 0.0))
    }
}
