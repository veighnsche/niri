# Phase S1.4: FocusState Completion

> **Goal**: FocusState is already well-designed. Minor additions to complete encapsulation.

## Current State ✅

FocusState is the **best-designed subsystem**:
- ✅ Private fields
- ✅ Proper accessor methods
- ✅ **Actual logic**: `compute_focus()` method (~110 LOC)
- ✅ Helper types: `FocusContext`, `LayerFocusCandidate`

This is the **model** other subsystems should follow.

---

## What's Already Good

### `compute_focus()` - The Right Pattern

```rust
impl FocusState {
    /// Computes what should have keyboard focus based on context.
    pub fn compute_focus(&self, ctx: &FocusContext) -> KeyboardFocus {
        // 110 lines of pure logic
        // Takes context struct, returns result
        // No side effects, no external mutations
    }
}
```

This is **exactly** how subsystem methods should work:
1. Takes immutable context
2. Returns computed result
3. Caller handles side effects

---

## Minor Additions Needed

### 1. Move `handle_focus_change` logic (mod.rs:957-965) - ~8 LOC

**Current:**
```rust
impl State {
    fn handle_focus_change(&mut self, old_focus: &KeyboardFocus, new_focus: &KeyboardFocus) {
        self.handle_keyboard_layout_tracking(old, new);
        self.handle_popup_grab_on_focus_change(new);
        self.update_window_focus_states(old, new);
    }
}
```

**This is orchestration** - should stay in State. But helpers could move:

---

### 2. Move `update_window_focus_states` (mod.rs:1041-1074) - ~33 LOC

**Current:** Updates window activated states when focus changes.

**Target:**
```rust
impl FocusState {
    /// Computes which windows need focus state updates.
    pub fn compute_focus_state_changes(
        &self,
        old: &KeyboardFocus,
        new: &KeyboardFocus,
    ) -> FocusStateChanges {
        // Returns what changed, caller applies
    }
}

pub struct FocusStateChanges {
    pub deactivate: Option<WindowId>,
    pub activate: Option<WindowId>,
}
```

---

### 3. Move `handle_keyboard_layout_tracking` (mod.rs:975-1018) - ~43 LOC

**Current:** Saves/restores keyboard layout per window.

**Consideration:** This is tightly coupled to Smithay keyboard API. May be better to keep in State as orchestration.

---

### 4. Add focus change event type

```rust
pub struct FocusChange {
    pub old: KeyboardFocus,
    pub new: KeyboardFocus,
    pub window_changes: FocusStateChanges,
}
```

---

## Already Complete

The following are already properly encapsulated:

| Method | Status |
|--------|--------|
| `compute_focus()` | ✅ In subsystem |
| `current()` | ✅ Accessor |
| `set_current()` | ✅ Setter |
| `layer_on_demand()` | ✅ Accessor |
| `set_layer_on_demand()` | ✅ Setter |
| `cleanup_layer_on_demand()` | ✅ In subsystem |
| `idle_inhibitors()` | ✅ Accessor |
| `shortcut_inhibitors()` | ✅ Accessor |
| `is_idle_inhibited()` | ✅ In subsystem |
| `are_shortcuts_inhibited()` | ✅ In subsystem |

---

## Recommended: Keep As-Is

FocusState is **already the best subsystem**. The remaining methods in `mod.rs` are legitimately orchestration that:
1. Needs Smithay seat/keyboard API
2. Needs multiple subsystem access
3. Handles side effects

**Recommendation:** Mark FocusState as complete, use it as the template for other subsystems.

---

## Optional Improvements

If we want to go further:

### Add `FocusManager` wrapper for State

```rust
impl State {
    /// Returns a focus manager for computing and applying focus changes.
    pub fn focus_manager(&mut self) -> FocusManager<'_> {
        FocusManager {
            focus: &mut self.niri.focus,
            seat: &self.niri.seat,
            layout: &mut self.niri.layout,
        }
    }
}

impl FocusManager<'_> {
    pub fn update(&mut self) {
        let ctx = self.build_context();
        let new_focus = self.focus.compute_focus(&ctx);
        if new_focus != *self.focus.current() {
            self.apply_focus_change(new_focus);
        }
    }
}
```

This is **nice to have** but not required.

---

## Success Criteria

- [x] Private fields ✅
- [x] Proper accessors ✅
- [x] Core logic in subsystem ✅
- [ ] (Optional) Move focus state change computation
- [x] Model for other subsystems ✅

---

## Estimated Effort

| Task | LOC | Time |
|------|-----|------|
| Review and document | 0 | 15 min |
| Optional: move compute_focus_state_changes | 33 | 20 min |
| **Total** | **~33** | **~35 min** |

**Note:** This phase is mostly verification that FocusState is already correct, with optional minor additions.
