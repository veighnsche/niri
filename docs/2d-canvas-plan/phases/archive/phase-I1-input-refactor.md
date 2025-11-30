# Phase I1: Input Module Refactor

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~4-5 hours  
> **Risk Level**: 🟡 Medium  
> **Prerequisite**: None  
> **Unblocks**: Cleaner input handling, easier feature additions

---

## Goal

Refactor `src/input/mod.rs` (5123 lines) into a modular `src/input/` directory following the same pattern as the successful `src/niri/` refactor.

---

## Current State

The `src/input/mod.rs` file is a monolithic 5123-line file containing:
- Input event dispatching
- Device management
- Keyboard handling
- **Action dispatch (~1550 lines!)** - the `do_action` match statement
- Pointer/mouse handling
- Tablet handling
- Gesture handling
- Touch handling
- Libinput configuration
- Bind matching logic
- Helper functions and predicates
- Tests

---

## Critical Assessment

### What we're actually achieving:

| Goal | Benefit | Rating |
|------|---------|--------|
| **Extract bind resolution** | Pure, testable module | ⭐⭐⭐ High |
| **Extract device settings** | Self-contained, isolated | ⭐⭐ Medium |
| **Split event handlers** | Navigability only | ⭐ Low |
| **Extract helpers** | Pure, testable functions | ⭐⭐ Medium |

### What we're NOT achieving:
- Better abstractions (handlers still need full State)
- Improved encapsulation (actions still reach into niri.layout)
- Testability for handlers (they need compositor context)

### Honest truth about `do_action`:
The 1550-line match statement looks scary, but:
1. The complexity lives in `Layout`, not here
2. Match statements are easy to navigate with IDE
3. Actions are thin wrappers: `Action::X => self.niri.layout.x()`
4. Splitting doesn't improve architecture, just moves code

---

## Phases

| Phase | Focus | Benefit | Time |
|-------|-------|---------|------|
| [I1.1](phase-I1.1-extract-binds.md) | Bind resolution | ⭐⭐⭐ Testable | 1h |
| [I1.2](phase-I1.2-extract-device.md) | Device management | ⭐⭐ Isolated | 45m |
| [I1.3](phase-I1.3-extract-handlers.md) | Event handlers | ⭐ Navigability | 2h |
| [I1.4](phase-I1.4-actions-decision.md) | Actions - decide | ❓ User choice | - |
| [I1.5](phase-I1.5-extract-helpers.md) | Helper functions | ⭐⭐ Testable | 30m |

---

## Target Architecture

```
src/input/
├── mod.rs              # Types, dispatcher, re-exports (~400 LOC)
├── binds.rs            # Bind resolution - PURE, TESTABLE (~300 LOC)
├── device.rs           # Device management, libinput (~350 LOC)
├── helpers.rs          # Predicates - PURE, TESTABLE (~150 LOC)
├── actions.rs          # do_action + handle_bind (~1600 LOC) [see I1.4]
├── keyboard.rs         # Keyboard events (~250 LOC)
├── pointer.rs          # Pointer events (~600 LOC)
├── tablet.rs           # Tablet events (~250 LOC)
├── gesture.rs          # Gesture events (~300 LOC)
├── touch.rs            # Touch events (~250 LOC)
└── [existing submodules stay as-is]
```

---

## Why This Order?

1. **I1.1 Binds first** - Highest value, pure functions, easy to test
2. **I1.2 Device next** - Self-contained, no dependencies
3. **I1.3 Handlers** - Mechanical, low risk
4. **I1.4 Actions decision** - Need user input on whether to split
5. **I1.5 Helpers last** - Small, clean up remaining clutter

---

## Success Criteria

- [ ] `binds.rs` has no State/Niri imports (pure)
- [ ] `helpers.rs` has no State/Niri imports (pure)
- [ ] All tests pass
- [ ] `cargo check` passes
- [ ] `mod.rs` is a thin dispatcher

---

## Design Principles

### 1. Thin Dispatcher
`mod.rs` should only contain:
- Type definitions (`TabletData`, `PointerOrTouchStartData`)
- `process_input_event` dispatcher
- Re-exports

### 2. Category-Based Actions
Actions are grouped by what they operate on (window, column, row, monitor, etc.) rather than by input type.

### 3. Consistent Patterns
Each handler module follows the same pattern:
```rust
impl State {
    pub fn on_<event_type><I: InputBackend>(&mut self, event: I::<EventType>) {
        // Handle the event
    }
}
```

### 4. Minimal Public API
Only expose what's needed by other modules. Keep implementation details private.

---

## Migration Notes

### Imports
The main challenge is managing imports. Each extracted module needs:
- Access to `State` and `Niri`
- Smithay input types
- Config types
- Layout types

### Method Visibility
Most methods stay as `impl State` but move to different files. Rust allows this naturally.

### Tests
Tests that use internal functions may need adjustment. Consider:
- Moving tests with their functions
- Making some test helpers `pub(crate)`

---

## Quick Reference

| File | Responsibility |
|------|----------------|
| `mod.rs` | Types, dispatcher, re-exports |
| `binds.rs` | Bind resolution |
| `device.rs` | Device management, libinput |
| `helpers.rs` | Utilities and predicates |
| `actions.rs` | do_action + handle_bind |
| `keyboard.rs` | Keyboard events |
| `pointer.rs` | Pointer events |
| `tablet.rs` | Tablet events |
| `gesture.rs` | Gesture events |
| `touch.rs` | Touch events |
