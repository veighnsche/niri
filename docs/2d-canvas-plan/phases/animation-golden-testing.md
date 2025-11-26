# Animation Golden Testing Design

> **Purpose**: Prevent animation regressions by capturing animation behavior as deterministic timelines.
> **Status**: ✅ IMPLEMENTED — All edge animations captured

---

## Solution Summary

Animation timelines for **ALL edges of ALL tiles** are now captured as part of layout snapshots.
This enables RTL mirroring and complete animation regression testing.

### Key Files
- `src/layout/snapshot.rs` — Animation timeline snapshot types
- `src/layout/scrolling.rs` — `snapshot()` captures all animations
- `src/layout/tile.rs` — Test methods to expose tile animations
- `src/layout/column/mod.rs` — Test methods to expose column animations

---

## What's Captured

### For Each Tile (4 edges)

Edge names use `x_min/x_max/y_min/y_max` for RTL-safe coordinates:
- In **LTR**: x_min = left edge, x_max = right edge
- In **RTL**: after negating X values, semantics preserved

```yaml
animations:
  - target: tile_0_1_x_min    # Column 0, Tile 1, minimum X (left in LTR)
    from: 116                 # Starting X position
    to: 0                     # Target X position
    kind:
      type: Spring
      damping_ratio: 1
      stiffness: 800
    duration_ms: 325
  - target: tile_0_1_x_max    # Maximum X (right in LTR)
    from: 216
    to: 100
    ...
  - target: tile_0_1_y_min    # Minimum Y (top)
    from: 16
    to: 232
    ...
  - target: tile_0_1_y_max    # Maximum Y (bottom)
    from: 216
    to: 432
    ...
```

### Column Animations
```yaml
  - target: column_1_move_x   # Column horizontal movement
    from: 116
    to: 0
    ...
```

### View Offset (Camera)
```yaml
  - target: view_offset       # Camera X position
    from: -354
    to: -248
    ...
```

---

## Animation Sources

| Animation Type | What Triggers It | Edges Affected |
|---------------|------------------|----------------|
| `view_offset` | Focus change, gestures, center column | Camera X |
| `column_move_x` | Move column, add/remove column | All tiles in column X |
| `tile resize` | Mod+R preset, consume/expel | Width → right edge, Height → bottom edge |
| `tile move_x` | Consume/expel window | Left + right edges |
| `tile move_y` | Move window up/down in column | Top + bottom edges |

---

## RTL Mirroring

All X-axis animations can be mirrored for RTL by negating `from` and `to`:

```rust
impl AnimationTimelineSnapshot {
    pub fn is_x_axis(&self) -> bool {
        self.target == "view_offset"
            || self.target.ends_with("_left")
            || self.target.ends_with("_right")
            || self.target.ends_with("_width")
            || self.target.ends_with("_move_x")
    }

    pub fn mirror_rtl(&self) -> Self {
        if self.is_x_axis() {
            Self { from: -self.from, to: -self.to, ... }
        } else {
            self.clone()
        }
    }
}
```

---

## Test Strategy

1. **Tests WITHOUT `CompleteAnimations`** → Capture in-progress animations
2. **Tests WITH `CompleteAnimations`** → Final layout only (animations = [])

All existing golden test scenarios now capture animations. Any operation that triggers animation will have its edge movements recorded.

---

## Example Snapshots

### Consume Window Into Column (e1)
Window 2 moves from column 1 to column 0, stacking below window 1:
- `tile_0_1_x_min`: 116 → 0 (moves left)
- `tile_0_1_y_min`: 16 → 232 (moves down)

### Move Column Left (g1)
Column 3 swaps with column 2:
- `column_1_move_x`: 116 → 0 (moved column animates from original position)
- `column_2_move_x`: -116 → 0 (pushed column animates from offset)

### Center Column (d1)
Camera pans to center the focused column:
- `view_offset`: -16 → -590 (camera moves)

---

## Verification

All 59 golden tests pass with comprehensive animation capture:

```bash
cargo test --lib golden
# test result: ok. 59 passed
```

---

*Implemented by TEAM_010 — Animation Golden Testing Infrastructure*
