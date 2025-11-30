# Phase T1.4a: DeviceManager Struct & Accessors

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟢 Low  
> **Architectural Benefit**: ⭐⭐⭐ High - creates the core subsystem structure

---

## Goal

Create the `DeviceManager` struct that **OWNS** all DRM device state. This phase creates the struct and accessor methods only - lifecycle methods come in T1.4b.

**Key principle**: `DeviceManager` owns its state privately and exposes an intentional API.

---

## What DeviceManager Owns

Move these fields from `Tty` to `DeviceManager`:

```rust
// FROM Tty:
pub struct Tty {
    // ... keep these:
    config: Rc<RefCell<Config>>,
    session: LibSeatSession,
    udev_dispatcher: Dispatcher<'static, UdevBackend, State>,
    libinput: Libinput,
    
    // MOVE these to DeviceManager:
    gpu_manager: GpuManager<...>,        // → DeviceManager
    primary_node: DrmNode,               // → DeviceManager
    primary_render_node: DrmNode,        // → DeviceManager
    ignored_nodes: HashSet<DrmNode>,     // → DeviceManager
    devices: HashMap<DrmNode, OutputDevice>, // → DeviceManager
    dmabuf_global: Option<DmabufGlobal>, // → DeviceManager
    
    // ... keep these (move to OutputManager in T1.7):
    update_output_config_on_resume: bool,
    update_ignored_nodes_on_resume: bool,
    debug_tint: bool,
    ipc_outputs: Arc<Mutex<IpcOutputMap>>,
}
```

---

## DeviceManager Struct

```rust
// src/backend/tty/devices.rs

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
```

---

## Accessor Methods to Add

```rust
impl DeviceManager {
    // === Constructor ===
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

    // === Device Access ===
    pub fn get(&self, node: &DrmNode) -> Option<&OutputDevice>
    pub fn get_mut(&mut self, node: &DrmNode) -> Option<&mut OutputDevice>
    pub fn iter(&self) -> impl Iterator<Item = (&DrmNode, &OutputDevice)>
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&DrmNode, &mut OutputDevice)>
    pub fn contains(&self, node: &DrmNode) -> bool
    pub fn insert(&mut self, node: DrmNode, device: OutputDevice)
    pub fn remove(&mut self, node: &DrmNode) -> Option<OutputDevice>

    // === Node Info ===
    pub fn primary_node(&self) -> DrmNode
    pub fn primary_render_node(&self) -> DrmNode
    pub fn is_ignored(&self, node: &DrmNode) -> bool
    pub fn ignored_nodes(&self) -> &HashSet<DrmNode>
    pub fn set_ignored_nodes(&mut self, ignored: HashSet<DrmNode>)

    // === GPU Manager ===
    pub fn gpu_manager(&self) -> &GpuManager<...>
    pub fn gpu_manager_mut(&mut self) -> &mut GpuManager<...>

    // === DmaBuf ===
    pub fn dmabuf_global(&self) -> Option<&DmabufGlobal>
    pub fn set_dmabuf_global(&mut self, global: Option<DmabufGlobal>)
    pub fn take_dmabuf_global(&mut self) -> Option<DmabufGlobal>
}
```

---

## Migration Steps

### Step 1: Create DeviceManager struct in devices.rs

Add after `OutputDevice`:

```rust
/// Device management subsystem.
pub struct DeviceManager {
    devices: HashMap<DrmNode, OutputDevice>,
    primary_node: DrmNode,
    primary_render_node: DrmNode,
    ignored_nodes: HashSet<DrmNode>,
    gpu_manager: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
    dmabuf_global: Option<DmabufGlobal>,
}

impl DeviceManager {
    pub fn new(...) -> Self { ... }
    // ... all accessor methods
}
```

### Step 2: Update Tty struct

```rust
pub struct Tty {
    config: Rc<RefCell<Config>>,
    session: LibSeatSession,
    udev_dispatcher: Dispatcher<'static, UdevBackend, State>,
    libinput: Libinput,
    
    // NEW: Subsystem
    pub(crate) devices: DeviceManager,
    
    // Keep for now (move to OutputManager in T1.7)
    update_output_config_on_resume: bool,
    update_ignored_nodes_on_resume: bool,
    debug_tint: bool,
    ipc_outputs: Arc<Mutex<IpcOutputMap>>,
}
```

### Step 3: Update Tty::new()

```rust
impl Tty {
    pub fn new(...) -> anyhow::Result<Self> {
        // ... existing setup code ...
        
        let devices = DeviceManager::new(
            primary_node,
            primary_render_node,
            ignored_nodes,
            gpu_manager,
        );
        
        Ok(Self {
            config,
            session,
            udev_dispatcher,
            libinput,
            devices,  // NEW
            update_output_config_on_resume: false,
            update_ignored_nodes_on_resume: false,
            debug_tint: false,
            ipc_outputs: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}
```

### Step 4: Update all field accesses

Search and replace in mod.rs:

| Old | New |
|-----|-----|
| `self.devices.get(` | `self.devices.get(` | (same - HashMap method) |
| `self.devices.get_mut(` | `self.devices.get_mut(` | (same) |
| `self.primary_node` | `self.devices.primary_node()` |
| `self.primary_render_node` | `self.devices.primary_render_node()` |
| `self.ignored_nodes` | `self.devices.ignored_nodes()` |
| `self.gpu_manager` | `self.devices.gpu_manager()` |
| `self.dmabuf_global` | `self.devices.dmabuf_global()` |

**Note**: The `devices` HashMap access stays similar because `DeviceManager` exposes the same methods.

---

## Verification Checklist

- [ ] `DeviceManager` struct created with all fields
- [ ] All accessor methods implemented
- [ ] `Tty` struct updated to use `DeviceManager`
- [ ] `Tty::new()` creates `DeviceManager`
- [ ] All field accesses updated
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/devices.rs` | Add `DeviceManager` struct (~150 LOC) |
| `src/backend/tty/mod.rs` | Update `Tty` to use `DeviceManager` |

---

## Next Phase

[Phase T1.4b: DeviceManager Lifecycle Methods](phase-T1.4b-device-manager-lifecycle.md)
