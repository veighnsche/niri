# Phase T1.1: Extract Types & Create Module Structure

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~30 minutes  
> **Risk Level**: 🟢 Low  
> **Architectural Benefit**: Foundation for subsystem pattern

---

## Goal

Create `src/backend/tty/` module directory and extract all type definitions. This establishes the module structure for the **subsystem ownership pattern**.

---

## Work Units

### Unit 1: Create Module Structure

**Key insight**: Can't have both `src/backend/tty.rs` and `src/backend/tty/` directory.

```bash
mkdir -p src/backend/tty
mv src/backend/tty.rs src/backend/tty/mod.rs
```

Add module header to `src/backend/tty/mod.rs` (at top):
```rust
//! TTY/DRM backend for native display.
//!
//! This module uses the **subsystem ownership pattern**:
//! - `DeviceManager` - owns all DRM device state
//! - `RenderManager` - owns rendering state
//! - `OutputManager` - owns IPC/config state
//!
//! `Tty` is a thin coordinator that dispatches events to subsystems.

mod types;
mod helpers;
mod devices;
mod render;
mod outputs;
mod gamma;

pub use types::{TtyRenderer, TtyFrame, TtyRendererError, SurfaceDmabufFeedback};
pub use devices::DeviceManager;
pub use render::RenderManager;
pub use outputs::OutputManager;
```

---

### Unit 2: Create `types.rs`

Extract all type definitions:
- `TtyRenderer`, `TtyFrame`, `TtyRendererError` (public)
- `GbmDrmCompositor`, `TtyOutputState`, `Surface` (internal)
- `SurfaceDmabufFeedback`, `ConnectorProperties`
- `SUPPORTED_COLOR_FORMATS` constant

---

### Unit 3: Create Stub Modules

Create empty stubs for subsystems (to be filled in later phases):

```rust
// devices.rs - stub
pub struct DeviceManager;

// render.rs - stub  
pub struct RenderManager;

// outputs.rs - stub
pub struct OutputManager;

// src/backend/tty/helpers.rs
//! Pure helper functions for TTY backend.

// src/backend/tty/gamma.rs
//! Gamma LUT management.
pub struct GammaProps {
    pub(super) crtc: smithay::reexports::drm::control::crtc::Handle,
    pub(super) gamma_lut: smithay::reexports::drm::control::property::Handle,
    pub(super) gamma_lut_size: u64,
    pub(super) previous_blob: Option<std::num::NonZeroU64>,
}
```

---

### Unit 4: Update mod.rs Structure

Update `mod.rs` to be a thin coordinator structure:

```rust
// src/backend/tty/mod.rs

//! TTY/DRM backend for native display.
//!
//! Uses subsystem ownership pattern.

mod types;
mod helpers;
mod devices;
mod render;
mod outputs;
mod gamma;

use std::cell::RefCell;
use std::rc::Rc;

use smithay::backend::libinput::LibinputInputBackend;
use smithay::backend::session::libseat::LibSeatSession;
use smithay::backend::udev::UdevBackend;
use smithay::reexports::calloop::Dispatcher;
use smithay::reexports::input::Libinput;

use niri_config::Config;

pub use types::{TtyRenderer, TtyFrame, TtyRendererError, SurfaceDmabufFeedback};
pub use devices::DeviceManager;
pub use render::RenderManager;
pub use outputs::OutputManager;

/// TTY/DRM backend - thin coordinator over subsystems.
///
/// This struct coordinates:
/// - Session management (VT switching)
/// - Udev events (device add/remove)
/// - Libinput (keyboard/mouse)
///
/// Actual state is owned by subsystems:
/// - `DeviceManager` - DRM devices and GPU management
/// - `RenderManager` - Frame rendering and vblank
/// - `OutputManager` - IPC, gamma, VRR, config
pub struct Tty {
    config: Rc<RefCell<Config>>,
    session: LibSeatSession,
    udev_dispatcher: Dispatcher<'static, UdevBackend, crate::niri::State>,
    libinput: Libinput,

    // Subsystems (OWN their state)
    pub(crate) devices: DeviceManager,
    pub(crate) render: RenderManager,
    pub(crate) outputs: OutputManager,
}

// Rest of Tty impl stays in mod.rs for now...
```

---

## Types to Extract

| Type | Visibility | Destination |
|------|------------|-------------|
| `TtyRenderer` | `pub` | `types.rs` |
| `TtyFrame` | `pub` | `types.rs` |
| `TtyRendererError` | `pub` | `types.rs` |
| `GbmDrmCompositor` | `pub(super)` | `types.rs` |
| `TtyOutputState` | `pub(super)` | `types.rs` |
| `Surface` | `pub(super)` | `types.rs` |
| `SurfaceDmabufFeedback` | `pub` | `types.rs` |
| `GammaProps` | `pub(super)` | `gamma.rs` |
| `ConnectorProperties` | `pub(super)` | `types.rs` |
| `SUPPORTED_COLOR_FORMATS` | `pub(super)` | `types.rs` |

---

## Verification Checklist

- [ ] `src/backend/tty/` directory exists
- [ ] `src/backend/tty/mod.rs` has subsystem structure
- [ ] `src/backend/tty/types.rs` has all types
- [ ] Stub modules exist for `devices`, `render`, `outputs`, `helpers`, `gamma`
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Files Created/Changed

| File | Change |
|------|--------|
| `src/backend/tty.rs` | Renamed to `src/backend/tty/mod.rs` |
| `src/backend/tty/mod.rs` | Subsystem coordinator structure |
| `src/backend/tty/types.rs` | Type definitions (~150 LOC) |
| `src/backend/tty/devices.rs` | Stub (~5 LOC) |
| `src/backend/tty/render.rs` | Stub (~5 LOC) |
| `src/backend/tty/outputs.rs` | Stub (~5 LOC) |
| `src/backend/tty/helpers.rs` | Stub (~5 LOC) |
| `src/backend/tty/gamma.rs` | GammaProps struct (~20 LOC) |

---

## Next Phase

After completing this phase, proceed to [Phase T1.2: Enrich OutputDevice](phase-T1.2-extract-device.md).
