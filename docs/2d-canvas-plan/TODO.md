# Global TODO List — 2D Canvas Refactor

> **Check this file first** to see where past teams planned to add features.
> This maintains architectural consistency across teams.

---

## src/layout/row/mod.rs

Row is a partial implementation. These methods need to be ported from `scrolling.rs`:

### Column Operations (DONE)
- [x] `add_tile`, `add_column`, `remove_column` — basic versions done
- [x] `move_left`, `move_right`, `move_column_to` — basic versions done
- [ ] TODO(TEAM_006): Animate column movement during add/remove/move
- [ ] TODO(TEAM_006): Port `consume_or_expel_window_left` from ScrollingSpace
- [ ] TODO(TEAM_006): Port `consume_or_expel_window_right` from ScrollingSpace

### View Offset / Animation
- [ ] TODO(TEAM_006): Port full `animate_view_offset_to_column` logic (current is stub)
- [ ] TODO(TEAM_006): Port `set_view_offset_with_animation` from ScrollingSpace
- [ ] TODO(TEAM_006): Port gesture handling (`view_offset_gesture_begin`, etc.)

### Rendering
- [ ] TODO(TEAM_006): Port `render_elements` from ScrollingSpace
- [ ] TODO(TEAM_006): Port `render_above_top_layer` from ScrollingSpace

### Interactive Resize
- [ ] TODO(TEAM_006): Port `interactive_resize_begin` from ScrollingSpace
- [ ] TODO(TEAM_006): Port `interactive_resize_update` from ScrollingSpace
- [ ] TODO(TEAM_006): Port `interactive_resize_end` from ScrollingSpace

---

## src/layout/canvas/mod.rs

Canvas2D depends on Row completion. Additional work needed:

### Window Operations (DONE)
- [x] `add_tile`, `add_tile_to_row` — done
- [x] `contains`, `find_window` — done

### Floating Layer
- [ ] TODO(TEAM_006): Integrate FloatingSpace (after Row is complete)
- [ ] TODO(TEAM_006): Add `toggle_floating` method

### Camera
- [ ] TODO(TEAM_006): Animate camera_y when changing rows (current is instant)
- [ ] TODO(TEAM_006): Add camera_x tracking for horizontal scroll

### Rendering
- [ ] TODO(TEAM_006): Add `render_elements` method

---

## How to Use This File

1. **Before starting work**: Check if your feature is already planned here
2. **When adding TODOs**: Use format `// TODO(TEAM_XXX): description`
3. **Before finishing**: Run `grep -rn "TODO(TEAM" src/layout/` and update this file
4. **When completing a TODO**: Mark it `[x]` here and remove from code

---

*Created by TEAM_006*
