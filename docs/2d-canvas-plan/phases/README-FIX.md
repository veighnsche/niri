# Fix Phases (A-O): Domain-Driven Decomposition

> **Status**: 🔴 **NOT STARTED**  
> **Priority**: 🔴 **URGENT** — Must be done before continuing P-phases  
> **Goal**: Transform Niri God Object (195 pub fields!) into composable subsystems  
> **Approach**: Extract **owned subsystems**, not more distributed `impl` blocks

---

## 🚨 THE PROBLEM: Previous Teams Made It Worse

Previous teams (TEAM_067-069) created 13 extracted modules:
```
src/niri/
├── mod.rs (3554 LOC) ← Still massive!
├── output.rs (287 LOC)
├── render.rs (415 LOC)
├── hit_test.rs (428 LOC)
├── init.rs (518 LOC)
├── lock.rs (291 LOC)
├── pointer.rs (193 LOC)
├── screenshot.rs (369 LOC)
├── screencopy.rs (210 LOC)
├── screencast.rs (290 LOC)
├── frame_callbacks.rs (252 LOC)
├── rules.rs (77 LOC)
├── mru.rs (60 LOC)
└── types.rs (260 LOC)
```

**But they all use the same anti-pattern**:
```rust
// Every extracted file does this:
impl Niri {
    pub fn some_method(&mut self, ...) {
        // Still accesses ALL 195 fields freely via self.*
    }
}
```

This achieves:
- ✅ Smaller files
- ❌ **No reduction in coupling** (every method still touches everything)
- ❌ **No encapsulation** (all 195 fields still `pub`)
- ❌ **No testability** (can't test without full compositor)

---

## THE SOLUTION: Owned Subsystems

Instead of distributing `impl Niri` blocks, **extract state into owned subsystems**:

### Current (Anti-Pattern)
```rust
pub struct Niri {
    // 195 pub fields mixed together
    pub global_space: Space<Window>,
    pub sorted_outputs: Vec<Output>,
    pub output_state: HashMap<Output, OutputState>,
    pub cursor_manager: CursorManager,
    pub pointer_visibility: PointerVisibility,
    pub pointer_contents: PointContents,
    // ... 189 more fields
}

// In output.rs - methods access everything
impl Niri {
    pub fn output_under(&self, pos: Point) {
        self.global_space.output_under(pos) // accesses self.*
    }
}
```

### Target (Owned Subsystems)
```rust
pub struct Niri {
    // ~30 core fields
    pub config: Rc<RefCell<Config>>,
    pub event_loop: LoopHandle<'static, State>,
    pub display_handle: DisplayHandle,
    pub clock: Clock,
    
    // Owned subsystems (~6 fields)
    pub outputs: OutputSubsystem,      // Owns output state
    pub cursor: CursorSubsystem,       // Owns cursor state
    pub focus: FocusModel,             // Owns focus state
    pub streaming: StreamingSubsystem, // Owns PipeWire/cast state
    pub ui: UiOverlays,                // Owns UI overlay state
    pub input: InputTracking,          // Owns input tracking state
    pub protocols: ProtocolStates,     // Owns Smithay protocol states
    
    // Already modular
    pub layout: Layout<Mapped>,
    pub seat: Seat<State>,
}

// In subsystems/outputs.rs - methods only access owned state
pub struct OutputSubsystem {
    global_space: Space<Window>,      // PRIVATE
    sorted: Vec<Output>,              // PRIVATE
    state: HashMap<Output, OutputState>, // PRIVATE
}

impl OutputSubsystem {
    pub fn under_position(&self, pos: Point) -> Option<&Output> {
        self.global_space.output_under(pos) // only accesses owned state
    }
}
```

---

## FIX PHASES

| Phase | Description | Fields Moved | Risk |
|-------|-------------|--------------|------|
| **[A](phase-A-protocol-states.md)** | Group 35+ Smithay protocol states | ~35 | 🟢 Low |
| **[B](phase-B-output-subsystem.md)** | Extract OutputSubsystem | ~8 | 🟡 Medium |
| **[C](phase-C-cursor-subsystem.md)** | Extract CursorSubsystem | ~15 | 🟡 Medium |
| **[D](phase-D-focus-model.md)** | Extract FocusModel | ~5 | 🔴 High |
| **[E](phase-E-streaming-subsystem.md)** | Extract StreamingSubsystem | ~6 | 🟢 Low |
| **[F](phase-F-ui-overlays.md)** | Extract UiOverlays | ~8 | 🟢 Low |
| **[G](phase-G-input-tracking.md)** | Extract InputTracking | ~12 | 🟡 Medium |
| **[H](phase-H-config-refactor.md)** | Refactor to use subsystems | 0 | 🟡 Medium |
| **[I](phase-I-final-cleanup.md)** | Final cleanup and docs | 0 | 🟢 Low |

**Expected result**: Niri from ~195 fields → ~40 fields

---

## EXECUTION ORDER

### Phase A: ProtocolStates (First - Easiest)
Group the 35+ Smithay protocol state fields:
```rust
// Before: 35 fields in Niri
pub compositor_state: CompositorState,
pub xdg_shell_state: XdgShellState,
pub layer_shell_state: WlrLayerShellState,
// ... 32 more protocol states

// After: 1 field in Niri
pub protocols: ProtocolStates,
```

### Phase B: OutputSubsystem (Most Impact)
Move output-related fields and methods:
```rust
// Before: 8 fields scattered
pub global_space: Space<Window>,
pub sorted_outputs: Vec<Output>,
pub output_state: HashMap<Output, OutputState>,
pub monitors_active: bool,
pub is_lid_closed: bool,
// + OutputState struct

// After: 1 field
pub outputs: OutputSubsystem,
```

### Phase C: CursorSubsystem
Move cursor/pointer fields:
```rust
// Before: 15 fields
pub cursor_manager: CursorManager,
pub cursor_texture_cache: CursorTextureCache,
pub pointer_visibility: PointerVisibility,
pub pointer_contents: PointContents,
pub pointer_inactivity_timer: ...
pub tablet_cursor_location: ...
pub dnd_icon: ...
// etc.

// After: 1 field
pub cursor: CursorSubsystem,
```

### Phase D: FocusModel (Highest Risk)
Move focus-related fields and refactor `update_keyboard_focus()`:
```rust
// Before: 5 fields
pub keyboard_focus: KeyboardFocus,
pub layer_shell_on_demand_focus: Option<LayerSurface>,
pub idle_inhibiting_surfaces: HashSet<WlSurface>,
pub keyboard_shortcuts_inhibiting_surfaces: HashMap<...>,
// + 260-line update_keyboard_focus() monster

// After: 1 field + testable focus logic
pub focus: FocusModel,
```

### Phase E: StreamingSubsystem
Move PipeWire/screencast fields:
```rust
// Before: 6 fields (some feature-gated)
pub casts: Vec<Cast>,
pub pipewire: Option<PipeWire>,
pub pw_to_niri: Sender<PwToNiri>,
pub mapped_cast_output: HashMap<Window, Output>,
pub dynamic_cast_id_for_portal: MappedId,

// After: 1 field
pub streaming: StreamingSubsystem,
```

### Phase F: UiOverlays
Move UI overlay fields:
```rust
// Before: 8 fields
pub screenshot_ui: ScreenshotUi,
pub config_error_notification: ConfigErrorNotification,
pub hotkey_overlay: HotkeyOverlay,
pub exit_confirm_dialog: ExitConfirmDialog,
pub window_mru_ui: WindowMruUi,
pub pending_mru_commit: Option<...>,
pub pick_window: Option<Sender<...>>,
pub pick_color: Option<Sender<...>>,

// After: 1 field
pub ui: UiOverlays,
```

### Phase G: InputTracking
Move scroll/gesture tracking:
```rust
// Before: 12 fields
pub gesture_swipe_3f_cumulative: Option<(f64, f64)>,
pub overview_scroll_swipe_gesture: ScrollSwipeGesture,
pub vertical_wheel_tracker: ScrollTracker,
pub horizontal_wheel_tracker: ScrollTracker,
pub mods_with_mouse_binds: HashSet<Modifiers>,
pub mods_with_wheel_binds: HashSet<Modifiers>,
pub vertical_finger_scroll_tracker: ScrollTracker,
pub horizontal_finger_scroll_tracker: ScrollTracker,
pub mods_with_finger_scroll_binds: HashSet<Modifiers>,
// etc.

// After: 1 field
pub input: InputTracking,
```

### Phase H: Config Refactor
Update config reload to use subsystems:
```rust
// Before: reload_config() touches 50+ fields directly
// After: reload_config() calls subsystem methods
fn reload_config(&mut self, config: Config) {
    self.outputs.apply_config(&config);
    self.cursor.apply_config(&config);
    self.ui.apply_config(&config);
    // etc.
}
```

### Phase I: Final Cleanup
- Remove dead code
- Update documentation
- Verify all tests pass

---

## SUCCESS CRITERIA

| Metric | Before | After |
|--------|--------|-------|
| Niri fields | 195 | ~40 |
| Subsystem structs | 0 | 7 |
| Fields encapsulated | 0% | 80%+ |
| Testable subsystems | 0 | 7 |

---

## FILE STRUCTURE

### Before
```
src/niri/
├── mod.rs (3554 LOC) ← God Object
├── output.rs         ← impl Niri methods
├── render.rs         ← impl Niri methods
└── ... (12 more impl Niri files)
```

### After
```
src/niri/
├── mod.rs (~600 LOC)        # Clean Niri + State
├── subsystems/
│   ├── mod.rs               # Re-exports
│   ├── outputs.rs (~400)    # OutputSubsystem
│   ├── cursor.rs (~300)     # CursorSubsystem
│   ├── focus.rs (~350)      # FocusModel
│   ├── streaming.rs (~250)  # StreamingSubsystem
│   ├── ui.rs (~200)         # UiOverlays
│   └── input.rs (~250)      # InputTracking
├── protocols.rs (~150)      # ProtocolStates
├── config.rs (~350)         # Config reload
├── init.rs (~450)           # Niri::new
└── (existing files kept)    # render.rs, hit_test.rs, etc.
```

---

## MIGRATION STRATEGY

For each subsystem extraction:

1. **Create subsystem struct** with private fields
2. **Create subsystem module** in `subsystems/`
3. **Move fields** from Niri to subsystem
4. **Add subsystem field** to Niri
5. **Update access patterns** throughout codebase
6. **Move related methods** to subsystem impl
7. **Verify**: `cargo check` && `cargo test`

---

## QUICK REFERENCE

| Command | Purpose |
|---------|---------|
| `cargo check` | Verify compilation |
| `cargo test` | Run all tests |
| `grep -c "pub " src/niri/mod.rs` | Count pub fields |
| `wc -l src/niri/*.rs` | Check line counts |

---

## RELATIONSHIP TO P-PHASES

The **P-phases** (phase-P1 through phase-P9) are now **obsolete**. They were created with the same anti-pattern (distributed `impl` blocks).

After completing the **A-I fix phases**, the architecture will be clean enough that:
- Further splitting may not be necessary
- If needed, can proceed with P-phases using the new subsystem pattern

**Order of operations**:
1. ✅ Complete A-I fix phases (this plan)
2. ⏸️ Re-evaluate if P-phases are still needed
3. ➡️ Proceed to feature work (zoom, bookmarks, etc.)
