# Phase P1: Move OutputState to types.rs

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~20 minutes  
> **Risk Level**: 🟢 Low (pure data movement)  
> **Prerequisite**: None  
> **Unblocks**: Phase P2 (output management extraction)

---

## Goal

Move `OutputState` struct from `mod.rs` to `types.rs` to unblock output management extraction.

Currently `OutputState` is defined in `mod.rs` which prevents moving output functions.

---

## Current Location

**File**: `src/niri/mod.rs` lines ~449-501 (~52 lines)

```rust
pub struct OutputState {
    pub global: GlobalId,
    pub redraw_state: RedrawState,
    pub on_demand_vrr_enabled: bool,
    pub unfinished_animations_remain: bool,
    pub frame_clock: FrameClock,
    pub last_drm_sequence: Option<(u32, Duration)>,
    pub vblank_throttle: VBlankThrottle,
    pub frame_callback_sequence: u32,
    pub backdrop_buffer: SolidColorBuffer,
    pub lock_render_state: LockRenderState,
    pub lock_surface: Option<LockSurface>,
    pub lock_color_buffer: SolidColorBuffer,
    pub screen_transition: Option<ScreenTransition>,
    pub debug_damage_tracker: OutputDamageTracker,
}
```

Also move the constant:
```rust
const CLEAR_COLOR_LOCKED: [f32; 4] = [0.3, 0.3, 0.3, 1.];
```

---

## Work Units

### Unit 1: Add Imports to types.rs

Add necessary imports at the top of `src/niri/types.rs`:

```rust
use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::reexports::wayland_server::backend::GlobalId;
use smithay::wayland::session_lock::LockSurface;

use crate::frame_clock::FrameClock;
use crate::render_helpers::solid_color::SolidColorBuffer;
use crate::ui::screen_transition::ScreenTransition;
use crate::utils::vblank_throttle::VBlankThrottle;
```

**Verify**: `cargo check` (expect errors - OutputState not yet moved)

---

### Unit 2: Move OutputState Struct

Cut the `OutputState` struct from `mod.rs` and paste into `types.rs`.

Add the constant above the struct:
```rust
/// Background color when session is locked.
pub const CLEAR_COLOR_LOCKED: [f32; 4] = [0.3, 0.3, 0.3, 1.];

/// Per-output compositor state.
pub struct OutputState {
    pub global: GlobalId,
    pub redraw_state: RedrawState,
    pub on_demand_vrr_enabled: bool,
    pub unfinished_animations_remain: bool,
    pub frame_clock: FrameClock,
    pub last_drm_sequence: Option<(u32, Duration)>,
    pub vblank_throttle: VBlankThrottle,
    pub frame_callback_sequence: u32,
    pub backdrop_buffer: SolidColorBuffer,
    pub lock_render_state: LockRenderState,
    pub lock_surface: Option<LockSurface>,
    pub lock_color_buffer: SolidColorBuffer,
    pub screen_transition: Option<ScreenTransition>,
    pub debug_damage_tracker: OutputDamageTracker,
}
```

**Verify**: `cargo check`

---

### Unit 3: Update mod.rs Imports

In `mod.rs`, the type is already re-exported via `pub use types::*;`, so no import changes needed.

Remove the now-duplicate definition from `mod.rs`.

**Verify**: `cargo check` && `cargo test`

---

### Unit 4: Verify No Breaking Changes

```bash
cargo test
```

All 270 tests should pass.

---

## Verification Checklist

- [ ] `OutputState` exists in `types.rs`
- [ ] `CLEAR_COLOR_LOCKED` exists in `types.rs`  
- [ ] No duplicate definitions in `mod.rs`
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)
- [ ] No new warnings

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/types.rs` | +60 lines (struct + imports + constant) |
| `src/niri/mod.rs` | -55 lines |

---

## Next Phase

After completing this phase, proceed to [Phase P2: Output Management](phase-P2-output-mgmt.md).

The `OutputState` type being in `types.rs` means `output.rs` can now import it and receive the output management functions.
