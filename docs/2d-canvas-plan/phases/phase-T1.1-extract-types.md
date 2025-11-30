# Phase T1.1: Extract Types & Create Module Structure

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~30 minutes  
> **Risk Level**: 🟢 Low  
> **Architectural Benefit**: Foundation for all other phases

---

## Goal

Create `src/backend/tty/` module directory and extract all type definitions. This is the foundation phase — it establishes the module structure for all subsequent phases.

---

## Work Units

### Unit 1: Create Module Structure

**Key insight**: Can't have both `src/backend/tty.rs` and `src/backend/tty/` directory. Must rename file.

```bash
mkdir -p src/backend/tty
mv src/backend/tty.rs src/backend/tty/mod.rs
```

Add module header to `src/backend/tty/mod.rs` (at top):
```rust
//! TTY/DRM backend for native display.
//!
//! This module handles:
//! - DRM device management
//! - Output enumeration and configuration
//! - Frame rendering and presentation

mod types;

pub use types::*;
```

**Verify**: `cargo check` should pass.

---

### Unit 2: Extract Type Aliases (~10 min)

**Source**: `tty.rs` lines ~108-131

```rust
// src/backend/tty/types.rs

use smithay::backend::drm::DrmDeviceFd;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::multigpu::gbm::GbmGlesBackend;
use smithay::backend::renderer::multigpu::{MultiFrame, MultiRenderer};
use smithay::backend::renderer::RendererSuper;
use smithay::backend::allocator::gbm::{GbmAllocator, GbmDevice};
use smithay::backend::drm::composer::{DrmCompositor, PrimaryPlaneElement};
use smithay::desktop::utils::OutputPresentationFeedback;
use std::time::Duration;

pub type TtyRenderer<'render> = MultiRenderer<
    'render,
    'render,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
>;

pub type TtyFrame<'render, 'frame, 'buffer> = MultiFrame<
    'render,
    'render,
    'frame,
    'buffer,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
>;

pub type TtyRendererError<'render> = <TtyRenderer<'render> as RendererSuper>::Error;

pub(super) type GbmDrmCompositor = DrmCompositor<
    GbmAllocator<DrmDeviceFd>,
    GbmFramebufferExporter<DrmDeviceFd>,
    (OutputPresentationFeedback, Duration),
    DrmDeviceFd,
>;
```

---

### Unit 3: Extract Internal Structs (~10 min)

**Source**: `tty.rs` lines ~371-412

```rust
// Add to src/backend/tty/types.rs

use smithay::wayland::dmabuf::DmabufFeedback;
use smithay::reexports::drm::control::{crtc, property};
use std::num::NonZeroU64;

pub(super) struct TtyOutputState {
    // ... fields from original
}

pub(super) struct Surface {
    // ... fields from original
}

pub struct SurfaceDmabufFeedback {
    pub render: DmabufFeedback,
    pub scanout: DmabufFeedback,
}

pub(super) struct GammaProps {
    pub(super) crtc: crtc::Handle,
    pub(super) gamma_lut: property::Handle,
    pub(super) gamma_lut_size: u64,
    pub(super) previous_blob: Option<NonZeroU64>,
}

pub(super) struct ConnectorProperties<'a> {
    // ... fields from original
}
```

---

### Unit 4: Extract Constants (~5 min)

```rust
// Add to src/backend/tty/types.rs

use smithay::backend::allocator::Fourcc;

pub(super) const SUPPORTED_COLOR_FORMATS: [Fourcc; 4] = [
    Fourcc::Xrgb8888,
    Fourcc::Xbgr8888,
    Fourcc::Argb8888,
    Fourcc::Abgr8888,
];
```

---

### Unit 5: Update mod.rs imports

Replace the type definitions in `mod.rs` with imports:

```rust
// At top of mod.rs
mod types;

pub use types::{TtyRenderer, TtyFrame, TtyRendererError, SurfaceDmabufFeedback};
use types::*;
```

Delete the original type definitions from `mod.rs`.

**Verify**: `cargo check` and `cargo test`

---

## Types to Extract

| Type | Visibility | Notes |
|------|------------|-------|
| `TtyRenderer` | `pub` | Used externally |
| `TtyFrame` | `pub` | Used externally |
| `TtyRendererError` | `pub` | Used externally |
| `GbmDrmCompositor` | `pub(super)` | Internal type alias |
| `TtyOutputState` | `pub(super)` | Internal state |
| `Surface` | `pub(super)` | Internal surface state |
| `SurfaceDmabufFeedback` | `pub` | Used by niri |
| `GammaProps` | `pub(super)` | Internal gamma handling |
| `ConnectorProperties` | `pub(super)` | Internal connector state |
| `SUPPORTED_COLOR_FORMATS` | `pub(super)` | Internal constant |

---

## Verification Checklist

- [ ] `src/backend/tty/mod.rs` exists (was `src/backend/tty.rs`, renamed)
- [ ] `src/backend/tty/types.rs` exists with all types
- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] No duplicate type definitions

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty.rs` | Renamed to `src/backend/tty/mod.rs` |
| `src/backend/tty/mod.rs` | Added module header |
| `src/backend/tty/types.rs` | Created (~150 LOC) |

---

## Next Phase

After completing this phase, proceed to [Phase T1.2: Extract OutputDevice](phase-T1.2-extract-device.md).
