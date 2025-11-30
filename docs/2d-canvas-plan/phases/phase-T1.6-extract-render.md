# Phase T1.6: Extract Render Pipeline

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟡 Medium  
> **Architectural Benefit**: ⭐⭐ Medium - isolates rendering logic

---

## Goal

Extract rendering and vblank handling into `src/backend/tty/render.rs`.

This includes:
- The main `render()` function
- VBlank event handling
- Estimated vblank timer handling

---

## What Moves

### render (lines 1795-1953, ~160 lines)

```rust
impl Tty {
    pub fn render(
        &mut self,
        niri: &mut Niri,
        output: &Output,
        target_presentation_time: Duration,
    ) -> RenderResult
}
```

This is the main rendering entry point that:
1. Gets the surface for the output
2. Sets up the renderer
3. Calls niri's render function
4. Submits the frame
5. Handles presentation feedback

### on_vblank (lines 1558-1746, ~190 lines)

```rust
impl Tty {
    fn on_vblank(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        crtc: crtc::Handle,
        metadata: DrmEventMetadata,
    )
}
```

Handles:
1. Presentation feedback
2. Damage tracking
3. Redraw scheduling
4. VRR adjustments

### on_estimated_vblank_timer (lines 1747-1778, ~30 lines)

```rust
impl Tty {
    fn on_estimated_vblank_timer(&self, niri: &mut Niri, output: Output)
}
```

Handles the fallback timer for estimated vblank when DRM doesn't provide timing.

---

## Why This is Good Architecture

1. **Clear rendering pipeline** - All render code in one place
2. **Performance critical** - Easy to profile and optimize
3. **Separate from device management** - Rendering doesn't care about hotplug
4. **Testable frame logic** - Could add frame timing tests

---

## Target: `src/backend/tty/render.rs`

```rust
//! Rendering pipeline for TTY backend.
//!
//! Handles:
//! - Frame rendering and submission
//! - VBlank event processing
//! - Presentation timing and feedback

use std::time::Duration;
use smithay::backend::drm::{DrmEventMetadata, DrmNode};
use smithay::output::Output;
use smithay::reexports::drm::control::crtc;

use super::Tty;
use super::types::Surface;
use crate::backend::RenderResult;
use crate::niri::Niri;

impl Tty {
    /// Render a frame for the given output.
    ///
    /// Returns the render result indicating success/failure and
    /// whether damage was present.
    pub fn render(
        &mut self,
        niri: &mut Niri,
        output: &Output,
        target_presentation_time: Duration,
    ) -> RenderResult {
        let _span = tracy_client::span!("Tty::render");

        // 1. Find the surface for this output
        let Some((node, device, surface)) = self.find_output_surface(output) else {
            return RenderResult::Error;
        };

        // 2. Get the renderer
        let primary_renderer = self.gpu_manager.single_renderer(&device.drm).unwrap();
        let mut renderer = if let Some(render_node) = device.render_node {
            self.gpu_manager.renderer(&render_node, &device.drm).unwrap()
        } else {
            primary_renderer
        };

        // 3. Start the frame
        let elements = niri.render(
            &mut renderer,
            output,
            target_presentation_time,
        );

        // 4. Submit to compositor
        match surface.compositor.render_frame(&mut renderer, &elements, ...) {
            Ok(result) => {
                // Handle presentation feedback
                // Submit frame
                RenderResult::Submitted
            }
            Err(err) => {
                warn!("error rendering frame: {err:?}");
                RenderResult::Error
            }
        }
    }

    /// Handle a VBlank event from DRM.
    pub(super) fn on_vblank(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        crtc: crtc::Handle,
        metadata: DrmEventMetadata,
    ) {
        let _span = tracy_client::span!("Tty::on_vblank");

        // 1. Find the surface
        let Some(device) = self.devices.get_mut(&node) else {
            return;
        };
        let Some(surface) = device.surfaces.get_mut(&crtc) else {
            return;
        };

        // 2. Process presentation feedback
        // ...

        // 3. Update frame clock
        // ...

        // 4. Schedule next redraw if needed
        // ...
    }

    /// Handle the estimated vblank timer firing.
    pub(super) fn on_estimated_vblank_timer(&self, niri: &mut Niri, output: Output) {
        let _span = tracy_client::span!("Tty::on_estimated_vblank_timer");

        // Send frame callbacks even without real vblank
        niri.send_frame_callbacks(&output);
    }

    /// Find the surface for an output.
    fn find_output_surface(&mut self, output: &Output) -> Option<(DrmNode, &mut OutputDevice, &mut Surface)> {
        // ...
    }
}
```

---

## Module-Level Function

The `queue_estimated_vblank_timer` function (lines 2880-2924) should also move here since it's render-related:

```rust
pub(super) fn queue_estimated_vblank_timer(
    niri: &mut Niri,
    output: Output,
    target_presentation_time: Duration,
) {
    // ...
}
```

---

## Verification

- [ ] Rendering works correctly
- [ ] VBlank timing is accurate
- [ ] Presentation feedback is correct
- [ ] VRR works when enabled
- [ ] `cargo check` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/render.rs` | Created (~420 LOC) |
| `src/backend/tty/mod.rs` | Removed render methods, added `mod render` |

---

## Next Phase

After completing this phase, proceed to [Phase T1.7: Extract Output Management](phase-T1.7-extract-output.md).
