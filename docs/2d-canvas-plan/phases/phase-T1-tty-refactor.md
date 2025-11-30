# Phase T1: TTY Backend Refactor (Subsystem Pattern)

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~6-8 hours  
> **Risk Level**: 🟡 Medium  
> **Prerequisite**: None  
> **Unblocks**: Cleaner backend code, easier hardware support, testable subsystems

---

## Goal

Refactor `src/backend/tty.rs` (3473 lines) into a modular `src/backend/tty/` directory using the **subsystem ownership pattern** from `src/niri/subsystems/`. Each subsystem OWNS its state and provides a focused API.

**This is NOT just file splitting** — we're creating proper abstractions that encapsulate state and behavior.

---

## Current State

The `src/backend/tty.rs` file is a monolithic 3473-line god object containing:
- **Tty struct** (~130 lines) - 15+ fields, owns everything
- **OutputDevice struct** (~240 lines) - already a separate type
- **impl Tty** (~2120 lines!) - all logic mixed together
- **Helper functions** (~400 lines) - pure DRM/mode functions
- **GammaProps impl** (~140 lines) - gamma LUT management
- **ConnectorProperties** (~170 lines) - connector state
- **Tests** (~130 lines)

### Problems with Current Code

1. **God object** - `Tty` owns 15+ fields directly
2. **No encapsulation** - All state accessible everywhere
3. **Untestable** - Can't test device lifecycle without full compositor
4. **Tangled concerns** - Rendering, device management, IPC all mixed

---

## Target Architecture

### Subsystem Ownership Pattern

Following `src/niri/subsystems/`, each subsystem:
- **OWNS its state** (private fields)
- **Exposes intentional API** (public methods)
- **Has single responsibility**
- **Can be tested in isolation**

```
src/backend/tty/
├── mod.rs              # Tty thin coordinator (~200 LOC)
├── types.rs            # Type definitions (~150 LOC)
├── helpers.rs          # Pure functions (~400 LOC)
├── devices.rs          # DeviceManager subsystem (~700 LOC)
├── render.rs           # RenderManager subsystem (~500 LOC)
├── outputs.rs          # OutputManager subsystem (~650 LOC)
└── gamma.rs            # GammaProps (~150 LOC)
```

### Before vs After

```rust
// BEFORE: God object with 15+ fields
pub struct Tty {
    config: Rc<RefCell<Config>>,
    session: LibSeatSession,
    udev_dispatcher: Dispatcher<...>,
    libinput: Libinput,
    gpu_manager: GpuManager<...>,
    primary_node: DrmNode,
    primary_render_node: DrmNode,
    ignored_nodes: HashSet<DrmNode>,
    devices: HashMap<DrmNode, OutputDevice>,
    dmabuf_global: Option<DmabufGlobal>,
    update_output_config_on_resume: bool,
    update_ignored_nodes_on_resume: bool,
    debug_tint: bool,
    ipc_outputs: Arc<Mutex<IpcOutputMap>>,
}

// AFTER: Thin coordinator with subsystems
pub struct Tty {
    config: Rc<RefCell<Config>>,
    session: LibSeatSession,
    udev_dispatcher: Dispatcher<...>,
    libinput: Libinput,
    
    // Subsystems that OWN their state
    pub(crate) devices: DeviceManager,
    pub(crate) render: RenderManager,
    pub(crate) outputs: OutputManager,
}
```

---

## Subsystem Definitions

### 1. DeviceManager (~700 LOC)

**Owns**: All DRM device state and lifecycle.

```rust
pub struct DeviceManager {
    // Private state
    devices: HashMap<DrmNode, OutputDevice>,
    primary_node: DrmNode,
    primary_render_node: DrmNode,
    ignored_nodes: HashSet<DrmNode>,
    gpu_manager: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
    dmabuf_global: Option<DmabufGlobal>,
}

impl DeviceManager {
    // Lifecycle
    pub fn device_added(&mut self, ...) -> anyhow::Result<()>
    pub fn device_changed(&mut self, ...)
    pub fn device_removed(&mut self, ...)
    
    // Connectors
    pub fn connector_connected(&mut self, ...) -> anyhow::Result<()>
    pub fn connector_disconnected(&mut self, ...)
    
    // Access
    pub fn get(&self, node: &DrmNode) -> Option<&OutputDevice>
    pub fn get_mut(&mut self, node: &DrmNode) -> Option<&mut OutputDevice>
    pub fn primary_node(&self) -> DrmNode
    pub fn gpu_manager(&self) -> &GpuManager<...>
}
```

### 2. RenderManager (~500 LOC)

**Owns**: Frame rendering, vblank handling, presentation feedback.

```rust
pub struct RenderManager {
    debug_tint: bool,
    // Frame state tracked per-surface in OutputDevice.surfaces
}

impl RenderManager {
    pub fn render(&mut self, devices: &mut DeviceManager, ...) -> RenderResult
    pub fn on_vblank(&mut self, devices: &mut DeviceManager, ...)
    pub fn on_estimated_vblank_timer(&self, ...)
    pub fn set_debug_tint(&mut self, enabled: bool)
}
```

### 3. OutputManager (~650 LOC)

**Owns**: IPC output reporting, gamma control, VRR, config changes.

```rust
pub struct OutputManager {
    ipc_outputs: Arc<Mutex<IpcOutputMap>>,
    update_config_on_resume: bool,
    update_ignored_nodes_on_resume: bool,
}

impl OutputManager {
    // IPC
    pub fn refresh_ipc_outputs(&self, devices: &DeviceManager, ...)
    pub fn ipc_outputs(&self) -> Arc<Mutex<IpcOutputMap>>
    
    // Gamma
    pub fn get_gamma_size(&self, devices: &DeviceManager, ...) -> anyhow::Result<u32>
    pub fn set_gamma(&mut self, devices: &mut DeviceManager, ...) -> anyhow::Result<()>
    
    // VRR
    pub fn set_output_on_demand_vrr(&mut self, devices: &mut DeviceManager, ...)
    
    // Config
    pub fn on_output_config_changed(&mut self, devices: &mut DeviceManager, ...)
    pub fn set_monitors_active(&mut self, devices: &mut DeviceManager, active: bool)
}
```

---

## Phases

| Phase | Focus | Benefit | Time |
|-------|-------|---------|------|
| [T1.1](phase-T1.1-extract-types.md) | Types & module structure | Foundation | 30m |
| [T1.2](phase-T1.2-extract-device.md) | OutputDevice enrichment | Better encapsulation | 45m |
| [T1.3](phase-T1.3-extract-helpers.md) | Pure helper functions | ⭐⭐⭐ Testable | 45m |
| [T1.4](phase-T1.4-extract-lifecycle.md) | DeviceManager subsystem | ⭐⭐⭐ Owns device state | 1.5h |
| [T1.5](phase-T1.5-extract-connectors.md) | Connectors into DeviceManager | Unified lifecycle | 45m |
| [T1.6](phase-T1.6-extract-render.md) | RenderManager subsystem | ⭐⭐⭐ Owns render state | 1h |
| [T1.7](phase-T1.7-extract-output.md) | OutputManager subsystem | ⭐⭐⭐ Owns IPC/config | 1h |

---

## Why This Order?

1. **T1.1 Types first** - Creates module structure and shared types
2. **T1.2 OutputDevice** - Enrich the existing type before subsystems use it
3. **T1.3 Helpers** - Pure functions needed by all subsystems
4. **T1.4 DeviceManager** - Core subsystem, biggest extraction
5. **T1.5 Connectors** - Integrate into DeviceManager
6. **T1.6 RenderManager** - Depends on DeviceManager existing
7. **T1.7 OutputManager** - Final piece, IPC/config management

---

## Success Criteria

- [ ] `Tty` struct has ≤8 fields (down from 15+)
- [ ] `DeviceManager` owns all device state privately
- [ ] `RenderManager` owns render state privately
- [ ] `OutputManager` owns IPC/config state privately
- [ ] `helpers.rs` has no Tty/State imports (pure functions)
- [ ] Each subsystem can be understood in isolation
- [ ] All tests pass
- [ ] `cargo check` passes

---

## Design Principles

### 1. Subsystem Ownership

Each subsystem OWNS its state:
```rust
pub struct DeviceManager {
    devices: HashMap<...>,  // PRIVATE - not pub(crate)
}

impl DeviceManager {
    pub fn get(&self, node: &DrmNode) -> Option<&OutputDevice>  // Intentional API
}
```

### 2. Thin Coordinator

`Tty` only does:
- Hold subsystems
- Dispatch events to appropriate subsystem
- Handle session/udev integration

```rust
impl Tty {
    fn on_udev_event(&mut self, niri: &mut Niri, event: UdevEvent) {
        match event {
            UdevEvent::Added { device_id, path } => {
                self.devices.device_added(device_id, path, ...);
            }
            // ...
        }
    }
}
```

### 3. Pure Helpers

`helpers.rs` contains only pure functions:
- DRM mode calculations
- Property lookups  
- Node discovery
- EDID parsing

### 4. Cross-Subsystem Access

When subsystems need to interact, pass references:
```rust
impl RenderManager {
    pub fn render(
        &mut self,
        devices: &mut DeviceManager,  // Borrow what you need
        niri: &mut Niri,
        output: &Output,
    ) -> RenderResult { ... }
}
```

---

## Comparison: Old vs New

| Metric | Old (file split) | New (subsystems) |
|--------|------------------|------------------|
| `Tty` fields | 15+ | ≤8 |
| Encapsulation | None | ✅ Private fields |
| Testability | Only helpers | ✅ Each subsystem |
| God object | Still exists | ✅ Eliminated |
| New abstractions | 0 | 3 subsystems |
| Cognitive load | Same | ⬇️ Lower per-subsystem |

---

## Quick Reference

| File | Responsibility | LOC |
|------|----------------|-----|
| `mod.rs` | Tty thin coordinator | ~200 |
| `types.rs` | Type definitions | ~150 |
| `helpers.rs` | Pure DRM/mode functions | ~400 |
| `devices.rs` | DeviceManager subsystem | ~700 |
| `render.rs` | RenderManager subsystem | ~500 |
| `outputs.rs` | OutputManager subsystem | ~650 |
| `gamma.rs` | GammaProps | ~150 |

**Total: ~2750 LOC** (reduced from 3473 through better organization)
