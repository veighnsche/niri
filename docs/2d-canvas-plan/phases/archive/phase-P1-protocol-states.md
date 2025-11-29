# Phase P1: Extract ProtocolStates Container

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~30 minutes  
> **Risk Level**: 🟢 Low (pure mechanical grouping)  
> **Prerequisite**: None  
> **Unblocks**: Cleaner Niri struct, easier subsystem extraction

---

## Goal

Group the 25+ Smithay protocol state fields from `Niri` into a single `ProtocolStates` container struct.

This is the safest first step: pure mechanical refactoring with no behavioral changes.

---

## Why This First?

The `Niri` struct has **25+ protocol state fields** that:
- Are all initialized the same way (in `Niri::new`)
- Are all accessed the same way (`self.xdg_shell_state`, etc.)
- Have no logic of their own (just Smithay state containers)
- Clutter the `Niri` struct definition

Grouping them reduces `Niri` fields by ~25 and makes the struct easier to understand.

---

## Current State (in mod.rs)

```rust
pub struct Niri {
    // ... 75 other fields ...
    
    // Smithay state (lines ~292-330, ~38 lines)
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
}
```

---

## Target State

### New File: `src/niri/protocols.rs`

```rust
//! Smithay protocol state container.
//!
//! Groups all Wayland protocol states into a single struct to reduce
//! clutter in the main Niri struct.

use smithay::wayland::compositor::CompositorState;
use smithay::wayland::shell::xdg::XdgShellState;
// ... all other imports

use super::State;

/// Container for all Smithay protocol states.
///
/// These are initialized once in `Niri::new` and provide the Wayland
/// protocol implementations. Grouping them here keeps the main `Niri`
/// struct focused on compositor logic rather than protocol plumbing.
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

impl ProtocolStates {
    /// Creates all protocol states for the given display.
    pub fn new(display: &DisplayHandle, ...) -> Self {
        // Move initialization from Niri::new here
        Self { ... }
    }
}
```

### Updated Niri Struct

```rust
pub struct Niri {
    // ... other fields ...
    
    /// All Wayland protocol states.
    pub protocols: ProtocolStates,
    
    // Removed: 25+ individual protocol fields
}
```

---

## Work Units

### Unit 1: Create protocols.rs

Create `src/niri/protocols.rs` with:
1. Module documentation
2. All necessary imports
3. `ProtocolStates` struct definition
4. Empty `impl ProtocolStates` block

**Verify**: `cargo check` (will fail - not connected yet)

---

### Unit 2: Add Module Declaration

In `mod.rs`:
```rust
mod protocols;
pub use protocols::ProtocolStates;
```

---

### Unit 3: Move Fields to ProtocolStates

Cut the protocol state fields from `Niri` struct, paste into `ProtocolStates`.

Rename fields to shorter names (remove `_state` suffix):
- `compositor_state` → `compositor`
- `xdg_shell_state` → `xdg_shell`
- etc.

**Verify**: `cargo check` (will fail - access patterns need updating)

---

### Unit 4: Update All Access Patterns

Search and replace throughout codebase:
```bash
# Find all usages
grep -rn "\.compositor_state" src/
grep -rn "\.xdg_shell_state" src/
# etc.
```

Update patterns:
- `self.compositor_state` → `self.protocols.compositor`
- `niri.xdg_shell_state` → `niri.protocols.xdg_shell`

**Verify**: `cargo check` after each batch

---

### Unit 5: Move Initialization to ProtocolStates::new

Extract protocol initialization from `Niri::new` into `ProtocolStates::new`.

In `init.rs`:
```rust
let protocols = ProtocolStates::new(&display_handle, ...);

Niri {
    protocols,
    // ...
}
```

**Verify**: `cargo check` && `cargo test`

---

## Verification Checklist

- [ ] `protocols.rs` exists with `ProtocolStates` struct
- [ ] All 25+ protocol fields moved from `Niri`
- [ ] `Niri` has single `protocols: ProtocolStates` field
- [ ] All access patterns updated (`self.protocols.xyz`)
- [ ] Initialization moved to `ProtocolStates::new`
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/protocols.rs` | +200 lines (new file) |
| `src/niri/mod.rs` | -40 lines (fields), +3 lines (mod + use) |
| `src/niri/init.rs` | Refactored initialization |
| Various handlers | Updated access patterns |

---

## Benefits

1. **Niri struct clarity**: 25 fewer fields to scroll past
2. **Logical grouping**: All protocol states in one place
3. **Future-proof**: Easy to add new protocols
4. **Prepares for subsystems**: Establishes pattern of grouping related state

---

## Next Phase

After completing this phase, proceed to [Phase P2: OutputSubsystem](phase-P2-output-subsystem.md).

This establishes the pattern of extracting domain-specific state into dedicated structs.
