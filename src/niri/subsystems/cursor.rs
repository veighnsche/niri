//! Cursor/pointer management subsystem.
//!
//! Handles cursor visibility, positioning, rendering, and input device state.

use std::time::Duration;

use calloop::RegistrationToken;
use smithay::input::pointer::CursorImageStatus;
use smithay::output::Output;
use smithay::utils::{Logical, Point};

use crate::cursor::{CursorManager, CursorTextureCache, RenderCursor};
use super::super::types::{CenterCoords, DndIcon, PointContents, PointerVisibility};

/// Cursor/pointer management subsystem.
///
/// Models the cursor as a state machine with visibility states,
/// manages cursor textures, and tracks what's under the pointer.
pub struct CursorSubsystem {
    /// Cursor theme manager.
    manager: CursorManager,
    
    /// Cached cursor textures.
    texture_cache: CursorTextureCache,
    
    /// Current visibility state.
    visibility: PointerVisibility,
    
    /// What's currently under the cursor.
    contents: PointContents,
    
    /// Drag-and-drop icon surface.
    dnd_icon: Option<DndIcon>,
    
    /// Tablet cursor location (if using tablet).
    tablet_location: Option<Point<f64, Logical>>,
    
    /// Inactivity timer for auto-hide.
    inactivity_timer: Option<RegistrationToken>,
    
    /// Whether the inactivity timer was reset this iteration.
    timer_reset_this_iter: bool,
    
    /// Whether cursor is inside the hot corner.
    inside_hot_corner: bool,
}

impl CursorSubsystem {
    /// Creates a new cursor subsystem.
    pub fn new(manager: CursorManager) -> Self {
        Self {
            manager,
            texture_cache: CursorTextureCache::default(),
            visibility: PointerVisibility::default(),
            contents: PointContents::default(),
            dnd_icon: None,
            tablet_location: None,
            inactivity_timer: None,
            timer_reset_this_iter: false,
            inside_hot_corner: false,
        }
    }
    
    // =========================================================================
    // Visibility State Machine
    // =========================================================================
    
    /// Returns current visibility state.
    pub fn visibility(&self) -> PointerVisibility {
        self.visibility
    }
    
    /// Returns whether the cursor is visible.
    pub fn is_visible(&self) -> bool {
        self.visibility.is_visible()
    }
    
    /// Sets visibility state directly.
    pub fn set_visibility(&mut self, visibility: PointerVisibility) {
        self.visibility = visibility;
    }
    
    /// Hides the cursor due to inactivity (retains focus).
    pub fn hide_for_inactivity(&mut self) {
        if self.visibility == PointerVisibility::Visible {
            self.visibility = PointerVisibility::Hidden;
        }
    }
    
    /// Shows the cursor (from hidden or disabled).
    pub fn show(&mut self) {
        self.visibility = PointerVisibility::Visible;
    }
    
    /// Disables the cursor completely (loses focus).
    pub fn disable(&mut self) {
        self.visibility = PointerVisibility::Disabled;
    }
    
    // =========================================================================
    // Rendering
    // =========================================================================
    
    /// Gets the cursor for rendering at the given scale.
    pub fn get_render_cursor(&self, scale: i32) -> RenderCursor {
        self.manager.get_render_cursor(scale)
    }
    
    /// Returns a reference to the cursor manager.
    pub fn manager(&self) -> &CursorManager {
        &self.manager
    }
    
    /// Returns a mutable reference to the cursor manager.
    pub fn manager_mut(&mut self) -> &mut CursorManager {
        &mut self.manager
    }
    
    /// Returns a reference to the texture cache.
    pub fn texture_cache(&self) -> &CursorTextureCache {
        &self.texture_cache
    }
    
    /// Returns a mutable reference to the texture cache.
    pub fn texture_cache_mut(&mut self) -> &mut CursorTextureCache {
        &mut self.texture_cache
    }
    
    // =========================================================================
    // Contents Under Cursor
    // =========================================================================
    
    /// Returns what's currently under the cursor.
    pub fn contents(&self) -> &PointContents {
        &self.contents
    }
    
    /// Updates what's under the cursor.
    pub fn update_contents(&mut self, contents: PointContents) {
        self.contents = contents;
    }
    
    // =========================================================================
    // Tablet Support
    // =========================================================================
    
    /// Returns the tablet cursor location if active.
    pub fn tablet_location(&self) -> Option<Point<f64, Logical>> {
        self.tablet_location
    }
    
    /// Sets the tablet cursor location.
    pub fn set_tablet_location(&mut self, location: Option<Point<f64, Logical>>) {
        self.tablet_location = location;
    }
    
    // =========================================================================
    // Drag and Drop
    // =========================================================================
    
    /// Returns the current DnD icon.
    pub fn dnd_icon(&self) -> Option<&DndIcon> {
        self.dnd_icon.as_ref()
    }
    
    /// Sets the DnD icon.
    pub fn set_dnd_icon(&mut self, icon: Option<DndIcon>) {
        self.dnd_icon = icon;
    }
    
    // =========================================================================
    // Inactivity Timer
    // =========================================================================
    
    /// Returns the inactivity timer token.
    pub fn inactivity_timer(&self) -> Option<RegistrationToken> {
        self.inactivity_timer
    }
    
    /// Sets the inactivity timer token.
    pub fn set_inactivity_timer(&mut self, token: Option<RegistrationToken>) {
        self.inactivity_timer = token;
    }
    
    /// Returns whether the timer was reset this iteration.
    pub fn timer_reset_this_iter(&self) -> bool {
        self.timer_reset_this_iter
    }
    
    /// Marks the timer as reset for this iteration.
    pub fn mark_timer_reset(&mut self) {
        self.timer_reset_this_iter = true;
    }
    
    /// Clears the timer reset flag (call at end of event loop iteration).
    pub fn clear_timer_reset_flag(&mut self) {
        self.timer_reset_this_iter = false;
    }
    
    // =========================================================================
    // Hot Corner
    // =========================================================================
    
    /// Returns whether cursor is in the hot corner.
    pub fn inside_hot_corner(&self) -> bool {
        self.inside_hot_corner
    }
    
    /// Sets whether cursor is in the hot corner.
    pub fn set_inside_hot_corner(&mut self, inside: bool) {
        self.inside_hot_corner = inside;
    }
    
    // =========================================================================
    // Lifecycle
    // =========================================================================
    
    /// Checks if the cursor image surface is still alive.
    pub fn check_cursor_image_alive(&mut self) {
        self.manager.check_cursor_image_surface_alive();
    }
}
