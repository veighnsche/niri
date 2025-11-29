# Phase P5: Extract StreamingSubsystem

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1 hour  
> **Risk Level**: 🟢 Low (already somewhat isolated)  
> **Prerequisite**: Phase P4 complete  
> **Creates**: `StreamingSubsystem` struct

---

## Goal

Extract PipeWire, screencast, and screencopy state into a dedicated `StreamingSubsystem` that:
- **Owns** all streaming-related state (casts, PipeWire, mapped outputs)
- **Encapsulates** streaming lifecycle management
- **Isolates** feature-gated code (`xdp-gnome-screencast`)

---

## Why This Is Low Risk

The streaming code is already:
- Feature-gated (`#[cfg(feature = "xdp-gnome-screencast")]`)
- Relatively isolated from core compositor logic
- Has clear boundaries (PipeWire messages, cast lifecycle)

---

## Current State Analysis

### Fields to Move from Niri

```rust
// Streaming state (mod.rs lines ~432-444)
pub casts: Vec<Cast>,
pub pipewire: Option<PipeWire>,
#[cfg(feature = "xdp-gnome-screencast")]
pub pw_to_niri: calloop::channel::Sender<PwToNiri>,
#[cfg(feature = "xdp-gnome-screencast")]
pub mapped_cast_output: HashMap<Window, Output>,
#[cfg(feature = "xdp-gnome-screencast")]
pub dynamic_cast_id_for_portal: MappedId,
```

### Methods Currently Scattered

```rust
// In mod.rs/screencast.rs
pub fn on_pw_msg(&mut self, msg: PwToNiri);
fn redraw_cast(&mut self, stream_id: usize);
pub fn set_dynamic_cast_target(&mut self, target: CastTarget);
pub fn on_screen_cast_msg(&mut self, msg: ScreenCastToNiri);
pub fn refresh_mapped_cast_window_rules(&mut self);
pub fn refresh_mapped_cast_outputs(&mut self);
```

---

## Target Architecture

### New File: `src/niri/subsystems/streaming.rs`

```rust
//! Streaming subsystem (screencast, screencopy, PipeWire).
//!
//! Handles screen capture streams for portals and protocols.

use std::collections::HashMap;

use smithay::desktop::Window;
use smithay::output::Output;

use crate::pw_utils::{Cast, PipeWire};
#[cfg(feature = "xdp-gnome-screencast")]
use crate::pw_utils::PwToNiri;
use crate::window::mapped::MappedId;

/// Streaming subsystem for screencast and screencopy.
///
/// Manages PipeWire streams, cast sessions, and the mapping between
/// windows and their screencast outputs.
pub struct StreamingSubsystem {
    /// Active screen cast sessions.
    casts: Vec<Cast>,
    
    /// PipeWire connection (if initialized).
    pipewire: Option<PipeWire>,
    
    /// Channel to send messages from PipeWire thread.
    #[cfg(feature = "xdp-gnome-screencast")]
    pw_sender: Option<calloop::channel::Sender<PwToNiri>>,
    
    /// Screencast output for each mapped window.
    #[cfg(feature = "xdp-gnome-screencast")]
    mapped_cast_output: HashMap<Window, Output>,
    
    /// Window ID for the "dynamic cast" special window for the xdp-gnome picker.
    #[cfg(feature = "xdp-gnome-screencast")]
    dynamic_cast_id: Option<MappedId>,
}

impl StreamingSubsystem {
    /// Creates a new streaming subsystem.
    pub fn new() -> Self {
        Self {
            casts: Vec::new(),
            pipewire: None,
            #[cfg(feature = "xdp-gnome-screencast")]
            pw_sender: None,
            #[cfg(feature = "xdp-gnome-screencast")]
            mapped_cast_output: HashMap::new(),
            #[cfg(feature = "xdp-gnome-screencast")]
            dynamic_cast_id: None,
        }
    }
    
    // =========================================================================
    // Cast Management
    // =========================================================================
    
    /// Returns the active casts.
    pub fn casts(&self) -> &[Cast] {
        &self.casts
    }
    
    /// Returns mutable access to active casts.
    pub fn casts_mut(&mut self) -> &mut Vec<Cast> {
        &mut self.casts
    }
    
    /// Adds a new cast session.
    pub fn add_cast(&mut self, cast: Cast) {
        self.casts.push(cast);
    }
    
    /// Removes a cast session by stream ID.
    pub fn remove_cast(&mut self, stream_id: usize) -> Option<Cast> {
        self.casts
            .iter()
            .position(|c| c.stream_id() == stream_id)
            .map(|idx| self.casts.remove(idx))
    }
    
    /// Finds a cast by stream ID.
    pub fn find_cast(&self, stream_id: usize) -> Option<&Cast> {
        self.casts.iter().find(|c| c.stream_id() == stream_id)
    }
    
    /// Finds a cast by stream ID (mutable).
    pub fn find_cast_mut(&mut self, stream_id: usize) -> Option<&mut Cast> {
        self.casts.iter_mut().find(|c| c.stream_id() == stream_id)
    }
    
    // =========================================================================
    // PipeWire
    // =========================================================================
    
    /// Returns the PipeWire connection.
    pub fn pipewire(&self) -> Option<&PipeWire> {
        self.pipewire.as_ref()
    }
    
    /// Returns mutable access to PipeWire.
    pub fn pipewire_mut(&mut self) -> Option<&mut PipeWire> {
        self.pipewire.as_mut()
    }
    
    /// Sets the PipeWire connection.
    pub fn set_pipewire(&mut self, pw: Option<PipeWire>) {
        self.pipewire = pw;
    }
    
    /// Returns whether PipeWire is initialized.
    pub fn has_pipewire(&self) -> bool {
        self.pipewire.is_some()
    }
    
    // =========================================================================
    // Mapped Cast Outputs (feature-gated)
    // =========================================================================
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn pw_sender(&self) -> Option<&calloop::channel::Sender<PwToNiri>> {
        self.pw_sender.as_ref()
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn set_pw_sender(&mut self, sender: calloop::channel::Sender<PwToNiri>) {
        self.pw_sender = Some(sender);
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn mapped_cast_output(&self, window: &Window) -> Option<&Output> {
        self.mapped_cast_output.get(window)
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn set_mapped_cast_output(&mut self, window: Window, output: Output) {
        self.mapped_cast_output.insert(window, output);
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn remove_mapped_cast_output(&mut self, window: &Window) {
        self.mapped_cast_output.remove(window);
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn mapped_cast_outputs(&self) -> &HashMap<Window, Output> {
        &self.mapped_cast_output
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn mapped_cast_outputs_mut(&mut self) -> &mut HashMap<Window, Output> {
        &mut self.mapped_cast_output
    }
    
    // =========================================================================
    // Dynamic Cast
    // =========================================================================
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn dynamic_cast_id(&self) -> Option<MappedId> {
        self.dynamic_cast_id
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn set_dynamic_cast_id(&mut self, id: Option<MappedId>) {
        self.dynamic_cast_id = id;
    }
}

impl Default for StreamingSubsystem {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Work Units

### Unit 1: Add StreamingSubsystem to subsystems/mod.rs

```rust
mod cursor;
mod focus;
mod outputs;
mod streaming;

pub use cursor::CursorSubsystem;
pub use focus::{FocusModel, FocusContext};
pub use outputs::OutputSubsystem;
pub use streaming::StreamingSubsystem;
```

---

### Unit 2: Create StreamingSubsystem Struct

Create `src/niri/subsystems/streaming.rs` with:
1. Struct definition with private fields
2. Feature-gated fields for `xdp-gnome-screencast`
3. Basic accessors

**Verify**: `cargo check`

---

### Unit 3: Move Fields from Niri

1. Remove streaming-related fields from `Niri` struct
2. Add `pub streaming: StreamingSubsystem` field
3. Update `Niri::new` to create `StreamingSubsystem`

**Verify**: `cargo check` (will fail - access patterns need updating)

---

### Unit 4: Update Access Patterns

```rust
// Before
self.casts.push(cast);
self.pipewire.as_ref();
self.mapped_cast_output.get(&window);

// After
self.streaming.add_cast(cast);
self.streaming.pipewire();
self.streaming.mapped_cast_output(&window);
```

---

### Unit 5: Refactor screencast.rs Methods

The screencast methods should delegate to the subsystem:

```rust
// Before (in screencast.rs)
impl Niri {
    pub fn refresh_mapped_cast_outputs(&mut self) {
        for (window, output) in &mut self.mapped_cast_output {
            // ...
        }
    }
}

// After
impl Niri {
    pub fn refresh_mapped_cast_outputs(&mut self) {
        for (window, output) in self.streaming.mapped_cast_outputs_mut() {
            // ...
        }
    }
}
```

---

## Verification Checklist

- [ ] `StreamingSubsystem` struct exists with private fields
- [ ] Feature-gated fields properly handled
- [ ] Streaming fields removed from `Niri`
- [ ] `Niri.streaming: StreamingSubsystem` field added
- [ ] All access patterns updated
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)
- [ ] Feature-gated build still works

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/streaming.rs` | +200 lines (new) |
| `src/niri/subsystems/mod.rs` | +3 lines |
| `src/niri/mod.rs` | -10 lines (fields), +2 lines (field) |
| `src/niri/screencast.rs` | Updated access patterns |

---

## Benefits Achieved

1. **Isolation**: Streaming code in one place
2. **Feature clarity**: Feature-gated code clearly marked
3. **Reduced Niri complexity**: 5 fewer fields
4. **Testability**: Cast management logic can be tested

---

## Next Phase

After completing this phase, proceed to [Phase P6: UiOverlays](phase-P6-ui-overlays.md).
