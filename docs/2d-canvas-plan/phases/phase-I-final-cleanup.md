# Phase I: Final Cleanup

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~30 minutes  
> **Risk Level**: 🟢 Low  
> **Prerequisite**: Phase H complete

---

## Goal

Final cleanup:
1. Remove dead code
2. Update documentation
3. Verify all tests pass
4. Count remaining Niri fields

---

## Work Units

### Unit 1: Count Fields

```bash
grep -c "^    pub " src/niri/mod.rs
```

**Target**: ~40 fields (down from 195)

---

### Unit 2: Verify Subsystem Structure

```
src/niri/subsystems/
├── mod.rs
├── outputs.rs (~400 LOC)
├── cursor.rs (~300 LOC)
├── focus.rs (~350 LOC)
├── streaming.rs (~200 LOC)
├── ui.rs (~150 LOC)
└── input.rs (~200 LOC)
```

---

### Unit 3: Clean Unused Imports

```bash
cargo check 2>&1 | grep "unused import"
```

Fix all warnings.

---

### Unit 4: Add Module Documentation

Each subsystem should have module-level docs:

```rust
//! Output management subsystem.
//!
//! This module owns all state related to physical outputs (monitors):
//! - Global coordinate space
//! - Sorted output list
//! - Per-output state
```

---

### Unit 5: Run Full Test Suite

```bash
cargo check
cargo test
cargo clippy
```

---

### Unit 6: Update Documentation

Update `README-FIX.md`:
- Mark all phases as COMPLETE
- Update actual field counts
- Document any deviations

---

## Verification Checklist

- [ ] Niri has ~40 fields (was 195)
- [ ] 7 subsystem structs created
- [ ] No unused imports
- [ ] All modules documented
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)
- [ ] `cargo clippy` passes

---

## Success Metrics

| Metric | Before | After |
|--------|--------|-------|
| Niri fields | 195 | ~40 |
| Subsystems | 0 | 7 |
| Private fields | 0 | ~80 |
| Testable units | 0 | 7 |

---

## Final Architecture

```rust
pub struct Niri {
    // Core infrastructure (~15 fields)
    pub config: Rc<RefCell<Config>>,
    pub event_loop: LoopHandle<'static, State>,
    pub display_handle: DisplayHandle,
    pub clock: Clock,
    pub start_time: Instant,
    pub socket_name: Option<OsString>,
    pub is_session_instance: bool,
    pub is_at_startup: bool,
    
    // Domain subsystems (~7 fields)
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
    
    // Remaining (~15 fields)
    // Things that don't fit neatly into subsystems
    pub unmapped_windows: HashMap<...>,
    pub mapped_layer_surfaces: HashMap<...>,
    pub devices: HashSet<input::Device>,
    pub tablets: HashMap<...>,
    pub suppressed_keys: HashSet<Keycode>,
    // etc.
}
```

---

## What's Next?

After completing the A-I fix phases:

1. **Re-evaluate P-phases** — May no longer be needed
2. **Proceed to features** — Camera zoom, bookmarks, etc.
3. **Add unit tests** — For each subsystem
