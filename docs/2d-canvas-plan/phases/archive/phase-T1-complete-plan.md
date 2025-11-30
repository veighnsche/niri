# Complete TTY Refactor Plan: T1.4b, T1.5, T1.6, T1.7

> **Created by**: TEAM_089  
> **Date**: Nov 30, 2025  
> **Goal**: Complete the TTY backend refactor by moving all remaining methods from mod.rs to subsystems

---

## Current State

| File | Current LOC | Target LOC |
|------|-------------|------------|
| `mod.rs` | 2594 | ~600 |
| `devices.rs` | 573 | ~1200 |
| `render.rs` | 71 | ~400 |
| `outputs.rs` | 87 | ~350 |

---

## Phase T1.4b: Device Lifecycle Methods

**Move ~470 LOC from mod.rs to devices.rs**

### Methods to Move

| Method | Lines in mod.rs | LOC |
|--------|-----------------|-----|
| `device_added()` | 539-734 | ~196 |
| `device_changed()` | 737-899 | ~163 |
| `device_removed()` | 901-1008 | ~108 |

### New Signatures in DeviceManager

```rust
impl DeviceManager {
    pub fn device_added(
        &mut self,
        device_id: dev_t,
        path: &Path,
        session: &LibSeatSession,
        event_loop: &LoopHandle<State>,
        config: &Rc<RefCell<Config>>,
        niri: &mut Niri,
        render: &RenderManager,  // for debug_tint
    ) -> anyhow::Result<()>

    pub fn device_changed(
        &mut self,
        device_id: dev_t,
        niri: &mut Niri,
        config: &Rc<RefCell<Config>>,
        cleanup: bool,
        // Callbacks for connector handling
        on_connect: impl FnMut(&mut Niri, DrmNode, connector::Info, crtc::Handle) -> anyhow::Result<()>,
        on_output_config_changed: impl FnMut(&mut Niri),
        should_disable_laptop_panels: impl Fn() -> bool,
    )

    pub fn device_removed(
        &mut self,
        device_id: dev_t,
        niri: &mut Niri,
        event_loop: &LoopHandle<State>,
        session: &LibSeatSession,
        on_refresh_ipc: impl FnMut(&mut Niri),
    )
}
```

### Key Transformations

| Old (in Tty) | New (in DeviceManager) |
|--------------|------------------------|
| `self.session` | `session` parameter |
| `self.config.borrow()` | `config.borrow()` parameter |
| `self.devices.xxx()` | `self.xxx()` |
| `self.render.debug_tint()` | `render.debug_tint()` parameter |
| `niri.event_loop` | `event_loop` parameter |

### Delegation in Tty

```rust
impl Tty {
    fn device_added(&mut self, device_id: dev_t, path: &Path, niri: &mut Niri) -> anyhow::Result<()> {
        self.devices.device_added(
            device_id,
            path,
            &self.session,
            &niri.event_loop,
            &self.config,
            niri,
            &self.render,
        )
    }
}
```

---

## Phase T1.5: Connector Connected Method

**Move ~330 LOC from mod.rs to devices.rs**

### Method to Move

| Method | Lines in mod.rs | LOC |
|--------|-----------------|-----|
| `connector_connected()` | 1011-1357 | ~347 |

Note: `connector_disconnected()` is already in devices.rs (done previously).

### New Signature

```rust
impl DeviceManager {
    pub fn connector_connected(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
        config: &Rc<RefCell<Config>>,
        render: &RenderManager,  // for debug_tint
    ) -> anyhow::Result<()>
}
```

### Key Transformations

| Old (in Tty) | New (in DeviceManager) |
|--------------|------------------------|
| `self.config.borrow()` | `config.borrow()` |
| `self.render.debug_tint()` | `render.debug_tint()` |
| `self.devices.xxx()` | `self.xxx()` |

---

## Phase T1.6: RenderManager Methods

**Move ~400 LOC from mod.rs to render.rs**

### Methods to Move

| Method | Lines in mod.rs | LOC |
|--------|-----------------|-----|
| `render()` | 1598-1778 | ~181 |
| `on_vblank()` | 1359-1546 | ~188 |
| `on_estimated_vblank_timer()` | 1548-1579 | ~32 |

### New Signatures

```rust
impl RenderManager {
    pub fn render(
        &self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        output: &Output,
        target_presentation_time: Duration,
        config: &Rc<RefCell<Config>>,
    ) -> RenderResult

    pub fn on_vblank(
        &self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        node: DrmNode,
        crtc: crtc::Handle,
        meta: DrmEventMetadata,
        config: &Rc<RefCell<Config>>,
    )

    pub fn on_estimated_vblank_timer(
        &self,
        niri: &mut Niri,
        output: Output,
    )
}
```

### Delegation in Tty

```rust
impl Tty {
    pub fn render(&mut self, niri: &mut Niri, output: &Output, target: Duration) -> RenderResult {
        self.render.render(&mut self.devices, niri, output, target, &self.config)
    }
}
```

---

## Phase T1.7: OutputManager Methods

**Move ~350 LOC from mod.rs to outputs.rs**

### Methods to Move

| Method | Lines in mod.rs | LOC |
|--------|-----------------|-----|
| `refresh_ipc_outputs()` | 1875-1984 | ~110 |
| `get_gamma_size()` | 1830-1849 | ~20 |
| `set_gamma()` | 1851-1873 | ~23 |
| `set_monitors_active()` | 2003-2020 | ~18 |
| `set_output_on_demand_vrr()` | 2022-2054 | ~33 |
| `on_output_config_changed()` | 2129-2320 | ~192 |

### New Signatures

```rust
impl OutputManager {
    pub fn refresh_ipc_outputs(
        &self,
        devices: &DeviceManager,
        niri: &mut Niri,
        config: &Rc<RefCell<Config>>,
    )

    pub fn get_gamma_size(
        &self,
        devices: &DeviceManager,
        output: &Output,
    ) -> anyhow::Result<u32>

    pub fn set_gamma(
        &mut self,
        devices: &mut DeviceManager,
        output: &Output,
        ramp: Option<Vec<u16>>,
        session: &LibSeatSession,
    ) -> anyhow::Result<()>

    pub fn set_monitors_active(
        &self,
        devices: &mut DeviceManager,
        active: bool,
    )

    pub fn set_output_on_demand_vrr(
        &mut self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        output: &Output,
        enable_vrr: bool,
        config: &Rc<RefCell<Config>>,
    )

    pub fn on_output_config_changed(
        &mut self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        config: &Rc<RefCell<Config>>,
        session: &LibSeatSession,
        render: &RenderManager,
        // Callbacks
        should_disable_laptop_panels: impl Fn(bool) -> bool,
        on_connector_connected: impl FnMut(...) -> anyhow::Result<()>,
    )
}
```

---

## Execution Order

### Step 1: T1.4b — Device Lifecycle (~1.5h)

1. Add imports to devices.rs
2. Move `device_added()` with signature changes
3. Move `device_changed()` with signature changes  
4. Move `device_removed()` with signature changes
5. Update Tty to delegate
6. `cargo check` + `cargo test`

### Step 2: T1.5 — Connector Connected (~1h)

1. Move `connector_connected()` to devices.rs
2. Update Tty to delegate
3. `cargo check` + `cargo test`

### Step 3: T1.6 — RenderManager (~1h)

1. Add imports to render.rs
2. Move `render()` method
3. Move `on_vblank()` method
4. Move `on_estimated_vblank_timer()` method
5. Move `queue_estimated_vblank_timer()` helper
6. Update Tty to delegate
7. `cargo check` + `cargo test`

### Step 4: T1.7 — OutputManager (~1h)

1. Add imports to outputs.rs
2. Move `refresh_ipc_outputs()` method
3. Move gamma methods
4. Move `set_monitors_active()` method
5. Move `set_output_on_demand_vrr()` method
6. Move `on_output_config_changed()` method
7. Update Tty to delegate
8. `cargo check` + `cargo test`

### Step 5: Final Cleanup (~30m)

1. Remove dead code from mod.rs
2. Update doc comments
3. Final `cargo check` + `cargo test`
4. Update team file

---

## Risk Mitigation

### Borrow Checker Issues

The main challenge is that methods access multiple `self` fields. Solutions:

1. **Pass parameters explicitly** — Most reliable
2. **Split borrows** — Use `gpu_manager_and_devices_mut()` pattern
3. **Callbacks** — For cross-subsystem calls

### Testing Strategy

After each method move:
1. `cargo check` — Must pass
2. `cargo test` — Must pass
3. Manual test if touching render/vblank (optional)

---

## Expected Final State

```
src/backend/tty/
├── mod.rs          # ~600 LOC - Tty coordinator + remaining helpers
├── types.rs        # ~150 LOC - Type definitions (unchanged)
├── helpers.rs      # ~600 LOC - Pure functions (unchanged)
├── devices.rs      # ~1200 LOC - DeviceManager + OutputDevice + lifecycle
├── render.rs       # ~400 LOC - RenderManager + render/vblank
└── outputs.rs      # ~350 LOC - OutputManager + IPC/gamma/VRR
```

**Total reduction**: mod.rs from 2594 → ~600 LOC (-77%)

---

## Verification Checklist

- [ ] T1.4b: `device_added`, `device_changed`, `device_removed` in DeviceManager
- [ ] T1.5: `connector_connected` in DeviceManager
- [ ] T1.6: `render`, `on_vblank`, `on_estimated_vblank_timer` in RenderManager
- [ ] T1.7: IPC/gamma/VRR/config methods in OutputManager
- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] mod.rs < 700 LOC
