// TEAM_064: Fullscreen/maximize operations extracted from mod.rs
//!
//! Fullscreen and maximize operations for Row.

use niri_ipc::ColumnDisplay;
use smithay::utils::{Logical, Size};

use super::super::LayoutElement;
use super::Row;

impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // Fullscreen operations
    // =========================================================================

    /// Set fullscreen state for a window.
    /// TEAM_059: Updated to return bool indicating if window should restore to floating
    pub fn set_fullscreen(&mut self, id: &W::Id, is_fullscreen: bool) -> bool {
        // Find the column containing this window
        let col_idx = self
            .columns
            .iter()
            .position(|col| col.contains(id));
        
        let Some(mut idx) = col_idx else {
            return false;
        };

        // Check if state is already the same
        if is_fullscreen == self.columns[idx].is_pending_fullscreen {
            return false;
        }

        let is_tabbed = self.columns[idx].display_mode == ColumnDisplay::Tabbed;
        let has_multiple_tiles = self.columns[idx].tiles.len() > 1;

        // Cancel any ongoing resize for this column
        if let Some(resize) = &mut self.interactive_resize {
            if &resize.window == id {
                self.interactive_resize = None;
            }
        }

        // TEAM_054: If setting fullscreen and column has multiple tiles, extract the window
        if is_fullscreen && has_multiple_tiles && !is_tabbed {
            // This wasn't the only window in its column; extract it into a separate column.
            self.consume_or_expel_window_right(Some(id));
            idx += 1;
        }

        // Check if we need to restore maximize state BEFORE setting fullscreen to false
        let should_restore_maximized = !is_fullscreen && self.columns[idx].is_pending_maximized;

        // TEAM_059: Check if we should restore to floating when unmaximizing (not unfullscreening)
        let mut should_restore_to_floating = false;
        if !is_fullscreen && !self.columns[idx].is_pending_maximized {
            // Only restore to floating if we're unfullscreening AND the window is not maximized
            // Check if the active tile in this column should restore to floating
            if let Some(tile) = self.columns[idx].tiles_iter().nth(self.columns[idx].active_tile_idx) {
                should_restore_to_floating = tile.should_restore_to_floating();
            }
        }

        self.columns[idx].set_fullscreen(is_fullscreen);

        // Update column data
        self.data[idx].update(&self.columns[idx]);
        
        // If unfullscreening and the column was previously maximized, restore maximize state
        if should_restore_maximized {
            // Store the window ID before calling set_maximized since it might move the column
            let window_id = id.clone();
            self.set_maximized(&window_id, true);
            // Ignore the return value since we're restoring maximize, not going to floating
        }
        
        // TEAM_050: View offset animation is handled in update_window() after the window
        // acknowledges the fullscreen state. This ensures view_offset_to_restore is saved
        // with the correct (pre-fullscreen) offset before any animation starts.
        
        should_restore_to_floating
    }
    
    /// Toggle fullscreen state for a window.
    /// TEAM_059: Updated to return bool indicating if window should restore to floating
    pub fn toggle_fullscreen(&mut self, id: &W::Id) -> bool {
        // Find the column containing this window
        let col_idx = self
            .columns
            .iter()
            .position(|col| col.contains(id));
        
        let Some(col_idx) = col_idx else {
            return false;
        };

        let current_state = self.columns[col_idx].is_pending_fullscreen;
        self.set_fullscreen(id, !current_state)
    }

    // =========================================================================
    // Maximize operations
    // =========================================================================
    
    /// Set maximized state for a window.
    /// TEAM_059: Updated to return bool indicating if window should restore to floating
    pub fn set_maximized(&mut self, id: &W::Id, maximize: bool) -> bool {
        // Find the column containing this window
        let col_idx = self
            .columns
            .iter()
            .position(|col| col.contains(id));
        
        let Some(mut idx) = col_idx else {
            return false;
        };

        // Check if state is already the same
        if maximize == self.columns[idx].is_pending_maximized {
            return false;
        }

        let is_tabbed = self.columns[idx].display_mode == ColumnDisplay::Tabbed;
        let has_multiple_tiles = self.columns[idx].tiles.len() > 1;

        // Cancel any ongoing resize for this column
        if let Some(resize) = &mut self.interactive_resize {
            if &resize.window == id {
                self.interactive_resize = None;
            }
        }

        // If setting maximize and column has multiple tiles, extract the window
        if maximize && has_multiple_tiles && !is_tabbed {
            // This wasn't the only window in its column; extract it into a separate column.
            self.consume_or_expel_window_right(Some(id));
            idx += 1;
        }

        // TEAM_059: Check if we should restore to floating when unmaximizing
        let mut should_restore_to_floating = false;
        if !maximize && !self.columns[idx].is_pending_fullscreen {
            // Only restore to floating if we're unmaximizing AND the window is not fullscreen
            // Check if the active tile in this column should restore to floating
            if let Some(tile) = self.columns[idx].tiles_iter().nth(self.columns[idx].active_tile_idx) {
                should_restore_to_floating = tile.should_restore_to_floating();
            }
        }

        self.columns[idx].set_maximized(maximize);

        // Update column data
        self.data[idx].update(&self.columns[idx]);
        
        should_restore_to_floating
    }
    
    /// Toggle maximized state for a window.
    /// TEAM_059: Updated to return bool indicating if window should restore to floating
    pub fn toggle_maximized(&mut self, id: &W::Id) -> bool {
        // Find the column containing this window
        let col_idx = self
            .columns
            .iter()
            .position(|col| col.contains(id));
        
        let Some(col_idx) = col_idx else {
            return false;
        };

        let current_state = self.columns[col_idx].is_pending_maximized;
        self.set_maximized(id, !current_state)
    }

    // =========================================================================
    // Fullscreen size helpers
    // =========================================================================
    
    /// TEAM_053: Get fullscreen size for a window before removing it
    /// This is needed by Canvas2D::toggle_floating_window_by_id to preserve fullscreen dimensions
    pub fn get_fullscreen_size_for_window(&self, id: &W::Id) -> Option<Size<i32, Logical>> {
        // Find the column containing this window
        let col_idx = self
            .columns
            .iter()
            .position(|col| col.contains(id));
        
        let Some(col_idx) = col_idx else {
            eprintln!("TEAM_053 DEBUG: No column found for window");
            return None;
        };

        let col = &self.columns[col_idx];
        
        // TEAM_053: Debug both column and window fullscreen states
        eprintln!("TEAM_053 DEBUG: col.is_pending_fullscreen: {}", col.is_pending_fullscreen);
        if let Some(tile) = col.tiles.iter().find(|tile| tile.window().id() == id) {
            eprintln!("TEAM_053 DEBUG: window.pending_sizing_mode(): {:?}", tile.window().pending_sizing_mode());
            eprintln!("TEAM_053 DEBUG: window.expected_size(): {:?}", tile.window().expected_size());
        }
        
        // TEAM_053: Check fullscreen state at column level, not tile level
        // Fullscreen state is stored in col.is_pending_fullscreen, not in window.pending_sizing_mode()
        if col.is_pending_fullscreen {
            eprintln!("TEAM_053 DEBUG: Column is pending fullscreen, getting size from active tile");
            // Return the fullscreen size (expected_size of the fullscreen window)
            col.tiles.iter().find(|tile| tile.window().id() == id)
                .and_then(|tile| tile.window().expected_size())
        } else {
            eprintln!("TEAM_053 DEBUG: Column is not pending fullscreen, checking window state instead");
            // Fallback: check if window itself reports fullscreen
            if let Some(tile) = col.tiles.iter().find(|tile| tile.window().id() == id) {
                if tile.window().pending_sizing_mode().is_fullscreen() {
                    eprintln!("TEAM_053 DEBUG: Window reports fullscreen, using its size");
                    return tile.window().expected_size();
                }
            }
            None
        }
    }
}
