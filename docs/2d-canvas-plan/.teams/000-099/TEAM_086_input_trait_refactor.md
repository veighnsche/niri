# TEAM_086: Input Handler Trait Refactor

## Objective
Replace `pub(super)` impl methods on `State` with proper trait-based handlers following Rust idioms (similar to Smithay's handler pattern).

## Problem
TEAM_085 extracted input handlers into separate files but used `pub(super)` on `impl State` methods. This:
- Violates Rule 7: "No `pub(super)` — if external code needs it, make a proper getter"
- Creates tight coupling between parent and child modules
- Looks like extending State's public API when it's really internal

## Solution
Create `pub(crate)` traits for each input category:
- `KeyboardInput<I: InputBackend>` - keyboard.rs
- `PointerInput<I: InputBackend>` - pointer.rs  
- `TouchInput<I: InputBackend>` - touch.rs
- `GestureInput<I: InputBackend + 'static>` - gesture.rs (where I::Device: 'static)
- `TabletInput<I: InputBackend>` - tablet.rs (where I::Device: 'static)
- `DeviceInput` - device.rs (non-generic)

Use explicit trait dispatch syntax: `TraitName::<I>::method(self, args)`.

## Files Modified
- src/input/keyboard.rs
- src/input/pointer.rs
- src/input/touch.rs
- src/input/gesture.rs
- src/input/tablet.rs
- src/input/device.rs
- src/input/mod.rs

## Technical Details

### Pattern Used
The traits are generic over `I: InputBackend` to handle the different event types.
Call sites use fully qualified trait syntax for type inference:
```rust
KeyboardInput::<I>::on_keyboard(self, event, &mut consumed_by_a11y)
```

### Non-Generic Helpers
Methods that don't depend on the backend type parameter (like `start_key_repeat` 
and `hide_cursor_if_needed`) are placed in a separate non-trait `impl State` block.

## Status
- [x] keyboard.rs - KeyboardInput trait
- [x] pointer.rs - PointerInput trait
- [x] touch.rs - TouchInput trait
- [x] gesture.rs - GestureInput trait
- [x] tablet.rs - TabletInput trait
- [x] device.rs - DeviceInput trait
- [x] mod.rs - Import traits and update calls
- [x] Test compilation

## Handoff
- [x] Code compiles (`cargo check`)
- [x] Main build works (`cargo build`)
- [ ] Tests pass - Pre-existing test issues with missing ColumnDisplay/PresetSize imports
- [x] Team file complete
