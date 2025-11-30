# Phase 09: Move Gamma Control to OutputManager

> **Status**: ⏳ PENDING  
> **LOC**: ~43  
> **Target**: `src/backend/tty/outputs.rs`  
> **Source**: `src/backend/tty/mod.rs` lines 1830-1873

---

## Overview

Move the gamma control methods from `Tty` to `OutputManager`:
- `get_gamma_size` (~20 LOC)
- `set_gamma` (~23 LOC)

---

## Current Signatures (in Tty)

```rust
pub fn get_gamma_size(&self, output: &Output) -> anyhow::Result<u32>

pub fn set_gamma(&mut self, output: &Output, ramp: Option<Vec<u16>>) -> anyhow::Result<()>
```

---

## New Signatures (in OutputManager)

```rust
pub fn get_gamma_size(
    &self,
    devices: &DeviceManager,
    output: &Output,
) -> anyhow::Result<u32>

pub fn set_gamma(
    &self,
    devices: &mut DeviceManager,
    output: &Output,
    ramp: Option<Vec<u16>>,
    session_active: bool,
) -> anyhow::Result<()>
```

---

## Required Transformations

| Old Code | New Code |
|----------|----------|
| `self.devices.get(&node)` | `devices.get(&node)` |
| `self.devices.get_mut(&node)` | `devices.get_mut(&node)` |
| `self.session.is_active()` | `session_active` parameter |

---

## Implementation Steps

### Step 1: Add imports to outputs.rs

```rust
use anyhow::Context;
use smithay::output::Output;
use smithay::reexports::drm::control::crtc;

use super::helpers::set_gamma_for_crtc;
```

### Step 2: Move `get_gamma_size`

```rust
pub fn get_gamma_size(
    &self,
    devices: &DeviceManager,
    output: &Output,
) -> anyhow::Result<u32> {
    let tty_state = output.user_data().get::<TtyOutputState>().unwrap();
    let crtc = tty_state.crtc;

    let device = devices
        .get(&tty_state.node)
        .context("missing device")?;

    let surface = device.surfaces.get(&crtc).context("missing surface")?;
    if let Some(gamma_props) = &surface.gamma_props {
        gamma_props.gamma_size(&device.drm)
    } else {
        let info = device
            .drm
            .get_crtc(crtc)
            .context("error getting crtc info")?;
        Ok(info.gamma_length())
    }
}
```

### Step 3: Move `set_gamma`

```rust
pub fn set_gamma(
    &self,
    devices: &mut DeviceManager,
    output: &Output,
    ramp: Option<Vec<u16>>,
    session_active: bool,
) -> anyhow::Result<()> {
    let tty_state = output.user_data().get::<TtyOutputState>().unwrap();
    let crtc = tty_state.crtc;

    let device = devices
        .get_mut(&tty_state.node)
        .context("missing device")?;
    let surface = device.surfaces.get_mut(&crtc).context("missing surface")?;

    // Cannot change properties while the device is inactive.
    if !session_active {
        surface.pending_gamma_change = Some(ramp);
        return Ok(());
    }

    let ramp = ramp.as_deref();
    if let Some(gamma_props) = &mut surface.gamma_props {
        gamma_props.set_gamma(&device.drm, ramp)
    } else {
        set_gamma_for_crtc(&device.drm, crtc, ramp)
    }
}
```

### Step 4: Create delegations in Tty

```rust
impl Tty {
    pub fn get_gamma_size(&self, output: &Output) -> anyhow::Result<u32> {
        self.outputs.get_gamma_size(&self.devices, output)
    }

    pub fn set_gamma(&mut self, output: &Output, ramp: Option<Vec<u16>>) -> anyhow::Result<()> {
        self.outputs.set_gamma(&mut self.devices, output, ramp, self.session.is_active())
    }
}
```

---

## Verification

```bash
cargo check
cargo test
```

Test: color temperature / night light apps should work.

---

## Dependencies

- **Requires**: Phase 04 (DeviceManager complete)
- **Blocks**: None (can be done in parallel with 10-12)

---

## Notes

- Supports both atomic GAMMA_LUT and legacy gamma
- Defers gamma changes when session is inactive
- Used by color management / night light features
