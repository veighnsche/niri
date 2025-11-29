# niri/mod.rs Refactor — Domain-Driven Decomposition

> **Status**: 🔴 **NOT STARTED**  
> **Goal**: Transform God Object into composable subsystems  
> **Approach**: Domain-Driven Decomposition (not file reorganization)  
> **Current State**: `mod.rs` is 3554 LOC with **195 pub fields**

---

## 🚨 The Real Problem

The current `Niri` struct is a **God Object** with:
- **195 pub fields** mixing unrelated domains (verified via `grep -c "^    pub " mod.rs`)
- **60+ methods** scattered across 13 files via distributed `impl Niri` blocks
- Previous teams (TEAM_067-069) split files but used the **anti-pattern** — they didn't create subsystems

**Moving methods to different files doesn't fix this.** It's still one massive struct that everything depends on.

### What's Wrong with Distributed impl Blocks

```rust
// output.rs
impl Niri { fn output_under() {...} }

// render.rs  
impl Niri { fn pointer_element() {...} }

// cursor.rs
impl Niri { fn move_cursor() {...} }
```

This achieves:
- ✅ Smaller files
- ❌ No reduction in coupling
- ❌ No improvement in testability
- ❌ No encapsulation
- ❌ No separation of concerns

Every module still accesses every field of `Niri`. There's no architectural improvement.

---

## The Solution: Domain Subsystems

Instead of one God Object, decompose into **owned subsystems** where each:
- Owns its related state (private fields)
- Exposes a minimal public API
- Can be tested in isolation
- Has clear boundaries

### Current vs Proposed

| Aspect | Current (Distributed impl) | Proposed (Subsystems) |
|--------|---------------------------|----------------------|
| Coupling | All methods touch all fields | Methods only touch owned state |
| Testability | Requires full compositor | Subsystems testable in isolation |
| Encapsulation | None (all 195 fields `pub`) | Subsystems hide implementation |
| Discoverability | "Where is cursor code?" → grep | `niri.cursor.*` |
| Adding features | Touches god object | Touches subsystem |

---

## Phase Plan

| Phase | Description | Fields Moved | Risk | Time |
|-------|-------------|--------------|------|------|
| **[P1](phase-P1-protocol-states.md)** | Extract ProtocolStates container | ~35 | 🟢 Low | 30m |
| **[P2](phase-P2-output-subsystem.md)** | Extract OutputSubsystem | ~8 | 🟡 Medium | 2h |
| **[P3](phase-P3-cursor-subsystem.md)** | Extract CursorSubsystem | ~10 | 🟡 Medium | 1.5h |
| **[P4a](phase-P4a-focus-state.md)** | Extract FocusState container | ~4 | 🟢 Low | 30m |
| **[P4b](phase-P4b-focus-computation.md)** | Extract focus computation logic | 0 | 🟡 Medium | 45m |
| **[P4c](phase-P4c-focus-change.md)** | Extract focus change handling | 0 | 🟡 Medium | 30m |
| **[P4d](phase-P4d-layer-cleanup.md)** | Extract layer on-demand cleanup | 0 | 🟢 Low | 15m |
| **[P5](phase-P5-streaming.md)** | Extract StreamingSubsystem | ~6 | 🟢 Low | 1h |
| **[P6](phase-P6-ui-overlays.md)** | Extract UiOverlays | ~8 | 🟢 Low | 45m |
| **[P7](phase-P7-config-manager.md)** | Refactor Config Reload | 0 | 🟡 Medium | 1.5h |
| **[P7.5](phase-P7.5-input-tracking.md)** | Extract InputTracking | ~12 | 🟡 Medium | 1h |
| **[P9](phase-P9-cleanup.md)** | Final cleanup and documentation | 0 | 🟢 Low | 30m |

**Expected result**: Niri from **195 fields → ~40 fields**  
**Total estimated time**: ~11 hours

> **Note**: Phase P4 was split into P4a-P4d to reduce risk. Phase P8 (StateContext) was removed — it's over-engineering.

---

## Target Architecture

### Before (God Object)
```rust
pub struct Niri {
    // 195 pub fields mixed together
    pub config: Rc<RefCell<Config>>,
    pub cursor_manager: CursorManager,
    pub cursor_texture_cache: CursorTextureCache,
    pub pointer_visibility: PointerVisibility,
    pub global_space: Space<Window>,
    pub sorted_outputs: Vec<Output>,
    // ... 189 more fields
}
```

### After (Composable Subsystems)
```rust
pub struct Niri {
    // Core infrastructure (~15 fields)
    pub config: Rc<RefCell<Config>>,
    pub event_loop: LoopHandle<'static, State>,
    pub display_handle: DisplayHandle,
    pub clock: Clock,
    
    // Domain subsystems (~7 fields, own ~85 fields internally)
    pub outputs: OutputSubsystem,
    pub cursor: CursorSubsystem,
    pub focus: FocusModel,
    pub streaming: StreamingSubsystem,
    pub ui: UiOverlays,
    pub input: InputTracking,
    pub protocols: ProtocolStates,
    
    // Already modular (~5 fields)
    pub layout: Layout<Mapped>,
    pub seat: Seat<State>,
    pub popups: PopupManager,
    
    // Remaining (~15 fields that don't fit neatly)
}
```

---

## Success Criteria

### Architecture Goals
- [ ] `Niri` struct < 50 fields (down from 195)
- [ ] 7 subsystem structs created
- [ ] Each subsystem independently testable
- [ ] Clear ownership boundaries
- [ ] No distributed `impl Niri` blocks for unrelated functionality

### Technical Goals
- [ ] `cargo check` passes
- [ ] All tests pass
- [ ] No circular dependencies
- [ ] Each module < 500 LOC

### Final File Structure
```
src/niri/
├── mod.rs (~600)           # Clean Niri + State structs
├── subsystems/
│   ├── mod.rs (~50)        # Subsystem re-exports
│   ├── outputs.rs (~400)   # OutputSubsystem
│   ├── cursor.rs (~300)    # CursorSubsystem
│   ├── focus.rs (~350)     # FocusModel
│   ├── streaming.rs (~250) # StreamingSubsystem
│   ├── ui.rs (~200)        # UiOverlays
│   └── input.rs (~250)     # InputTracking
├── protocols.rs (~150)     # ProtocolStates container
├── config.rs (~350)        # Config reload logic
├── init.rs (~450)          # Niri::new
└── (existing files kept)   # render.rs, hit_test.rs, etc.
```

---

## Key Design Principles

### 1. Ownership, Not Access
```rust
// ❌ Bad: Niri owns cursor fields, methods scattered
impl Niri {
    pub cursor_manager: CursorManager,      // in Niri
    pub fn move_cursor() { ... }             // in cursor.rs
}

// ✅ Good: CursorSubsystem owns everything cursor-related
pub struct CursorSubsystem {
    manager: CursorManager,        // PRIVATE
    visibility: PointerVisibility, // PRIVATE
}
impl CursorSubsystem {
    pub fn move_to(&mut self, pos: Point) { ... }
}
```

### 2. Minimal Public API
```rust
// ❌ Bad: All fields public (current state)
pub struct Niri {
    pub global_space: Space<Window>,
    pub sorted_outputs: Vec<Output>,
}

// ✅ Good: Private fields, intentional API
pub struct OutputSubsystem {
    global_space: Space<Window>,      // private
    sorted_outputs: Vec<Output>,      // private
}
impl OutputSubsystem {
    pub fn add(&mut self, output: Output, config: &Config) { ... }
    pub fn under_position(&self, pos: Point) -> Option<&Output> { ... }
}
```

### 3. Testable in Isolation
```rust
#[test]
fn test_focus_priority() {
    // Can test FocusModel without full compositor
    let mut focus = FocusModel::default();
    let ctx = FocusContext { exit_dialog_open: true, ..Default::default() };
    assert_eq!(focus.compute(&ctx), KeyboardFocus::ExitConfirmDialog);
}
```

---

## Quick Reference

| Command | Purpose |
|---------|---------|
| `cargo check` | Verify compilation |
| `cargo test` | Run all tests |
| `wc -l src/niri/*.rs` | Check line counts |
| `grep -c "^    pub " src/niri/mod.rs` | Count pub fields (currently 195) |

---

## Team Registration

Before starting a phase:
1. Check `.teams/` for latest team number
2. Create your team file: `.teams/TEAM_XXX_phase_name.md`
3. Follow the phase instructions
4. Update this README with progress
