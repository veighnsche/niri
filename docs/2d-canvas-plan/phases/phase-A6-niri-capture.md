# Phase A6: Extract Screen Capture Modules

> **Status**: ⏳ PENDING
> **Estimated Time**: 1.5 hours
> **Risk Level**: 🟡 Medium (feature-gated, protocol-specific)
> **Prerequisite**: Phase A5 complete

---

## Goal

Extract screen capture functionality into three files:
- `src/niri/screenshot.rs` — Screenshot capture and saving
- `src/niri/screencopy.rs` — zwlr_screencopy protocol
- `src/niri/screencast.rs` — PipeWire screencasting (feature-gated)

User: Remember that when a item is too big of a refactor than planned. that I want you to make it smaller and do it in multiple steps. Write it down in this folder as broken down steps in the phase file...

---

## Work Units

### Unit 1: Create screenshot.rs (5 min)

Create `src/niri/screenshot.rs`:
```rust
//! Screenshot capture and saving for the Niri compositor.

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;
use smithay::utils::{Physical, Size, Transform};

use crate::niri::Niri;
use crate::render_helpers::RenderTarget;
use crate::utils::write_png_rgba8;
use crate::window::Mapped;
```

Update `src/niri/mod.rs`:
```rust
mod frame_callbacks;
mod hit_test;
mod lock;
mod output;
mod render;
mod screencast;
mod screencopy;
mod screenshot;
mod types;

pub use types::*;
```

---

### Unit 2: Extract screenshot (15 min)

**Source**: `niri.rs` ~lines 5658-5693

```rust
impl Niri {
    pub fn screenshot(
        &mut self,
        renderer: &mut GlesRenderer,
        output: &Output,
        write_to_disk: bool,
        include_pointer: bool,
        path: Option<String>,
    ) -> anyhow::Result<()> {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 3: Extract screenshot_window (10 min)

**Source**: `niri.rs` ~lines 5695-5735

```rust
impl Niri {
    pub fn screenshot_window(
        &self,
        renderer: &mut GlesRenderer,
        output: &Output,
        mapped: &Mapped,
        write_to_disk: bool,
        path: Option<String>,
    ) -> anyhow::Result<()> {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 4: Extract save_screenshot (15 min)

**Source**: `niri.rs` ~lines 5737-5842

```rust
impl Niri {
    pub fn save_screenshot(
        &self,
        size: Size<i32, Physical>,
        pixels: Vec<u8>,
        write_to_disk: bool,
        path_arg: Option<String>,
    ) -> anyhow::Result<()> {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 5: Extract screenshot_all_outputs (10 min)

**Source**: `niri.rs` ~lines 5844-5915 (feature-gated)

```rust
impl Niri {
    #[cfg(feature = "dbus")]
    pub fn screenshot_all_outputs(
        &mut self,
        renderer: &mut GlesRenderer,
        include_pointer: bool,
        on_done: impl FnOnce(PathBuf) + Send + 'static,
    ) -> anyhow::Result<()> {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 6: Extract capture_screenshots (10 min)

**Source**: `niri.rs` ~lines 5590-5656

```rust
impl Niri {
    pub fn capture_screenshots<'a>(
        &'a self,
        renderer: &'a mut GlesRenderer,
    ) -> impl Iterator<Item = (Output, [OutputScreenshot; 3])> + 'a {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 7: Create screencopy.rs (5 min)

Create `src/niri/screencopy.rs`:
```rust
//! zwlr_screencopy protocol implementation for the Niri compositor.

use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;

use crate::niri::Niri;
use crate::protocols::screencopy::Screencopy;
```

---

### Unit 8: Extract screencopy Methods (20 min)

**Source**: `niri.rs` ~lines 5332-5502

```rust
impl Niri {
    pub fn render_for_screencopy_with_damage(
        &mut self,
        renderer: &mut GlesRenderer,
        output: &Output,
    ) {
        // ... (copy from niri.rs)
    }

    pub fn render_for_screencopy_without_damage(
        &mut self,
        renderer: &mut GlesRenderer,
        manager: &ZwlrScreencopyManagerV1,
        screencopy: Screencopy,
    ) -> anyhow::Result<()> {
        // ... (copy from niri.rs)
    }

    fn render_for_screencopy_internal<'a>(
        renderer: &mut GlesRenderer,
        output: &Output,
        elements: &[OutputRenderElements<GlesRenderer>],
        with_damage: bool,
        damage_tracker: &'a mut OutputDamageTracker,
        screencopy: &Screencopy,
    ) -> anyhow::Result<(Option<SyncPoint>, Option<&'a Vec<Rectangle<i32, Physical>>>)> {
        // ... (copy from niri.rs)
    }

    pub fn remove_screencopy_output(&mut self, output: &Output) {
        // ... (copy from niri.rs)
    }
}
```

---

### Unit 9: Create screencast.rs (5 min)

Create `src/niri/screencast.rs`:
```rust
//! PipeWire screencasting for the Niri compositor.
//!
//! All code in this module is gated behind `#[cfg(feature = "xdp-gnome-screencast")]`.

#![cfg(feature = "xdp-gnome-screencast")]

use std::time::Duration;

use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;

use crate::niri::{CastTarget, Niri};
```

---

### Unit 10: Extract screencast Methods (15 min)

**Source**: `niri.rs` ~lines 5213-5330, 5504-5569

```rust
impl Niri {
    fn render_for_screen_cast(
        &mut self,
        renderer: &mut GlesRenderer,
        output: &Output,
        target_presentation_time: Duration,
    ) {
        // ... (copy from niri.rs)
    }

    fn render_windows_for_screen_cast(
        &mut self,
        renderer: &mut GlesRenderer,
        output: &Output,
        target_presentation_time: Duration,
    ) {
        // ... (copy from niri.rs)
    }

    fn stop_cast(&mut self, session_id: usize) {
        // ... (copy from niri.rs)
    }

    pub fn stop_casts_for_target(&mut self, target: CastTarget) {
        // ... (copy from niri.rs)
    }
}
```

Also add the non-feature-gated stub:
```rust
// In niri/screencast.rs or a separate section
#[cfg(not(feature = "xdp-gnome-screencast"))]
impl Niri {
    pub fn stop_casts_for_target(&mut self, _target: CastTarget) {}
}
```

---

### Unit 11: Remove from niri.rs (10 min)

Delete all the extracted methods from `niri.rs`.

**Verify**: `cargo check` and `cargo test`

---

## Verification Checklist

- [ ] `src/niri/screenshot.rs` exists (~400 LOC)
- [ ] `src/niri/screencopy.rs` exists (~300 LOC)
- [ ] `src/niri/screencast.rs` exists (~300 LOC)
- [ ] Screenshots work correctly
- [ ] Screencopy protocol works
- [ ] Screencasting works (if feature enabled)
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Files Created

| File | LOC | Feature Gate |
|------|-----|--------------|
| `niri/screenshot.rs` | ~400 | None |
| `niri/screencopy.rs` | ~300 | None |
| `niri/screencast.rs` | ~300 | `xdp-gnome-screencast` |

---

## Next Phase

After completing this phase, proceed to [Phase A7: Input & Rules](phase-A7-niri-input.md).
