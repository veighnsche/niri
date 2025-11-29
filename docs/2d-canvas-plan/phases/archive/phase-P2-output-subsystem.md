# Phase P2: Extract OutputSubsystem

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~2 hours  
> **Risk Level**: 🟡 Medium (most impactful change)  
> **Prerequisite**: Phase P1 complete  
> **Creates**: `OutputSubsystem` struct with encapsulated state

---

## Goal

Extract all output-related state and logic from `Niri` into a dedicated `OutputSubsystem` that:
- **Owns** all output-related state (not just accesses it)
- **Encapsulates** implementation details
- **Exposes** a clean, minimal public API
- **Can be tested** in isolation (future goal)

---

## Why a Subsystem, Not Just Moving Methods?

### Current Anti-Pattern
```rust
// Scattered across Niri
pub struct Niri {
    pub global_space: Space<Window>,
    pub sorted_outputs: Vec<Output>,
    pub output_state: HashMap<Output, OutputState>,
    pub monitors_active: bool,
    pub is_lid_closed: bool,
}

// In output.rs - just impl Niri
impl Niri {
    pub fn output_under(&self, pos: Point) { ... }  // accesses self.*
}
```

### Target Pattern
```rust
// OutputSubsystem owns its state
pub struct OutputSubsystem {
    global_space: Space<Window>,      // private
    sorted: Vec<Output>,              // private
    state: HashMap<Output, OutputState>, // private
    monitors_active: bool,            // private
    lid_closed: bool,                 // private
}

impl OutputSubsystem {
    // Clean public API
    pub fn add(&mut self, output: Output, config: &OutputConfig) -> Result<(), OutputError> { ... }
    pub fn remove(&mut self, output: &Output) { ... }
    pub fn under_position(&self, pos: Point<f64, Logical>) -> Option<&Output> { ... }
    pub fn reposition(&mut self, config: &Config) { ... }
    
    // Controlled access to internals when needed
    pub fn space(&self) -> &Space<Window> { &self.global_space }
    pub fn state_mut(&mut self, output: &Output) -> Option<&mut OutputState> { ... }
}
```

---

## Current State Analysis

### Fields to Move from Niri

```rust
// src/niri/mod.rs lines ~249-286
pub global_space: Space<Window>,           // ~1 line
pub sorted_outputs: Vec<Output>,           // ~1 line
pub output_state: HashMap<Output, OutputState>,  // ~1 line
pub monitors_active: bool,                 // ~1 line
pub is_lid_closed: bool,                   // ~1 line
```

### Types to Move to types.rs (if not already)

```rust
pub struct OutputState { ... }  // ~52 lines - may already be in types.rs
```

### Methods Currently on Niri (in output.rs)

```rust
// Query methods (~150 lines)
pub fn output_under(&self, pos) -> Option<(&Output, Point)>
pub fn output_under_cursor(&self) -> Option<Output>
pub fn output_left_of(&self, current: &Output) -> Option<Output>
pub fn output_right_of(&self, current: &Output) -> Option<Output>
pub fn output_up_of(&self, current: &Output) -> Option<Output>
pub fn output_down_of(&self, current: &Output) -> Option<Output>
// ... more query methods

// Management methods (in mod.rs, ~300 lines)
pub fn reposition_outputs(&mut self, new_output: Option<&Output>)
pub fn add_output(&mut self, output: Output, refresh_interval: Option<Duration>, vrr: bool)
pub fn remove_output(&mut self, output: &Output)
```

---

## Target Architecture

### New File: `src/niri/subsystems/outputs.rs`

```rust
//! Output management subsystem.
//!
//! Owns all state related to physical outputs (monitors) and provides
//! a clean API for output lifecycle management and spatial queries.

use std::collections::HashMap;
use std::time::Duration;

use smithay::desktop::Space;
use smithay::output::Output;
use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::frame_clock::FrameClock;
use crate::render_helpers::solid_color::SolidColorBuffer;

use super::super::types::{OutputState, RedrawState};

/// Output management subsystem.
///
/// This struct owns all output-related state and encapsulates the logic
/// for adding, removing, repositioning, and querying outputs.
pub struct OutputSubsystem {
    /// Global compositor space containing all outputs.
    global_space: Space<smithay::desktop::Window>,
    
    /// Outputs sorted by name and position.
    sorted: Vec<Output>,
    
    /// Per-output state (frame clock, redraw state, etc.).
    state: HashMap<Output, OutputState>,
    
    /// Whether monitors are currently active (not powered off for idle).
    monitors_active: bool,
    
    /// Whether the laptop lid is closed.
    lid_closed: bool,
}

impl OutputSubsystem {
    /// Creates a new output subsystem.
    pub fn new() -> Self {
        Self {
            global_space: Space::default(),
            sorted: Vec::new(),
            state: HashMap::new(),
            monitors_active: true,
            lid_closed: false,
        }
    }
    
    // =========================================================================
    // Lifecycle Management
    // =========================================================================
    
    /// Adds a new output to the compositor.
    pub fn add(
        &mut self,
        output: Output,
        refresh_interval: Option<Duration>,
        vrr: bool,
        display_handle: &DisplayHandle,
        config: &Config,
    ) -> GlobalId {
        // Implementation moved from Niri::add_output
    }
    
    /// Removes an output from the compositor.
    pub fn remove(&mut self, output: &Output) {
        // Implementation moved from Niri::remove_output
    }
    
    /// Repositions all outputs based on configuration.
    pub fn reposition(&mut self, new_output: Option<&Output>, config: &Config) {
        // Implementation moved from Niri::reposition_outputs
    }
    
    // =========================================================================
    // Spatial Queries
    // =========================================================================
    
    /// Returns the output under the given global position.
    pub fn under_position(&self, pos: Point<f64, Logical>) -> Option<(&Output, Point<f64, Logical>)> {
        let output = self.global_space.output_under(pos).next()?;
        let geo = self.global_space.output_geometry(output)?;
        Some((output, pos - geo.loc.to_f64()))
    }
    
    /// Returns the output to the left of the given output.
    pub fn left_of(&self, current: &Output) -> Option<&Output> {
        // Implementation moved from Niri::output_left_of
    }
    
    /// Returns the output to the right of the given output.
    pub fn right_of(&self, current: &Output) -> Option<&Output> {
        // Implementation moved from Niri::output_right_of
    }
    
    /// Returns the output above the given output.
    pub fn above(&self, current: &Output) -> Option<&Output> {
        // Implementation moved from Niri::output_up_of
    }
    
    /// Returns the output below the given output.
    pub fn below(&self, current: &Output) -> Option<&Output> {
        // Implementation moved from Niri::output_down_of
    }
    
    // =========================================================================
    // State Access
    // =========================================================================
    
    /// Returns an iterator over all outputs.
    pub fn iter(&self) -> impl Iterator<Item = &Output> {
        self.sorted.iter()
    }
    
    /// Returns the global space (read-only).
    pub fn space(&self) -> &Space<smithay::desktop::Window> {
        &self.global_space
    }
    
    /// Returns a mutable reference to the global space.
    pub fn space_mut(&mut self) -> &mut Space<smithay::desktop::Window> {
        &mut self.global_space
    }
    
    /// Returns the state for a specific output.
    pub fn state(&self, output: &Output) -> Option<&OutputState> {
        self.state.get(output)
    }
    
    /// Returns mutable state for a specific output.
    pub fn state_mut(&mut self, output: &Output) -> Option<&mut OutputState> {
        self.state.get_mut(output)
    }
    
    /// Returns whether monitors are active.
    pub fn monitors_active(&self) -> bool {
        self.monitors_active
    }
    
    /// Sets whether monitors are active.
    pub fn set_monitors_active(&mut self, active: bool) {
        self.monitors_active = active;
    }
    
    /// Returns whether the lid is closed.
    pub fn lid_closed(&self) -> bool {
        self.lid_closed
    }
    
    /// Sets the lid closed state.
    pub fn set_lid_closed(&mut self, closed: bool) {
        self.lid_closed = closed;
    }
    
    // =========================================================================
    // Redraw Management
    // =========================================================================
    
    /// Queues a redraw for a specific output.
    pub fn queue_redraw(&mut self, output: &Output) {
        if let Some(state) = self.state.get_mut(output) {
            state.redraw_state = std::mem::take(&mut state.redraw_state).queue_redraw();
        }
    }
    
    /// Queues a redraw for all outputs.
    pub fn queue_redraw_all(&mut self) {
        for state in self.state.values_mut() {
            state.redraw_state = std::mem::take(&mut state.redraw_state).queue_redraw();
        }
    }
}

impl Default for OutputSubsystem {
    fn default() -> Self {
        Self::new()
    }
}
```

### Updated Niri Struct

```rust
pub struct Niri {
    // ... other fields ...
    
    /// Output management subsystem.
    pub outputs: OutputSubsystem,
    
    // REMOVED:
    // pub global_space: Space<Window>,
    // pub sorted_outputs: Vec<Output>,
    // pub output_state: HashMap<Output, OutputState>,
    // pub monitors_active: bool,
    // pub is_lid_closed: bool,
}
```

---

## Work Units

### Unit 1: Create Subsystems Directory

```bash
mkdir -p src/niri/subsystems
```

Create `src/niri/subsystems/mod.rs`:
```rust
//! Compositor subsystems.
//!
//! Each subsystem owns a domain of state and exposes a clean API.

mod outputs;

pub use outputs::OutputSubsystem;
```

---

### Unit 2: Create OutputSubsystem Struct

Create `src/niri/subsystems/outputs.rs` with:
1. Struct definition with private fields
2. `new()` and `Default` implementations
3. Stub methods that return unimplemented!()

**Verify**: `cargo check`

---

### Unit 3: Move Fields from Niri

1. Remove output-related fields from `Niri` struct in `mod.rs`
2. Add `pub outputs: OutputSubsystem` field
3. Update `Niri::new` in `init.rs` to create `OutputSubsystem`

**Verify**: `cargo check` (will fail - access patterns need updating)

---

### Unit 4: Update Access Patterns (Batch 1: Simple Reads)

Update all simple field accesses:
```rust
// Before
self.global_space.output_under(pos)
self.sorted_outputs.iter()
self.monitors_active

// After
self.outputs.space().output_under(pos)
self.outputs.iter()
self.outputs.monitors_active()
```

**Verify**: `cargo check` after each file

---

### Unit 5: Update Access Patterns (Batch 2: State Access)

Update output state accesses:
```rust
// Before
self.output_state.get(&output)
self.output_state.get_mut(&output)

// After
self.outputs.state(&output)
self.outputs.state_mut(&output)
```

---

### Unit 6: Move Method Implementations

Move the actual method implementations from `output.rs` and `mod.rs` into `OutputSubsystem`:
1. `add_output` → `OutputSubsystem::add`
2. `remove_output` → `OutputSubsystem::remove`
3. `reposition_outputs` → `OutputSubsystem::reposition`
4. Spatial query methods

**Verify**: `cargo check` && `cargo test`

---

### Unit 7: Update Call Sites

Replace `impl Niri` method calls with subsystem calls:
```rust
// Before
self.niri.add_output(output, interval, vrr);
self.niri.output_under(pos);

// After  
self.niri.outputs.add(output, interval, vrr, &self.niri.display_handle, &config);
self.niri.outputs.under_position(pos);
```

---

## Verification Checklist

- [ ] `src/niri/subsystems/` directory exists
- [ ] `OutputSubsystem` struct with private fields
- [ ] All output fields removed from `Niri`
- [ ] `Niri.outputs: OutputSubsystem` field added
- [ ] All access patterns updated
- [ ] All method implementations moved
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/mod.rs` | +10 lines (new) |
| `src/niri/subsystems/outputs.rs` | +400 lines (new) |
| `src/niri/mod.rs` | -15 lines (fields), +5 lines (mod + field) |
| `src/niri/output.rs` | Refactored or removed |
| `src/niri/init.rs` | Updated initialization |
| Various files | Updated access patterns |

---

## Benefits Achieved

1. **Single owner**: All output state in one place
2. **Encapsulation**: Private fields with intentional API
3. **Testability**: `OutputSubsystem` can be tested without full Niri
4. **Discoverability**: `niri.outputs.` gives all output operations
5. **Reduced Niri complexity**: 5 fewer fields

---

## Next Phase

After completing this phase, proceed to [Phase P3: CursorSubsystem](phase-P3-cursor-subsystem.md).
