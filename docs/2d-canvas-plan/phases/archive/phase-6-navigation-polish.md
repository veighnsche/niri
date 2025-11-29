# Phase 6: Navigation & Polish

> **Status**: ⏳ PENDING (after Phase 5)
> **Goal**: Geometric navigation, origin-based edges, final polish

---

## Related TODOs from Codebase

These TODOs will be resolved by this phase:
- `TODO: Implement move window between rows` (monitor/navigation.rs:124,129,139)
- `TODO: Implement move column between rows` (monitor/navigation.rs:148,153,158)
- `TODO(TEAM_018): Implement back-and-forth logic` (canvas/navigation.rs:324)
- `TODO(TEAM_022): Implement proper insert hint rendering` (monitor/render.rs:45)
- `TODO(TEAM_023): Implement proper row geometry calculation` (monitor/hit_test.rs:22,41)

---

## Overview

Final polish phase for the 2D canvas system:
- Geometric navigation (find nearest tile in direction)
- Origin-based leading edges for animations
- Spawn direction based on quadrant
- Documentation and testing

---

## Step 6.1: Geometric Navigation

### Current Behavior
"Down" moves to same column index in row below.

### Target Behavior
"Down" finds the window whose top edge is closest to the current window's bottom edge center.

```
┌─────────────┐
│   Current   │
│   Window    │
└──────●──────┘  ← probe point (bottom edge center)
       ↓
       ↓ find nearest below
       ↓
    ┌──●────────────┐
    │  Nearest Tile │
    └───────────────┘
```

### Tasks

- [ ] **6.1.1**: Define `Direction` enum (Up, Down, Left, Right)
- [ ] **6.1.2**: Implement `probe_point(tile_rect, direction) -> Point`
- [ ] **6.1.3**: Implement `tiles_in_direction(probe, direction) -> Iterator`
- [ ] **6.1.4**: Implement `distance_to_tile(probe, tile, direction) -> f64`
- [ ] **6.1.5**: Implement `find_nearest_in_direction(direction) -> Option<TileRef>`
- [ ] **6.1.6**: Wire up to focus_up/down/left/right

---

## Step 6.2: Origin-Based Leading Edge

### Concept

The origin (0,0) determines which edge is "leading" for animations:

| Position | Horizontal Leading | Vertical Leading |
|----------|-------------------|------------------|
| Right of origin | Right edge | — |
| Left of origin | Left edge | — |
| Above origin | — | Top edge |
| Below origin | — | Bottom edge |

### Tasks

- [ ] **6.2.1**: Add `Canvas2D::quadrant_of(tile) -> Quadrant`
- [ ] **6.2.2**: Define `Quadrant { h: HorizontalSide, v: VerticalSide }`
- [ ] **6.2.3**: Use quadrant in resize animations
- [ ] **6.2.4**: Use quadrant in window movement animations

---

## Step 6.3: Spawn Direction

### Logic

When spawning a new window "next to" focused:
- Right of origin → spawn to the right
- Left of origin → spawn to the left

### Tasks

- [ ] **6.3.1**: Modify `add_window()` to consider quadrant
- [ ] **6.3.2**: Implement horizontal spawn direction
- [ ] **6.3.3**: Handle edge case: spawning at origin

---

## Step 6.4: Documentation

### Tasks

- [ ] **6.4.1**: Update wiki with 2D canvas concepts
- [ ] **6.4.2**: Document new keybinds
- [ ] **6.4.3**: Document new config options
- [ ] **6.4.4**: Create migration guide from workspaces
- [ ] **6.4.5**: Update README

### Documentation Structure

```
docs/wiki/
├── 2D-Canvas.md           # Overview of 2D canvas
├── 2D-Canvas-Keybinds.md  # Keybind reference
├── 2D-Canvas-Config.md    # Configuration options
└── Migration-to-2D.md     # Migration guide
```

---

## Step 6.5: Testing

### Tasks

- [ ] **6.5.1**: Port existing layout tests to Canvas2D
- [ ] **6.5.2**: Add tests for row spanning
- [ ] **6.5.3**: Add tests for geometric navigation
- [ ] **6.5.4**: Add tests for camera behavior
- [ ] **6.5.5**: Add tests for bookmarks
- [ ] **6.5.6**: Manual testing checklist

### Manual Testing Checklist

```markdown
## 2D Canvas Manual Testing

### Basic Operations
- [ ] Create new window → appears in correct position
- [ ] Close window → layout adjusts
- [ ] Move window left/right → works
- [ ] Move window up/down (to new row) → works

### Row Spanning
- [ ] Increase row span → window grows to 2 rows
- [ ] Decrease row span → window shrinks
- [ ] Camera zooms out when focusing 2-row window

### Navigation
- [ ] Focus down → goes to nearest below
- [ ] Focus up → goes to nearest above
- [ ] Focus left/right → works within row

### Camera & Bookmarks
- [ ] Mod+ScrollDown → zooms out
- [ ] Mod+ScrollUp → zooms in
- [ ] Mod+1/2/3 → jump to bookmarks
- [ ] Mod+Shift+1/2/3 → save bookmarks
```

---

## Success Criteria

- [ ] Geometric navigation finds nearest tile
- [ ] Resize pins correct edge based on quadrant
- [ ] New windows spawn in correct direction
- [ ] All documentation complete
- [ ] All tests pass
- [ ] Manual testing checklist complete

---

*Phase 6 - Navigation & Polish*
