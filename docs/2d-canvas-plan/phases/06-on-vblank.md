# Phase 06: Move `on_vblank` to RenderManager

> **Status**: ✅ COMPLETE  
> **LOC**: ~188  
> **Target**: `src/backend/tty/render.rs`  
> **Source**: `src/backend/tty/mod.rs` lines 1359-1546

---

## Overview

Move the `on_vblank` method from `Tty` to `RenderManager`. This handles DRM vblank events and presentation timing.

---

## Current Signature (in Tty)

```rust
fn on_vblank(
    &mut self,
    niri: &mut Niri,
    node: DrmNode,
    crtc: crtc::Handle,
    meta: DrmEventMetadata,
)
```

---

## New Signature (in RenderManager)

```rust
pub fn on_vblank(
    &self,
    devices: &mut DeviceManager,
    niri: &mut Niri,
    node: DrmNode,
    crtc: crtc::Handle,
    meta: DrmEventMetadata,
    config: &Rc<RefCell<Config>>,
)
```

---

## Required Transformations

| Old Code | New Code |
|----------|----------|
| `self.devices.get_mut(&node)` | `devices.get_mut(&node)` |
| `niri.config.borrow()` | `config.borrow()` |

---

## Implementation Steps

### Step 1: Add imports to render.rs

```rust
use smithay::backend::drm::{DrmEventMetadata, DrmEventTime};
use smithay::wayland::presentation::Refresh;
use wayland_protocols::wp::presentation_time::server::wp_presentation_feedback;

use crate::utils::get_monotonic_time;
```

### Step 2: Move method body

Copy lines 1359-1546 from mod.rs, applying transformations.

### Step 3: Handle vblank_throttle callback

The method uses `output_state.vblank_throttle.throttle(...)` with a callback that calls `tty.on_vblank(...)`. This creates a circular dependency.

**Solution**: Keep the callback in mod.rs:
```rust
// In mod.rs, the event loop callback for DRM events:
DrmEvent::VBlank(crtc) => {
    let meta = meta.expect("VBlank events must have metadata");
    tty.render.on_vblank(&mut tty.devices, &mut state.niri, node, crtc, meta, &tty.config);
}
```

The throttle callback needs special handling — it reschedules a call to on_vblank. We may need to keep the throttle callback in Tty.

### Step 4: Create delegation in Tty

```rust
impl Tty {
    fn on_vblank(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        crtc: crtc::Handle,
        meta: DrmEventMetadata,
    ) {
        self.render.on_vblank(
            &mut self.devices,
            niri,
            node,
            crtc,
            meta,
            &self.config,
        )
    }
}
```

---

## Verification

```bash
cargo check
cargo test
```

---

## Dependencies

- **Requires**: Phase 05 (render)
- **Blocks**: Phase 07 (estimated_vblank_timer)

---

## Notes

- Processes DRM vblank events
- Updates frame clock with presentation time
- Handles presentation feedback for Wayland protocol
- Manages redraw state machine
- Queues next redraw if animations remain
- Complex timing logic with Tracy profiling
