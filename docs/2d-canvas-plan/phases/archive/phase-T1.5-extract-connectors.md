# Phase T1.5: Integrate Connectors into DeviceManager

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟡 Medium  
> **Architectural Benefit**: ⭐⭐ Medium - unified device lifecycle

---

## Goal

Add connector connect/disconnect handling to `DeviceManager`. These belong with device lifecycle since they're called during device scanning.

---

## Methods to Add to DeviceManager

```rust
impl DeviceManager {
    // === Connector Lifecycle ===
    
    /// Handle a connector becoming connected (monitor plugged in).
    pub fn connector_connected(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
        config: &Config,
    ) -> anyhow::Result<()>

    /// Handle a connector becoming disconnected (monitor unplugged).
    pub fn connector_disconnected(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        crtc: crtc::Handle,
    )
}
```

---

## Implementation

```rust
impl DeviceManager {
    pub fn connector_connected(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
        config: &Config,
    ) -> anyhow::Result<()> {
        let connector_name = helpers::format_connector_name(&connector);
        tracing::debug!("connecting connector: {connector_name}");

        let device = self.devices.get_mut(&node).context("missing device")?;

        // 1. Get output name
        let output_name = device.known_crtc_name(crtc, &connector, config);

        // 2. Check for non-desktop (VR headsets)
        let non_desktop = helpers::find_drm_property(device.drm(), connector.handle(), "non-desktop")
            .and_then(|(_, info, value)| info.value_type().convert_value(value).as_boolean())
            .unwrap_or(false);

        if non_desktop {
            // Handle VR connector
            if let Some(lease_state) = device.lease_state_mut() {
                lease_state.add_connector::<State>(...);
            }
            device.add_non_desktop(connector.handle(), crtc);
            return Ok(());
        }

        // 3. Pick display mode
        let mode = helpers::pick_mode(&connector, config.mode)?;

        // 4. Create DRM surface
        let surface = device.drm().create_surface(crtc, mode, &[connector.handle()])?;

        // 5. Configure VRR
        // ... 

        // 6. Create smithay Output
        let output = Output::new(...);

        // 7. Build dmabuf feedback
        let feedback = helpers::surface_dmabuf_feedback(...)?;

        // 8. Store surface
        device.insert_surface(crtc, Surface { ... });

        // 9. Notify niri
        niri.add_output(output, ...);

        Ok(())
    }

    pub fn connector_disconnected(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        crtc: crtc::Handle,
    ) {
        let Some(device) = self.devices.get_mut(&node) else {
            return;
        };

        let Some(surface) = device.remove_surface(crtc) else {
            return;
        };

        device.remove_known_crtc(crtc);
        niri.remove_output(&surface.output);
    }
}
```

---

## Why Connectors Belong in DeviceManager

1. **Called during device_changed** - Part of the device scan loop
2. **Needs device access** - `device.drm()`, `device.surfaces`, etc.
3. **Single responsibility** - Device lifecycle includes monitors
4. **Avoids cross-module calls** - Everything in one subsystem

---

## Verification Checklist

- [ ] Monitor hotplug works (unplug/replug)
- [ ] VR headset lease works
- [ ] Correct mode is selected
- [ ] VRR is configured when available
- [ ] `cargo check` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/devices.rs` | Add connector methods (~400 LOC) |

---

## Next Phase

[Phase T1.6: Create RenderManager Subsystem](phase-T1.6-extract-render.md)
