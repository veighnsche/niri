# Phase T1.6: Create RenderManager Subsystem

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1 hour  
> **Risk Level**: 🟡 Medium  
> **Architectural Benefit**: ⭐⭐⭐ High - isolates render pipeline

---

## Goal

Create the `RenderManager` subsystem that handles frame rendering, vblank events, and presentation feedback.

---

## What RenderManager Owns

```rust
pub struct RenderManager {
    debug_tint: bool,
    // Note: Per-surface frame state is in OutputDevice.surfaces
}
```

The render manager is lightweight because per-surface state (frame clock, damage) lives in `Surface` which is owned by `OutputDevice`.

---

## RenderManager API

```rust
impl RenderManager {
    pub fn new() -> Self

    /// Render a frame for the given output.
    pub fn render(
        &mut self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        output: &Output,
        target_presentation_time: Duration,
    ) -> RenderResult

    /// Handle a VBlank event from DRM.
    pub fn on_vblank(
        &mut self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        node: DrmNode,
        crtc: crtc::Handle,
        metadata: DrmEventMetadata,
    )

    /// Handle the estimated vblank timer firing.
    pub fn on_estimated_vblank_timer(&self, niri: &mut Niri, output: Output)

    /// Enable or disable debug tinting.
    pub fn set_debug_tint(&mut self, enabled: bool)

    pub fn debug_tint(&self) -> bool
}
```

---

## Implementation: `src/backend/tty/render.rs`

```rust
//! Rendering subsystem for TTY backend.
//!
//! Handles frame rendering, vblank events, and presentation timing.

use std::time::Duration;

use smithay::backend::drm::{DrmEventMetadata, DrmNode};
use smithay::output::Output;
use smithay::reexports::drm::control::crtc;

use super::devices::DeviceManager;
use crate::backend::RenderResult;
use crate::niri::Niri;

/// Rendering subsystem.
///
/// Handles:
/// - Frame rendering and submission
/// - VBlank event processing
/// - Presentation timing and feedback
pub struct RenderManager {
    debug_tint: bool,
}

impl RenderManager {
    pub fn new() -> Self {
        Self { debug_tint: false }
    }

    /// Render a frame for the given output.
    pub fn render(
        &mut self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        output: &Output,
        target_presentation_time: Duration,
    ) -> RenderResult {
        let _span = tracy_client::span!("RenderManager::render");

        // 1. Find the surface for this output
        let (node, crtc) = Self::find_output_surface(output)?;
        let device = devices.get_mut(&node)?;
        let surface = device.surface_mut(crtc)?;

        // 2. Get the renderer
        let render_node = device.render_node().unwrap_or(devices.primary_render_node());
        let mut renderer = devices.gpu_manager_mut()
            .renderer(&render_node, device.drm())?;

        // 3. Render elements
        let elements = niri.render(&mut renderer, output, target_presentation_time);

        // 4. Submit to compositor
        match surface.compositor.render_frame(&mut renderer, &elements, ...) {
            Ok(result) => {
                // Handle presentation feedback
                RenderResult::Submitted
            }
            Err(err) => {
                tracing::warn!("error rendering frame: {err:?}");
                RenderResult::Error
            }
        }
    }

    /// Handle a VBlank event from DRM.
    pub fn on_vblank(
        &mut self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        node: DrmNode,
        crtc: crtc::Handle,
        metadata: DrmEventMetadata,
    ) {
        let _span = tracy_client::span!("RenderManager::on_vblank");

        let Some(device) = devices.get_mut(&node) else {
            return;
        };
        let Some(surface) = device.surface_mut(crtc) else {
            return;
        };

        // 1. Process presentation feedback
        // ...

        // 2. Update frame clock
        // ...

        // 3. Schedule next redraw if needed
        // ...
    }

    /// Handle the estimated vblank timer firing.
    pub fn on_estimated_vblank_timer(&self, niri: &mut Niri, output: Output) {
        niri.send_frame_callbacks(&output);
    }

    pub fn set_debug_tint(&mut self, enabled: bool) {
        self.debug_tint = enabled;
    }

    pub fn debug_tint(&self) -> bool {
        self.debug_tint
    }

    fn find_output_surface(output: &Output) -> Option<(DrmNode, crtc::Handle)> {
        let state = output.user_data().get::<super::types::TtyOutputState>()?;
        Some((state.node, state.crtc))
    }
}
```

---

## Migration: Update Tty

```rust
// BEFORE
impl Tty {
    pub fn render(&mut self, niri: &mut Niri, output: &Output, ...) -> RenderResult {
        // 400 lines of render logic
    }
}

// AFTER  
impl Tty {
    pub fn render(&mut self, niri: &mut Niri, output: &Output, target_time: Duration) -> RenderResult {
        self.render.render(&mut self.devices, niri, output, target_time)
    }
}
```

---

## Cross-Subsystem Access Pattern

`RenderManager` needs `DeviceManager` to access surfaces:

```rust
pub fn render(
    &mut self,
    devices: &mut DeviceManager,  // Borrow what you need
    niri: &mut Niri,
    output: &Output,
) -> RenderResult
```

This is the standard pattern when subsystems need to interact.

---

## Verification Checklist

- [ ] Frame rendering works correctly
- [ ] VBlank timing is accurate
- [ ] Presentation feedback is correct
- [ ] VRR works when enabled
- [ ] Debug tint toggle works
- [ ] `cargo check` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/render.rs` | `RenderManager` subsystem (~500 LOC) |
| `src/backend/tty/mod.rs` | `Tty` delegates to `render` |

---

## Next Phase

[Phase T1.7: Create OutputManager Subsystem](phase-T1.7-extract-output.md)
