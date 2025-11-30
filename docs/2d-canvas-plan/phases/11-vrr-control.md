# Phase 11: Move `set_output_on_demand_vrr` to OutputManager

> **Status**: ✅ COMPLETE  
> **LOC**: ~33  
> **Target**: `src/backend/tty/outputs.rs`  
> **Source**: `src/backend/tty/mod.rs` lines 2022-2054

---

## Overview

Move the VRR (Variable Refresh Rate) control method from `Tty` to `OutputManager`.

---

## Current Signature (in Tty)

```rust
pub fn set_output_on_demand_vrr(&mut self, niri: &mut Niri, output: &Output, enable_vrr: bool)
```

---

## New Signature (in OutputManager)

```rust
pub fn set_output_on_demand_vrr(
    &self,
    devices: &mut DeviceManager,
    niri: &mut Niri,
    output: &Output,
    enable_vrr: bool,
) -> bool  // Returns true if refresh_ipc_outputs should be called
```

---

## Implementation Steps

### Step 1: Move method body

```rust
impl OutputManager {
    pub fn set_output_on_demand_vrr(
        &self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        output: &Output,
        enable_vrr: bool,
    ) -> bool {
        let _span = tracy_client::span!("OutputManager::set_output_on_demand_vrr");

        let output_state = niri.outputs.state_mut(output).unwrap();
        output_state.on_demand_vrr_enabled = enable_vrr;
        if output_state.frame_clock.vrr() == enable_vrr {
            return false;
        }
        
        let tty_state: &TtyOutputState = output.user_data().get().unwrap();
        let target_node = tty_state.node;
        let target_crtc = tty_state.crtc;

        let mut found = false;
        if let Some(device) = devices.get_mut(&target_node) {
            if let Some(surface) = device.surfaces.get_mut(&target_crtc) {
                let word = if enable_vrr { "enabling" } else { "disabling" };
                if let Err(err) = surface.compositor.use_vrr(enable_vrr) {
                    warn!(
                        "output {:?}: error {} VRR: {err:?}",
                        surface.name.connector, word
                    );
                }
                output_state
                    .frame_clock
                    .set_vrr(surface.compositor.vrr_enabled());
                found = true;
            }
        }

        found
    }
}
```

### Step 2: Create delegation in Tty

```rust
impl Tty {
    pub fn set_output_on_demand_vrr(&mut self, niri: &mut Niri, output: &Output, enable_vrr: bool) {
        let needs_refresh = self.outputs.set_output_on_demand_vrr(
            &mut self.devices,
            niri,
            output,
            enable_vrr,
        );
        
        if needs_refresh {
            self.refresh_ipc_outputs(niri);
        }
    }
}
```

---

## Verification

```bash
cargo check
cargo test
```

Test: VRR toggle should work.

---

## Dependencies

- **Requires**: Phase 04 (DeviceManager), Phase 08 (refresh_ipc_outputs)
- **Blocks**: None

---

## Notes

- On-demand VRR enables/disables based on content
- Updates frame clock VRR state
- Triggers IPC refresh on change
