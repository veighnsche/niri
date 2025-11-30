# Phase T1.3: Extract Pure Helper Functions

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟢 Low  
> **Architectural Benefit**: ⭐⭐⭐ High - testable pure functions

---

## Goal

Extract all pure helper functions into `src/backend/tty/helpers.rs`. 

**Key principle**: These functions have **no `&self`**, **no Tty/Niri state access**, and are **pure computations**. This matches the pattern in `src/input/helpers.rs`.

---

## What Makes a Function "Pure"?

| Pure ✅ | Impure ❌ |
|---------|----------|
| `fn refresh_interval(mode: DrmMode) -> Duration` | `fn render(&mut self, ...)` |
| `fn pick_mode(connector: &Info, ...) -> Mode` | `fn device_added(&mut self, ...)` |
| Takes inputs, returns outputs | Mutates state |
| No `&self` or `&mut self` | Has `&self` or `&mut self` |

---

## Functions to Extract (~400 LOC total)

### DRM Node Discovery
```rust
pub(super) fn primary_node_from_render_node(path: &Path) -> Option<(DrmNode, DrmNode)>
pub(super) fn primary_node_from_config(config: &Config) -> Option<(DrmNode, DrmNode)>
pub(super) fn ignored_nodes_from_config(config: &Config) -> HashSet<DrmNode>
```

### DRM Property Helpers
```rust
pub(super) fn find_drm_property(drm: &DrmDevice, resource: impl ResourceHandle, name: &str) 
    -> Option<(property::Handle, property::Info, property::RawValue)>
pub(super) fn get_drm_property(drm: &DrmDevice, resource: impl ResourceHandle, prop: property::Handle) 
    -> Option<property::RawValue>
```

### Mode Calculations
```rust
pub(super) fn refresh_interval(mode: DrmMode) -> Duration
pub fn calculate_drm_mode_from_modeline(modeline: &Modeline) -> anyhow::Result<DrmMode>
pub fn calculate_mode_cvt(width: u16, height: u16, refresh: f64) -> DrmMode
pub(super) fn pick_mode(connector: &connector::Info, config: Option<&ConfigMode>) -> Option<(DrmMode, bool)>
```

### Connector/EDID Helpers
```rust
pub(super) fn get_edid_info(edid_blob: &[u8]) -> (Option<String>, Option<String>, Option<String>)
pub(super) fn format_connector_name(connector: &connector::Info) -> String
pub(super) fn make_output_name(...) -> OutputName
```

### HDR/VRR/Gamma
```rust
pub(super) fn reset_hdr(props: &ConnectorProperties) -> anyhow::Result<()>
pub(super) fn is_vrr_capable(device: &DrmDevice, connector: connector::Handle) -> Option<bool>
pub(super) fn get_panel_orientation(props: &ConnectorProperties) -> anyhow::Result<Transform>
pub fn set_gamma_for_crtc(device: &DrmDevice, crtc: crtc::Handle, ramp: Option<&[u16]>) -> anyhow::Result<()>
```

### DmaBuf Feedback
```rust
pub(super) fn surface_dmabuf_feedback(
    compositor: &GbmDrmCompositor,
    primary_formats: FormatSet,
    primary_render_node: DrmNode,
    surface_render_node: Option<DrmNode>,
    surface_scanout_node: DrmNode,
) -> Result<SurfaceDmabufFeedback, io::Error>
```

---

## Why This is Excellent Architecture

1. **Pure functions** - No state mutation, predictable outputs
2. **Easily testable** - Unit test mode calculations, property lookups
3. **Reusable** - Could be used by other backends
4. **Clear contracts** - Input → Output, no side effects
5. **Matches existing pattern** - Same as `src/input/helpers.rs`

---

## Tests to Move

Move existing tests with the helpers:
- `test_calculate_drmmode_from_modeline`
- `test_calc_cvt`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_drmmode_from_modeline() { ... }

    #[test]
    fn test_calc_cvt() { ... }
}
```

---

## Verification Checklist

- [ ] All helper functions work correctly
- [ ] `cargo check` passes
- [ ] Existing tests pass
- [ ] **No Tty or State imports in helpers.rs**
- [ ] All functions are pure (no `&self`)

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/helpers.rs` | Created (~400 LOC) |
| `src/backend/tty/mod.rs` | Added `mod helpers`, removed inline helpers |

---

## Next Phase

[Phase T1.4: Create DeviceManager Subsystem](phase-T1.4-extract-lifecycle.md)
