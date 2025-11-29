# Phase A1: Extract niri/types.rs

> **Status**: 🔄 CURRENT
> **Estimated Time**: 45 minutes
> **Risk Level**: 🟢 Low (pure data types, no behavior)
> **Prerequisite**: None

---

## Goal

Create `src/niri/` module directory and extract all pure data types from `niri.rs`.

This is the foundation phase — it establishes the module structure for all subsequent phases.

---

## Work Units

### Unit 1: Create Module Structure (5 min)

Create the directory and mod.rs:

```bash
mkdir -p src/niri
```

Create `src/niri/mod.rs`:
```rust
//! Niri compositor state modules.
//!
//! This module contains the implementation of the Niri compositor,
//! split into focused submodules for maintainability.

mod types;

pub use types::*;
```

Add to `src/niri.rs` (near top, after other mod declarations):
```rust
mod niri;
pub use crate::niri::*;
```

**Verify**: `cargo check` should pass.

---

### Unit 2: Extract PointerVisibility (5 min)

**Source**: `niri.rs` lines ~427-432

```rust
// src/niri/types.rs
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PointerVisibility {
    #[default]
    Visible,
    Hidden,
    Disabled,
}

impl PointerVisibility {
    pub fn is_visible(self) -> bool {
        matches!(self, Self::Visible)
    }
}
```

**Verify**: `cargo check`

---

### Unit 3: Extract DndIcon (5 min)

**Source**: `niri.rs` lines ~434-437

```rust
// Add to src/niri/types.rs
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point};

pub struct DndIcon {
    pub surface: WlSurface,
    pub offset: Point<i32, Logical>,
}
```

---

### Unit 4: Extract CenterCoords (5 min)

**Source**: `niri.rs` lines ~439-453

```rust
// Add to src/niri/types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CenterCoords {
    /// Center each coordinate separately.
    Separately,
    /// Center both coordinates together, but only if both are requested.
    Both,
    /// Center both coordinates together, even if only one is requested.
    BothAlways,
}

impl CenterCoords {
    pub fn should_center_together(self, center_x: bool, center_y: bool) -> bool {
        match self {
            CenterCoords::Separately => false,
            CenterCoords::Both => center_x && center_y,
            CenterCoords::BothAlways => center_x || center_y,
        }
    }
}
```

---

### Unit 5: Extract RedrawState (10 min)

**Source**: `niri.rs` lines ~495-550 (approximately)

```rust
// Add to src/niri/types.rs
use std::time::Duration;
use calloop::RegistrationToken;

#[derive(Debug, Default)]
pub enum RedrawState {
    /// The compositor is idle.
    #[default]
    Idle,
    /// A redraw is queued.
    Queued,
    /// Waiting for the estimated VBlank, and a redraw is queued.
    WaitingForEstimatedVBlankAndQueued(RegistrationToken),
    /// Waiting for the VBlank.
    WaitingForVBlank {
        /// The redraw was queued after the last VBlank.
        redraw_needed: bool,
    },
    /// Waiting for the estimated VBlank.
    WaitingForEstimatedVBlank(RegistrationToken),
}

impl RedrawState {
    pub fn queue_redraw(self) -> Self {
        match self {
            RedrawState::Idle => RedrawState::Queued,
            RedrawState::WaitingForEstimatedVBlank(token) => {
                RedrawState::WaitingForEstimatedVBlankAndQueued(token)
            }
            RedrawState::WaitingForVBlank { .. } => RedrawState::WaitingForVBlank {
                redraw_needed: true,
            },
            other => other,
        }
    }
}
```

---

### Unit 6: Extract LockState and LockRenderState (10 min)

**Source**: `niri.rs` lines ~552-598 (approximately)

```rust
// Add to src/niri/types.rs
use smithay::wayland::session_lock::SessionLocker;
use smithay::reexports::wayland_protocols::ext::session_lock::v1::server::ext_session_lock_v1::ExtSessionLockV1;

#[derive(Debug, Default)]
pub enum LockState {
    #[default]
    Unlocked,
    WaitingForSurfaces {
        confirmation: SessionLocker,
        deadline_token: RegistrationToken,
    },
    Locking(SessionLocker),
    Locked(ExtSessionLockV1),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockRenderState {
    Unlocked,
    Locked,
}
```

---

### Unit 7: Extract KeyboardFocus (5 min)

**Source**: `niri.rs` lines ~600-620 (approximately)

```rust
// Add to src/niri/types.rs
use crate::window::Mapped;
use smithay::desktop::LayerSurface;

#[derive(Debug, Clone, PartialEq)]
pub enum KeyboardFocus {
    /// Focus is on a window in the layout.
    Layout {
        surface: Option<WlSurface>,
        window: Option<Mapped>,
    },
    /// Focus is on a layer-shell surface.
    LayerShell { surface: LayerSurface },
    /// Focus is on a lock screen surface.
    LockScreen { surface: WlSurface },
}
```

---

### Unit 8: Extract PointContents (5 min)

**Source**: `niri.rs` lines ~622-640 (approximately)

```rust
// Add to src/niri/types.rs
use crate::layout::monitor::HitType;
use crate::window::Window;
use smithay::output::Output;

#[derive(Debug, Default)]
pub struct PointContents {
    pub output: Option<Output>,
    pub surface: Option<(WlSurface, Point<f64, Logical>)>,
    pub window: Option<(Window, HitType)>,
    pub layer: Option<LayerSurface>,
    pub hot_corner: bool,
}
```

---

### Unit 9: Extract CastTarget (5 min)

**Source**: `niri.rs` (search for `CastTarget`)

```rust
// Add to src/niri/types.rs
use smithay::output::WeakOutput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CastTarget {
    Nothing,
    Output(WeakOutput),
    Window { id: u64 },
}
```

---

### Unit 10: Update niri.rs Imports (10 min)

Replace the type definitions in `niri.rs` with imports from the new module:

```rust
// At top of niri.rs, add:
use crate::niri::{
    CastTarget, CenterCoords, DndIcon, KeyboardFocus, LockRenderState, 
    LockState, PointContents, PointerVisibility, RedrawState,
};
```

Delete the original type definitions from `niri.rs`.

**Verify**: `cargo check` and `cargo test`

---

## Verification Checklist

- [ ] `src/niri/mod.rs` exists
- [ ] `src/niri/types.rs` exists with all types
- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] No duplicate type definitions

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/mod.rs` | Created |
| `src/niri/types.rs` | Created (~200 LOC) |
| `src/niri.rs` | Removed type definitions, added imports |

---

## Next Phase

After completing this phase, proceed to [Phase A2: niri/output.rs](phase-A2-niri-output.md).
