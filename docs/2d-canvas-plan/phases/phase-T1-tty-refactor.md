# Phase T1: TTY Backend Refactor

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~4-5 hours  
> **Risk Level**: 🟡 Medium  
> **Prerequisite**: None  
> **Unblocks**: Cleaner backend code, easier hardware support additions

---

## Goal

Refactor `src/backend/tty.rs` (3473 lines) into a modular `src/backend/tty/` directory following the same pattern as the successful `src/niri/` and `src/input/` refactors.

---

## Current State

The `src/backend/tty.rs` file is a monolithic 3473-line file containing:
- **Tty struct and types** (~130 lines)
- **OutputDevice struct and impl** (~240 lines)
- **impl Tty** (~2120 lines!) - the main logic
- **Helper functions** (~400 lines) - DRM, modes, properties
- **GammaProps impl** (~140 lines)
- **ConnectorProperties** (~170 lines)
- **Tests** (~130 lines)

---

## Critical Assessment

### What we're achieving:

| Goal | Benefit | Rating |
|------|---------|--------|
| **Extract types** | Clear module boundary | ⭐⭐ Medium |
| **Extract device lifecycle** | Isolated, focused code | ⭐⭐⭐ High |
| **Extract helpers** | Pure, testable functions | ⭐⭐⭐ High |
| **Extract render** | Clear rendering pipeline | ⭐⭐ Medium |

### Complexity analysis:

The `impl Tty` block is 2120 lines with these categories:
- **Initialization** (130 lines): `new()`, `init()`
- **Event handling** (170 lines): `on_udev_event()`, `on_session_event()`
- **Device lifecycle** (470 lines): `device_added()`, `device_changed()`, `device_removed()`
- **Connector handling** (380 lines): `connector_connected()`, `connector_disconnected()`
- **Render/VBlank** (400 lines): `render()`, `on_vblank()`, `on_estimated_vblank_timer()`
- **Output management** (570 lines): IPC, gamma, VRR, config changes

---

## Phases

| Phase | Focus | Benefit | Time |
|-------|-------|---------|------|
| [T1.1](phase-T1.1-extract-types.md) | Types & module structure | Foundation | 30m |
| [T1.2](phase-T1.2-extract-device.md) | OutputDevice | ⭐⭐ Isolated | 45m |
| [T1.3](phase-T1.3-extract-helpers.md) | Helper functions | ⭐⭐⭐ Testable | 45m |
| [T1.4](phase-T1.4-extract-lifecycle.md) | Device lifecycle | ⭐⭐⭐ Focused | 1h |
| [T1.5](phase-T1.5-extract-connectors.md) | Connector handling | ⭐⭐ Isolated | 45m |
| [T1.6](phase-T1.6-extract-render.md) | Render pipeline | ⭐⭐ Clear | 45m |
| [T1.7](phase-T1.7-extract-output.md) | Output management | ⭐⭐ Organized | 30m |

---

## Target Architecture

```
src/backend/tty/
├── mod.rs              # Tty struct, init, thin dispatcher (~300 LOC)
├── types.rs            # Type definitions, type aliases (~150 LOC)
├── device.rs           # OutputDevice, CrtcInfo (~250 LOC)
├── helpers.rs          # Pure functions - DRM, modes, properties (~400 LOC)
├── lifecycle.rs        # device_added/changed/removed, udev/session events (~650 LOC)
├── connectors.rs       # connector_connected/disconnected (~400 LOC)
├── render.rs           # render(), on_vblank(), vblank timer (~420 LOC)
├── output.rs           # IPC, gamma, VRR, config (~600 LOC)
└── gamma.rs            # GammaProps impl (~150 LOC)
```

---

## Why This Order?

1. **T1.1 Types first** - Foundation, creates module structure
2. **T1.2 Device next** - Self-contained, OutputDevice is its own thing
3. **T1.3 Helpers** - Pure functions, no State dependency, easy to extract
4. **T1.4 Lifecycle** - Biggest piece, but well-bounded (device add/remove)
5. **T1.5 Connectors** - Depends on lifecycle being done
6. **T1.6 Render** - Core rendering, somewhat standalone
7. **T1.7 Output last** - Miscellaneous remaining pieces

---

## Success Criteria

- [ ] `helpers.rs` has no Tty/State imports (pure functions)
- [ ] `types.rs` has only type definitions
- [ ] All tests pass
- [ ] `cargo check` passes
- [ ] `mod.rs` is a thin coordinator

---

## Design Principles

### 1. Thin Coordinator
`mod.rs` should only contain:
- `Tty` struct definition
- `new()` and `init()` 
- Event dispatch (delegating to other modules)
- Re-exports

### 2. Pure Helpers
Helper functions in `helpers.rs` should be pure - no `&mut self`, no State access. These include:
- DRM mode calculations
- Property lookups
- Node discovery

### 3. Consistent Patterns
Each module follows the same pattern as `src/input/`:
```rust
impl Tty {
    pub(super) fn device_added(...) { ... }
}
```

### 4. Minimal Public API
Only expose what's needed. Most methods are `pub(super)` or `pub(crate)`.

---

## Quick Reference

| File | Responsibility | LOC |
|------|----------------|-----|
| `mod.rs` | Tty struct, init, dispatcher | ~300 |
| `types.rs` | Type definitions | ~150 |
| `device.rs` | OutputDevice, leases | ~250 |
| `helpers.rs` | Pure DRM/mode functions | ~400 |
| `lifecycle.rs` | Device add/change/remove | ~650 |
| `connectors.rs` | Connect/disconnect | ~400 |
| `render.rs` | Render, vblank | ~420 |
| `output.rs` | IPC, gamma, VRR | ~600 |
| `gamma.rs` | GammaProps | ~150 |

**Total: ~3320 LOC** (similar to original, but organized)
