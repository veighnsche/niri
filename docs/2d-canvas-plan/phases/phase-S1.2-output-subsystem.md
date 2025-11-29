# Phase S1.2: OutputSubsystem Logic Migration

> **Goal**: Move all output management logic FROM `mod.rs` INTO `OutputSubsystem`

## Current State

OutputSubsystem has:
- ✅ Private fields (space, sorted, state, monitors_active, lid_closed)
- ✅ Accessor methods (state(), states(), space(), iter(), etc.)
- ❌ No actual logic - just data container with getters/setters

Logic still lives in:
- `src/niri/mod.rs` - add_output, remove_output, reposition_outputs
- `src/niri/output.rs` - output_resized, activate_monitors, deactivate_monitors

---

## Methods to Move

### 1. `reposition_outputs` (mod.rs:1883-1999) - ~116 LOC

**Current signature:**
```rust
impl Niri {
    pub fn reposition_outputs(&mut self, new_output: Option<&Output>) { ... }
}
```

**Target signature:**
```rust
impl OutputSubsystem {
    /// Repositions all outputs based on config. Call after add/remove.
    pub fn reposition(&mut self, new_output: Option<&Output>, config: &Config) -> Vec<Output> {
        // Returns outputs that changed position (for redraw)
    }
}
```

**Dependencies:**
- `config.outputs.find(name)` - pass config as parameter
- `self.global_space` → `self.space`
- `self.sorted_outputs` → `self.sorted`
- `self.ipc_outputs_changed` → return changed outputs, caller sets flag
- `self.queue_redraw(&output)` → return changed outputs, caller queues redraw

**Migration steps:**
1. Copy method body to `OutputSubsystem::reposition()`
2. Change `self.global_space` to `self.space`
3. Change `self.sorted_outputs` to `self.sorted`
4. Remove `self.ipc_outputs_changed = true` - return changed outputs instead
5. Remove `self.queue_redraw()` calls - return changed outputs instead
6. Update `Niri::reposition_outputs()` to call subsystem and handle returns

---

### 2. `add_output` (mod.rs:2005-2099) - ~94 LOC

**Current signature:**
```rust
impl Niri {
    pub fn add_output(&mut self, output: Output, refresh_interval: Option<Duration>, vrr: bool) { ... }
}
```

**Target signature:**
```rust
impl OutputSubsystem {
    /// Adds a new output. Returns the global ID and whether reposition is needed.
    pub fn add(
        &mut self,
        output: Output,
        refresh_interval: Option<Duration>,
        vrr: bool,
        config: &OutputConfig,
        display_handle: &DisplayHandle,
        event_loop: &LoopHandle<'static, State>,
    ) -> GlobalId {
        // Creates global, sets up state, returns global ID
    }
}
```

**Dependencies:**
- `output.create_global::<State>(&self.display_handle)` - pass display_handle
- `self.config.borrow()` - pass config
- `self.layout.add_output()` - caller handles this
- `self.config.borrow().rows` - pass config
- `self.is_locked()` - pass lock state
- `self.event_loop` - pass event_loop
- `self.reposition_outputs()` - caller handles this

**Migration steps:**
1. Split into two parts:
   - `OutputSubsystem::add()` - handles output state creation
   - `Niri::add_output()` - orchestrates layout, rows, reposition
2. Move OutputState creation logic into subsystem
3. Keep layout/row/reposition orchestration in Niri

---

### 3. `remove_output` (mod.rs:2101-2175) - ~74 LOC

**Current signature:**
```rust
impl Niri {
    pub fn remove_output(&mut self, output: &Output) { ... }
}
```

**Target signature:**
```rust
impl OutputSubsystem {
    /// Removes an output. Returns the OutputState for cleanup.
    pub fn remove(&mut self, output: &Output) -> Option<OutputState> {
        // Unmaps from space, removes from sorted, removes state
    }
}
```

**Dependencies:**
- `layer_map_for_output(output)` - caller handles layer cleanup
- `self.layout.remove_output(output)` - caller handles
- `self.protocols.gamma_control.output_removed(output)` - caller handles
- `self.event_loop.remove(token)` - caller handles timer cleanup
- `self.stop_casts_for_target()` - caller handles
- `self.remove_screencopy_output()` - caller handles
- `self.display_handle.disable_global()` - caller handles
- Lock state handling - caller handles

**Migration steps:**
1. `OutputSubsystem::remove()` does:
   - Unmap from space
   - Remove from sorted list
   - Remove and return OutputState
2. `Niri::remove_output()` handles:
   - Layer surface cleanup
   - Layout removal
   - Protocol cleanup
   - Timer cleanup
   - Screencast/screencopy cleanup
   - Global disable/remove
   - Lock state handling

---

### 4. `output_resized` (output.rs:225-263) - ~38 LOC

**Current location:** `src/niri/output.rs`

**Target signature:**
```rust
impl OutputSubsystem {
    /// Called when an output is resized. Updates backdrop buffers.
    pub fn resized(&mut self, output: &Output) {
        // Updates backdrop and lock color buffers
    }
}
```

**Migration steps:**
1. Move method body to OutputSubsystem
2. Update call sites

---

### 5. `activate_monitors` / `deactivate_monitors` (output.rs:267-286) - ~20 LOC

**Current location:** `src/niri/output.rs`

**Target signature:**
```rust
impl OutputSubsystem {
    /// Activates all monitors.
    pub fn activate(&mut self, backend: &mut Backend) {
        if !self.monitors_active { return; }
        self.monitors_active = true;
        backend.set_monitors_active(true);
    }
    
    /// Deactivates all monitors.
    pub fn deactivate(&mut self, backend: &mut Backend) {
        if self.monitors_active { return; }
        self.monitors_active = false;
        backend.set_monitors_active(false);
    }
}
```

**Note:** These still need Backend reference - consider if this is the right place.

---

## Implementation Order

1. **Move `output_resized`** - simplest, few dependencies
2. **Move `activate/deactivate_monitors`** - simple, consider Backend coupling
3. **Move `remove` logic** - medium complexity
4. **Move `add` logic** - complex, many dependencies
5. **Move `reposition` logic** - complex, most dependencies

---

## File Changes

### `src/niri/subsystems/outputs.rs`
- Add `reposition()` method (~60 LOC after refactor)
- Add `add_state()` method (~30 LOC)
- Add `remove()` method (~15 LOC)
- Add `resized()` method (~20 LOC)

### `src/niri/mod.rs`
- Simplify `add_output()` to orchestration (~40 LOC)
- Simplify `remove_output()` to orchestration (~50 LOC)
- Simplify `reposition_outputs()` to orchestration (~20 LOC)

### `src/niri/output.rs`
- Remove `output_resized()` (moved to subsystem)
- Remove `activate/deactivate_monitors()` (moved to subsystem)
- Keep query methods (output_left/right/above/below)

---

## Success Criteria

- [ ] `OutputSubsystem` contains all output state management logic
- [ ] `mod.rs` only contains orchestration that crosses subsystems
- [ ] No direct field access from outside subsystem
- [ ] All tests pass
- [ ] `cargo check` succeeds

---

## Estimated Effort

| Task | LOC | Time |
|------|-----|------|
| Move output_resized | 38 | 15 min |
| Move activate/deactivate | 20 | 10 min |
| Move remove logic | 74 | 30 min |
| Move add logic | 94 | 45 min |
| Move reposition logic | 116 | 60 min |
| Testing & fixes | - | 30 min |
| **Total** | **~342** | **~3 hrs** |
