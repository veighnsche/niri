# Phase P2: Move Output Management to output.rs

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟡 Medium (complex dependencies)  
> **Prerequisite**: Phase P1 complete  
> **Depends on**: `OutputState` in types.rs

---

## Goal

Move the three major output management functions from `mod.rs` to `output.rs`:
- `reposition_outputs()` (~120 lines)
- `add_output()` (~95 lines)
- `remove_output()` (~85 lines)

Total: ~300 lines moved from mod.rs

---

## Functions to Move

### 1. reposition_outputs (mod.rs ~lines 2316-2437)

```rust
pub fn reposition_outputs(&mut self, new_output: Option<&Output>) {
    // ~120 lines
    // Uses: config, global_space, sorted_outputs, ipc_outputs_changed, queue_redraw
}
```

### 2. add_output (mod.rs ~lines 2439-2534)

```rust
pub fn add_output(&mut self, output: Output, refresh_interval: Option<Duration>, vrr: bool) {
    // ~95 lines  
    // Uses: display_handle, config, layout, output_state, reposition_outputs
}
```

### 3. remove_output (mod.rs ~lines 2535-2618)

```rust
pub fn remove_output(&mut self, output: &Output) {
    // ~85 lines
    // Uses: layer_map_for_output, layout, global_space, gamma_control, output_state,
    //       event_loop, lock_state, screenshot_ui, window_mru_ui, cancel_mru
}
```

---

## Work Units

### Unit 1: Add Required Imports to output.rs

Add imports at the top of `src/niri/output.rs`:

```rust
use std::mem;
use std::time::Duration;

use smithay::desktop::layer_map_for_output;
use smithay::output::{Output, Scale as OutputScale};
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay::utils::{Logical, Point, Rectangle, Size, Transform};

use crate::backend::Backend;
use crate::frame_clock::FrameClock;
use crate::handlers::configure_lock_surface;
use crate::render_helpers::solid_color::SolidColorBuffer;
use crate::utils::scale::{closest_representable_scale, guess_monitor_scale};
use crate::utils::{ipc_transform_to_smithay, output_matches_name, output_size, panel_orientation};
use crate::utils::vblank_throttle::VBlankThrottle;

use super::{
    CastTarget, LockRenderState, LockState, Niri, OutputState, RedrawState, State,
    CLEAR_COLOR_LOCKED,
};
```

**Verify**: `cargo check` (some imports may error until functions are moved)

---

### Unit 2: Move reposition_outputs

Cut from `mod.rs` and paste into `output.rs` inside an `impl Niri` block:

```rust
// =============================================================================
// Output Management Methods
// =============================================================================

impl Niri {
    /// Repositions all outputs, optionally adding a new output.
    pub fn reposition_outputs(&mut self, new_output: Option<&Output>) {
        // ... paste full implementation
    }
}
```

**Verify**: `cargo check`

---

### Unit 3: Move add_output

Add to the same `impl Niri` block:

```rust
    pub fn add_output(&mut self, output: Output, refresh_interval: Option<Duration>, vrr: bool) {
        // ... paste full implementation
    }
```

**Note**: This function uses `State` for `create_global`. May need to change signature or keep a thin wrapper in mod.rs.

**Verify**: `cargo check`

---

### Unit 4: Move remove_output

Add to the same `impl Niri` block:

```rust
    pub fn remove_output(&mut self, output: &Output) {
        // ... paste full implementation
    }
```

**Verify**: `cargo check` && `cargo test`

---

### Unit 5: Handle State References

If any function needs `State` (not just `Niri`), you have two options:

**Option A**: Change signature to take additional parameters
```rust
pub fn add_output(&mut self, display_handle: &DisplayHandle, ...)
```

**Option B**: Keep a thin wrapper in mod.rs
```rust
// In mod.rs
pub fn add_output_with_state(&mut self, ...) {
    // Call the Niri method
}
```

---

## Verification Checklist

- [ ] `reposition_outputs` in output.rs
- [ ] `add_output` in output.rs  
- [ ] `remove_output` in output.rs
- [ ] No duplicate definitions in mod.rs
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/output.rs` | +300 lines |
| `src/niri/mod.rs` | -300 lines |

---

## Dependencies

These functions use many types and call many other methods:

**Types**: `OutputState`, `OutputName`, `FrameClock`, `VBlankThrottle`, `SolidColorBuffer`

**Methods called**:
- `layout.add_output()`, `layout.remove_output()`
- `gamma_control_manager_state.output_removed()`
- `queue_redraw()`, `queue_redraw_all()`
- `reposition_outputs()` (called by others)
- `cancel_mru()`

---

## Next Phase

After completing this phase, proceed to [Phase P3: Render Types](phase-P3-render-types.md).
