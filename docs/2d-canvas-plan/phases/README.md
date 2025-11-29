# niri/mod.rs Refactor — Domain-Driven Decomposition

> **Status**: 🔴 **NOT STARTED**  
> **Goal**: Transform God Object into composable subsystems  
> **Approach**: Domain-Driven Decomposition (not file reorganization)

---

## 🚨 The Real Problem

The current `Niri` struct is a **God Object** with:
- **237 lines** of field definitions
- **100+ fields** mixing unrelated domains
- **60+ methods** scattered across 13 files via distributed `impl Niri` blocks

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
- Owns its related state
- Exposes a minimal public API
- Can be tested in isolation
- Has clear boundaries

### Current vs Proposed

| Aspect | Current (Distributed impl) | Proposed (Subsystems) |
|--------|---------------------------|----------------------|
| Coupling | All methods touch all fields | Methods only touch owned state |
| Testability | Requires full compositor | Subsystems testable in isolation |
| Encapsulation | None (all `pub`) | Subsystems hide implementation |
| Discoverability | "Where is cursor code?" → grep | `niri.cursor.*` |
| Adding features | Touches god object | Touches subsystem |

---

## Phase Plan

| Phase | Description | Subsystem Created | Risk |
|-------|-------------|-------------------|------|
| **P1** | Extract ProtocolStates container | `ProtocolStates` | 🟢 Low |
| **P2** | Extract OutputSubsystem | `OutputSubsystem` | 🟡 Medium |
| **P3** | Extract CursorSubsystem | `CursorSubsystem` | 🟡 Medium |
| **P4** | Extract FocusModel | `FocusModel` | 🔴 High |
| **P5** | Extract StreamingSubsystem | `StreamingSubsystem` | 🟢 Low |
| **P6** | Extract UiOverlays | `UiOverlays` | 🟢 Low |
| **P7** | Extract ConfigManager | `ConfigManager` | 🟡 Medium |
| **P8** | Refactor State to use Context pattern | `StateContext` | 🟡 Medium |
| **P9** | Final cleanup and documentation | - | 🟢 Low |

---

## Target Architecture

### Before (God Object)
```rust
pub struct Niri {
    // 100+ fields mixed together
    pub config: Rc<RefCell<Config>>,
    pub cursor_manager: CursorManager,
    pub cursor_texture_cache: CursorTextureCache,
    pub pointer_visibility: PointerVisibility,
    pub pointer_contents: PointContents,
    pub pointer_inactivity_timer: Option<RegistrationToken>,
    // ... 95+ more fields
}
```

### After (Composable Subsystems)
```rust
pub struct Niri {
    // Core infrastructure
    pub config: Rc<RefCell<Config>>,
    pub event_loop: LoopHandle<'static, State>,
    pub display_handle: DisplayHandle,
    pub clock: Clock,
    
    // Domain subsystems (owned, encapsulated)
    pub outputs: OutputSubsystem,
    pub cursor: CursorSubsystem,
    pub focus: FocusModel,
    pub streaming: StreamingSubsystem,
    pub ui: UiOverlays,
    
    // Already modular
    pub layout: Layout<Mapped>,
    pub seat: Seat<State>,
    
    // Grouped protocol states
    pub protocols: ProtocolStates,
}
```

---

## Detailed Phases

### [Phase P1: ProtocolStates Container](phase-P1-protocol-states.md)
Group 25+ Smithay protocol states into one container struct.
- **Creates**: `ProtocolStates` struct
- **Risk**: 🟢 Low (pure mechanical grouping)
- **Time**: ~30 minutes

### [Phase P2: OutputSubsystem](phase-P2-output-subsystem.md)
Extract output-related fields and methods into owned subsystem.
- **Creates**: `OutputSubsystem` struct with encapsulated state
- **Risk**: 🟡 Medium (most impactful change)
- **Time**: ~2 hours

### [Phase P3: CursorSubsystem](phase-P3-cursor-subsystem.md)
Extract cursor/pointer state into state machine with proper API.
- **Creates**: `CursorSubsystem` struct
- **Risk**: 🟡 Medium (state machine design)
- **Time**: ~1.5 hours

### [Phase P4: FocusModel](phase-P4-focus-model.md)
Extract focus logic into dedicated domain model.
- **Creates**: `FocusModel` struct
- **Risk**: 🔴 High (complex focus logic)
- **Time**: ~2 hours

### [Phase P5: StreamingSubsystem](phase-P5-streaming.md)
Group PipeWire, casts, screencopy into streaming subsystem.
- **Creates**: `StreamingSubsystem` struct
- **Risk**: 🟢 Low (already somewhat isolated)
- **Time**: ~1 hour

### [Phase P6: UiOverlays](phase-P6-ui-overlays.md)
Group UI overlay state (screenshot, hotkey, exit dialog, MRU).
- **Creates**: `UiOverlays` struct
- **Risk**: 🟢 Low (minimal coupling)
- **Time**: ~45 minutes

### [Phase P7: ConfigManager](phase-P7-config-manager.md)
Extract config reload logic into dedicated manager.
- **Creates**: `ConfigManager` or config reload methods
- **Risk**: 🟡 Medium (many interactions)
- **Time**: ~1.5 hours

### [Phase P8: State Context Pattern](phase-P8-state-context.md)
Refactor `State` to pass context instead of `&mut self`.
- **Refactors**: `impl State` methods to use context pattern
- **Risk**: 🟡 Medium (API changes)
- **Time**: ~2 hours

### [Phase P9: Final Cleanup](phase-P9-cleanup.md)
Documentation, remove dead code, final verification.
- **Risk**: 🟢 Low
- **Time**: ~30 minutes

---

## Success Criteria

### Architecture Goals ✓
- [ ] `Niri` struct < 50 fields (down from 100+)
- [ ] Each subsystem independently testable
- [ ] Clear ownership boundaries
- [ ] No distributed `impl Niri` blocks for unrelated functionality

### Technical Goals ✓
- [ ] `cargo check` passes
- [ ] All 270 tests pass
- [ ] No circular dependencies
- [ ] Each module < 500 LOC

### Final File Structure
```
src/niri/
├── mod.rs (~400)           # Niri + State structs, coordination
├── subsystems/
│   ├── mod.rs (~50)        # Subsystem re-exports
│   ├── outputs.rs (~400)   # OutputSubsystem
│   ├── cursor.rs (~300)    # CursorSubsystem
│   ├── focus.rs (~350)     # FocusModel
│   ├── streaming.rs (~300) # StreamingSubsystem
│   └── ui.rs (~200)        # UiOverlays
├── protocols.rs (~150)     # ProtocolStates container
├── config.rs (~350)        # Config reload logic
├── init.rs (~450)          # Niri::new
├── render.rs (~400)        # Rendering coordination
├── hit_test.rs (~400)      # Hit testing
├── lock.rs (~290)          # Session lock
├── screenshot.rs (~350)    # Screenshots
├── screencopy.rs (~200)    # Screencopy protocol
├── screencast.rs (~250)    # Screencast
├── frame_callbacks.rs (~250) # Frame callbacks
├── pointer.rs (~200)       # Pointer constraints
├── rules.rs (~80)          # Window rules
├── mru.rs (~60)            # MRU switcher
└── types.rs (~300)         # Shared types
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
    manager: CursorManager,
    visibility: PointerVisibility,
    // ... all cursor state private
}
impl CursorSubsystem {
    pub fn move_to(&mut self, pos: Point) { ... }
}
```

### 2. Minimal Public API
```rust
// ❌ Bad: All fields public
pub struct OutputSubsystem {
    pub global_space: Space<Window>,
    pub sorted_outputs: Vec<Output>,
    pub output_state: HashMap<Output, OutputState>,
}

// ✅ Good: Private fields, intentional API
pub struct OutputSubsystem {
    global_space: Space<Window>,
    sorted_outputs: Vec<Output>,
    state: HashMap<Output, OutputState>,
}
impl OutputSubsystem {
    pub fn add(&mut self, output: Output, config: &OutputConfig) { ... }
    pub fn remove(&mut self, output: &Output) { ... }
    pub fn under_position(&self, pos: Point) -> Option<&Output> { ... }
}
```

### 3. Testable in Isolation
```rust
#[test]
fn test_focus_priority() {
    // Can test FocusModel without full compositor
    let mut focus = FocusModel::default();
    focus.set_layer_focus(Some(layer_surface));
    assert_eq!(focus.current(), KeyboardFocus::LayerShell { ... });
}
```

---

## Quick Reference

| Command | Purpose |
|---------|---------|
| `cargo check` | Verify compilation |
| `cargo test` | Run all tests |
| `wc -l src/niri/*.rs` | Check line counts |
| `grep -c "pub " src/niri/mod.rs` | Count public fields |

---

## Team Registration

Before starting a phase:
1. Check `.teams/` for latest team number
2. Create your team file: `.teams/TEAM_XXX_phase_name.md`
3. Follow the phase instructions
4. Update this README with progress
