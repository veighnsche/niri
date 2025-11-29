# Phase P5: Create cursor.rs

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟡 Medium (State method patterns)  
> **Prerequisite**: None

---

## Goal

Create new `cursor.rs` module for cursor movement methods from `impl State`.

These methods manipulate cursor position and need access to both `State.backend` and `State.niri`.

Total: ~200 lines moved from mod.rs

---

## Functions to Move

All these are `impl State` methods:

### 1. move_cursor (mod.rs ~lines 652-690)
```rust
pub fn move_cursor(&mut self, location: Point<f64, Logical>) {
    // ~40 lines - sets cursor position and updates pointer contents
}
```

### 2. move_cursor_to_rect (mod.rs ~lines 691-726)
```rust
fn move_cursor_to_rect(&mut self, rect: Rectangle<f64, Logical>, mode: CenterCoords) -> bool {
    // ~35 lines - moves cursor to be within a rectangle
}
```

### 3. move_cursor_to_focused_tile (mod.rs ~lines 727-753)
```rust
pub fn move_cursor_to_focused_tile(&mut self, mode: CenterCoords) -> bool {
    // ~25 lines - warps cursor to active tile
}
```

### 4. maybe_warp_cursor_to_focus (mod.rs ~lines 804-815)
```rust
pub fn maybe_warp_cursor_to_focus(&mut self) -> bool {
    // ~12 lines - conditional warp based on config
}
```

### 5. maybe_warp_cursor_to_focus_centered (mod.rs ~lines 816-827)
```rust
pub fn maybe_warp_cursor_to_focus_centered(&mut self) -> bool {
    // ~12 lines - conditional centered warp
}
```

### 6. move_cursor_to_output (mod.rs ~lines 906-910)
```rust
pub fn move_cursor_to_output(&mut self, output: &Output) {
    // ~5 lines - moves cursor to output center
}
```

### 7. refresh_pointer_contents (mod.rs ~lines 828-859)
```rust
pub fn refresh_pointer_contents(&mut self) {
    // ~30 lines - updates pointer focus state
}
```

### 8. update_pointer_contents (mod.rs ~lines 860-905)
```rust
pub fn update_pointer_contents(&mut self) -> bool {
    // ~45 lines - updates what's under the pointer
}
```

---

## Work Units

### Unit 1: Create cursor.rs File

Create `src/niri/cursor.rs`:

```rust
//! Cursor movement and positioning for the Niri compositor.
//!
//! This module handles cursor/pointer movement methods that need access
//! to both the backend and Niri state.

use niri_config::WarpMouseToFocusMode;
use smithay::input::pointer::MotionEvent;
use smithay::output::Output;
use smithay::utils::{Logical, Point, Rectangle, SERIAL_COUNTER};

use crate::utils::{center_f64, get_monotonic_time};

use super::{CenterCoords, Niri, PointerVisibility, PointContents, State};

// =============================================================================
// Cursor Movement Methods (impl State)
// =============================================================================

impl State {
    // Methods will go here
}
```

---

### Unit 2: Add Module Declaration

In `mod.rs`, add:

```rust
mod cursor;
mod frame_callbacks;
// ... other mods
```

---

### Unit 3: Move move_cursor

```rust
impl State {
    /// Moves the cursor to the specified location.
    pub fn move_cursor(&mut self, location: Point<f64, Logical>) {
        let mut under = match self.niri.pointer_visibility {
            PointerVisibility::Disabled => PointContents::default(),
            _ => self.niri.contents_under(location),
        };

        // ... rest of implementation
    }
}
```

**Verify**: `cargo check`

---

### Unit 4: Move Remaining Cursor Methods

Move all the cursor-related methods one by one, verifying after each:

1. `move_cursor_to_rect`
2. `move_cursor_to_focused_tile`
3. `maybe_warp_cursor_to_focus`
4. `maybe_warp_cursor_to_focus_centered`
5. `move_cursor_to_output`
6. `refresh_pointer_contents`
7. `update_pointer_contents`

---

## Verification Checklist

- [ ] `cursor.rs` exists with module header
- [ ] All cursor methods moved
- [ ] Module declared in mod.rs
- [ ] No duplicate definitions
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/cursor.rs` | +200 lines (new file) |
| `src/niri/mod.rs` | -200 lines, +1 line (mod declaration) |

**Expected mod.rs after P5**: ~2624 lines

---

## Technical Notes

### State vs Niri Methods

These are `impl State` methods (not `impl Niri`) because they need:
- `self.niri` - for Niri state
- Implicit access to pointer/seat via `self.niri.seat`

They could potentially be refactored to `impl Niri` methods that take the pointer as a parameter, but that's a larger change.

### Dependencies

**Niri fields used**:
- `pointer_visibility`
- `pointer_contents`
- `keyboard_focus`
- `tablet_cursor_location`
- `layout`
- `global_space`
- `seat`
- `config`

**Niri methods called**:
- `contents_under()` - in hit_test.rs
- `maybe_activate_pointer_constraint()` - in pointer.rs
- `queue_redraw_all()` - in render.rs (after P4)

---

## Next Phase

After completing this phase, proceed to [Phase P6: Create focus.rs](phase-P6-focus.md).
