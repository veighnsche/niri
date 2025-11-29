# Phase B: Extract OutputSubsystem

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~2 hours  
> **Risk Level**: 🟡 Medium (most impactful change)  
> **Prerequisite**: Phase A complete  
> **Creates**: `OutputSubsystem` struct

---

## Goal

Extract all output-related state and logic into an `OutputSubsystem` that:
- **Owns** output-related state (private fields)
- **Exposes** minimal public API
- **Can be tested** in isolation

---

## Fields to Move from Niri

```rust
// Output state (mod.rs lines ~250-285)
pub global_space: Space<Window>,
pub sorted_outputs: Vec<Output>,
pub output_state: HashMap<Output, OutputState>,
pub monitors_active: bool,
pub is_lid_closed: bool,
pub ipc_outputs_changed: bool,
```

Also move `OutputState` struct (~40 lines).

---

## Target Architecture

### New File: `src/niri/subsystems/outputs.rs`

```rust
//! Output management subsystem.
//!
//! Owns all state related to physical outputs (monitors):
//! - Global coordinate space
//! - Sorted output list
//! - Per-output state (frame clock, redraw, etc.)
//! - Monitor power state

use std::collections::HashMap;

use smithay::desktop::Space;
use smithay::output::Output;
use smithay::reexports::wayland_server::backend::GlobalId;
use smithay::utils::{Logical, Point};

use crate::frame_clock::FrameClock;
// ... other imports

/// Per-output state.
pub struct OutputState {
    pub global: GlobalId,
    pub frame_clock: FrameClock,
    pub redraw_state: RedrawState,
    // ... other fields
}

/// Output management subsystem.
pub struct OutputSubsystem {
    /// Global compositor coordinate space.
    global_space: Space<Window>,
    
    /// Outputs sorted by name and position.
    sorted: Vec<Output>,
    
    /// Per-output state.
    state: HashMap<Output, OutputState>,
    
    /// Whether monitors are powered on.
    monitors_active: bool,
    
    /// Whether laptop lid is closed.
    lid_closed: bool,
    
    /// Flag for IPC output change notification.
    ipc_changed: bool,
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
            ipc_changed: false,
        }
    }
    
    // =========================================================================
    // Query Methods
    // =========================================================================
    
    /// Returns the output under the given position.
    pub fn under_position(&self, pos: Point<f64, Logical>) -> Option<(&Output, Point<f64, Logical>)> {
        let output = self.global_space.output_under(pos).next()?;
        let output_geo = self.global_space.output_geometry(output)?;
        let pos_within = pos - output_geo.loc.to_f64();
        Some((output, pos_within))
    }
    
    /// Returns the output to the left of the given output.
    pub fn left_of(&self, current: &Output) -> Option<&Output> {
        // Implementation moved from output.rs
    }
    
    /// Returns the output to the right of the given output.
    pub fn right_of(&self, current: &Output) -> Option<&Output> {
        // Implementation moved from output.rs
    }
    
    /// Returns the output above the given output.
    pub fn above(&self, current: &Output) -> Option<&Output> {
        // Implementation moved from output.rs
    }
    
    /// Returns the output below the given output.
    pub fn below(&self, current: &Output) -> Option<&Output> {
        // Implementation moved from output.rs
    }
    
    /// Returns output matching the given name pattern.
    pub fn by_name(&self, pattern: &OutputName) -> Option<&Output> {
        // Implementation moved from output.rs
    }
    
    /// Returns the previous output in sorted order.
    pub fn previous(&self, current: &Output) -> Option<&Output> {
        // Implementation moved from output.rs
    }
    
    /// Returns the next output in sorted order.
    pub fn next(&self, current: &Output) -> Option<&Output> {
        // Implementation moved from output.rs
    }
    
    /// Returns all outputs.
    pub fn all(&self) -> &[Output] {
        &self.sorted
    }
    
    /// Returns mutable access to the global space.
    pub fn global_space_mut(&mut self) -> &mut Space<Window> {
        &mut self.global_space
    }
    
    /// Returns the global space.
    pub fn global_space(&self) -> &Space<Window> {
        &self.global_space
    }
    
    // =========================================================================
    // State Access
    // =========================================================================
    
    /// Returns state for the given output.
    pub fn state(&self, output: &Output) -> Option<&OutputState> {
        self.state.get(output)
    }
    
    /// Returns mutable state for the given output.
    pub fn state_mut(&mut self, output: &Output) -> Option<&mut OutputState> {
        self.state.get_mut(output)
    }
    
    /// Returns whether monitors are active.
    pub fn monitors_active(&self) -> bool {
        self.monitors_active
    }
    
    /// Sets monitor active state.
    pub fn set_monitors_active(&mut self, active: bool) {
        self.monitors_active = active;
    }
    
    /// Returns whether lid is closed.
    pub fn lid_closed(&self) -> bool {
        self.lid_closed
    }
    
    /// Sets lid closed state.
    pub fn set_lid_closed(&mut self, closed: bool) {
        self.lid_closed = closed;
    }
    
    // =========================================================================
    // Lifecycle Methods
    // =========================================================================
    
    /// Adds an output.
    pub fn add(
        &mut self,
        output: Output,
        refresh_interval: Option<Duration>,
        vrr: bool,
        display_handle: &DisplayHandle,
        config: &Config,
    ) {
        // Implementation moved from mod.rs add_output()
    }
    
    /// Removes an output.
    pub fn remove(&mut self, output: &Output) {
        // Implementation moved from mod.rs remove_output()
    }
    
    /// Repositions all outputs based on configuration.
    pub fn reposition(&mut self, new_output: Option<&Output>, config: &Config) {
        // Implementation moved from mod.rs reposition_outputs()
    }
    
    // =========================================================================
    // Redraw Management
    // =========================================================================
    
    /// Queues a redraw for the given output.
    pub fn queue_redraw(&mut self, output: &Output) {
        if let Some(state) = self.state.get_mut(output) {
            state.redraw_state = RedrawState::Queued;
        }
    }
    
    /// Queues a redraw for all outputs.
    pub fn queue_redraw_all(&mut self) {
        for state in self.state.values_mut() {
            state.redraw_state = RedrawState::Queued;
        }
    }
    
    /// Marks IPC outputs as changed.
    pub fn mark_ipc_changed(&mut self) {
        self.ipc_changed = true;
    }
    
    /// Takes and clears the IPC changed flag.
    pub fn take_ipc_changed(&mut self) -> bool {
        std::mem::take(&mut self.ipc_changed)
    }
}

impl Default for OutputSubsystem {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Work Units

### Unit 1: Create subsystems directory

```bash
mkdir -p src/niri/subsystems
```

Create `src/niri/subsystems/mod.rs`:
```rust
//! Domain subsystems for the Niri compositor.

mod outputs;

pub use outputs::{OutputSubsystem, OutputState};
```

---

### Unit 2: Create OutputSubsystem struct

Create `src/niri/subsystems/outputs.rs` with:
1. Struct definition with private fields
2. Constructor
3. Basic query methods

**Verify**: `cargo check`

---

### Unit 3: Move OutputState struct

Move `OutputState` from `mod.rs` to `subsystems/outputs.rs`.

---

### Unit 4: Move fields from Niri

1. Remove output-related fields from `Niri` struct
2. Add `pub outputs: OutputSubsystem` field
3. Update `Niri::new()` to create `OutputSubsystem`

**Verify**: `cargo check` (will fail — access patterns need updating)

---

### Unit 5: Update access patterns

```rust
// Before
self.global_space.output_under(pos)
self.sorted_outputs.iter()
self.output_state.get(&output)
self.monitors_active
self.ipc_outputs_changed = true;

// After
self.outputs.global_space().output_under(pos)
self.outputs.all().iter()
self.outputs.state(&output)
self.outputs.monitors_active()
self.outputs.mark_ipc_changed();
```

---

### Unit 6: Move methods from output.rs

Move existing `impl Niri` methods from `output.rs` to `OutputSubsystem`:
- `output_under()` → `outputs.under_position()`
- `output_left_of()` → `outputs.left_of()`
- `output_right_of()` → `outputs.right_of()`
- etc.

---

### Unit 7: Move lifecycle methods from mod.rs

Move from `mod.rs`:
- `add_output()` → `outputs.add()`
- `remove_output()` → `outputs.remove()`
- `reposition_outputs()` → `outputs.reposition()`
- `queue_redraw()` → `outputs.queue_redraw()`
- `queue_redraw_all()` → `outputs.queue_redraw_all()`

---

### Unit 8: Verify

```bash
cargo check
cargo test
```

---

## Verification Checklist

- [ ] `subsystems/` directory exists
- [ ] `OutputSubsystem` struct with private fields
- [ ] `OutputState` moved to subsystems
- [ ] All output fields removed from Niri
- [ ] `Niri.outputs: OutputSubsystem` field added
- [ ] Query methods implemented
- [ ] Lifecycle methods implemented
- [ ] All access patterns updated
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/mod.rs` | **NEW** |
| `src/niri/subsystems/outputs.rs` | **NEW** ~400 lines |
| `src/niri/mod.rs` | -8 fields, -OutputState |
| `src/niri/output.rs` | Methods → OutputSubsystem |
| Various files | Updated access patterns |

---

## Benefits

1. **-8 fields** from Niri struct
2. **Encapsulated** output state
3. **Testable** output logic
4. **Clear ownership** of output-related code

---

## Next Phase

After completing this phase, proceed to [Phase C: CursorSubsystem](phase-C-cursor-subsystem.md).
