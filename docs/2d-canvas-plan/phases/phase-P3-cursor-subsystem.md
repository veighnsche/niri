# Phase P3: Extract CursorSubsystem

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1.5 hours  
> **Risk Level**: 🟡 Medium (state machine design)  
> **Prerequisite**: Phase P2 complete  
> **Creates**: `CursorSubsystem` struct

---

## Goal

Extract all cursor/pointer-related state from `Niri` into a dedicated `CursorSubsystem` that:
- **Models cursor as a state machine** (visible, hidden, disabled)
- **Owns** all cursor-related state
- **Encapsulates** cursor management logic
- **Provides** clear API for cursor operations

---

## Why a Subsystem?

### Current State: Scattered Fields
```rust
pub struct Niri {
    // Cursor state scattered across 10+ fields
    pub cursor_manager: CursorManager,
    pub cursor_texture_cache: CursorTextureCache,
    pub dnd_icon: Option<DndIcon>,
    pub pointer_contents: PointContents,
    pub pointer_visibility: PointerVisibility,
    pub pointer_inactivity_timer: Option<RegistrationToken>,
    pub pointer_inactivity_timer_got_reset: bool,
    pub pointer_inside_hot_corner: bool,
    pub tablet_cursor_location: Option<Point<f64, Logical>>,
    // ... spread across struct
}
```

### Target: Cohesive Subsystem
```rust
pub struct CursorSubsystem {
    // All cursor state owned here
    manager: CursorManager,
    texture_cache: CursorTextureCache,
    visibility: PointerVisibility,
    contents: PointContents,
    dnd_icon: Option<DndIcon>,
    tablet_location: Option<Point<f64, Logical>>,
    
    // Inactivity tracking
    inactivity_timer: Option<RegistrationToken>,
    timer_reset_this_iter: bool,
    
    // Hot corner state
    inside_hot_corner: bool,
}

impl CursorSubsystem {
    pub fn set_visibility(&mut self, vis: PointerVisibility);
    pub fn hide_for_inactivity(&mut self);
    pub fn show(&mut self);
    pub fn get_render_cursor(&self, scale: i32) -> RenderCursor;
    pub fn update_contents(&mut self, under: PointContents);
    // ...
}
```

---

## Current State Analysis

### Fields to Move from Niri (mod.rs lines ~357-388)

```rust
pub cursor_manager: CursorManager,                    // Cursor theme loading
pub cursor_texture_cache: CursorTextureCache,         // Texture caching
pub cursor_shape_manager_state: CursorShapeManagerState, // Move to ProtocolStates
pub dnd_icon: Option<DndIcon>,                        // Drag-and-drop icon
pub pointer_contents: PointContents,                  // What's under cursor
pub pointer_visibility: PointerVisibility,            // Visible/Hidden/Disabled
pub pointer_inactivity_timer: Option<RegistrationToken>,
pub pointer_inactivity_timer_got_reset: bool,
pub pointer_inside_hot_corner: bool,
pub tablet_cursor_location: Option<Point<f64, Logical>>,
```

### Methods Currently on State (cursor-related)

```rust
// In mod.rs (~200 lines)
pub fn move_cursor(&mut self, location: Point<f64, Logical>);
fn move_cursor_to_rect(&mut self, rect: Rectangle<f64, Logical>, mode: CenterCoords) -> bool;
pub fn move_cursor_to_focused_tile(&mut self, mode: CenterCoords) -> bool;
pub fn maybe_warp_cursor_to_focus(&mut self) -> bool;
pub fn maybe_warp_cursor_to_focus_centered(&mut self) -> bool;
pub fn move_cursor_to_output(&mut self, output: &Output);
pub fn refresh_pointer_contents(&mut self);
pub fn update_pointer_contents(&mut self) -> bool;
```

---

## Target Architecture

### New File: `src/niri/subsystems/cursor.rs`

```rust
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
```

---

## Work Units

### Unit 1: Add CursorSubsystem to subsystems/mod.rs

```rust
mod cursor;
mod outputs;

pub use cursor::CursorSubsystem;
pub use outputs::OutputSubsystem;
```

---

### Unit 2: Create CursorSubsystem Struct

Create `src/niri/subsystems/cursor.rs` with:
1. Struct definition with private fields
2. Constructor and accessor methods
3. Visibility state machine methods

**Verify**: `cargo check`

---

### Unit 3: Move Fields from Niri

1. Remove cursor-related fields from `Niri` struct
2. Add `pub cursor: CursorSubsystem` field
3. Update `Niri::new` to create `CursorSubsystem`

**Verify**: `cargo check` (will fail - access patterns need updating)

---

### Unit 4: Update Access Patterns

```rust
// Before
self.cursor_manager.get_render_cursor(scale)
self.pointer_visibility.is_visible()
self.pointer_contents

// After
self.cursor.get_render_cursor(scale)
self.cursor.is_visible()
self.cursor.contents()
```

---

### Unit 5: Refactor State Cursor Methods

The `impl State` cursor methods need to be refactored:
- Some logic moves into `CursorSubsystem`
- Coordination logic stays in `State` but calls subsystem

```rust
// Before (in mod.rs)
impl State {
    pub fn move_cursor(&mut self, location: Point<f64, Logical>) {
        // 40 lines of mixed logic
    }
}

// After (coordination in State, logic in subsystem)
impl State {
    pub fn move_cursor(&mut self, location: Point<f64, Logical>) {
        let contents = match self.niri.cursor.visibility() {
            PointerVisibility::Disabled => PointContents::default(),
            _ => self.niri.contents_under(location),
        };
        self.niri.cursor.update_contents(contents);
        // ... pointer motion handling
    }
}
```

---

## Verification Checklist

- [ ] `CursorSubsystem` struct exists with private fields
- [ ] All cursor fields removed from `Niri`
- [ ] `Niri.cursor: CursorSubsystem` field added
- [ ] Visibility state machine methods work
- [ ] All access patterns updated
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/cursor.rs` | +300 lines (new) |
| `src/niri/subsystems/mod.rs` | +3 lines |
| `src/niri/mod.rs` | -15 lines (fields), +2 lines (field) |
| `src/niri/init.rs` | Updated initialization |
| Various files | Updated access patterns |

---

## Benefits Achieved

1. **State machine clarity**: Visibility states explicitly modeled
2. **Single owner**: All cursor state in one place
3. **Encapsulation**: Private fields with intentional API
4. **Reduced Niri complexity**: 10 fewer fields
5. **Discoverability**: `niri.cursor.` gives all cursor operations

---

## Next Phase

After completing this phase, proceed to [Phase P4: FocusModel](phase-P4-focus-model.md).
