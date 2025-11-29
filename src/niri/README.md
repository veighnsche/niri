# niri/ Module Architecture

This module implements the core compositor state, organized into domain subsystems.

## Overview

The niri module has been refactored from a god object with 195 public fields to a clean architecture with just 48 public fields, organized into focused subsystems.

## Subsystems

### OutputSubsystem (`subsystems/outputs.rs`)
Manages physical outputs (monitors), their positions, and per-output state.

**Responsibilities:**
- Global coordinate space and output positioning
- Per-output frame clocks and redraw state
- Spatial queries (find outputs under positions)
- Power management and lid detection

### CursorSubsystem (`subsystems/cursor.rs`)
Handles cursor visibility, positioning, and the cursor state machine.

**Responsibilities:**
- Cursor visibility management
- Position tracking and movement
- Cursor rendering and texture caching
- Input device state coordination

### FocusModel (`subsystems/focus.rs`)
Computes and tracks keyboard focus based on priority rules.

**Responsibilities:**
- Focus priority computation
- Layer surface focus handling
- Keyboard shortcut inhibition
- Focus context management

### StreamingSubsystem (`subsystems/streaming.rs`)
Manages PipeWire streams for screencast and screencopy.

**Responsibilities:**
- Screen capture stream management
- PipeWire integration
- Stream mapping and cleanup
- Portal protocol support

### UiOverlays (`subsystems/ui.rs`)
Groups modal UI elements (screenshot, hotkey, exit dialog, MRU).

**Responsibilities:**
- Screenshot UI management
- Hotkey overlay display
- Exit confirmation dialog
- MRU window switcher
- Config error notifications

### InputTracking (`subsystems/input.rs`)
Tracks input gestures, scroll accumulators, and modifier bindings.

**Responsibilities:**
- Gesture tracking (3-finger swipe, overview scroll)
- Scroll accumulators for binding detection
- Modifier combination tracking
- Config-driven binding updates

### ProtocolStates (`protocols.rs`)
Container for all Smithay protocol states.

**Responsibilities:**
- Protocol lifecycle management
- State container for all Wayland protocols
- Protocol initialization and cleanup

## Supporting Modules

### Layout (`layout/`)
Already modular - handles window tiling and spatial arrangement.

### Config (`config.rs`)
Configuration reload logic, broken into focused helper methods.

### Initialization (`init.rs`)
Niri constructor and compositor setup.

## Design Principles

### 1. Ownership
Each subsystem owns its state (private fields) and exposes an intentional public API.

### 2. Encapsulation
Minimal public APIs with clear boundaries between subsystems.

### 3. Testability
Subsystems can be tested in isolation without requiring a full compositor.

### 4. Clear Boundaries
One responsibility per subsystem, with well-defined interfaces.

## Usage Examples

```rust
// Output management
niri.outputs.add(output, refresh_interval, vrr, &display_handle, &config);
if let Some((output, _)) = niri.outputs.under_position(cursor_pos) {
    niri.queue_redraw(&output);
}

// Focus handling
let focus = niri.focus.compute(&context);
match focus {
    KeyboardFocus::Window(window) => { /* handle window focus */ }
    KeyboardFocus::Layer(surface) => { /* handle layer focus */ }
}

// Input tracking
if niri.input.mods_with_mouse_binds().contains(&modifiers) {
    // Handle mouse binding
}
let ticks = niri.input.vertical_wheel_mut().accumulate(delta);
```

## Architecture Benefits

- **Reduced Complexity**: 75% reduction in Niri struct fields
- **Improved Testability**: Each subsystem can be tested independently
- **Better Maintainability**: Clear ownership and encapsulation
- **Enhanced Readability**: Focused, single-responsibility modules
- **Easier Extension**: New features can be added to appropriate subsystems

## Migration History

This architecture was achieved through the 2D Canvas Plan refactoring (Phases P1-P7.5), which extracted domain-specific subsystems from a monolithic god object pattern.
