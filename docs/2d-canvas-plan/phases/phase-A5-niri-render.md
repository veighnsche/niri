# Phase A5: Extract niri/render.rs + frame_callbacks.rs

> **Status**: ⏳ PENDING
> **Estimated Time**: 1.5 hours
> **Risk Level**: 🟡 Medium (core rendering path)
> **Prerequisite**: Phase A4 complete

---

## Goal

Extract rendering and frame callback methods into two files:
- `src/niri/render.rs` — Core rendering logic
- `src/niri/frame_callbacks.rs` — Frame callbacks and presentation feedback

---

## Work Units

### Unit 1: Create render.rs with Imports (5 min)

Create `src/niri/render.rs`:
```rust
//! Core rendering logic for the Niri compositor.

use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;
use smithay::utils::Scale;

use crate::niri::Niri;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::RenderTarget;
```

Update `src/niri/mod.rs`:
```rust
mod frame_callbacks;
mod hit_test;
mod lock;
mod output;
mod render;
mod types;

pub use types::*;
```

---

### Unit 2: Extract pointer_element (15 min)

**Source**: `niri.rs` ~lines 3905-3991

```rust
impl Niri {
    pub fn pointer_element<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        output: &Output,
    ) -> Vec<OutputRenderElements<R>> {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 3: Extract render (20 min)

**Source**: `niri.rs` ~lines 3993-4200 (approximately)

This is the main render method — large but cohesive.

```rust
impl Niri {
    pub fn render<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        output: &Output,
        include_pointer: bool,
        target: RenderTarget,
    ) -> Vec<OutputRenderElements<R>> {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 4: Extract update_render_elements (10 min)

**Source**: `niri.rs` (search for `update_render_elements`)

```rust
impl Niri {
    pub fn update_render_elements(&mut self, output: Option<&Output>) {
        // ... (copy from niri.rs)
    }
}
```

---

### Unit 5: Extract redraw Methods (10 min)

**Source**: `niri.rs` ~lines 3877-3903

```rust
impl Niri {
    /// Schedules an immediate redraw on all outputs if one is not already scheduled.
    pub fn queue_redraw_all(&mut self) {
        for state in self.output_state.values_mut() {
            state.redraw_state = mem::take(&mut state.redraw_state).queue_redraw();
        }
    }

    /// Schedules an immediate redraw if one is not already scheduled.
    pub fn queue_redraw(&mut self, output: &Output) {
        let state = self.output_state.get_mut(output).unwrap();
        state.redraw_state = mem::take(&mut state.redraw_state).queue_redraw();
    }

    pub fn redraw_queued_outputs(&mut self, backend: &mut Backend) {
        // ... (copy from niri.rs)
    }
}
```

---

### Unit 6: Create frame_callbacks.rs (5 min)

Create `src/niri/frame_callbacks.rs`:
```rust
//! Frame callbacks and presentation feedback for the Niri compositor.

use std::time::Duration;

use smithay::backend::renderer::element::RenderElementStates;
use smithay::desktop::layer_map_for_output;
use smithay::output::Output;
use smithay::wayland::compositor::{
    send_frames_surface_tree, with_surface_tree_downward, TraversalAction,
};
use smithay::wayland::dmabuf::DmabufFeedback;

use crate::niri::Niri;
```

---

### Unit 7: Extract send_frame_callbacks (15 min)

**Source**: `niri.rs` ~lines 4991-5079

```rust
impl Niri {
    pub fn send_frame_callbacks(&mut self, output: &Output) {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 8: Extract send_frame_callbacks_on_fallback_timer (10 min)

**Source**: `niri.rs` ~lines 5081-5148

```rust
impl Niri {
    pub fn send_frame_callbacks_on_fallback_timer(&mut self) {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 9: Extract send_dmabuf_feedbacks (10 min)

**Source**: `niri.rs` ~lines 4901-4989

```rust
impl Niri {
    pub fn send_dmabuf_feedbacks(
        &self,
        output: &Output,
        feedback: &SurfaceDmabufFeedback,
        render_element_states: &RenderElementStates,
    ) {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 10: Extract update_primary_scanout_output (10 min)

**Source**: `niri.rs` ~lines 4750-4899

```rust
impl Niri {
    pub fn update_primary_scanout_output(
        &self,
        output: &Output,
        render_element_states: &RenderElementStates,
    ) {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 11: Extract take_presentation_feedbacks (10 min)

**Source**: `niri.rs` ~lines 5150-5211

```rust
impl Niri {
    pub fn take_presentation_feedbacks(
        &mut self,
        output: &Output,
        render_element_states: &RenderElementStates,
    ) -> OutputPresentationFeedback {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 12: Remove from niri.rs (10 min)

Delete all the extracted methods from `niri.rs`.

**Verify**: `cargo check` and `cargo test`

---

## Verification Checklist

- [ ] `src/niri/render.rs` exists (~400 LOC)
- [ ] `src/niri/frame_callbacks.rs` exists (~350 LOC)
- [ ] Rendering works correctly
- [ ] Frame callbacks fire properly
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Files Created

| File | LOC | Description |
|------|-----|-------------|
| `niri/render.rs` | ~400 | Core rendering |
| `niri/frame_callbacks.rs` | ~350 | Frame callbacks & feedback |

---

## Next Phase

After completing this phase, proceed to [Phase A6: Screen Capture](phase-A6-niri-capture.md).
