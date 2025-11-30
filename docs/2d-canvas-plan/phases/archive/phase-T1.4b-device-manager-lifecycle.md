# Phase T1.4b: DeviceManager Lifecycle Methods

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1 hour  
> **Risk Level**: 🟡 Medium  
> **Architectural Benefit**: ⭐⭐⭐ High - encapsulates device lifecycle

---

## Goal

Move the device lifecycle methods (`device_added`, `device_changed`, `device_removed`) into `DeviceManager`. These are the core methods that handle GPU hotplug.

**Prerequisite**: Phase T1.4a must be complete.

---

## Methods to Move

| Method | Lines | Complexity |
|--------|-------|------------|
| `device_added` | ~195 | High - creates DRM device |
| `device_changed` | ~164 | Medium - scans connectors |
| `device_removed` | ~109 | Medium - cleanup |
| **Total** | **~468** | |

---

## Method Signatures

```rust
impl DeviceManager {
    /// Add a new DRM device (GPU hotplug).
    pub fn device_added(
        &mut self,
        device_id: dev_t,
        path: &Path,
        session: &LibSeatSession,
        event_loop: &LoopHandle<State>,
        config: &Config,
        niri: &mut Niri,
    ) -> anyhow::Result<()>

    /// Handle device change (connector scan).
    pub fn device_changed(
        &mut self,
        device_id: dev_t,
        niri: &mut Niri,
        config: &Config,
        cleanup: bool,
    )

    /// Remove a DRM device (GPU unplug).
    pub fn device_removed(
        &mut self,
        device_id: dev_t,
        niri: &mut Niri,
        event_loop: &LoopHandle<State>,
        session: &LibSeatSession,
    )
}
```

---

## Migration Steps

### Step 1: Move `device_added` (~195 lines)

**Location in mod.rs**: Lines 643-837

```rust
// BEFORE (in impl Tty)
fn device_added(
    &mut self,
    device_id: dev_t,
    path: &Path,
    niri: &mut Niri,
) -> anyhow::Result<()> {
    // ... 195 lines
}

// AFTER (in impl DeviceManager)
pub fn device_added(
    &mut self,
    device_id: dev_t,
    path: &Path,
    session: &LibSeatSession,
    event_loop: &LoopHandle<State>,
    config: &Config,
    niri: &mut Niri,
) -> anyhow::Result<()> {
    // Same implementation, but:
    // - self.session → session parameter
    // - self.config.borrow() → config parameter
    // - self.devices.insert() → self.insert()
    // - self.gpu_manager → self.gpu_manager()
}
```

**Key changes inside the function**:
- `self.session.open(...)` → `session.open(...)`
- `self.config.borrow()` → `config`
- `self.devices.insert(node, device)` → `self.insert(node, device)`
- `self.gpu_manager.as_mut()` → `self.gpu_manager_mut()`
- `self.primary_node` → `self.primary_node()`
- `self.primary_render_node` → `self.primary_render_node()`

### Step 2: Move `device_changed` (~164 lines)

**Location in mod.rs**: Lines 838-1001

```rust
// BEFORE (in impl Tty)
fn device_changed(&mut self, device_id: dev_t, niri: &mut Niri, cleanup: bool) {
    // ... 164 lines
}

// AFTER (in impl DeviceManager)
pub fn device_changed(
    &mut self,
    device_id: dev_t,
    niri: &mut Niri,
    config: &Config,
    cleanup: bool,
) {
    // Same implementation with parameter changes
}
```

**Note**: This method calls `connector_connected` and `connector_disconnected` which will be moved in T1.5. For now, these will need to be called via a callback or remain as separate methods.

### Step 3: Move `device_removed` (~109 lines)

**Location in mod.rs**: Lines 1002-1110

```rust
// BEFORE (in impl Tty)
fn device_removed(&mut self, device_id: dev_t, niri: &mut Niri) {
    // ... 109 lines
}

// AFTER (in impl DeviceManager)
pub fn device_removed(
    &mut self,
    device_id: dev_t,
    niri: &mut Niri,
    event_loop: &LoopHandle<State>,
    session: &LibSeatSession,
) {
    // Same implementation with parameter changes
}
```

### Step 4: Update Tty to delegate

```rust
impl Tty {
    fn on_udev_event(&mut self, niri: &mut Niri, event: UdevEvent) {
        match event {
            UdevEvent::Added { device_id, path } => {
                if !self.session.is_active() {
                    debug!("skipping UdevEvent::Added as session is inactive");
                    return;
                }

                if let Err(err) = self.devices.device_added(
                    device_id,
                    &path,
                    &self.session,
                    &niri.event_loop,
                    &self.config.borrow(),
                    niri,
                ) {
                    warn!("error adding device: {err:?}");
                }
            }
            UdevEvent::Changed { device_id } => {
                if !self.session.is_active() {
                    debug!("skipping UdevEvent::Changed as session is inactive");
                    return;
                }

                self.devices.device_changed(
                    device_id,
                    niri,
                    &self.config.borrow(),
                    false,
                );
            }
            UdevEvent::Removed { device_id } => {
                if !self.session.is_active() {
                    debug!("skipping UdevEvent::Removed as session is inactive");
                    return;
                }

                self.devices.device_removed(
                    device_id,
                    niri,
                    &niri.event_loop,
                    &self.session,
                );
            }
        }
    }
}
```

---

## Handling Connector Callbacks

`device_changed` calls `connector_connected` and `connector_disconnected`. Since those move in T1.5, we have two options:

### Option A: Temporary trait/callback (Complex)
Pass a callback for connector handling.

### Option B: Keep connector methods in Tty for now (Recommended)
Leave `connector_connected`/`connector_disconnected` in `Tty` until T1.5. `device_changed` can call them via a mutable reference pattern.

```rust
// In DeviceManager
pub fn device_changed(
    &mut self,
    device_id: dev_t,
    niri: &mut Niri,
    config: &Config,
    cleanup: bool,
    // Callbacks for connector handling (temporary until T1.5)
    on_connect: impl FnMut(&mut Niri, DrmNode, connector::Info, crtc::Handle),
    on_disconnect: impl FnMut(&mut Niri, DrmNode, crtc::Handle),
) {
    // ... scan connectors ...
    // Call on_connect/on_disconnect as needed
}
```

**Or simpler**: Return a list of events and let Tty handle them:

```rust
pub enum ConnectorEvent {
    Connected { node: DrmNode, connector: connector::Info, crtc: crtc::Handle },
    Disconnected { node: DrmNode, crtc: crtc::Handle },
}

pub fn device_changed(...) -> Vec<ConnectorEvent> {
    // Return events instead of calling methods directly
}
```

---

## Verification Checklist

- [ ] `device_added` moved to `DeviceManager`
- [ ] `device_changed` moved to `DeviceManager`
- [ ] `device_removed` moved to `DeviceManager`
- [ ] `Tty::on_udev_event` delegates to `DeviceManager`
- [ ] GPU hotplug works (if you have a way to test)
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Testing

Since GPU hotplug is hard to test automatically:

1. **Compile check**: `cargo check`
2. **Run tests**: `cargo test`
3. **Manual test** (if possible): 
   - Start niri
   - Plug/unplug a monitor
   - Verify it's detected

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/devices.rs` | Add lifecycle methods (~470 LOC) |
| `src/backend/tty/mod.rs` | `Tty` delegates to `DeviceManager` |

---

## Next Phase

[Phase T1.5: Integrate Connectors into DeviceManager](phase-T1.5-extract-connectors.md)
