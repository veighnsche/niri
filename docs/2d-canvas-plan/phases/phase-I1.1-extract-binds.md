# Phase I1.1: Extract Bind Resolution

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1 hour  
> **Risk Level**: 🟢 Low  
> **Architectural Benefit**: ⭐⭐⭐ High - creates testable, pure module

---

## Goal

Extract bind resolution logic into `src/input/binds.rs`. This is the highest-value extraction because:

1. **Pure functions** - no State dependency, just config + input → bind
2. **Testable** - can unit test bind matching without compositor
3. **Clear boundary** - input comes in, optional Bind comes out

---

## What Moves

From `mod.rs` (lines ~4017-4200, ~4664-4744):

```rust
// These are pure functions - they don't need State
fn should_intercept_key(...) -> FilterResult<Option<Bind>>
fn find_bind(...) -> Option<Bind>
fn find_configured_bind(...) -> Option<Bind>
fn find_configured_switch_action(...) -> Option<Action>
fn modifiers_from_state(mods: ModifiersState) -> Modifiers

// These build on the pure functions
pub fn mods_with_binds(...) -> HashSet<Modifiers>
pub fn mods_with_mouse_binds(...) -> HashSet<Modifiers>
pub fn mods_with_wheel_binds(...) -> HashSet<Modifiers>
pub fn mods_with_finger_scroll_binds(...) -> HashSet<Modifiers>
fn make_binds_iter(...) -> impl Iterator<Item = &Bind>
```

---

## Why This is Good Architecture

1. **Separation of concerns** - Bind resolution is distinct from action execution
2. **Testability** - The existing tests in mod.rs can move with these functions
3. **No coupling** - These functions only depend on config types, not State
4. **Single responsibility** - binds.rs answers "what action for this input?"

---

## Target: `src/input/binds.rs`

```rust
//! Keyboard and mouse bind resolution.
//!
//! This module handles matching input events to configured bindings.
//! It is intentionally free of State dependencies for testability.

use std::collections::HashSet;
use niri_config::{Bind, Binds, Config, Key, Modifiers, ModKey, SwitchBinds, Trigger};
use smithay::backend::input::{Switch, SwitchState};
use smithay::input::keyboard::{Keysym, ModifiersState};

/// Convert XKB modifier state to our Modifiers type.
pub fn modifiers_from_state(mods: ModifiersState) -> Modifiers { ... }

/// Find a bind for the given key input.
pub fn find_bind(...) -> Option<Bind> { ... }

/// Find a configured bind matching the trigger and modifiers.
pub fn find_configured_bind(...) -> Option<Bind> { ... }

/// Find switch action (lid open/close, tablet mode).
pub fn find_switch_action(...) -> Option<Action> { ... }

/// Get modifiers that have bindings for given triggers.
pub fn mods_with_binds(...) -> HashSet<Modifiers> { ... }

// ... etc
```

---

## Verification

- [ ] All bind-related tests pass
- [ ] `cargo check` passes
- [ ] No State or Niri imports in binds.rs (keeps it pure)
