// TEAM_008: Window operations split from mod.rs
//!
//! This module handles adding, removing, and finding windows in the canvas.

use std::rc::Rc;

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::Point;
use crate::layout::canvas::Canvas2D;
use crate::layout::row::Row;
use super::super::floating::FloatingSpace;
use super::super::tile::Tile;
use super::super::LayoutElement;
use super::super::Options;
// TEAM_060: Using RowId directly instead of WorkspaceId alias
use super::super::row_types::RowId;
use super::super::ColumnWidth;
use niri_ipc::PositionChange;

impl<W: LayoutElement> Canvas2D<W> {
    // =========================================================================
    // Row Management
    // =========================================================================

    /// Creates a new row at the specified index if it doesn't exist.
    /// TEAM_057: Fixed row ID generation to avoid collisions across canvases.
    /// Each canvas gets a unique base ID from Layout, and we use a large stride
    /// to ensure IDs don't collide when multiple canvases create rows.
    pub fn ensure_row(&mut self, row_idx: i32) -> &mut Row<W> {
        self.rows.entry(row_idx).or_insert_with(|| {
            // TEAM_057: Generate unique row ID for new row
            // Use a large stride (1000) to avoid collisions with other canvases
            // Each canvas starts with a unique base from Layout.next_row_id()
            self.row_id_counter += 1000;
            let row_id = RowId(self.row_id_counter);
            
            Row::new(
                row_idx,
                row_id,
                self.view_size,
                self.parent_area,
                self.scale,
                self.clock.clone(),
                self.options.clone(),
            )
        })
    }

    /// Removes empty rows (except row 0 and named rows which are always kept).
    /// TEAM_042: Keep named rows to preserve workspace compatibility during transition
    pub fn cleanup_empty_rows(&mut self) {
        self.rows.retain(|&idx, row| idx == 0 || row.name().is_some() || !row.is_empty());
    }
    
    /// Renumbers rows to maintain contiguous indices starting from 0.
    /// This should be called after cleanup_empty_rows to ensure workspaces are contiguous.
    /// TEAM_059: Added to fix move_to_workspace_by_idx_does_not_leave_empty_workspaces test
    pub fn renumber_rows(&mut self) {
        // Take all rows out of the map
        let old_rows = std::mem::take(&mut self.rows);
        
        // Collect and sort by index
        let mut sorted_rows: Vec<(i32, Row<W>)> = old_rows.into_iter().collect();
        sorted_rows.sort_by_key(|(idx, _)| *idx);
        
        // Re-insert with contiguous indices starting from 0
        for (new_idx, (old_idx, mut row)) in sorted_rows.into_iter().enumerate() {
            let new_idx = new_idx as i32;
            if old_idx != new_idx {
                row.set_idx(new_idx);
            }
            self.rows.insert(new_idx, row);
        }
        
        // Update active_row_idx to point to a valid row
        if !self.rows.contains_key(&self.active_row_idx) {
            self.active_row_idx = self.rows.keys().next().copied().unwrap_or(0);
        }
    }
    
    /// Cleans up empty rows and renumbers remaining rows to be contiguous.
    /// TEAM_059: Combined cleanup and renumber for convenience
    pub fn cleanup_and_renumber_rows(&mut self) {
        self.cleanup_empty_rows();
        self.renumber_rows();
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

    /// TEAM_057: Find a row by its workspace/row ID.
    /// Returns the row index if found.
    pub fn find_row_by_id(&self, ws_id: super::super::row_types::RowId) -> Option<i32> {
        self.rows.iter()
            .find(|(_, row)| row.id() == ws_id)
            .map(|(&idx, _)| idx)
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

    /// TEAM_057: Set the height of a window in the canvas.
    /// Finds the row containing the window and calls set_window_height on it.
    pub fn set_window_height(&mut self, window_id: &W::Id, change: super::super::SizeChange) {
        // Find which row contains this window
        let row_idx = self.rows.iter()
            .find(|(_, row)| row.contains(window_id))
            .map(|(&idx, _)| idx);
        
        if let Some(idx) = row_idx {
            if let Some(row) = self.rows.get_mut(&idx) {
                row.set_window_height(Some(window_id), change);
            }
        }
    }

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
                    // TEAM_019: Row doesn't have individual layout configs - use Canvas options
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
            // TEAM_019: Implemented start_open_animation for Row
            // Use Row's start_open_animation which returns bool
            if row.start_open_animation(id) {
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
        if let Some(id) = id {
            // Center specific window
            if self.floating.has_window(id) {
                self.floating.center_window(Some(id));
            } else {
                // Find the row containing this tiled window and center its column
                for row in self.rows.values_mut() {
                    if row.contains(id) {
                        row.center_column();
                        break;
                    }
                }
            }
        } else {
            // Center active window
            if self.floating_is_active {
                self.floating.center_window(None);
            } else if let Some(row) = self.active_row_mut() {
                // TEAM_019: Implemented proper centering for tiled windows
                // Center the active column in the active row
                row.center_column();
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

    // TEAM_044: switch_focus_floating_tiling moved to floating.rs

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
                // TEAM_019: Implemented actual column reordering 
                // Move active column to first position
                row.move_column_to_index(0);
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
                // TEAM_019: Implemented actual column reordering
                // Move active column to last position
                row.move_column_to_index(last_idx);
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
    // TEAM_054: Fixed to also search floating space
    pub fn find_wl_surface(&self, wl_surface: &WlSurface) -> Option<&W> {
        // Check tiled windows
        if let Some(window) = self.tiled_tiles()
            .find(|tile| tile.window().is_wl_surface(wl_surface))
            .map(|tile| tile.window()) 
        {
            return Some(window);
        }
        
        // Check floating windows
        self.floating.tiles()
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

    /// Workspace equivalent: set fullscreen for window
    /// TEAM_050: Implemented - delegate to row containing the window
    /// TEAM_054: Added floating window support - move to tiled first
    /// TEAM_059: Added restore_to_floating support for proper state preservation
    pub fn set_fullscreen(&mut self, id: &W::Id, is_fullscreen: bool) {
        // Check if window is floating
        if self.floating.has_window(id) {
            if is_fullscreen {
                // Move from floating to tiled, then fullscreen
                // Note: floating.remove_tile() already stores floating_window_size via expected_size()
                let removed = self.floating.remove_tile(id);
                let mut tile = removed.tile;
                
                tile.animate_move_from(Point::from((0., 0.)));
                
                // TEAM_059: Mark this tile to restore to floating when unfullscreened
                tile.set_restore_to_floating(true);
                
                let width = removed.width;
                let is_full_width = removed.is_full_width;
                self.add_tile(tile, true, width, is_full_width);
                
                // TEAM_054: Switch to tiled mode since window is no longer floating
                if self.floating.is_empty() {
                    self.floating_is_active = false;
                }
                
                // Now fullscreen the window in its new tiled location
                for row in self.rows.values_mut() {
                    if row.has_window(id) {
                        row.set_fullscreen(id, true);
                        return;
                    }
                }
            }
            // If unsetting fullscreen on a floating window, nothing to do
            return;
        }
        
        // Find the row containing this window and delegate
        for row in self.rows.values_mut() {
            if row.has_window(id) {
                let should_restore_to_floating = row.set_fullscreen(id, is_fullscreen);
                
                // TEAM_059: If the row indicates we should restore to floating, move the window back
                if should_restore_to_floating {
                    self.toggle_floating_window_by_id(Some(id));
                }
                return;
            }
        }
    }

    /// Workspace equivalent: toggle fullscreen for window
    /// TEAM_050: Implemented - delegate to row containing the window
    /// TEAM_054: Added floating window support
    /// TEAM_059: Added restore_to_floating support
    pub fn toggle_fullscreen(&mut self, id: &W::Id) {
        // Check if window is floating
        if self.floating.has_window(id) {
            // Move from floating to tiled, then fullscreen
            // Note: floating.remove_tile() already stores floating_window_size via expected_size()
            let removed = self.floating.remove_tile(id);
            let mut tile = removed.tile;
            
            tile.animate_move_from(Point::from((0., 0.)));
            
            // TEAM_059: Mark this tile to restore to floating when unfullscreened
            tile.set_restore_to_floating(true);
            
            let width = removed.width;
            let is_full_width = removed.is_full_width;
            self.add_tile(tile, true, width, is_full_width);
            
            // TEAM_054: Switch to tiled mode since window is no longer floating
            if self.floating.is_empty() {
                self.floating_is_active = false;
            }
            
            // Now fullscreen the window in its new tiled location
            for row in self.rows.values_mut() {
                if row.has_window(id) {
                    row.toggle_fullscreen(id);
                    return;
                }
            }
            return;
        }
        
        // Find the row containing this window and delegate
        for row in self.rows.values_mut() {
            if row.has_window(id) {
                let should_restore_to_floating = row.toggle_fullscreen(id);
                
                // TEAM_059: If the row indicates we should restore to floating, move the window back
                if should_restore_to_floating {
                    self.toggle_floating_window_by_id(Some(id));
                }
                return;
            }
        }
    }

    /// Workspace equivalent: set maximized for window
    /// TEAM_050: Implemented - delegate to row containing the window
    /// TEAM_054: Added floating window support - move to tiled first
    /// TEAM_059: Added restore_to_floating support for proper state preservation
    pub fn set_maximized(&mut self, id: &W::Id, maximize: bool) {
        // Check if window is floating
        if self.floating.has_window(id) {
            if maximize {
                // Move from floating to tiled, then maximize
                // Note: floating.remove_tile() already stores floating_window_size via expected_size()
                let removed = self.floating.remove_tile(id);
                let mut tile = removed.tile;
                
                tile.animate_move_from(Point::from((0., 0.)));
                
                // TEAM_059: Mark this tile to restore to floating when unmaximized
                tile.set_restore_to_floating(true);
                
                let width = removed.width;
                let is_full_width = removed.is_full_width;
                self.add_tile(tile, true, width, is_full_width);
                
                // TEAM_054: Switch to tiled mode since window is no longer floating
                if self.floating.is_empty() {
                    self.floating_is_active = false;
                }
                
                // Now maximize the window in its new tiled location
                for row in self.rows.values_mut() {
                    if row.has_window(id) {
                        row.set_maximized(id, true);
                        return;
                    }
                }
            }
            // If unsetting maximized on a floating window, nothing to do
            return;
        }
        
        // Find the row containing this window and delegate
        for row in self.rows.values_mut() {
            if row.has_window(id) {
                let should_restore_to_floating = row.set_maximized(id, maximize);
                
                // TEAM_059: If the row indicates we should restore to floating, move the window back
                if should_restore_to_floating {
                    self.toggle_floating_window_by_id(Some(id));
                }
                return;
            }
        }
    }

    /// Workspace equivalent: toggle maximized for window
    /// TEAM_050: Implemented - delegate to row containing the window
    /// TEAM_054: Added floating window support
    /// TEAM_059: Added restore_to_floating support
    pub fn toggle_maximized(&mut self, id: &W::Id) {
        // Check if window is floating
        if self.floating.has_window(id) {
            // Move from floating to tiled, then maximize
            // Note: floating.remove_tile() already stores floating_window_size via expected_size()
            let removed = self.floating.remove_tile(id);
            let mut tile = removed.tile;
            
            tile.animate_move_from(Point::from((0., 0.)));
            
            // TEAM_059: Mark this tile to restore to floating when unmaximized
            tile.set_restore_to_floating(true);
            
            let width = removed.width;
            let is_full_width = removed.is_full_width;
            self.add_tile(tile, true, width, is_full_width);
            
            // TEAM_054: Switch to tiled mode since window is no longer floating
            if self.floating.is_empty() {
                self.floating_is_active = false;
            }
            
            // Now maximize the window in its new tiled location
            for row in self.rows.values_mut() {
                if row.has_window(id) {
                    row.toggle_maximized(id);
                    return;
                }
            }
            return;
        }
        
        // Find the row containing this window and delegate
        for row in self.rows.values_mut() {
            if row.has_window(id) {
                let should_restore_to_floating = row.toggle_maximized(id);
                
                // TEAM_059: If the row indicates we should restore to floating, move the window back
                if should_restore_to_floating {
                    self.toggle_floating_window_by_id(Some(id));
                }
                return;
            }
        }
    }

    /// Workspace equivalent: check if any window is urgent
    pub fn has_urgent_window(&self) -> bool {
        self.tiled_tiles().any(|tile| tile.window().is_urgent())
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
                let active_row_y = self.active_row()
                    .map(|row| row.y_offset())
                    .unwrap_or(0.0);
                let target_row_y = row.y_offset();
                target_row_y - active_row_y
            }
        } else {
            0.0 // Window not found, no scroll
        }
    }

    /// Workspace equivalent: get popup target rect
    pub fn popup_target_rect(&self, window: &W) -> smithay::utils::Rectangle<f64, smithay::utils::Logical> {
        // TEAM_021: Implemented proper popup positioning
        // Popups are positioned relative to their parent window, not canvas coordinates
        let size = window.size();
        smithay::utils::Rectangle::new((0.0, 0.0).into(), (size.w as f64, size.h as f64).into())
    }

    /// Workspace equivalent: descendants added check - TEAM_021
    pub fn descendants_added(&self, id: &W::Id) -> bool {
        // Find the window and check if it has any descendants
        if let Some((_, _, tile)) = self.find_window(id) {
            // For now, return false - descendants logic can be implemented later
            // This is typically used for popup windows and child surfaces
            false
        } else {
            false
        }
    }

    /// Workspace equivalent: clean up all empty rows - TEAM_021
    pub fn clean_up_workspaces(&mut self) {
        self.cleanup_empty_rows();
    }

    /// Workspace equivalent: get active workspace (row) - TEAM_021
    pub fn active_workspace(&self) -> Option<&Row<W>> {
        self.active_row()
    }

    /// Workspace equivalent: get active workspace (row) mutable - TEAM_021
    pub fn active_workspace_mut(&mut self) -> Option<&mut Row<W>> {
        self.active_row_mut()
    }

    /// Workspace equivalent: end DND scroll gesture on all rows - TEAM_021
    /// TEAM_033: Fixed to destructure tuple from rows_mut()
    pub fn dnd_scroll_gesture_end(&mut self) {
        for (_, row) in self.rows_mut() {
            row.dnd_scroll_gesture_end();
        }
    }

    /// Workspace equivalent: begin DND scroll gesture on all rows - TEAM_021
    /// TEAM_033: Fixed to destructure tuple from rows_mut()
    pub fn dnd_scroll_gesture_begin(&mut self) {
        for (_, row) in self.rows_mut() {
            row.dnd_scroll_gesture_begin();
        }
    }

    /// Workspace equivalent: start open animation for window - TEAM_021
    pub fn start_open_animation(&mut self, window: &W::Id) -> bool {
        // Find the window and start open animation
        if let Some((_, tile)) = self.find_window_mut(window) {
            tile.start_open_animation();
            true
        } else {
            false
        }
    }

    /// Check if any transitions are ongoing - TEAM_021
    pub fn are_transitions_ongoing(&self) -> bool {
        // Check all rows for ongoing transitions
        for row in self.rows() {
            if row.1.are_transitions_ongoing() {
                return true;
            }
        }
        false
    }
}
