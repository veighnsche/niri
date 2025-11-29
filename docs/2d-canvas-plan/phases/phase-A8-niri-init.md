# Phase A8: Extract niri/init.rs

> **Status**: ⏳ PENDING
> **Estimated Time**: 1 hour
> **Risk Level**: 🟡 Medium (large function, many dependencies)
> **Prerequisite**: Phase A7 complete

---

## Goal

Extract `Niri::new()` constructor into `src/niri/init.rs`.

This is the final phase of Part A. After this, `niri.rs` should be ~600 LOC.

User: Remember that when a item is too big of a refactor than planned. that I want you to make it smaller and do it in multiple steps. Write it down in this folder as broken down steps in the phase file...

---

## Work Units

### Unit 1: Create init.rs with Imports (10 min)

Create `src/niri/init.rs`:
```rust
//! Niri compositor initialization.
//!
//! Contains the `Niri::new()` constructor and related initialization helpers.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use calloop::LoopHandle;
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::input::{Seat, SeatState};
use smithay::output::Output;
use smithay::reexports::wayland_server::DisplayHandle;
use smithay::wayland::compositor::CompositorState;
// ... (add all necessary imports)

use crate::animation::Clock;
use crate::layout::Layout;
use crate::niri::Niri;
use crate::State;
```

Update `src/niri/mod.rs`:
```rust
mod frame_callbacks;
mod hit_test;
mod init;
mod lock;
mod output;
mod pointer;
mod render;
mod rules;
mod screencast;
mod screencopy;
mod screenshot;
mod types;

pub use types::*;
```

---

### Unit 2: Identify Helper Functions (10 min)

Before extracting `Niri::new()`, identify any helper functions it uses:

```rust
// These may need to move to init.rs or stay in niri.rs:
fn configure_lock_surface(surface: &LockSurface, output: &Output) { ... }
fn make_screenshot_path(config: &Config) -> anyhow::Result<Option<PathBuf>> { ... }
// etc.
```

List all helpers and decide where they belong.

---

### Unit 3: Extract Niri::new() Part 1 — Setup (15 min)

**Source**: `niri.rs` ~lines 700-900 (first third of new())

Extract the initial setup portion:
- Display handle setup
- Compositor state creation
- Protocol state initialization

```rust
impl Niri {
    pub fn new(
        config: Rc<RefCell<Config>>,
        event_loop: LoopHandle<'static, State>,
        stop_signal: calloop::ping::Ping,
        display: DisplayHandle,
        backend: &Backend,
    ) -> Self {
        // Part 1: Basic setup
        let _span = tracy_client::span!("Niri::new");

        let clock = Clock::new();
        
        // Compositor state
        let compositor_state = CompositorState::new::<State>(&display);
        
        // ... (first portion of initialization)
    }
}
```

---

### Unit 4: Extract Niri::new() Part 2 — Protocols (15 min)

**Source**: `niri.rs` ~lines 900-1100 (middle third)

Continue with protocol initialization:
- XDG shell
- Layer shell
- Session lock
- Screencopy
- etc.

---

### Unit 5: Extract Niri::new() Part 3 — Finalization (15 min)

**Source**: `niri.rs` ~lines 1100-1300 (final third)

Complete with:
- Input seat setup
- IPC server
- Event loop sources
- Final struct construction

---

### Unit 6: Extract Helper Functions (10 min)

Move any helper functions that are only used by `new()`:

```rust
// In init.rs
fn setup_wayland_globals(display: &DisplayHandle) -> ... {
    // ...
}

fn create_input_seat(display: &DisplayHandle, name: &str) -> Seat<State> {
    // ...
}
```

---

### Unit 7: Remove from niri.rs (10 min)

Delete `Niri::new()` and related helpers from `niri.rs`.

**Verify**: `cargo check` and `cargo test`

---

### Unit 8: Final Cleanup (15 min)

1. **Verify niri.rs size**: Should be ~600 LOC
2. **Check all modules**: Each should be <500 LOC
3. **Remove unused imports**: Clean up any warnings
4. **Run full test suite**: `cargo test`

---

## Verification Checklist

- [ ] `src/niri/init.rs` exists (~500 LOC)
- [ ] `src/niri.rs` is now ~600 LOC
- [ ] All modules are <500 LOC
- [ ] Niri initializes correctly
- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] No unused imports

---

## Final Module Structure

After Phase A8, the structure should be:

```
src/
├── niri.rs (~600 LOC)          # Core Niri + State structs
└── niri/
    ├── mod.rs (~50 LOC)        # Re-exports
    ├── types.rs (~200 LOC)     # Data types
    ├── output.rs (~500 LOC)    # Output management
    ├── hit_test.rs (~450 LOC)  # Hit testing
    ├── lock.rs (~350 LOC)      # Session lock
    ├── render.rs (~400 LOC)    # Rendering
    ├── frame_callbacks.rs (~350 LOC)  # Frame callbacks
    ├── screenshot.rs (~400 LOC)       # Screenshots
    ├── screencopy.rs (~300 LOC)       # Screencopy
    ├── screencast.rs (~300 LOC)       # Screencasting
    ├── pointer.rs (~350 LOC)   # Pointer management
    ├── rules.rs (~150 LOC)     # Window/layer rules
    └── init.rs (~500 LOC)      # Constructor
```

**Total**: ~4950 LOC across 13 files (avg ~380 LOC each)

---

## Part A Complete! 🎉

After this phase, Part A (Modularization) is complete.

### Success Criteria Met:
- [x] niri.rs < 700 LOC
- [x] Each module < 500 LOC
- [x] No `pub(super)`
- [x] All tests pass
- [x] No unused imports

### Next Steps

Part B (Features) is now unblocked:
- [Phase B1](phase-B1-camera-zoom.md) — Camera zoom system
- [Phase B2](phase-B2-camera-bookmarks.md) — Camera bookmarks
- [Phase B3](phase-B3-ipc-migration.md) — IPC/Protocol migration
- [Phase B4](phase-B4-row-spanning.md) — Row spanning

---

## Handoff Notes

For future teams:
1. The niri module is now well-organized
2. Each file has a clear responsibility
3. Follow the same pattern for future extractions
4. Keep files under 500 LOC
