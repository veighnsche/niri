# Phase 07: Move `on_estimated_vblank_timer` to RenderManager

> **Status**: ✅ COMPLETE  
> **LOC**: ~32 (method) + ~45 (helper)  
> **Target**: `src/backend/tty/render.rs`  
> **Source**: `src/backend/tty/mod.rs` lines 1548-1579, 2511-2555

---

## Overview

Move the estimated vblank timer handling from `Tty` to `RenderManager`. This includes:
1. `on_estimated_vblank_timer` method
2. `queue_estimated_vblank_timer` helper function

---

## Current Signatures (in Tty/mod.rs)

```rust
// Method
fn on_estimated_vblank_timer(&self, niri: &mut Niri, output: Output)

// Free function
fn queue_estimated_vblank_timer(
    niri: &mut Niri,
    output: Output,
    target_presentation_time: Duration,
)
```

---

## New Signatures (in RenderManager)

```rust
impl RenderManager {
    pub fn on_estimated_vblank_timer(&self, niri: &mut Niri, output: Output)
}

// Keep as associated function or free function in render.rs
pub fn queue_estimated_vblank_timer(
    niri: &mut Niri,
    output: Output,
    target_presentation_time: Duration,
)
```

---

## Implementation Steps

### Step 1: Move `on_estimated_vblank_timer`

Copy lines 1548-1579 from mod.rs to render.rs.

This method is simple — it just:
1. Updates frame callback sequence
2. Handles redraw state transition
3. Queues redraw or sends frame callbacks

No transformations needed — it only uses `niri`.

### Step 2: Move `queue_estimated_vblank_timer`

Copy lines 2511-2555 from mod.rs to render.rs.

**Issue**: The callback calls `tty.on_estimated_vblank_timer(...)`:
```rust
let token = niri
    .event_loop
    .insert_source(timer, move |_, _, data| {
        data.backend
            .tty()
            .on_estimated_vblank_timer(&mut data.niri, output.clone());
        TimeoutAction::Drop
    })
```

This needs to call `tty.render.on_estimated_vblank_timer(...)` instead.

**Solution**: Keep the timer callback in mod.rs, but move the logic to render.rs:

```rust
// In render.rs
pub fn queue_estimated_vblank_timer(
    niri: &mut Niri,
    output: Output,
    target_presentation_time: Duration,
    on_timer: impl FnOnce(&mut State) + 'static,
) {
    // ... setup logic ...
    let token = niri
        .event_loop
        .insert_source(timer, move |_, _, data| {
            on_timer(data);
            TimeoutAction::Drop
        })
        .unwrap();
    // ...
}
```

Or simpler: keep the callback creation in Tty but move state handling to RenderManager.

### Step 3: Update render() to use new location

The `render()` method calls `queue_estimated_vblank_timer`. After moving, update the call.

### Step 4: Create delegation in Tty

```rust
impl Tty {
    fn on_estimated_vblank_timer(&self, niri: &mut Niri, output: Output) {
        self.render.on_estimated_vblank_timer(niri, output)
    }
}
```

---

## Verification

```bash
cargo check
cargo test
```

---

## Dependencies

- **Requires**: Phase 05, Phase 06
- **Blocks**: Phase 08-12 (output management)

---

## Notes

- Used when render fails to queue a frame
- Estimates when vblank would have occurred
- Sends frame callbacks at estimated time
- Prevents clients from stalling
