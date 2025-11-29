# Phase S1.3: CursorSubsystem Logic Migration

> **Goal**: Move all cursor/pointer logic FROM `mod.rs` and `render.rs` INTO `CursorSubsystem`

## Current State

CursorSubsystem has:
- ✅ Private fields (manager, texture_cache, visibility, contents, dnd_icon, tablet_location)
- ✅ Basic accessor methods
- ✅ Some behavior (hide_for_inactivity, show, disable)
- ❌ Cursor movement logic still in `mod.rs` (on `impl State`)
- ❌ Pointer rendering logic still in `render.rs`

---

## Methods to Move

### From `mod.rs` (impl State)

#### 1. `move_cursor` (mod.rs:568-594) - ~26 LOC

**Current signature:**
```rust
impl State {
    pub fn move_cursor(&mut self, location: Point<f64, Logical>) { ... }
}
```

**Problem:** This is on `impl State`, not `impl Niri`. It needs both `niri` and `backend` access.

**Target approach:**
```rust
impl CursorSubsystem {
    /// Updates cursor position and contents. Returns true if contents changed.
    pub fn move_to(
        &mut self,
        location: Point<f64, Logical>,
        space: &Space<Window>,
        seat: &Seat<State>,
    ) -> CursorMoveResult {
        // Returns what changed so caller can emit events
    }
}

pub struct CursorMoveResult {
    pub contents_changed: bool,
    pub surface: Option<(WlSurface, Point<f64, Logical>)>,
    pub needs_redraw: bool,
}
```

**Migration steps:**
1. Extract contents computation into CursorSubsystem
2. Keep Smithay pointer.motion() call in State (needs pointer grab context)
3. Return result struct so State can handle side effects

---

#### 2. `move_cursor_to_rect` (mod.rs:596-631) - ~35 LOC

**Current signature:**
```rust
impl State {
    fn move_cursor_to_rect(&mut self, rect: Rectangle<f64, Logical>, mode: CenterCoords) -> bool { ... }
}
```

**Target signature:**
```rust
impl CursorSubsystem {
    /// Computes new cursor position within rect. Returns None if already in bounds.
    pub fn compute_warp_to_rect(
        &self,
        current: Point<f64, Logical>,
        rect: Rectangle<f64, Logical>,
        mode: CenterCoords,
    ) -> Option<Point<f64, Logical>> {
        // Pure computation, no side effects
    }
}
```

**Migration:** Pure computation, easy to move.

---

#### 3. `move_cursor_to_focused_tile` (mod.rs:633-668) - ~35 LOC

**Current signature:**
```rust
impl State {
    pub fn move_cursor_to_focused_tile(&mut self, mode: CenterCoords) -> bool { ... }
}
```

**Dependencies:**
- `self.niri.focus.current()` - focus state
- `self.niri.tablet_cursor_location` - cursor subsystem
- `self.niri.layout.active_output()` - layout
- `self.niri.layout.monitor_for_output()` - layout
- `monitor.active_tile_visual_rectangle()` - layout
- `self.niri.global_space` - outputs

**Target approach:** Keep in State as orchestration, but use cursor subsystem for computation.

---

#### 4. `move_cursor_to_output` (mod.rs:822-826) - ~5 LOC

Simple wrapper, keep in State.

---

#### 5. `refresh_pointer_contents` (mod.rs:744-774) - ~30 LOC

**Target signature:**
```rust
impl CursorSubsystem {
    /// Refreshes contents under pointer and updates visibility.
    pub fn refresh_contents(
        &mut self,
        location: Point<f64, Logical>,
        space: &Space<Window>,
    ) -> bool {
        // Returns true if contents changed
    }
}
```

---

#### 6. `update_pointer_contents` (mod.rs:776-820) - ~44 LOC

**Current:** Complex method that updates pointer based on contents.

**Target:** Split into:
- `CursorSubsystem::compute_contents()` - pure computation
- State orchestration for pointer focus updates

---

### From `render.rs`

#### 7. `pointer_element` (render.rs:120-280) - ~160 LOC

**Current:** Creates pointer render elements.

**Target signature:**
```rust
impl CursorSubsystem {
    /// Renders pointer for the given output.
    pub fn render<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        output: &Output,
        output_scale: Scale<f64>,
        location: Point<f64, Logical>,
        include_pointer: bool,
    ) -> Vec<OutputRenderElements<R>> {
        // Creates cursor and DnD icon elements
    }
}
```

**Migration:** Large method but self-contained.

---

#### 8. `refresh_pointer_outputs` (render.rs:282-345) - ~63 LOC

**Current:** Refreshes DnD icon output enter/leave.

**Target signature:**
```rust
impl CursorSubsystem {
    /// Updates DnD icon surface outputs for damage tracking.
    pub fn refresh_dnd_outputs(&mut self, space: &Space<Window>, location: Point<f64, Logical>) {
        // Handles output enter/leave for DnD icon
    }
}
```

---

## Architecture Decision

### Problem: `impl State` vs `impl Niri`

Cursor methods are on `impl State` because they need:
1. `self.niri` - cursor state
2. `self.backend` - for rendering (sometimes)
3. `pointer.motion()` - Smithay seat API

### Solution: Layered Approach

```
┌─────────────────────────────────────────┐
│ impl State (orchestration)              │
│   - move_cursor() calls:                │
│     1. cursor.compute_move()            │
│     2. pointer.motion()                 │
│     3. niri.queue_redraw_all()          │
└─────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────┐
│ CursorSubsystem (logic)                 │
│   - compute_move() - pure computation   │
│   - compute_warp() - pure computation   │
│   - render() - render elements          │
│   - refresh_contents() - state update   │
└─────────────────────────────────────────┘
```

---

## New CursorSubsystem API

```rust
impl CursorSubsystem {
    // === Visibility ===
    pub fn is_visible(&self) -> bool;
    pub fn show(&mut self);
    pub fn hide(&mut self);
    pub fn disable(&mut self);
    
    // === Contents ===
    pub fn contents(&self) -> &PointContents;
    pub fn update_contents(&mut self, contents: PointContents);
    pub fn compute_contents(&self, location: Point, space: &Space) -> PointContents;
    
    // === Movement computation ===
    pub fn compute_warp_to_rect(
        &self,
        current: Point,
        rect: Rectangle,
        mode: CenterCoords,
    ) -> Option<Point>;
    
    // === Rendering ===
    pub fn render<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        output: &Output,
        scale: Scale,
        location: Point,
    ) -> Vec<OutputRenderElements<R>>;
    
    pub fn refresh_dnd_outputs(&mut self, space: &Space, location: Point);
    
    // === DnD icon ===
    pub fn dnd_icon(&self) -> Option<&DndIcon>;
    pub fn set_dnd_icon(&mut self, icon: Option<DndIcon>);
    
    // === Tablet ===
    pub fn tablet_location(&self) -> Option<Point>;
    pub fn set_tablet_location(&mut self, location: Option<Point>);
    
    // === Manager access (for cursor image changes) ===
    pub fn set_cursor_image(&mut self, image: CursorImageStatus);
    pub fn get_render_cursor(&self, scale: i32) -> RenderCursor;
}
```

---

## Implementation Order

1. **Add `compute_contents()`** - pure function, easy
2. **Add `compute_warp_to_rect()`** - pure function, easy
3. **Move `pointer_element()` → `render()`** - large but self-contained
4. **Move `refresh_pointer_outputs()` → `refresh_dnd_outputs()`** - medium
5. **Refactor `refresh_pointer_contents()`** - use new compute method
6. **Refactor `update_pointer_contents()`** - use new compute method
7. **Update State methods** - use new subsystem methods

---

## File Changes

### `src/niri/subsystems/cursor.rs`
- Add `compute_contents()` method
- Add `compute_warp_to_rect()` method
- Add `render()` method (~160 LOC)
- Add `refresh_dnd_outputs()` method (~60 LOC)
- Add `set_cursor_image()` method

### `src/niri/render.rs`
- Remove `pointer_element()` (moved)
- Remove `refresh_pointer_outputs()` (moved)
- Update render to call `cursor.render()`

### `src/niri/mod.rs`
- Simplify `refresh_pointer_contents()` to use subsystem
- Simplify `update_pointer_contents()` to use subsystem
- Keep `move_cursor()` etc. as thin orchestration

---

## Success Criteria

- [ ] All cursor rendering in `CursorSubsystem::render()`
- [ ] All contents computation in subsystem
- [ ] State methods are thin orchestration only
- [ ] No direct field access from outside
- [ ] `cargo check` succeeds

---

## Estimated Effort

| Task | LOC | Time |
|------|-----|------|
| compute_contents | 20 | 15 min |
| compute_warp_to_rect | 30 | 15 min |
| Move render (pointer_element) | 160 | 45 min |
| Move refresh_dnd_outputs | 60 | 20 min |
| Refactor State methods | 50 | 30 min |
| Testing & fixes | - | 30 min |
| **Total** | **~320** | **~2.5 hrs** |
