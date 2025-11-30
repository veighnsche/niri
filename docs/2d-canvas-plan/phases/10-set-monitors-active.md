# Phase 10: Move `set_monitors_active` to OutputManager

> **Status**: ✅ COMPLETE  
> **LOC**: ~18  
> **Target**: `src/backend/tty/outputs.rs`  
> **Source**: `src/backend/tty/mod.rs` lines 2003-2020

---

## Overview

Move the `set_monitors_active` method from `Tty` to `OutputManager`. This method controls DPMS (monitor power state).

---

## Current Signature (in Tty)

```rust
pub fn set_monitors_active(&mut self, active: bool)
```

---

## New Signature (in OutputManager)

```rust
pub fn set_monitors_active(&self, devices: &mut DeviceManager, active: bool)
```

---

## Implementation Steps

### Step 1: Move method body

```rust
impl OutputManager {
    pub fn set_monitors_active(&self, devices: &mut DeviceManager, active: bool) {
        // We only disable the CRTC here, this will also reset the
        // surface state so that the next call to `render_frame` will
        // always produce a new frame and `queue_frame` will change
        // the CRTC to active. This makes sure we always enable a CRTC
        // within an atomic operation.
        if active {
            return;
        }

        for device in devices.values_mut() {
            for surface in device.surfaces.values_mut() {
                if let Err(err) = surface.compositor.clear() {
                    warn!("error clearing drm surface: {err:?}");
                }
            }
        }
    }
}
```

### Step 2: Create delegation in Tty

```rust
impl Tty {
    pub fn set_monitors_active(&mut self, active: bool) {
        self.outputs.set_monitors_active(&mut self.devices, active)
    }
}
```

---

## Verification

```bash
cargo check
cargo test
```

Test: DPMS / screen blanking should work.

---

## Dependencies

- **Requires**: Phase 04 (DeviceManager complete)
- **Blocks**: None (can be done in parallel)

---

## Notes

- Only handles deactivation (clearing)
- Activation happens automatically on next render
- Used for screen blanking / power saving
