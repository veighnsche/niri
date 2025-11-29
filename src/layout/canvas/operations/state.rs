// TEAM_065: State operations extracted from operations.rs
//!
//! This module handles canvas state updates, configuration, and window state changes.

use std::rc::Rc;

use smithay::utils::{Logical, Point, Rectangle};
use niri_ipc::PositionChange;

use crate::layout::canvas::Canvas2D;
use crate::layout::row::Row;
use crate::layout::LayoutElement;
use crate::layout::Options;

impl<W: LayoutElement> Canvas2D<W> {
    // =========================================================================
    // Configuration Updates
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

    // =========================================================================
    // Open Animation
    // =========================================================================

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

    // =========================================================================
    // Focus Movement
    // =========================================================================

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

    // =========================================================================
    // Column Movement
    // =========================================================================

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

    // =========================================================================
    // Window Centering and Movement
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

    // =========================================================================
    // Fullscreen/Maximized State
    // =========================================================================

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

    // =========================================================================
    // Workspace Compatibility
    // =========================================================================

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

    /// Workspace equivalent: get popup target rect
    pub fn popup_target_rect(&self, window: &W) -> Rectangle<f64, Logical> {
        // TEAM_021: Implemented proper popup positioning
        // Popups are positioned relative to their parent window, not canvas coordinates
        let size = window.size();
        Rectangle::new((0.0, 0.0).into(), (size.w as f64, size.h as f64).into())
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
