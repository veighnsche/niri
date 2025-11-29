# Phase P9: Final Cleanup and Documentation

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~30 minutes  
> **Risk Level**: 🟢 Low  
> **Prerequisite**: Phases P1-P7 complete (P8 optional)

---

## Goal

Final cleanup pass to:
1. Verify all subsystems are working
2. Clean up unused imports and dead code
3. Add documentation to new subsystems
4. Update project documentation
5. Verify target achieved: Niri fields < 50

---

## Expected State After P7

```
src/niri/
├── mod.rs (~400)           # Niri + State structs, coordination
├── subsystems/
│   ├── mod.rs (~50)        # Subsystem re-exports
│   ├── outputs.rs (~400)   # OutputSubsystem
│   ├── cursor.rs (~300)    # CursorSubsystem
│   ├── focus.rs (~350)     # FocusModel
│   ├── streaming.rs (~200) # StreamingSubsystem
│   └── ui.rs (~200)        # UiOverlays
├── protocols.rs (~150)     # ProtocolStates container
├── config.rs (~400)        # Config reload
├── init.rs (~450)          # Niri::new
├── render.rs (~400)        # Rendering
├── hit_test.rs (~400)      # Hit testing
├── lock.rs (~290)          # Session lock
├── screenshot.rs (~350)    # Screenshots
├── screencopy.rs (~200)    # Screencopy
├── screencast.rs (~250)    # Screencast
├── frame_callbacks.rs (~250) # Frame callbacks
├── pointer.rs (~200)       # Pointer constraints
├── rules.rs (~80)          # Window rules
├── mru.rs (~60)            # MRU switcher
└── types.rs (~300)         # Shared types
```

---

## Work Units

### Unit 1: Verify Subsystem Integration

Check that all subsystems are properly integrated:

```bash
# Verify compilation
cargo check

# Run all tests
cargo test

# Check for subsystem-related issues
grep -rn "TODO.*subsystem" src/niri/
```

---

### Unit 2: Clean Up Dead Code

Remove code that's no longer needed after subsystem extraction:

```bash
# Find unused functions
cargo clippy -- -W dead_code

# Find unused imports
cargo check 2>&1 | grep "unused import"
```

---

### Unit 3: Add Subsystem Documentation

Ensure each subsystem has proper module-level docs:

```rust
//! Output management subsystem.
//!
//! This module owns all state related to physical outputs (monitors):
//! - `global_space`: The compositor's global coordinate space
//! - `sorted_outputs`: Outputs sorted by name and position
//! - `state`: Per-output state (frame clock, redraw, etc.)
//! - `monitors_active`: Whether monitors are powered on
//! - `lid_closed`: Laptop lid state
//!
//! # Example
//!
//! ```ignore
//! // Add an output
//! niri.outputs.add(output, refresh_interval, vrr, &display_handle, &config);
//!
//! // Query outputs
//! if let Some((output, pos)) = niri.outputs.under_position(cursor_pos) {
//!     // ...
//! }
//! ```
```

---

### Unit 4: Update Phase Documentation

Update `phases/README.md`:
- Mark completed phases as DONE
- Update line counts
- Document any deviations

---

### Unit 5: Create Architecture Documentation

Create `src/niri/README.md` or update module docs:

```markdown
# niri/ Module Architecture

This module implements the core compositor state, organized into subsystems.

## Subsystems

### OutputSubsystem (`subsystems/outputs.rs`)
Manages physical outputs (monitors), their positions, and per-output state.

### CursorSubsystem (`subsystems/cursor.rs`)
Handles cursor visibility, positioning, and the cursor state machine.

### FocusModel (`subsystems/focus.rs`)
Computes and tracks keyboard focus based on priority rules.

### StreamingSubsystem (`subsystems/streaming.rs`)
Manages PipeWire streams for screencast and screencopy.

### UiOverlays (`subsystems/ui.rs`)
Groups modal UI elements (screenshot, hotkey, exit dialog, MRU).

### ProtocolStates (`protocols.rs`)
Container for all Smithay protocol states.

## Design Principles

1. **Ownership**: Each subsystem owns its state (private fields)
2. **Encapsulation**: Minimal public API
3. **Testability**: Subsystems can be tested in isolation
4. **Clear boundaries**: One responsibility per subsystem
```

---

### Unit 6: Final Verification

```bash
# Full compilation check
cargo check

# Run all tests
cargo test

# Check for warnings
cargo clippy

# Verify line counts
wc -l src/niri/*.rs src/niri/subsystems/*.rs | sort -n

# Count Niri fields (should be < 50)
grep -c "pub " src/niri/mod.rs
```

---

## Verification Checklist

- [ ] All subsystems compile and work
- [ ] `Niri` struct has < 50 fields (down from 100+)
- [ ] Each subsystem < 500 LOC
- [ ] No dead code or unused imports
- [ ] All modules have documentation
- [ ] `cargo check` passes cleanly
- [ ] `cargo test` passes (270 tests)
- [ ] `cargo clippy` has no new warnings

---

## Success Criteria

### Achieved ✓

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Niri fields | 100+ | <50 | 50%+ reduction |
| Largest impl block | 60+ methods | <20 | Split into subsystems |
| Testability | Requires full compositor | Subsystems testable | ✓ |
| Code organization | God object | Domain subsystems | ✓ |

### Final Niri Struct

```rust
pub struct Niri {
    // Core infrastructure (~10 fields)
    pub config: Rc<RefCell<Config>>,
    pub event_loop: LoopHandle<'static, State>,
    pub display_handle: DisplayHandle,
    pub clock: Clock,
    pub start_time: Instant,
    // ...
    
    // Domain subsystems (~6 fields)
    pub outputs: OutputSubsystem,
    pub cursor: CursorSubsystem,
    pub focus: FocusModel,
    pub streaming: StreamingSubsystem,
    pub ui: UiOverlays,
    pub protocols: ProtocolStates,
    
    // Already modular (~5 fields)
    pub layout: Layout<Mapped>,
    pub seat: Seat<State>,
    pub popups: PopupManager,
    // ...
    
    // Remaining (~20-30 fields)
    // Things that don't fit neatly into subsystems
}
```

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                          State                               │
│  ┌─────────────────────────────────────────────────────────┐│
│  │                          Niri                           ││
│  │  ┌───────────────┐  ┌───────────────┐  ┌─────────────┐ ││
│  │  │ OutputSubsys  │  │ CursorSubsys  │  │ FocusModel  │ ││
│  │  │  - space      │  │  - manager    │  │  - current  │ ││
│  │  │  - outputs    │  │  - visibility │  │  - layer    │ ││
│  │  │  - state      │  │  - contents   │  │  - inhibit  │ ││
│  │  └───────────────┘  └───────────────┘  └─────────────┘ ││
│  │  ┌───────────────┐  ┌───────────────┐  ┌─────────────┐ ││
│  │  │ StreamingSub  │  │  UiOverlays   │  │ProtocolStat │ ││
│  │  │  - casts      │  │  - screenshot │  │  - xdg_shell│ ││
│  │  │  - pipewire   │  │  - hotkey     │  │  - layer    │ ││
│  │  │  - mapped     │  │  - exit       │  │  - seat     │ ││
│  │  └───────────────┘  └───────────────┘  └─────────────┘ ││
│  │  ┌─────────────────────────────────────────────────────┐││
│  │  │                     Layout                          │││
│  │  │           (already its own module)                  │││
│  │  └─────────────────────────────────────────────────────┘││
│  └─────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────┐│
│  │                        Backend                          ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

---

## Lessons Learned

Document insights for future work:

1. **Subsystem pattern works**: Grouping related state + behavior is effective
2. **Start with low-risk phases**: ProtocolStates was a good warm-up
3. **Focus model is complex**: May need further iteration
4. **Distributed impl is anti-pattern**: Avoided this trap
5. **Testing improves**: Subsystems are independently testable

---

## Future Improvements

Consider for later:
- [ ] Add unit tests for each subsystem
- [ ] Extract more subsystems (Input, Window lifecycle)
- [ ] Implement Context pattern from Phase P8
- [ ] Further split large files (render.rs, init.rs)
