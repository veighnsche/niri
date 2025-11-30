# Phase T1.4: Create DeviceManager Subsystem

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1.5 hours  
> **Risk Level**: 🟡 Medium  
> **Architectural Benefit**: ⭐⭐⭐ High - core subsystem owning device state

---

## Goal

Create the `DeviceManager` subsystem that **OWNS** all DRM device state. This is the most important subsystem extraction.

**Key principle**: `DeviceManager` owns its state privately and exposes an intentional API.

---

## What DeviceManager Owns

```rust
pub struct DeviceManager {
    // ALL PRIVATE - encapsulated state
    devices: HashMap<DrmNode, OutputDevice>,
    primary_node: DrmNode,
    primary_render_node: DrmNode,
    ignored_nodes: HashSet<DrmNode>,
    gpu_manager: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
    dmabuf_global: Option<DmabufGlobal>,
}
```

---

## DeviceManager API

```rust
impl DeviceManager {
    // === Constructor ===
    pub fn new(
        primary_node: DrmNode,
        primary_render_node: DrmNode,
        ignored_nodes: HashSet<DrmNode>,
        gpu_manager: GpuManager<...>,
    ) -> Self

    // === Device Lifecycle ===
    pub fn device_added(
        &mut self,
        device_id: dev_t,
        path: &Path,
        session: &LibSeatSession,
        event_loop: &LoopHandle<State>,
        niri: &mut Niri,
    ) -> anyhow::Result<()>

    pub fn device_changed(&mut self, device_id: dev_t, niri: &mut Niri, cleanup: bool)

    pub fn device_removed(&mut self, device_id: dev_t, niri: &mut Niri, event_loop: &LoopHandle<State>)

    // === Session Events ===
    pub fn pause_devices(&mut self)
    pub fn resume_devices(&mut self, niri: &mut Niri)

    // === Access ===
    pub fn get(&self, node: &DrmNode) -> Option<&OutputDevice>
    pub fn get_mut(&mut self, node: &DrmNode) -> Option<&mut OutputDevice>
    pub fn iter(&self) -> impl Iterator<Item = (&DrmNode, &OutputDevice)>
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&DrmNode, &mut OutputDevice)>

    // === Node Info ===
    pub fn primary_node(&self) -> DrmNode
    pub fn primary_render_node(&self) -> DrmNode
    pub fn is_ignored(&self, node: &DrmNode) -> bool
    pub fn update_ignored_nodes(&mut self, ignored: HashSet<DrmNode>)

    // === GPU Manager ===
    pub fn gpu_manager(&self) -> &GpuManager<...>
    pub fn gpu_manager_mut(&mut self) -> &mut GpuManager<...>

    // === DmaBuf ===
    pub fn dmabuf_global(&self) -> Option<&DmabufGlobal>
    pub fn set_dmabuf_global(&mut self, global: Option<DmabufGlobal>)
}
```

---

## Implementation: `src/backend/tty/devices.rs`

```rust
//! Device management subsystem.
//!
//! This subsystem OWNS all DRM device state and provides the device lifecycle API.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use libc::dev_t;
use smithay::backend::drm::{DrmDeviceFd, DrmNode};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::multigpu::gbm::GbmGlesBackend;
use smithay::backend::renderer::multigpu::GpuManager;
use smithay::backend::session::libseat::LibSeatSession;
use smithay::reexports::calloop::LoopHandle;
use smithay::wayland::dmabuf::DmabufGlobal;

use super::helpers::{ignored_nodes_from_config, primary_node_from_config};
use crate::niri::{Niri, State};

/// Device management subsystem.
///
/// OWNS all DRM device state:
/// - Connected devices (GPUs)
/// - Primary/render node tracking
/// - GPU manager for multi-GPU
/// - DmaBuf global
pub struct DeviceManager {
    devices: HashMap<DrmNode, OutputDevice>,
    primary_node: DrmNode,
    primary_render_node: DrmNode,
    ignored_nodes: HashSet<DrmNode>,
    gpu_manager: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
    dmabuf_global: Option<DmabufGlobal>,
}

impl DeviceManager {
    pub fn new(
        primary_node: DrmNode,
        primary_render_node: DrmNode,
        ignored_nodes: HashSet<DrmNode>,
        gpu_manager: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
    ) -> Self {
        Self {
            devices: HashMap::new(),
            primary_node,
            primary_render_node,
            ignored_nodes,
            gpu_manager,
            dmabuf_global: None,
        }
    }

    // === Device Lifecycle ===

    pub fn device_added(
        &mut self,
        device_id: dev_t,
        path: &Path,
        session: &LibSeatSession,
        event_loop: &LoopHandle<State>,
        niri: &mut Niri,
    ) -> anyhow::Result<()> {
        // Implementation moved from Tty::device_added
        // ... ~200 lines
        todo!()
    }

    pub fn device_changed(&mut self, device_id: dev_t, niri: &mut Niri, cleanup: bool) {
        // Implementation moved from Tty::device_changed
        // ... ~160 lines
        todo!()
    }

    pub fn device_removed(
        &mut self,
        device_id: dev_t,
        niri: &mut Niri,
        event_loop: &LoopHandle<State>,
    ) {
        // Implementation moved from Tty::device_removed
        // ... ~110 lines
        todo!()
    }

    // === Session Events ===

    pub fn pause_devices(&mut self) {
        for device in self.devices.values_mut() {
            device.drm_mut().pause();
            if let Some(lease_state) = device.lease_state_mut() {
                lease_state.suspend();
            }
        }
    }

    pub fn resume_devices(&mut self, niri: &mut Niri) {
        for (node, device) in self.devices.iter_mut() {
            if let Err(err) = device.drm_mut().activate(false) {
                tracing::warn!("error activating DRM device: {err:?}");
            }
            if let Some(lease_state) = device.lease_state_mut() {
                lease_state.resume::<State>();
            }
        }
    }

    // === Access ===

    pub fn get(&self, node: &DrmNode) -> Option<&OutputDevice> {
        self.devices.get(node)
    }

    pub fn get_mut(&mut self, node: &DrmNode) -> Option<&mut OutputDevice> {
        self.devices.get_mut(node)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DrmNode, &OutputDevice)> {
        self.devices.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&DrmNode, &mut OutputDevice)> {
        self.devices.iter_mut()
    }

    // === Node Info ===

    pub fn primary_node(&self) -> DrmNode {
        self.primary_node
    }

    pub fn primary_render_node(&self) -> DrmNode {
        self.primary_render_node
    }

    pub fn is_ignored(&self, node: &DrmNode) -> bool {
        self.ignored_nodes.contains(node)
    }

    // === GPU Manager ===

    pub fn gpu_manager(&self) -> &GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>> {
        &self.gpu_manager
    }

    pub fn gpu_manager_mut(&mut self) -> &mut GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>> {
        &mut self.gpu_manager
    }

    // === DmaBuf ===

    pub fn dmabuf_global(&self) -> Option<&DmabufGlobal> {
        self.dmabuf_global.as_ref()
    }

    pub fn set_dmabuf_global(&mut self, global: Option<DmabufGlobal>) {
        self.dmabuf_global = global;
    }
}
```

---

## Migration: Update Tty

```rust
// BEFORE
impl Tty {
    fn device_added(&mut self, ...) { ... }
    fn device_removed(&mut self, ...) { ... }
}

// AFTER
impl Tty {
    fn on_udev_event(&mut self, niri: &mut Niri, event: UdevEvent) {
        match event {
            UdevEvent::Added { device_id, path } => {
                // Delegate to subsystem
                if let Err(err) = self.devices.device_added(
                    device_id, &path, &self.session, &niri.event_loop, niri
                ) {
                    tracing::warn!("error adding device: {err:?}");
                }
            }
            // ...
        }
    }
}
```

---

## Verification Checklist

- [ ] `DeviceManager` owns all device state privately
- [ ] All device lifecycle goes through `DeviceManager` API
- [ ] `Tty` delegates to `DeviceManager`
- [ ] GPU hotplug works
- [ ] `cargo check` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/devices.rs` | `DeviceManager` subsystem (~700 LOC) |
| `src/backend/tty/mod.rs` | `Tty` delegates to `devices` |

---

## Next Phase

[Phase T1.5: Integrate Connectors into DeviceManager](phase-T1.5-extract-connectors.md)
