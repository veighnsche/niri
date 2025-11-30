# Phase 05: Move `render` to RenderManager

> **Status**: ✅ COMPLETE  
> **LOC**: ~181  
> **Target**: `src/backend/tty/render.rs`  
> **Source**: `src/backend/tty/mod.rs` lines 1598-1778

---

## Overview

Move the `render` method from `Tty` to `RenderManager`. This is the core frame rendering logic.

---

## Current Signature (in Tty)

```rust
pub fn render(
    &mut self,
    niri: &mut Niri,
    output: &Output,
    target_presentation_time: Duration,
) -> RenderResult
```

---

## New Signature (in RenderManager)

```rust
pub fn render(
    &self,
    devices: &mut DeviceManager,
    niri: &mut Niri,
    output: &Output,
    target_presentation_time: Duration,
    config: &Rc<RefCell<Config>>,
) -> RenderResult
```

---

## Required Transformations

| Old Code | New Code |
|----------|----------|
| `self.devices.primary_render_node()` | `devices.primary_render_node()` |
| `self.devices.get(&node)` | `devices.get(&node)` |
| `self.devices.get_mut(&node)` | `devices.get_mut(&node)` |
| `self.devices.gpu_manager_and_devices_mut()` | `devices.gpu_manager_and_devices_mut()` |
| `self.config.borrow()` | `config.borrow()` |

---

## Implementation Steps

### Step 1: Add imports to render.rs

```rust
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use std::time::Duration;

use niri_config::Config;
use smithay::backend::drm::compositor::{FrameFlags, PrimaryPlaneElement};
use smithay::backend::drm::DrmNode;
use smithay::output::Output;
use smithay::reexports::drm::control::crtc;

use super::devices::DeviceManager;
use super::types::TtyOutputState;
use crate::backend::RenderResult;
use crate::niri::{Niri, RedrawState};
use crate::render_helpers::debug::draw_damage;
use crate::render_helpers::RenderTarget;
```

### Step 2: Move method body

Copy lines 1598-1778 from mod.rs, applying transformations.

### Step 3: Handle queue_estimated_vblank_timer

The method calls `queue_estimated_vblank_timer(...)` which is a free function. Keep it in mod.rs or move to render.rs — it's closely related to render.

**Recommendation**: Move it to render.rs as a method or associated function.

### Step 4: Create delegation in Tty

```rust
impl Tty {
    pub fn render(
        &mut self,
        niri: &mut Niri,
        output: &Output,
        target_presentation_time: Duration,
    ) -> RenderResult {
        self.render.render(
            &mut self.devices,
            niri,
            output,
            target_presentation_time,
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

Manual test: verify display works.

---

## Dependencies

- **Requires**: Phase 04 (connector_connected)
- **Blocks**: Phase 06 (on_vblank)

---

## Notes

- Uses split borrow pattern via `gpu_manager_and_devices_mut()`
- Creates renderer from GPU manager
- Calls niri.render() for element generation
- Handles DRM compositor frame submission
- Sets up presentation feedback
- Queues vblank timer on failure
