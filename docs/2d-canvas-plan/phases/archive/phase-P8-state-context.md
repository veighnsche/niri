# Phase P8: State Context Pattern — SKIPPED

> **Status**: ❌ **SKIPPED**  
> **Reason**: Over-engineering with questionable benefit  
> **Decision**: Made by consolidation review (see README.md note)

---

## Why Skipped

This phase was removed from the plan because:
1. **Over-engineering**: Adds complexity without clear benefit
2. **Invasive**: Requires updating many call sites
3. **Incremental**: Can be done later if ever needed
4. **Subsystems sufficient**: P1-P7.5 already achieve the main goals

The subsystem extraction (P1-P7.5) provides:
- Encapsulation
- Testability
- Clear ownership

The Context pattern can be added incrementally later if a specific need arises.

---

## Original Goal (for reference)

---

## Goal

Refactor `impl State` methods to use a context pattern instead of `&mut self`, enabling better testing and clearer dependencies.

**This phase is optional.** The previous phases already achieve significant improvement. This phase is for further architectural cleanup if time permits.

---

## The Problem with `impl State`

Currently, `State` methods take `&mut self` even when they only need specific parts:

```rust
impl State {
    pub fn move_cursor(&mut self, location: Point<f64, Logical>) {
        // Actually only needs:
        // - self.niri.cursor (subsystem)
        // - self.niri.seat (for pointer)
        // - self.niri (for contents_under)
        // Does NOT need self.backend
    }
}
```

This makes testing difficult and hides actual dependencies.

---

## The Context Pattern

Instead of `&mut self`, pass a context struct with only what's needed:

```rust
/// Context for cursor operations.
pub struct CursorContext<'a> {
    pub cursor: &'a mut CursorSubsystem,
    pub seat: &'a Seat<State>,
    pub layout: &'a Layout<Mapped>,
    pub outputs: &'a OutputSubsystem,
}

impl CursorContext<'_> {
    pub fn move_to(&mut self, location: Point<f64, Logical>) {
        // Now clear what this operation needs
    }
}
```

---

## When to Use This Pattern

Use context pattern for:
- Methods that don't need backend
- Methods that only need specific subsystems
- Methods that should be testable

Keep `&mut self` for:
- Methods that need backend access
- High-level coordination methods
- Methods that touch many subsystems

---

## Example Refactoring

### Before

```rust
impl State {
    pub fn maybe_warp_cursor_to_focus(&mut self) -> bool {
        let config = self.niri.config.borrow();
        match config.input.warp_mouse_to_focus {
            WarpMouseToFocusMode::Never => false,
            WarpMouseToFocusMode::OnFocusChange => {
                self.move_cursor_to_focused_tile(CenterCoords::Separately)
            }
        }
    }
    
    pub fn move_cursor_to_focused_tile(&mut self, mode: CenterCoords) -> bool {
        let Some(rect) = self.niri.layout.focused_tile_rect() else {
            return false;
        };
        self.move_cursor_to_rect(rect, mode)
    }
}
```

### After

```rust
impl State {
    /// Creates a cursor context for cursor operations.
    fn cursor_context(&mut self) -> CursorContext<'_> {
        CursorContext {
            cursor: &mut self.niri.cursor,
            seat: &self.niri.seat,
            layout: &self.niri.layout,
            outputs: &self.niri.outputs,
            config: &self.niri.config,
        }
    }
    
    pub fn maybe_warp_cursor_to_focus(&mut self) -> bool {
        self.cursor_context().maybe_warp_to_focus()
    }
}

impl CursorContext<'_> {
    pub fn maybe_warp_to_focus(&mut self) -> bool {
        let config = self.config.borrow();
        match config.input.warp_mouse_to_focus {
            WarpMouseToFocusMode::Never => false,
            WarpMouseToFocusMode::OnFocusChange => {
                self.warp_to_focused_tile(CenterCoords::Separately)
            }
        }
    }
    
    pub fn warp_to_focused_tile(&mut self, mode: CenterCoords) -> bool {
        let Some(rect) = self.layout.focused_tile_rect() else {
            return false;
        };
        self.warp_to_rect(rect, mode)
    }
}
```

---

## Contexts to Consider

### CursorContext

For cursor movement operations:
```rust
pub struct CursorContext<'a> {
    pub cursor: &'a mut CursorSubsystem,
    pub seat: &'a Seat<State>,
    pub layout: &'a Layout<Mapped>,
    pub outputs: &'a OutputSubsystem,
    pub config: &'a Rc<RefCell<Config>>,
}
```

### FocusContext

Already created in Phase P4 for focus computation:
```rust
pub struct FocusContext<'a> {
    pub exit_dialog_open: bool,
    pub is_locked: bool,
    // ...
}
```

### RenderContext

For rendering operations:
```rust
pub struct RenderContext<'a, R: NiriRenderer> {
    pub renderer: &'a mut R,
    pub output: &'a Output,
    pub outputs: &'a OutputSubsystem,
    pub cursor: &'a CursorSubsystem,
    pub ui: &'a UiOverlays,
    pub layout: &'a Layout<Mapped>,
}
```

---

## Work Units

### Unit 1: Define CursorContext

Create `src/niri/contexts.rs`:

```rust
//! Context structs for operation-specific state access.

pub struct CursorContext<'a> { ... }
pub struct RenderContext<'a, R> { ... }
```

---

### Unit 2: Refactor Cursor Methods

Move cursor operations to use CursorContext:
1. `move_cursor_to_rect`
2. `move_cursor_to_focused_tile`
3. `maybe_warp_cursor_to_focus`

---

### Unit 3: Add Context Constructor to State

```rust
impl State {
    fn cursor_context(&mut self) -> CursorContext<'_> { ... }
}
```

---

### Unit 4: Update Call Sites

Update callers to use the context pattern or the State wrapper methods.

---

## Verification Checklist

- [ ] `contexts.rs` exists with context structs
- [ ] CursorContext operations work
- [ ] Existing tests still pass
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/contexts.rs` | +150 lines (new) |
| `src/niri/mod.rs` | +20 lines (context constructors) |
| Various files | Updated to use contexts |

---

## Benefits

1. **Clear dependencies**: Context struct shows exactly what's needed
2. **Testability**: Can create test contexts without full State
3. **Documentation**: Context fields document the API contract
4. **Borrow checker friendly**: Avoids conflicting borrows

---

## Why Optional?

This phase:
- Is more invasive than previous phases
- May not be needed if subsystems are sufficient
- Can be done incrementally later
- Requires updating many call sites

The subsystem extraction (P1-P7) already provides most benefits.

---

## Next Phase

After completing this phase, proceed to [Phase P9: Final Cleanup](phase-P9-cleanup.md).
