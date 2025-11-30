# TEAM_067: niri.rs Refactor Masterplan

> **Created**: Nov 29, 2025
> **Status**: ✅ Planning Complete — Phase files created
> **Priority**: 🔴 **#1 BLOCKING** — No new features until this is complete
> **Target**: `/home/vince/Projects/niri/src/niri.rs` (6604 LOC → multiple <500 LOC modules)
> **Phases**: See `docs/2d-canvas-plan/phases/` for detailed work units
> **Compilation**: ✅ `cargo check` passes (48 unused import warnings only)

---

## Problem Statement

`src/niri.rs` is a 6604-line monolith containing:
- `State` struct (backend + niri pair)
- `Niri` struct (~100 fields!)
- Mixed concerns: rendering, output management, locking, screenshots, IPC, etc.

Goal: Split into focused modules with clear ownership, <500 LOC each, proper encapsulation (no `pub(super)` smell).

---

## Analysis: Current Method Groups

### 1. **State Methods** (~750 LOC)
- `State::new()` - initialization
- `refresh_and_flush_clients()`, `refresh()` - event loop
- `set_lid_closed()`, `reload_config()`, `reload_output_config()`
- `move_cursor*()`, `focus_*()` methods
- `update_keyboard_focus()`, `refresh_popup_grab()`
- `on_pw_msg()`, `on_screen_cast_msg()`, `on_*_msg()` - message handlers

### 2. **Output Management** (~500 LOC)
- `add_output()`, `remove_output()`, `output_resized()`
- `reposition_outputs()`
- `output_left/right/up/down/previous/next_of()`
- `output_under()`, `output_by_name_match()`, `output_for_*`

### 3. **Rendering** (~400 LOC)
- `render()` - main render loop
- `pointer_element()`, `update_render_elements()`
- `update_primary_scanout_output()`

### 4. **Screencopy & Screencasting** (~700 LOC)
- `render_for_screen_cast()`, `render_windows_for_screen_cast()`
- `render_for_screencopy_*()` methods
- `stop_cast()`, `stop_casts_for_target()`
- `set_dynamic_cast_target()`

### 5. **Screenshots** (~500 LOC)
- `screenshot()`, `screenshot_window()`, `screenshot_all_outputs()`
- `save_screenshot()`, `capture_screenshots()`
- `open_screenshot_ui()`, `confirm_screenshot()`

### 6. **Frame Callbacks & Feedback** (~350 LOC)
- `send_frame_callbacks()`, `send_frame_callbacks_on_fallback_timer()`
- `send_dmabuf_feedbacks()`
- `take_presentation_feedbacks()`

### 7. **Lock Screen** (~350 LOC)
- `lock()`, `unlock()`, `is_locked()`
- `continue_to_locking()`, `maybe_continue_to_locking()`
- `new_lock_surface()`, `lock_surface_focus()`
- `update_locked_hint()`

### 8. **Hit Testing & Contents** (~450 LOC)
- `contents_under()` - comprehensive hit testing
- `window_under()`, `window_under_cursor()`
- `row_under()`, `row_under_cursor()`
- `is_*_obscured_under()` methods

### 9. **Pointer & Cursor** (~350 LOC)
- `reset_pointer_inactivity_timer()`, `notify_activity()`
- `maybe_activate_pointer_constraint()`
- `handle_focus_follows_mouse()`
- `refresh_pointer_contents()`

### 10. **Window & Layer Rules** (~150 LOC)
- `recompute_window_rules()`, `recompute_layer_rules()`
- `refresh_window_rules()` (called in refresh())

### 11. **Types & Helpers** (~200 LOC)
- `KeyboardFocus`, `PointContents`, `LockState`, `RedrawState`
- `OutputState`, `PopupGrabState`, `CastTarget`
- Various small helper enums

### 12. **Niri::new()** (~500 LOC)
- Massive initialization function

---

## Proposed Module Structure

```
src/
├── niri.rs (shrink to ~600 LOC)        # Core struct + mod declarations
├── niri/
│   ├── mod.rs                          # Re-exports
│   ├── types.rs (~200 LOC)             # KeyboardFocus, PointContents, etc.
│   ├── output.rs (~500 LOC)            # Output management
│   ├── render.rs (~400 LOC)            # Rendering logic
│   ├── screencopy.rs (~400 LOC)        # Screencopy (zwlr_screencopy)
│   ├── screencast.rs (~400 LOC)        # PipeWire screencasting
│   ├── screenshot.rs (~400 LOC)        # Screenshot capture & save
│   ├── frame_callbacks.rs (~350 LOC)   # Frame callbacks & feedback
│   ├── lock.rs (~350 LOC)              # Session lock
│   ├── hit_test.rs (~450 LOC)          # contents_under, window_under
│   ├── pointer.rs (~350 LOC)           # Pointer/cursor management
│   ├── rules.rs (~150 LOC)             # Window/layer rules
│   └── init.rs (~500 LOC)              # Niri::new() extracted
```

Total: 12 files, avg ~375 LOC each.

---

## Migration Strategy

### Step 1: Create `niri/types.rs`
Extract helper types that don't have behavior:
- `KeyboardFocus`
- `PointContents`
- `LockState`, `LockRenderState`
- `RedrawState`
- `OutputState`
- `PopupGrabState`
- `CastTarget`
- `PointerVisibility`
- `CenterCoords`
- `PendingMruCommit`
- `SurfaceFrameThrottlingState`
- `DndIcon`

**No behavior, just data** → Clean extraction.

### Step 2: Create `niri/output.rs`
Extract output management to `impl Niri`:
- `add_output()`
- `remove_output()`
- `output_resized()`
- `reposition_outputs()`
- `output_left/right/up/down/previous/next_of()`
- `output_under()`
- `output_by_name_match()`
- `output_for_*()` methods

**These form a cohesive unit** → Operates on `self.global_space`, `self.output_state`, `self.layout`.

### Step 3: Create `niri/hit_test.rs`
Extract hit testing:
- `contents_under()`
- `window_under()`, `window_under_cursor()`
- `row_under()`, `row_under_cursor()`
- `is_sticky_obscured_under()`, `is_layout_obscured_under()`
- `is_inside_hot_corner()`

**Pure queries, no mutations** → Easy extraction.

### Step 4: Create `niri/lock.rs`
Extract lock screen logic:
- `lock()`, `unlock()`, `is_locked()`
- `continue_to_locking()`, `maybe_continue_to_locking()`
- `new_lock_surface()`, `lock_surface_focus()`
- `update_locked_hint()`

**Self-contained subsystem** → Clear boundaries.

### Step 5: Create `niri/screenshot.rs`
Extract screenshot functionality:
- `screenshot()`, `screenshot_window()`
- `screenshot_all_outputs()` (#[cfg(feature = "dbus")])
- `save_screenshot()`
- `capture_screenshots()`
- `open_screenshot_ui()`, `confirm_screenshot()` (these are on State, not Niri)

**Heavy but isolated** → Uses renderer, file I/O, threads.

### Step 6: Create `niri/screencast.rs`
Extract PipeWire screencasting:
- `render_for_screen_cast()` (#[cfg(feature = "xdp-gnome-screencast")])
- `render_windows_for_screen_cast()`
- `stop_cast()`, `stop_casts_for_target()`
- `set_dynamic_cast_target()`

**Feature-gated subsystem** → Clean extraction.

### Step 7: Create `niri/screencopy.rs`
Extract zwlr_screencopy:
- `render_for_screencopy_with_damage()`
- `render_for_screencopy_without_damage()`
- `render_for_screencopy_internal()`
- `remove_screencopy_output()`

**Protocol-specific** → Clear boundaries.

### Step 8: Create `niri/frame_callbacks.rs`
Extract frame callback logic:
- `send_frame_callbacks()`
- `send_frame_callbacks_on_fallback_timer()`
- `send_dmabuf_feedbacks()`
- `update_primary_scanout_output()`
- `take_presentation_feedbacks()`

**Surface traversal patterns** → Cohesive unit.

### Step 9: Create `niri/render.rs`
Extract core rendering:
- `render()` (main render loop)
- `pointer_element()`
- `update_render_elements()`
- `redraw_queued_outputs()`

**Core compositor concern** → Central but focused.

### Step 10: Create `niri/pointer.rs`
Extract pointer management:
- `reset_pointer_inactivity_timer()`
- `notify_activity()`
- `maybe_activate_pointer_constraint()`
- `handle_focus_follows_mouse()`
- `refresh_pointer_outputs()` (if exists)

**Input/cursor state** → Clear boundary.

### Step 11: Create `niri/rules.rs`
Extract window/layer rules:
- `recompute_window_rules()`
- `recompute_layer_rules()`
- `refresh_window_rules()`

**Config-driven behavior** → Small but distinct.

### Step 12: Create `niri/init.rs`
Extract `Niri::new()`:
- The massive initialization function
- Helper functions for initializing Smithay state

**Construction logic** → Separate from runtime behavior.

---

## Existing Abstractions to Leverage

### Pattern 1: `layout_impl/` style
The layout module already demonstrates this pattern:
```
src/layout/
├── mod.rs           # Core Layout struct
└── layout_impl/
    ├── mod.rs       # impl Layout re-exports
    ├── focus.rs
    ├── navigation.rs
    ├── render.rs
    └── ...
```

Apply same pattern to niri:
```
src/
├── niri.rs          # Core Niri + State structs
└── niri/
    ├── mod.rs       # impl Niri re-exports
    ├── output.rs
    ├── render.rs
    └── ...
```

### Pattern 2: Protocol modules in `protocols/`
Already well-organized:
- `protocols/ext_workspace.rs`
- `protocols/foreign_toplevel.rs`
- `protocols/screencopy.rs`

Screencopy methods in niri.rs should use these, not duplicate.

### Pattern 3: Handler modules in `handlers/`
- `handlers/compositor.rs`
- `handlers/xdg_shell.rs`

Some methods on `State` might belong here instead.

---

## Dependencies Between New Modules

```
niri/types.rs     ← used by all modules
      ↓
niri/output.rs    ← hit_test, lock, screenshot, screencopy, screencast, render
      ↓
niri/hit_test.rs  ← pointer, render
      ↓
niri/pointer.rs   ← render
      ↓
niri/render.rs    ← screenshot, screencopy, screencast
      ↓
niri/frame_callbacks.rs  ← render (post-render)
```

---

## Items for TODO.md

### Missing Abstractions Needed:

1. **`niri/` module directory** - Does not exist
2. **State methods vs Niri methods** - Need clear ownership rules:
   - State methods: Coordinate backend + niri
   - Niri methods: Pure compositor state

3. **Feature-flag boundaries** - Screencast/screenshot methods are heavily
   `#[cfg(feature = "...")]` gated. Create separate files per feature.

4. **IPC refresh methods** - Currently scattered:
   - `refresh_ipc_outputs()` in State
   - `ipc_refresh_layout()` in State
   - Should consolidate to `niri/ipc.rs` or similar

---

## Implementation Order

1. ✅ **niri/types.rs** - No dependencies, pure data
2. **niri/output.rs** - Core infrastructure
3. **niri/hit_test.rs** - Pure queries
4. **niri/lock.rs** - Self-contained
5. **niri/rules.rs** - Small, config-driven
6. **niri/pointer.rs** - Input handling
7. **niri/frame_callbacks.rs** - Post-render
8. **niri/screenshot.rs** - Heavy, isolated
9. **niri/screencopy.rs** - Protocol-specific
10. **niri/screencast.rs** - Feature-gated
11. **niri/render.rs** - Central orchestration
12. **niri/init.rs** - Construction logic

---

## Success Criteria

- [ ] Each module <500 LOC
- [ ] No `pub(super)` - proper encapsulation
- [ ] Clear ownership: each module owns its state
- [ ] Consistent patterns with layout_impl/
- [ ] All tests pass
- [ ] No regression in functionality

---

## Handoff

- [x] Analysis complete
- [x] Module structure proposed
- [x] Migration order defined
- [x] Dependencies mapped
- [x] TODO.md updated
- [x] Phase files created (A1-A8)
- [x] Code compiles (`cargo check`)
- [ ] Implementation (future teams)

### For Next Team

**Go to [`phases/README.md`](../phases/README.md)** — the masterplan.

Current phase: **[Phase A1: niri/types.rs](../phases/phase-A1-niri-types.md)** 🔄

Each phase file contains:
- Detailed work units (5-15 min each)
- Exact code to extract
- Verification steps
- Links to next phase

### Phase Files Created

| Phase | File | Description |
|-------|------|-------------|
| [A1](../phases/phase-A1-niri-types.md) | niri/types.rs | Extract pure data types |
| [A2](../phases/phase-A2-niri-output.md) | niri/output.rs | Output management |
| [A3](../phases/phase-A3-niri-hit-test.md) | niri/hit_test.rs | Hit testing queries |
| [A4](../phases/phase-A4-niri-lock.md) | niri/lock.rs | Session lock |
| [A5](../phases/phase-A5-niri-render.md) | niri/render.rs | Rendering + frame callbacks |
| [A6](../phases/phase-A6-niri-capture.md) | screenshot/screencopy/screencast | Screen capture |
| [A7](../phases/phase-A7-niri-input.md) | niri/pointer.rs + rules.rs | Input & rules |
| [A8](../phases/phase-A8-niri-init.md) | niri/init.rs | Constructor extraction |

### Commands to Verify

```bash
cargo check
cargo test
```
