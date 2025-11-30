# Phase 03: Move `device_removed` to DeviceManager

> **Status**: ⏳ PENDING  
> **LOC**: ~108  
> **Target**: `src/backend/tty/devices.rs`  
> **Source**: `src/backend/tty/mod.rs` lines 901-1008

---

## Overview

Move the `device_removed` method from `Tty` to `DeviceManager`. This method handles GPU unplug cleanup.

---

## Current Signature (in Tty)

```rust
fn device_removed(&mut self, device_id: dev_t, niri: &mut Niri)
```

---

## New Signature (in DeviceManager)

```rust
pub fn device_removed(
    &mut self,
    device_id: dev_t,
    niri: &mut Niri,
    event_loop: &LoopHandle<State>,
    session: &LibSeatSession,
) -> Option<OwnedFd>
```

Returns the device FD that needs to be closed via session.

---

## Required Transformations

| Old Code | New Code |
|----------|----------|
| `self.devices.get_mut(&node)` | `self.get_mut(&node)` |
| `self.devices.connector_disconnected(...)` | `self.connector_disconnected(...)` |
| `self.devices.remove(&node)` | `self.remove(&node)` |
| `self.devices.values()` | `self.values()` |
| `self.devices.values_mut()` | `self.values_mut()` |
| `self.devices.gpu_manager_mut()` | `self.gpu_manager_mut()` |
| `self.devices.primary_render_node()` | `self.primary_render_node()` |
| `self.devices.take_dmabuf_global()` | `self.take_dmabuf_global()` |
| `niri.event_loop.remove(...)` | `event_loop.remove(...)` |
| `niri.event_loop.insert_source(...)` | `event_loop.insert_source(...)` |
| `self.session.close(fd)` | Return fd, caller closes |
| `self.refresh_ipc_outputs(niri)` | Caller handles |

---

## Implementation Steps

### Step 1: Move method body

Copy lines 901-1008 from mod.rs, applying transformations.

### Step 2: Return FD instead of closing

Instead of:
```rust
match TryInto::<OwnedFd>::try_into(device_fd) {
    Ok(fd) => {
        if let Err(err) = self.session.close(fd) {
            warn!("error closing DRM device fd: {err:?}");
        }
    }
    Err(_) => {
        error!("unable to close DRM device cleanly: fd has unexpected references");
    }
}
```

Return:
```rust
TryInto::<OwnedFd>::try_into(device_fd).ok()
```

### Step 3: Create delegation in Tty

```rust
impl Tty {
    fn device_removed(&mut self, device_id: dev_t, niri: &mut Niri) {
        let fd = self.devices.device_removed(
            device_id,
            niri,
            &niri.event_loop,
            &self.session,
        );
        
        if let Some(fd) = fd {
            if let Err(err) = self.session.close(fd) {
                warn!("error closing DRM device fd: {err:?}");
            }
        }
        
        self.refresh_ipc_outputs(niri);
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

- **Requires**: Phase 01, Phase 02
- **Blocks**: Phase 04 (connector_connected)

---

## Notes

- Disconnects all connectors on the device
- Removes device from internal map
- Cleans up render node from GPU manager
- Destroys dmabuf global if this was the primary GPU
- Removes event loop registration
