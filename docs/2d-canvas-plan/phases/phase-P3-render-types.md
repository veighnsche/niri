# Phase P3: Move OutputRenderElements to render.rs

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~30 minutes  
> **Risk Level**: 🟡 Medium (macro complexity)  
> **Prerequisite**: None (can be done in parallel with P1-P2)  
> **Unblocks**: Phase P4 (render function extraction)

---

## Goal

Move the `OutputRenderElements` type definition and helper function from `mod.rs` to `render.rs`.

This unblocks moving the `render()` function which returns `Vec<OutputRenderElements<R>>`.

---

## Current Location

**File**: `src/niri/mod.rs` lines ~3524-3554 (~30 lines)

```rust
fn scale_relocate_crop<E: Element>(
    elem: E,
    output_scale: Scale<f64>,
    zoom: f64,
    ws_geo: Rectangle<f64, Logical>,
) -> Option<CropRenderElement<RelocateRenderElement<RescaleRenderElement<E>>>> {
    // ...
}

niri_render_elements! {
    OutputRenderElements<R> => {
        Monitor = MonitorRenderElement<R>,
        RescaledTile = RescaleRenderElement<TileRenderElement<R>>,
        LayerSurface = LayerSurfaceRenderElement<R>,
        // ... more variants
    }
}
```

---

## Work Units

### Unit 1: Add Required Imports to render.rs

Add imports needed for the render elements macro:

```rust
use smithay::backend::renderer::element::memory::MemoryRenderBufferRenderElement;
use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::element::utils::{
    CropRenderElement, Relocate, RelocateRenderElement, RescaleRenderElement,
};
use smithay::backend::renderer::element::Element;
use smithay::utils::{Logical, Rectangle, Scale};

use crate::layer::mapped::LayerSurfaceRenderElement;
use crate::layout::tile::TileRenderElement;
use crate::layout::MonitorRenderElement;
use crate::niri_render_elements;
use crate::render_helpers::primary_gpu_texture::PrimaryGpuTextureRenderElement;
use crate::render_helpers::solid_color::SolidColorRenderElement;
use crate::ui::exit_confirm_dialog::ExitConfirmDialogRenderElement;
use crate::ui::mru::WindowMruUiRenderElement;
use crate::ui::screenshot_ui::ScreenshotUiRenderElement;
```

---

### Unit 2: Move scale_relocate_crop Helper

Cut from `mod.rs` and paste into `render.rs`:

```rust
// =============================================================================
// Render Element Types
// =============================================================================

pub(super) fn scale_relocate_crop<E: Element>(
    elem: E,
    output_scale: Scale<f64>,
    zoom: f64,
    ws_geo: Rectangle<f64, Logical>,
) -> Option<CropRenderElement<RelocateRenderElement<RescaleRenderElement<E>>>> {
    let ws_geo = ws_geo.to_physical_precise_round(output_scale);
    let elem = RescaleRenderElement::from_element(elem, Point::from((0, 0)), zoom);
    let elem = RelocateRenderElement::from_element(elem, ws_geo.loc, Relocate::Relative);
    CropRenderElement::from_element(elem, output_scale, ws_geo)
}
```

---

### Unit 3: Move niri_render_elements! Invocation

Cut the macro invocation from `mod.rs` and paste into `render.rs`:

```rust
niri_render_elements! {
    pub OutputRenderElements<R> => {
        Monitor = MonitorRenderElement<R>,
        RescaledTile = RescaleRenderElement<TileRenderElement<R>>,
        LayerSurface = LayerSurfaceRenderElement<R>,
        RelocatedLayerSurface = CropRenderElement<RelocateRenderElement<RescaleRenderElement<
            LayerSurfaceRenderElement<R>
        >>>,
        Wayland = WaylandSurfaceRenderElement<R>,
        NamedPointer = MemoryRenderBufferRenderElement<R>,
        SolidColor = SolidColorRenderElement,
        ScreenshotUi = ScreenshotUiRenderElement,
        WindowMruUi = WindowMruUiRenderElement<R>,
        ExitConfirmDialog = ExitConfirmDialogRenderElement,
        Texture = PrimaryGpuTextureRenderElement,
        RelocatedMemoryBuffer = RelocateRenderElement<MemoryRenderBufferRenderElement<R>>,
    }
}
```

**Note**: Make sure `pub` is added to make it visible outside the module.

---

### Unit 4: Re-export from mod.rs

In `mod.rs`, add re-export:

```rust
// At top with other module declarations
mod render;

// Re-export the type
pub use render::OutputRenderElements;
```

---

### Unit 5: Update All Import Sites

Search for files that import `OutputRenderElements`:

```bash
grep -rn "OutputRenderElements" src/
```

Update imports to use the new location. Most should work via the re-export.

**Verify**: `cargo check` && `cargo test`

---

## Verification Checklist

- [ ] `OutputRenderElements` defined in render.rs
- [ ] `scale_relocate_crop` in render.rs
- [ ] Re-exported from mod.rs
- [ ] No duplicate definitions
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/render.rs` | +40 lines (type + helper + imports) |
| `src/niri/mod.rs` | -30 lines |

---

## Technical Notes

The `niri_render_elements!` macro generates an enum with From implementations for each variant. The macro is defined elsewhere and just invoked here.

The generated type looks roughly like:
```rust
pub enum OutputRenderElements<R> {
    Monitor(MonitorRenderElement<R>),
    RescaledTile(RescaleRenderElement<TileRenderElement<R>>),
    // ...
}
```

---

## Next Phase

After completing this phase, proceed to [Phase P4: Render Functions](phase-P4-render-impl.md).
