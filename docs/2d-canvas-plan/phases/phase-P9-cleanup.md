# Phase P9: Final Cleanup

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~30 minutes  
> **Risk Level**: 🟢 Low  
> **Prerequisite**: Phases P1-P8 complete

---

## Goal

Final cleanup pass to:
1. Move any remaining small utilities
2. Clean up unused imports
3. Verify all modules are <500 LOC
4. Update documentation
5. Verify target achieved: mod.rs <700 LOC

---

## Expected State After P8

```
src/niri/
├── mod.rs        ~1724 LOC  ← Still over target!
├── config.rs      ~400 LOC
├── render.rs      ~845 LOC  ← Over 500!
├── output.rs      ~587 LOC  ← Over 500!
├── init.rs        ~518 LOC  ← Over 500!
├── focus.rs       ~300 LOC
├── cursor.rs      ~200 LOC
├── dbus.rs        ~200 LOC
├── hit_test.rs    ~428 LOC
├── screenshot.rs  ~369 LOC
├── lock.rs        ~291 LOC
├── screencast.rs  ~290 LOC
├── types.rs       ~312 LOC
├── frame_callbacks ~252 LOC
├── screencopy.rs  ~210 LOC
├── pointer.rs     ~193 LOC
├── rules.rs        ~77 LOC
└── mru.rs          ~60 LOC
```

---

## Work Units

### Unit 1: Assess Remaining mod.rs Content

After P8, mod.rs should contain only:
- `Niri` struct definition (~237 lines)
- `State` struct definition (~5 lines)
- `NewClient` and `ClientState` structs (~20 lines)
- Core State methods that can't be moved:
  - `new()` - in init.rs ✓
  - `refresh_and_flush_clients()` (~30 lines)
  - `refresh()` (~50 lines)
  - `set_lid_closed()` (~10 lines)
  - `notify_blocker_cleared()` (~10 lines)
  - `confirm_mru()` (~5 lines) - might go to mru.rs
- Core Niri methods that can't be moved:
  - `insert_client()` (~20 lines)
  - `inhibit_power_key()` (~25 lines)
  - `find_output_and_workspace_index()` (~20 lines)
  - `find_window_by_id()` (~10 lines)
  - Various small utilities

**Action**: Identify what's actually left and whether any can still be moved.

---

### Unit 2: Move Small Utilities

Candidates for moving:

#### To mru.rs:
- `confirm_mru()` from State

#### To output.rs:
- `find_output_and_workspace_index()`
- `on_ipc_outputs_changed()` (if not in dbus.rs)

#### To lock.rs:
- `inhibit_power_key()` (D-Bus login1 related)

#### To types.rs:
- `NewClient` struct
- `ClientState` struct
- `ClientData` impl

---

### Unit 3: Handle Over-Limit Files

Some files may exceed 500 LOC:

#### render.rs (~845 LOC)
Consider splitting:
- `render_elements.rs` - OutputRenderElements type, scale_relocate_crop
- `render_impl.rs` - render(), render_layer()
- `redraw.rs` - redraw(), queue_redraw*()

#### output.rs (~587 LOC)
Consider:
- Keeping as-is (87 lines over is acceptable)
- Moving VRR-related code to separate file

#### init.rs (~518 LOC)
Acceptable - only 18 lines over limit.

---

### Unit 4: Clean Up Unused Imports

```bash
# Find unused imports
cargo check 2>&1 | grep "unused import"

# Fix them in each file
```

Common cleanup:
- Remove imports for moved functions
- Remove duplicate imports
- Organize imports (std, external, crate)

---

### Unit 5: Verify Line Counts

```bash
wc -l src/niri/*.rs | sort -n
```

Target state:
- mod.rs: <700 LOC ✓
- All other files: <600 LOC (soft limit 500)

---

### Unit 6: Update Documentation

1. Update `phases/README.md`:
   - Mark all phases as DONE
   - Update final line counts
   - Document any deviations from plan

2. Update team file with final status

3. Consider adding module-level documentation to each file

---

### Unit 7: Final Verification

```bash
# Full compilation check
cargo check

# Run all tests
cargo test

# Check for warnings
cargo clippy
```

---

## Verification Checklist

- [ ] mod.rs < 700 LOC
- [ ] No file > 650 LOC (allowing some flexibility)
- [ ] No unused imports
- [ ] `cargo check` passes cleanly
- [ ] `cargo test` passes (270 tests)
- [ ] `cargo clippy` has no new warnings
- [ ] README.md updated with final state

---

## Success Criteria

### Achieved ✓
- mod.rs reduced from 3554 to <700 LOC (~80% reduction)
- 19 focused modules instead of 1 monolith
- Each module has a clear single responsibility
- All tests pass
- Clean compilation with minimal warnings

### Final Architecture

```
src/niri/
├── mod.rs (~600)       # Niri + State structs, core initialization
├── types.rs (~350)     # All data types
├── config.rs (~400)    # Config reload
├── render.rs (~500)    # Rendering (after potential split)
├── output.rs (~550)    # Output management
├── init.rs (~520)      # Niri::new
├── cursor.rs (~200)    # Cursor movement
├── focus.rs (~300)     # Keyboard focus
├── dbus.rs (~200)      # D-Bus handlers
├── hit_test.rs (~430)  # Hit testing
├── screenshot.rs (~370)# Screenshots
├── lock.rs (~290)      # Session lock
├── screencast.rs (~290)# Screencast
├── frame_callbacks(~250)# Frame callbacks
├── screencopy.rs (~210)# Screencopy
├── pointer.rs (~200)   # Pointer constraints
├── rules.rs (~80)      # Window rules
└── mru.rs (~60)        # MRU switcher
                       ─────────
                       ~5300 LOC total (vs 7204 before)
```

---

## What Remains in mod.rs

After all phases, mod.rs should contain only:

1. **Struct definitions** (~260 lines):
   - `Niri` struct
   - `State` struct

2. **Core lifecycle methods** (~100 lines):
   - `State::refresh_and_flush_clients()`
   - `State::refresh()`
   - `State::set_lid_closed()`
   - `State::notify_blocker_cleared()`

3. **Small utilities** (~50 lines):
   - `Niri::insert_client()`
   - `Niri::find_window_by_id()`
   - Various one-liner delegators

4. **Module re-exports** (~50 lines):
   - `mod` declarations
   - `pub use` statements

5. **Imports** (~100 lines):
   - Reduced from original ~200 lines

**Total**: ~560-650 LOC

---

## Lessons Learned

Document any insights for future refactoring:
1. What patterns worked well
2. What was harder than expected
3. What could be improved in the architecture
