# Phase 08: Move `refresh_ipc_outputs` to OutputManager

> **Status**: ⏳ PENDING  
> **LOC**: ~110  
> **Target**: `src/backend/tty/outputs.rs`  
> **Source**: `src/backend/tty/mod.rs` lines 1875-1984

---

## Overview

Move the `refresh_ipc_outputs` method from `Tty` to `OutputManager`. This method updates the IPC output map for external queries.

---

## Current Signature (in Tty)

```rust
fn refresh_ipc_outputs(&self, niri: &mut Niri)
```

---

## New Signature (in OutputManager)

```rust
pub fn refresh_ipc_outputs(
    &self,
    devices: &DeviceManager,
    niri: &mut Niri,
    config: &Rc<RefCell<Config>>,
)
```

---

## Required Transformations

| Old Code | New Code |
|----------|----------|
| `&self.devices` | `devices` parameter |
| `self.config.borrow()` | `config.borrow()` |
| `self.outputs.set_ipc_outputs(...)` | `self.set_ipc_outputs(...)` |

---

## Implementation Steps

### Step 1: Add imports to outputs.rs

```rust
use std::cell::RefCell;
use std::rc::Rc;

use niri_config::Config;
use smithay::backend::drm::{DrmNode, VrrSupport};
use smithay::reexports::drm::control::ModeFlags;
use smithay::reexports::drm::control::ModeTypeFlags;
use smithay::output::Mode;

use super::devices::{format_connector_name, DeviceManager};
use super::types::TtyOutputState;
use crate::backend::OutputId;
use crate::niri::Niri;
use crate::utils::logical_output;
```

### Step 2: Move method body

Copy lines 1875-1984 from mod.rs, applying transformations.

### Step 3: Create delegation in Tty

```rust
impl Tty {
    fn refresh_ipc_outputs(&self, niri: &mut Niri) {
        self.outputs.refresh_ipc_outputs(&self.devices, niri, &self.config)
    }
}
```

---

## Verification

```bash
cargo check
cargo test
```

Test: `niri msg outputs` should work.

---

## Dependencies

- **Requires**: Phase 04 (connector_connected — DeviceManager complete)
- **Blocks**: Phase 12 (on_output_config_changed)

---

## Notes

- Iterates all devices and connectors
- Builds IPC output info (modes, VRR, etc.)
- Updates shared IpcOutputMap
- Called after any output change
