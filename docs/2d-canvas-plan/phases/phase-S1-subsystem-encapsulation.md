# Phase S1: Subsystem Encapsulation

> **Goal**: Reduce public API surface by moving logic INTO subsystems, not just data.

## Problem Statement

Current state (TEAM_083 audit):
- **143 public functions** in `src/niri/*.rs`
- **58 pub fn** in `mod.rs` alone
- Subsystems are **hollow shells** - they hold data but logic lives elsewhere
- Inconsistent patterns: some use accessors, some have public fields, some have `unimplemented!()` stubs

This is **Option A architecture dressed up as Option B** - the worst of both worlds.

---

## Target Architecture: True Encapsulation (Option B)

### Principle: Subsystems Own Behavior

```rust
// WRONG (current): Logic in Niri, subsystem is data-only
impl Niri {
    pub fn add_output(&mut self, output: Output, ...) { // 95 LOC of logic }
    pub fn remove_output(&mut self, output: &Output) { // 85 LOC of logic }
    pub fn move_cursor(&mut self, location: Point) { // 40 LOC of logic }
}

// RIGHT (target): Logic in subsystem, Niri orchestrates
impl OutputSubsystem {
    pub fn add(&mut self, output: Output, config: &Config) -> GlobalId { ... }
    pub fn remove(&mut self, output: &Output) { ... }
}
impl Niri {
    // Only high-level orchestration that crosses subsystems
    pub fn handle_output_added(&mut self, output: Output) {
        let global = self.outputs.add(output, &self.config);
        self.layout.add_output(&output);
        self.queue_redraw_all();
    }
}
```

### Principle: Minimize Public API

```rust
// WRONG: Everything public
pub struct CursorSubsystem {
    pub manager: CursorManager,
    pub visibility: PointerVisibility,
    pub contents: PointContents,
}

// RIGHT: Private fields, focused public API
pub struct CursorSubsystem {
    manager: CursorManager,        // private
    visibility: PointerVisibility, // private
    contents: PointContents,       // private
}
impl CursorSubsystem {
    // Only expose what external code NEEDS
    pub fn move_to(&mut self, location: Point, seat: &Seat) { ... }
    pub fn warp_to_rect(&mut self, rect: Rectangle) -> bool { ... }
    pub fn is_visible(&self) -> bool { ... }
    pub fn render(&self, scale: i32) -> RenderCursor { ... }
}
```

---

## Subsystem Ownership Map

### OutputSubsystem should own:
| Method | Current Location | LOC |
|--------|-----------------|-----|
| `add_output` | mod.rs | 95 |
| `remove_output` | mod.rs | 85 |
| `reposition_outputs` | mod.rs | 123 |
| `output_resized` | output.rs | 45 |
| `activate_monitors` | output.rs | 15 |
| `deactivate_monitors` | output.rs | 15 |
| `output_left/right/above/below` | output.rs | 60 |
| **Total** | | **~440 LOC** |

**Public API after encapsulation:**
```rust
impl OutputSubsystem {
    pub fn add(&mut self, output: Output, config: &OutputConfig) -> GlobalId;
    pub fn remove(&mut self, output: &Output);
    pub fn reposition(&mut self);
    pub fn resize(&mut self, output: &Output);
    pub fn activate(&mut self);
    pub fn deactivate(&mut self);
    pub fn for_output(&self, name: &str) -> Option<&Output>;
    pub fn iter(&self) -> impl Iterator<Item = &Output>;
}
```
**Reduction: ~15 pub fn → 8 pub fn**

---

### CursorSubsystem should own:
| Method | Current Location | LOC |
|--------|-----------------|-----|
| `move_cursor` | mod.rs (State) | 40 |
| `move_cursor_to_rect` | mod.rs (State) | 35 |
| `move_cursor_to_focused_tile` | mod.rs (State) | 30 |
| `move_cursor_to_output` | mod.rs (State) | 10 |
| `refresh_pointer_contents` | render.rs | 25 |
| `update_pointer_contents` | render.rs | 40 |
| `pointer_element` | render.rs | 120 |
| `refresh_pointer_outputs` | render.rs | 50 |
| **Total** | | **~350 LOC** |

**Public API after encapsulation:**
```rust
impl CursorSubsystem {
    pub fn move_to(&mut self, location: Point, seat: &Seat, space: &Space);
    pub fn warp_to_rect(&mut self, rect: Rectangle, mode: CenterCoords) -> bool;
    pub fn warp_to_output(&mut self, output: &Output);
    pub fn update_contents(&mut self, space: &Space, location: Point);
    pub fn render<R: Renderer>(&self, output: &Output) -> Vec<CursorRenderElement>;
    pub fn is_visible(&self) -> bool;
    pub fn show(&mut self);
    pub fn hide(&mut self);
}
```
**Reduction: ~20 pub fn → 8 pub fn**

---

### FocusState should own:
| Method | Current Location | LOC |
|--------|-----------------|-----|
| `update_keyboard_focus` | mod.rs | 20 |
| `handle_focus_change` | mod.rs | 15 |
| `apply_keyboard_focus` | mod.rs | 15 |
| `handle_keyboard_layout_tracking` | mod.rs | 45 |
| `handle_popup_grab_on_focus_change` | mod.rs | 25 |
| `update_window_focus_states` | mod.rs | 35 |
| `build_focus_context` | mod.rs | 20 |
| `cleanup_layer_on_demand_focus` | mod.rs | 25 |
| **Total** | | **~200 LOC** |

**Public API after encapsulation:**
```rust
impl FocusState {
    pub fn update(&mut self, ctx: FocusContext) -> FocusChange;
    pub fn current(&self) -> &KeyboardFocus;
    pub fn apply(&mut self, seat: &Seat);
    pub fn is_inhibited(&self, surface: &WlSurface) -> bool;
}
```
**Reduction: ~12 pub fn → 4 pub fn**

---

### StreamingSubsystem should own:
| Method | Current Location | LOC |
|--------|-----------------|-----|
| `render_for_screen_cast` | screencast.rs | 80 |
| `render_windows_for_screen_cast` | screencast.rs | 60 |
| `stop_cast` | screencast.rs | 30 |
| `redraw_cast` | mod.rs (State) | 70 |
| `set_dynamic_cast_target` | mod.rs (State) | 50 |
| `on_screen_cast_msg` | mod.rs (State) | 140 |
| **Total** | | **~430 LOC** |

**Public API after encapsulation:**
```rust
impl StreamingSubsystem {
    pub fn add_cast(&mut self, cast: Cast);
    pub fn remove_cast(&mut self, session_id: usize);
    pub fn handle_message(&mut self, msg: ScreenCastMessage, ctx: &mut RenderContext);
    pub fn render_output<R: Renderer>(&self, output: &Output, renderer: &mut R);
    pub fn render_window<R: Renderer>(&self, window: &Window, renderer: &mut R);
}
```
**Reduction: ~10 pub fn → 5 pub fn**

---

## Implementation Order

### Phase S1.1: Make Fields Private (Preparation)
1. Change all `pub field` to `field` in subsystems
2. Add minimal accessor methods where needed
3. Fix compile errors by using accessors

### Phase S1.2: OutputSubsystem (Highest Impact)
1. Move `add_output` logic → `OutputSubsystem::add()`
2. Move `remove_output` logic → `OutputSubsystem::remove()`
3. Move `reposition_outputs` logic → `OutputSubsystem::reposition()`
4. Update all call sites in mod.rs

### Phase S1.3: CursorSubsystem
1. Move cursor movement logic from State impl
2. Move pointer rendering logic from render.rs
3. Consolidate pointer_element, refresh_pointer_outputs

### Phase S1.4: FocusState
1. Already has `compute_focus()` - most complete
2. Move remaining focus methods from mod.rs
3. Consolidate keyboard focus handling

### Phase S1.5: StreamingSubsystem
1. Move screencast logic
2. Move screencopy logic
3. Consolidate message handling

### Phase S1.6: Cleanup
1. Remove redundant accessor methods
2. Remove dead code
3. Verify public API surface reduced

---

## Success Metrics

| Metric | Before | Target |
|--------|--------|--------|
| `pub fn` in mod.rs | 58 | <20 |
| `pub fn` in niri/*.rs | 143 | <60 |
| LOC in mod.rs | 2427 | <800 |
| Subsystem logic (not just data) | 0% | 100% |

---

## Anti-Patterns to Avoid

### ❌ DON'T: Add accessors for everything
```rust
// This is just bureaucracy
pub fn manager(&self) -> &CursorManager { &self.manager }
pub fn manager_mut(&mut self) -> &mut CursorManager { &mut self.manager }
```

### ✅ DO: Add behavior methods
```rust
// This encapsulates the logic
pub fn set_cursor_image(&mut self, image: CursorImageStatus) {
    self.manager.set_cursor_image(image);
    self.visibility = PointerVisibility::Visible;
}
```

### ❌ DON'T: Pass subsystem internals to external code
```rust
// Leaks internal structure
let space = niri.outputs.space();
space.outputs().for_each(|o| ...);
```

### ✅ DO: Provide focused queries
```rust
// Hides internal structure
niri.outputs.for_each(|output| ...);
```

---

## Dependency Considerations

Some methods need cross-subsystem access. Solutions:

1. **Context structs**: Pass read-only context
   ```rust
   struct OutputContext<'a> {
       config: &'a Config,
       layout: &'a Layout,
   }
   self.outputs.add(output, ctx);
   ```

2. **Callbacks**: For mutations across subsystems
   ```rust
   self.outputs.add(output, |output| {
       self.layout.add_output(output);
   });
   ```

3. **Event-based**: Return what changed, let orchestrator handle
   ```rust
   let changes = self.outputs.add(output);
   for change in changes {
       match change {
           OutputChange::Added(o) => self.layout.add_output(o),
           OutputChange::Repositioned => self.queue_redraw_all(),
       }
   }
   ```

---

## Files to Modify

- `src/niri/subsystems/outputs.rs` - Add real implementations
- `src/niri/subsystems/cursor.rs` - Add cursor logic
- `src/niri/subsystems/focus.rs` - Already good, minor additions
- `src/niri/subsystems/streaming.rs` - Add screencast logic
- `src/niri/mod.rs` - Remove migrated methods, keep orchestration
- `src/niri/render.rs` - Move pointer rendering to cursor subsystem
- `src/niri/output.rs` - Merge into OutputSubsystem
- `src/niri/screencast.rs` - Merge into StreamingSubsystem

---

## Estimated Effort

| Phase | LOC to Move | Complexity | Time |
|-------|-------------|------------|------|
| S1.1 | 0 | Low | 30 min |
| S1.2 | ~440 | High | 3 hrs |
| S1.3 | ~350 | Medium | 2 hrs |
| S1.4 | ~200 | Low | 1 hr |
| S1.5 | ~430 | Medium | 2 hrs |
| S1.6 | 0 | Low | 1 hr |
| **Total** | **~1420** | | **~9.5 hrs** |

---

## Team Assignment

Each phase should be done by ONE team to avoid merge conflicts.
Mark your team number in code: `// TEAM_XXX: Migrated from mod.rs`
