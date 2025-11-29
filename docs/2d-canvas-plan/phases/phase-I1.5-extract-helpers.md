# Phase I1.5: Extract Helper Functions

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~30 minutes  
> **Risk Level**: 🟢 Low  
> **Architectural Benefit**: ⭐⭐ Medium - pure functions, testable

---

## Goal

Extract helper predicates and utilities into `src/input/helpers.rs`.

These are pure functions that answer simple questions about input events.

---

## What Moves

From `mod.rs` (lines ~4225-4376):

```rust
// Event predicates - pure functions
fn should_activate_monitors<I: InputBackend>(event: &InputEvent<I>) -> bool
fn should_hide_hotkey_overlay<I: InputBackend>(event: &InputEvent<I>) -> bool
fn should_hide_exit_confirm_dialog<I: InputBackend>(event: &InputEvent<I>) -> bool
fn should_notify_activity<I: InputBackend>(event: &InputEvent<I>) -> bool
fn should_reset_pointer_inactivity_timer<I: InputBackend>(event: &InputEvent<I>) -> bool

// Action predicates - pure functions
fn allowed_when_locked(action: &Action) -> bool
fn allowed_during_screenshot(action: &Action) -> bool

// Hardcoded binds
fn hardcoded_overview_bind(raw: Keysym, mods: ModifiersState) -> Option<Bind>
```

---

## Why This is Good

1. **Pure functions** - No State dependency
2. **Testable** - Can unit test predicates
3. **Clear intent** - Functions named after what they answer
4. **Removes clutter** - Main module focuses on dispatch

---

## Target: `src/input/helpers.rs`

```rust
//! Input event helper functions and predicates.
//!
//! Pure functions for classifying input events.

use niri_config::{Action, Bind};
use smithay::backend::input::InputEvent;

/// Should this event turn monitors back on?
pub fn should_activate_monitors<I: InputBackend>(event: &InputEvent<I>) -> bool { ... }

/// Is this action allowed when the session is locked?
pub fn allowed_when_locked(action: &Action) -> bool { ... }

// etc.
```

---

## Verification

- [ ] All predicates still work
- [ ] `cargo check` passes
