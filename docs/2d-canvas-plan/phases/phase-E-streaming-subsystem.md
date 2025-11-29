# Phase E: Extract StreamingSubsystem

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1 hour  
> **Risk Level**: 🟢 Low (already isolated)  
> **Prerequisite**: Phase D complete  
> **Creates**: `StreamingSubsystem` struct

---

## Goal

Extract PipeWire, screencast, and screencopy state into a `StreamingSubsystem` that:
- **Owns** streaming state
- **Isolates** feature-gated code
- **Simplifies** cast lifecycle management

---

## Why This Is Low Risk

The streaming code is:
- Feature-gated (`#[cfg(feature = "xdp-gnome-screencast")]`)
- Relatively isolated from core logic
- Has clear boundaries (PipeWire lifecycle, cast sessions)

---

## Fields to Move from Niri

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

/// Streaming subsystem.
pub struct StreamingSubsystem {
    /// Active screen cast sessions.
    casts: Vec<Cast>,
    
    /// PipeWire connection.
    pipewire: Option<PipeWire>,
    
    /// Channel for PipeWire messages.
    #[cfg(feature = "xdp-gnome-screencast")]
    pw_sender: Option<calloop::channel::Sender<PwToNiri>>,
    
    /// Screencast output per mapped window.
    #[cfg(feature = "xdp-gnome-screencast")]
    mapped_cast_output: HashMap<Window, Output>,
    
    /// Dynamic cast window ID for portal.
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
    
    /// Returns all casts.
    pub fn casts(&self) -> &[Cast] {
        &self.casts
    }
    
    /// Returns mutable casts.
    pub fn casts_mut(&mut self) -> &mut Vec<Cast> {
        &mut self.casts
    }
    
    /// Adds a cast.
    pub fn add_cast(&mut self, cast: Cast) {
        self.casts.push(cast);
    }
    
    /// Removes a cast by stream ID.
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
    
    /// Finds a mutable cast by stream ID.
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
    
    /// Returns mutable PipeWire.
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
    // Feature-Gated Methods
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

### Unit 1: Add to subsystems/mod.rs

```rust
mod streaming;
pub use streaming::StreamingSubsystem;
```

---

### Unit 2: Create StreamingSubsystem struct

---

### Unit 3: Move fields from Niri

---

### Unit 4: Update access patterns

```rust
// Before
self.casts.push(cast);
self.pipewire.as_ref()
self.mapped_cast_output.get(&window)

// After
self.streaming.add_cast(cast);
self.streaming.pipewire()
self.streaming.mapped_cast_output(&window)
```

---

### Unit 5: Update screencast.rs

---

### Unit 6: Verify

```bash
cargo check
cargo test
```

---

## Verification Checklist

- [ ] `StreamingSubsystem` struct created
- [ ] All streaming fields removed from Niri
- [ ] Feature-gated fields properly handled
- [ ] screencast.rs methods updated
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/streaming.rs` | **NEW** ~200 lines |
| `src/niri/mod.rs` | -6 fields |
| `src/niri/screencast.rs` | Updated |

---

## Next Phase

After completing this phase, proceed to [Phase F: UiOverlays](phase-F-ui-overlays.md).
