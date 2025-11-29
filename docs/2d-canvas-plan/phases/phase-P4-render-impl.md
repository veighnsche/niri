# Phase P4: Move Render Functions to render.rs

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1 hour  
> **Risk Level**: 🟡 Medium (many dependencies)  
> **Prerequisite**: Phase P3 complete (OutputRenderElements in render.rs)

---

## Goal

Move the core rendering functions from `mod.rs` to `render.rs`:
- `render()` (~250 lines)
- `render_layer()` (~25 lines)
- `redraw()` (~130 lines)
- `queue_redraw_all()` (~5 lines)
- `queue_redraw()` (~5 lines)
- `redraw_queued_outputs()` (~15 lines)

Total: ~430 lines moved from mod.rs

---

## Functions to Move

### 1. render (mod.rs ~lines 2682-2930)

The main render function - returns `Vec<OutputRenderElements<R>>`.

```rust
pub fn render<R: NiriRenderer>(
    &self,
    renderer: &mut R,
    output: &Output,
    include_pointer: bool,
    mut target: RenderTarget,
) -> Vec<OutputRenderElements<R>> {
    // ~250 lines
}
```

### 2. render_layer (mod.rs ~lines 2931-2955)

Helper for rendering layer-shell surfaces.

```rust
fn render_layer<R: NiriRenderer>(
    &self,
    renderer: &mut R,
    target: RenderTarget,
    layer_map: &LayerMap,
    layer: Layer,
    elements: &mut SplitElements<LayerSurfaceRenderElement<R>>,
    for_backdrop: bool,
) {
    // ~25 lines
}
```

### 3. redraw (mod.rs ~lines 2956-3084)

Per-output redraw orchestration.

```rust
fn redraw(&mut self, backend: &mut Backend, output: &Output) {
    // ~130 lines
    // Uses: output_state, clock, layout, config_error_notification,
    //       exit_confirm_dialog, screenshot_ui, window_mru_ui, cursor_manager,
    //       mapped_layer_surfaces, monitors_active, lock_state
}
```

### 4. queue_redraw* functions (mod.rs ~lines 2650-2675)

```rust
pub fn queue_redraw_all(&mut self) { /* ~5 lines */ }
pub fn queue_redraw(&mut self, output: &Output) { /* ~5 lines */ }
pub fn redraw_queued_outputs(&mut self, backend: &mut Backend) { /* ~15 lines */ }
```

---

## Work Units

### Unit 1: Add Required Imports

Add to `render.rs`:

```rust
use std::mem;

use smithay::backend::renderer::element::surface::render_elements_from_surface_tree;
use smithay::desktop::{layer_map_for_output, LayerMap};
use smithay::wayland::shell::wlr_layer::Layer;

use crate::backend::{Backend, RenderResult};
use crate::render_helpers::debug::draw_opaque_regions;
use crate::render_helpers::{RenderTarget, SplitElements};

use super::{LockRenderState, LockState, Niri, RedrawState};
```

---

### Unit 2: Move queue_redraw Functions

These are simple and have minimal dependencies:

```rust
impl Niri {
    /// Schedules an immediate redraw on all outputs.
    pub fn queue_redraw_all(&mut self) {
        for state in self.output_state.values_mut() {
            state.redraw_state = mem::take(&mut state.redraw_state).queue_redraw();
        }
    }

    /// Schedules an immediate redraw for one output.
    pub fn queue_redraw(&mut self, output: &Output) {
        let state = self.output_state.get_mut(output).unwrap();
        state.redraw_state = mem::take(&mut state.redraw_state).queue_redraw();
    }

    /// Redraws all outputs that have pending redraws.
    pub fn redraw_queued_outputs(&mut self, backend: &mut Backend) {
        let _span = tracy_client::span!("Niri::redraw_queued_outputs");
        
        while let Some((output, _)) = self.output_state.iter().find(|(_, state)| {
            matches!(
                state.redraw_state,
                RedrawState::Queued | RedrawState::WaitingForEstimatedVBlankAndQueued(_)
            )
        }) {
            trace!("redrawing output");
            let output = output.clone();
            self.redraw(backend, &output);
        }
    }
}
```

**Verify**: `cargo check`

---

### Unit 3: Move render_layer

```rust
impl Niri {
    #[allow(clippy::too_many_arguments)]
    fn render_layer<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        target: RenderTarget,
        layer_map: &LayerMap,
        layer: Layer,
        elements: &mut SplitElements<LayerSurfaceRenderElement<R>>,
        for_backdrop: bool,
    ) {
        // ... paste full implementation
    }
}
```

---

### Unit 4: Move render

This is the big one (~250 lines):

```rust
impl Niri {
    pub fn render<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        output: &Output,
        include_pointer: bool,
        mut target: RenderTarget,
    ) -> Vec<OutputRenderElements<R>> {
        // ... paste full implementation
    }
}
```

**Verify**: `cargo check`

---

### Unit 5: Move redraw

```rust
impl Niri {
    fn redraw(&mut self, backend: &mut Backend, output: &Output) {
        // ... paste full implementation (~130 lines)
    }
}
```

**Verify**: `cargo check` && `cargo test`

---

## Verification Checklist

- [ ] `render()` in render.rs
- [ ] `render_layer()` in render.rs
- [ ] `redraw()` in render.rs
- [ ] `queue_redraw*` functions in render.rs
- [ ] No duplicate definitions in mod.rs
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/render.rs` | +430 lines |
| `src/niri/mod.rs` | -430 lines |

**Expected mod.rs after P4**: ~2824 lines

---

## Dependencies

These functions use many fields and call many methods:

**Fields accessed**:
- `output_state`, `config`, `layout`, `global_space`
- `cursor_manager`, `mapped_layer_surfaces`
- `screenshot_ui`, `window_mru_ui`, `hotkey_overlay`
- `exit_confirm_dialog`, `config_error_notification`
- `debug_draw_opaque_regions`, `monitors_active`
- `clock`, `lock_state`

**Methods called**:
- `pointer_element()` - already in render.rs ✓
- `update_render_elements()` - already in render.rs ✓
- `send_frame_callbacks()` - in frame_callbacks.rs
- `render_for_screen_cast()` - in screencast.rs
- `render_for_screencopy_with_damage()` - in screencopy.rs

---

## Next Phase

After completing this phase, proceed to [Phase P5: Create cursor.rs](phase-P5-cursor.md).
