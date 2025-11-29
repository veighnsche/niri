# Phase A: Extract ProtocolStates Container

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟢 Low (pure mechanical grouping)  
> **Prerequisite**: None  
> **Creates**: `ProtocolStates` struct

---

## Goal

Group 35+ Smithay protocol state fields into a single `ProtocolStates` container.
This is the easiest phase — pure mechanical grouping with no logic changes.

---

## Why This Phase First

- **Zero behavior change** — just moving fields
- **Maximum field reduction** — removes ~35 fields from Niri
- **Simple verification** — if it compiles, it works
- **Builds confidence** — easy win before harder phases

---

## Fields to Move (35 fields)

```rust
// Current Niri fields (lines ~291-339)
pub compositor_state: CompositorState,
pub xdg_shell_state: XdgShellState,
pub xdg_decoration_state: XdgDecorationState,
pub kde_decoration_state: KdeDecorationState,
pub layer_shell_state: WlrLayerShellState,
pub session_lock_state: SessionLockManagerState,
pub foreign_toplevel_state: ForeignToplevelManagerState,
pub ext_workspace_state: ExtWorkspaceManagerState,
pub screencopy_state: ScreencopyManagerState,
pub output_management_state: OutputManagementManagerState,
pub viewporter_state: ViewporterState,
pub xdg_foreign_state: XdgForeignState,
pub shm_state: ShmState,
pub output_manager_state: OutputManagerState,
pub dmabuf_state: DmabufState,
pub fractional_scale_manager_state: FractionalScaleManagerState,
pub seat_state: SeatState<State>,
pub tablet_state: TabletManagerState,
pub text_input_state: TextInputManagerState,
pub input_method_state: InputMethodManagerState,
pub keyboard_shortcuts_inhibit_state: KeyboardShortcutsInhibitState,
pub virtual_keyboard_state: VirtualKeyboardManagerState,
pub virtual_pointer_state: VirtualPointerManagerState,
pub pointer_gestures_state: PointerGesturesState,
pub relative_pointer_state: RelativePointerManagerState,
pub pointer_constraints_state: PointerConstraintsState,
pub idle_notifier_state: IdleNotifierState<State>,
pub idle_inhibit_manager_state: IdleInhibitManagerState,
pub data_device_state: DataDeviceState,
pub primary_selection_state: PrimarySelectionState,
pub wlr_data_control_state: WlrDataControlState,
pub ext_data_control_state: ExtDataControlState,
pub presentation_state: PresentationState,
pub security_context_state: SecurityContextState,
pub gamma_control_manager_state: GammaControlManagerState,
pub activation_state: XdgActivationState,
pub mutter_x11_interop_state: MutterX11InteropManagerState,
pub cursor_shape_manager_state: CursorShapeManagerState,
#[cfg(test)]
pub single_pixel_buffer_state: SinglePixelBufferState,
```

---

## Target Architecture

### New File: `src/niri/protocols.rs`

```rust
//! Smithay protocol states container.
//!
//! Groups all Wayland protocol state objects that Smithay requires.
//! These are initialized once and rarely accessed directly.

use smithay::wayland::compositor::CompositorState;
use smithay::wayland::dmabuf::DmabufState;
// ... all other imports

use crate::niri::State;

/// Container for all Smithay protocol states.
///
/// This groups protocol state objects that are:
/// - Initialized once at startup
/// - Required by Smithay handlers
/// - Rarely accessed directly by compositor logic
pub struct ProtocolStates {
    pub compositor: CompositorState,
    pub xdg_shell: XdgShellState,
    pub xdg_decoration: XdgDecorationState,
    pub kde_decoration: KdeDecorationState,
    pub layer_shell: WlrLayerShellState,
    pub session_lock: SessionLockManagerState,
    pub foreign_toplevel: ForeignToplevelManagerState,
    pub ext_workspace: ExtWorkspaceManagerState,
    pub screencopy: ScreencopyManagerState,
    pub output_management: OutputManagementManagerState,
    pub viewporter: ViewporterState,
    pub xdg_foreign: XdgForeignState,
    pub shm: ShmState,
    pub output_manager: OutputManagerState,
    pub dmabuf: DmabufState,
    pub fractional_scale: FractionalScaleManagerState,
    pub seat: SeatState<State>,
    pub tablet: TabletManagerState,
    pub text_input: TextInputManagerState,
    pub input_method: InputMethodManagerState,
    pub keyboard_shortcuts_inhibit: KeyboardShortcutsInhibitState,
    pub virtual_keyboard: VirtualKeyboardManagerState,
    pub virtual_pointer: VirtualPointerManagerState,
    pub pointer_gestures: PointerGesturesState,
    pub relative_pointer: RelativePointerManagerState,
    pub pointer_constraints: PointerConstraintsState,
    pub idle_notifier: IdleNotifierState<State>,
    pub idle_inhibit: IdleInhibitManagerState,
    pub data_device: DataDeviceState,
    pub primary_selection: PrimarySelectionState,
    pub wlr_data_control: WlrDataControlState,
    pub ext_data_control: ExtDataControlState,
    pub presentation: PresentationState,
    pub security_context: SecurityContextState,
    pub gamma_control: GammaControlManagerState,
    pub activation: XdgActivationState,
    pub mutter_x11_interop: MutterX11InteropManagerState,
    pub cursor_shape: CursorShapeManagerState,
    #[cfg(test)]
    pub single_pixel_buffer: SinglePixelBufferState,
}
```

---

## Work Units

### Unit 1: Create protocols.rs file

Create `src/niri/protocols.rs` with:
1. All imports (copy from mod.rs)
2. `ProtocolStates` struct definition
3. No methods yet (just a container)

**Time**: ~10 minutes

---

### Unit 2: Add module declaration

In `src/niri/mod.rs`, add:
```rust
mod protocols;
pub use protocols::ProtocolStates;
```

**Time**: ~2 minutes

---

### Unit 3: Move fields from Niri

1. Remove the 35+ protocol state fields from `Niri` struct
2. Add single field: `pub protocols: ProtocolStates`

**Time**: ~10 minutes

---

### Unit 4: Update Niri::new()

In `src/niri/init.rs`, update the initialization:
```rust
// Before
compositor_state,
xdg_shell_state,
// ... 35 more

// After
protocols: ProtocolStates {
    compositor: compositor_state,
    xdg_shell: xdg_shell_state,
    // ... 35 more
},
```

**Time**: ~15 minutes

---

### Unit 5: Update all access patterns

Search and replace throughout codebase:
```rust
// Before
self.compositor_state
self.niri.compositor_state

// After
self.protocols.compositor
self.niri.protocols.compositor
```

Use these commands to find all usages:
```bash
grep -rn "compositor_state" src/
grep -rn "xdg_shell_state" src/
# ... for each field
```

**Time**: ~15 minutes

---

### Unit 6: Verify

```bash
cargo check
cargo test
```

---

## Verification Checklist

- [ ] `protocols.rs` exists with ProtocolStates struct
- [ ] All 35+ fields moved to ProtocolStates
- [ ] `Niri.protocols: ProtocolStates` field added
- [ ] `Niri::new()` updated to create ProtocolStates
- [ ] All access patterns updated
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/protocols.rs` | **NEW** ~150 lines |
| `src/niri/mod.rs` | -35 fields, +2 lines |
| `src/niri/init.rs` | Updated initialization |
| Various handlers | Updated access patterns |

---

## Common Access Pattern Updates

### In Smithay handlers

```rust
// Before (handlers/compositor.rs)
impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.niri.compositor_state
    }
}

// After
impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.niri.protocols.compositor
    }
}
```

### In Niri methods

```rust
// Before
self.xdg_shell_state.toplevel_surfaces()

// After
self.protocols.xdg_shell.toplevel_surfaces()
```

---

## Benefits

1. **-35 fields** from Niri struct
2. **Clear grouping** of protocol states
3. **Easy to find** protocol-related code
4. **Prepares** for further subsystem extraction

---

## Next Phase

After completing this phase, proceed to [Phase B: OutputSubsystem](phase-B-output-subsystem.md).
