# Phase 01: Move `device_added` to DeviceManager

> **Status**: ⏳ PENDING  
> **LOC**: ~196  
> **Target**: `src/backend/tty/devices.rs`  
> **Source**: `src/backend/tty/mod.rs` lines 539-734

---

## Overview

Move the `device_added` method from `Tty` to `DeviceManager`. This method handles GPU hotplug when a new DRM device is connected.

---

## Current Signature (in Tty)

```rust
fn device_added(
    &mut self,
    device_id: dev_t,
    path: &Path,
    niri: &mut Niri,
) -> anyhow::Result<()>
```

---

## New Signature (in DeviceManager)

```rust
pub fn device_added(
    &mut self,
    device_id: dev_t,
    path: &Path,
    session: &LibSeatSession,
    event_loop: &LoopHandle<State>,
    config: &Rc<RefCell<Config>>,
    niri: &mut Niri,
    debug_tint: bool,
) -> anyhow::Result<()>
```

---

## Required Transformations

| Old Code | New Code |
|----------|----------|
| `self.session.open(...)` | `session.open(...)` |
| `self.config.borrow()` | `config.borrow()` |
| `self.devices.primary_node()` | `self.primary_node()` |
| `self.devices.primary_render_node()` | `self.primary_render_node()` |
| `self.devices.gpu_manager_mut()` | `self.gpu_manager_mut()` |
| `self.devices.get(...)` | `self.get(...)` |
| `self.devices.insert(...)` | `self.insert(...)` |
| `self.devices.dmabuf_global()` | `self.dmabuf_global()` |
| `self.devices.set_dmabuf_global(...)` | `self.set_dmabuf_global(...)` |
| `self.devices.iter_mut()` | `self.iter_mut()` |
| `self.render.debug_tint()` | `debug_tint` parameter |
| `self.device_changed(...)` | Return flag, caller handles |

---

## Implementation Steps

### Step 1: Add imports to devices.rs

```rust
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use libc::dev_t;
use niri_config::Config;
use smithay::backend::session::libseat::LibSeatSession;
use smithay::backend::session::Session;
use smithay::reexports::calloop::LoopHandle;
use smithay::reexports::rustix::fs::OFlags;
// ... other imports as needed
```

### Step 2: Move the method body

Copy lines 539-734 from mod.rs to DeviceManager impl block, applying the transformations above.

### Step 3: Handle `device_changed` call

The method calls `self.device_changed(...)` at the end. Options:
- Return a flag indicating device_changed should be called
- Pass a callback

**Recommended**: Return `(Ok(()), device_id)` and have Tty call device_changed.

### Step 4: Create delegation in Tty

```rust
impl Tty {
    fn device_added(
        &mut self,
        device_id: dev_t,
        path: &Path,
        niri: &mut Niri,
    ) -> anyhow::Result<()> {
        self.devices.device_added(
            device_id,
            path,
            &self.session,
            &niri.event_loop,
            &self.config,
            niri,
            self.render.debug_tint(),
        )?;
        
        // device_added internally calls device_changed
        self.device_changed(device_id, niri, true);
        Ok(())
    }
}
```

### Step 5: Update device_changed call inside device_added

Change the end of device_added from:
```rust
self.device_changed(device_id, niri, true);
Ok(())
```

To just:
```rust
Ok(())
```

And have the Tty delegation handle it.

---

## Verification

```bash
cargo check
cargo test
```

---

## Dependencies

- **Requires**: Nothing (first in sequence)
- **Blocks**: Phase 02 (device_changed), Phase 03 (device_removed)

---

## Notes

- The method creates a DRM device from scratch
- Handles EGL display initialization
- Sets up dmabuf feedback
- Creates OutputDevice and inserts into map
- Registers event loop source for DRM events
