# Phase C: Extract CursorSubsystem

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1.5 hours  
> **Risk Level**: 🟡 Medium (state machine design)  
> **Prerequisite**: Phase B complete  
> **Creates**: `CursorSubsystem` struct

---

## Goal

Extract all cursor/pointer-related state into a `CursorSubsystem` that:
- **Owns** cursor state (visibility, position, contents)
- **Manages** cursor texture caching
- **Handles** pointer inactivity timer

---

## Fields to Move from Niri

```rust
// Cursor state (mod.rs lines ~357-388)
pub cursor_manager: CursorManager,
pub cursor_texture_cache: CursorTextureCache,
pub dnd_icon: Option<DndIcon>,
pub pointer_contents: PointContents,
pub pointer_visibility: PointerVisibility,
pub pointer_inactivity_timer: Option<RegistrationToken>,
pub pointer_inactivity_timer_got_reset: bool,
pub notified_activity_this_iteration: bool,
pub pointer_inside_hot_corner: bool,
pub tablet_cursor_location: Option<Point<f64, Logical>>,
```

---

## Target Architecture

### New File: `src/niri/subsystems/cursor.rs`

```rust
//! Cursor management subsystem.
//!
//! Owns all state related to the mouse cursor/pointer:
//! - Cursor theme and texture cache
//! - Pointer visibility state machine
//! - What's under the cursor
//! - Inactivity timer for auto-hide

use smithay::reexports::calloop::RegistrationToken;
use smithay::utils::{Logical, Point};

use crate::cursor::{CursorManager, CursorTextureCache};
use crate::niri::types::{DndIcon, PointContents, PointerVisibility};

/// Cursor management subsystem.
pub struct CursorSubsystem {
    /// Cursor theme manager.
    manager: CursorManager,
    
    /// Cached cursor textures.
    texture_cache: CursorTextureCache,
    
    /// Current DnD icon if dragging.
    dnd_icon: Option<DndIcon>,
    
    /// What's currently under the pointer.
    contents: PointContents,
    
    /// Pointer visibility state.
    visibility: PointerVisibility,
    
    /// Inactivity timer for auto-hiding cursor.
    inactivity_timer: Option<RegistrationToken>,
    
    /// Whether inactivity timer was reset this iteration.
    timer_reset_this_iteration: bool,
    
    /// Whether activity was notified this iteration.
    activity_notified_this_iteration: bool,
    
    /// Whether pointer is inside a hot corner.
    inside_hot_corner: bool,
    
    /// Tablet cursor location (if using tablet).
    tablet_location: Option<Point<f64, Logical>>,
}

impl CursorSubsystem {
    /// Creates a new cursor subsystem.
    pub fn new(config: &niri_config::Config) -> Self {
        Self {
            manager: CursorManager::new(config),
            texture_cache: CursorTextureCache::new(),
            dnd_icon: None,
            contents: PointContents::default(),
            visibility: PointerVisibility::default(),
            inactivity_timer: None,
            timer_reset_this_iteration: false,
            activity_notified_this_iteration: false,
            inside_hot_corner: false,
            tablet_location: None,
        }
    }
    
    // =========================================================================
    // Cursor Manager Access
    // =========================================================================
    
    /// Returns the cursor manager.
    pub fn manager(&self) -> &CursorManager {
        &self.manager
    }
    
    /// Returns mutable cursor manager.
    pub fn manager_mut(&mut self) -> &mut CursorManager {
        &mut self.manager
    }
    
    /// Returns the texture cache.
    pub fn texture_cache(&self) -> &CursorTextureCache {
        &self.texture_cache
    }
    
    /// Returns mutable texture cache.
    pub fn texture_cache_mut(&mut self) -> &mut CursorTextureCache {
        &mut self.texture_cache
    }
    
    // =========================================================================
    // Visibility State Machine
    // =========================================================================
    
    /// Returns current visibility state.
    pub fn visibility(&self) -> PointerVisibility {
        self.visibility
    }
    
    /// Returns whether cursor is visible.
    pub fn is_visible(&self) -> bool {
        self.visibility.is_visible()
    }
    
    /// Shows the cursor.
    pub fn show(&mut self) {
        self.visibility = PointerVisibility::Visible;
    }
    
    /// Hides the cursor due to inactivity.
    pub fn hide_due_to_inactivity(&mut self) {
        self.visibility = PointerVisibility::Hidden;
    }
    
    /// Disables cursor (e.g., when locked).
    pub fn disable(&mut self) {
        self.visibility = PointerVisibility::Disabled;
    }
    
    /// Sets visibility state.
    pub fn set_visibility(&mut self, visibility: PointerVisibility) {
        self.visibility = visibility;
    }
    
    // =========================================================================
    // Contents Under Pointer
    // =========================================================================
    
    /// Returns what's under the pointer.
    pub fn contents(&self) -> &PointContents {
        &self.contents
    }
    
    /// Updates what's under the pointer.
    pub fn set_contents(&mut self, contents: PointContents) {
        self.contents = contents;
    }
    
    // =========================================================================
    // DnD Icon
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
    
    /// Returns whether timer was reset this iteration.
    pub fn timer_reset_this_iteration(&self) -> bool {
        self.timer_reset_this_iteration
    }
    
    /// Marks timer as reset for this iteration.
    pub fn mark_timer_reset(&mut self) {
        self.timer_reset_this_iteration = true;
    }
    
    /// Clears the timer reset flag (call at start of iteration).
    pub fn clear_timer_reset_flag(&mut self) {
        self.timer_reset_this_iteration = false;
    }
    
    /// Returns whether activity was notified this iteration.
    pub fn activity_notified(&self) -> bool {
        self.activity_notified_this_iteration
    }
    
    /// Marks activity as notified for this iteration.
    pub fn mark_activity_notified(&mut self) {
        self.activity_notified_this_iteration = true;
    }
    
    /// Clears the activity notified flag.
    pub fn clear_activity_flag(&mut self) {
        self.activity_notified_this_iteration = false;
    }
    
    // =========================================================================
    // Hot Corner
    // =========================================================================
    
    /// Returns whether pointer is inside hot corner.
    pub fn inside_hot_corner(&self) -> bool {
        self.inside_hot_corner
    }
    
    /// Sets hot corner state.
    pub fn set_inside_hot_corner(&mut self, inside: bool) {
        self.inside_hot_corner = inside;
    }
    
    // =========================================================================
    // Tablet Location
    // =========================================================================
    
    /// Returns tablet cursor location if set.
    pub fn tablet_location(&self) -> Option<Point<f64, Logical>> {
        self.tablet_location
    }
    
    /// Sets tablet cursor location.
    pub fn set_tablet_location(&mut self, location: Option<Point<f64, Logical>>) {
        self.tablet_location = location;
    }
    
    /// Returns effective cursor location (tablet or pointer).
    pub fn effective_location(&self, pointer_location: Point<f64, Logical>) -> Point<f64, Logical> {
        self.tablet_location.unwrap_or(pointer_location)
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
pub use outputs::{OutputSubsystem, OutputState};
```

---

### Unit 2: Create CursorSubsystem struct

Create `src/niri/subsystems/cursor.rs` with struct and accessors.

**Verify**: `cargo check`

---

### Unit 3: Move fields from Niri

1. Remove cursor-related fields from `Niri` struct
2. Add `pub cursor: CursorSubsystem` field
3. Update `Niri::new()` to create `CursorSubsystem`

---

### Unit 4: Update access patterns

```rust
// Before
self.cursor_manager.get_render_cursor(scale)
self.pointer_visibility.is_visible()
self.pointer_contents = contents;

// After
self.cursor.manager().get_render_cursor(scale)
self.cursor.is_visible()
self.cursor.set_contents(contents);
```

---

### Unit 5: Update pointer.rs methods

Update methods in `pointer.rs` to use subsystem:
```rust
// pointer.rs
impl Niri {
    pub fn reset_pointer_inactivity_timer(&mut self) {
        // Use self.cursor instead of direct fields
    }
}
```

---

### Unit 6: Update render.rs

Update `pointer_element()` method:
```rust
impl Niri {
    pub fn pointer_element<R: NiriRenderer>(&self, ...) {
        if !self.cursor.is_visible() {
            return vec![];
        }
        let render_cursor = self.cursor.manager().get_render_cursor(scale);
        // ...
    }
}
```

---

### Unit 7: Verify

```bash
cargo check
cargo test
```

---

## Verification Checklist

- [ ] `CursorSubsystem` struct with private fields
- [ ] All cursor fields removed from Niri
- [ ] `Niri.cursor: CursorSubsystem` field added
- [ ] Visibility state machine works
- [ ] Inactivity timer management works
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/cursor.rs` | **NEW** ~300 lines |
| `src/niri/subsystems/mod.rs` | +2 lines |
| `src/niri/mod.rs` | -10 fields |
| `src/niri/pointer.rs` | Updated |
| `src/niri/render.rs` | Updated |

---

## Benefits

1. **-10 fields** from Niri struct
2. **Encapsulated** cursor state
3. **Clear visibility state machine**
4. **Testable** cursor logic

---

## Next Phase

After completing this phase, proceed to [Phase D: FocusModel](phase-D-focus-model.md).
