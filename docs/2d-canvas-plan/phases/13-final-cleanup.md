# Phase 13: Final Cleanup

> **Status**: ⏳ PENDING  
> **Target**: All TTY files  
> **Goal**: Clean up mod.rs and finalize architecture

---

## Overview

After all methods have been moved, clean up the remaining code:
1. Remove dead code from mod.rs
2. Update documentation
3. Verify final architecture
4. Run full test suite

---

## Cleanup Tasks

### Task 1: Remove Dead Imports from mod.rs

After moving methods, many imports will be unused:
```rust
// Remove unused imports like:
use std::mem;
use smithay::backend::drm::compositor::{FrameFlags, PrimaryPlaneElement};
// ... etc
```

Run `cargo check` to identify unused imports.

### Task 2: Update Module Documentation

Update the doc comment at the top of mod.rs:
```rust
//! TTY/DRM backend for native display.
//!
//! This module uses the **subsystem ownership pattern**:
//! - `DeviceManager` (devices.rs) - owns all DRM device state and lifecycle
//! - `RenderManager` (render.rs) - owns rendering and vblank handling
//! - `OutputManager` (outputs.rs) - owns IPC and output configuration
//!
//! `Tty` is a thin coordinator that:
//! - Holds session and udev state
//! - Dispatches events to subsystems
//! - Provides public API delegation
```

### Task 3: Verify Tty is Now a Thin Coordinator

After cleanup, Tty should only contain:
- Constructor (`new`)
- Event dispatching (`on_session_event`, `on_udev_event`)
- Public API delegation (one-liner methods)
- Session-specific code that can't be moved

### Task 4: Remove Unused Helper Functions

Check for helper functions in mod.rs that are now only used in subsystems:
- Move them to the appropriate subsystem
- Or to helpers.rs if shared

### Task 5: Update Type Visibility

Some types may need visibility changes:
```rust
// In types.rs, ensure proper visibility
pub(super) struct Surface { ... }  // Only needed within tty/
pub struct TtyOutputState { ... }   // Needed externally
```

### Task 6: Add Subsystem Tests (Optional)

Consider adding unit tests for subsystems:
```rust
// In devices.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_manager_creation() {
        // ...
    }
}
```

---

## Verification Checklist

### Compilation
- [ ] `cargo check` passes with no warnings
- [ ] `cargo clippy` passes
- [ ] No unused imports or dead code warnings

### Tests
- [ ] `cargo test` passes
- [ ] All existing tests still work

### Manual Testing
- [ ] niri starts correctly
- [ ] Monitor hotplug works
- [ ] Config reload works
- [ ] VRR toggle works
- [ ] Gamma/night light works
- [ ] `niri msg outputs` works

### Architecture
- [ ] mod.rs is ~600 LOC or less
- [ ] Each subsystem is self-contained
- [ ] No circular dependencies
- [ ] Clear ownership boundaries

---

## Final File Sizes

| File | Expected LOC |
|------|--------------|
| mod.rs | ~600 |
| devices.rs | ~1400 |
| render.rs | ~470 |
| outputs.rs | ~480 |
| helpers.rs | ~600 (unchanged) |
| types.rs | ~150 (unchanged) |

**Total**: ~3700 LOC (up from 3469 due to delegation code)

---

## Documentation Updates

### Update phases/README.md

Mark all phases as complete:
```markdown
| # | Phase | Status |
|---|-------|--------|
| 01-13 | All | ✅ Complete |
```

### Update team file

Document completion in TEAM_089 file.

---

## Dependencies

- **Requires**: All previous phases (01-12)
- **Blocks**: None (final phase)
