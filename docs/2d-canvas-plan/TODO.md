# Global TODO List — 2D Canvas Refactor

> **Check this file first** to see where past teams planned to add features.
> This maintains architectural consistency across teams.

**Last updated**: TEAM_009 (Phase 1.5.2 COMPLETE, Phase 1.5.3 next)

---

## src/layout/row/ — ✅ PHASE 1.5.1 COMPLETE

Row module is now feature-complete for Phase 1.5.1. All core ScrollingSpace methods have been ported.

### Column Operations (DONE)
- [x] `add_tile`, `add_column`, `remove_column` — basic versions done
- [x] `move_left`, `move_right`, `move_column_to` — basic versions done
- [x] `add_tile_to_column` — add tile to existing column (TEAM_008)
- [x] `add_tile_right_of` — add tile as new column right of window (TEAM_008)
- [x] `activate_column` — activate column with animation (TEAM_008)
- [x] `remove_tile` — remove tile by window ID (TEAM_008)
- [x] `remove_tile_by_idx` — remove tile by column/tile index with animations (TEAM_008)
- [x] `remove_active_column` — remove the active column (TEAM_008)
- [x] `remove_column_by_idx` — remove column with full animation support (TEAM_008)
- [x] `consume_or_expel_window_left` — consume into left column or expel as new column (TEAM_008)
- [x] `consume_or_expel_window_right` — consume into right column or expel as new column (TEAM_008)
- [x] `consume_into_column` — consume first tile from right column into active (TEAM_008)

### Remaining — ⚠️ ANIMATION GAP (See TEAM_009 questionnaire)
- [ ] TODO(TEAM_006): Animate column movement during add/remove/move (`operations/add.rs:157`, `operations/move_col.rs:48`)

### FIXMEs (Lower Priority)
- [ ] FIXME: Smarter height distribution (`resize.rs:111`)
- [ ] FIXME: Compute and use current velocity (`view_offset.rs:235`)
- [ ] FIXME: Tiles can move by X too in centered/resizing layout (`operations/remove.rs:54`)
- [ ] FIXME: Preserve activate_prev_column_on_removal (`operations/remove.rs:204`)

### View Offset / Animation
- [x] TODO(TEAM_007): Port full `animate_view_offset_to_column` logic — DONE
- [x] TODO(TEAM_007): Port `compute_new_view_offset_*` methods — DONE
- [x] TODO(TEAM_007): Port `animate_view_offset_with_config` — DONE
- [x] TODO(TEAM_007): Port gesture handling (`view_offset_gesture_begin`, etc.) — DONE

### Rendering
- [x] TODO(TEAM_007): Port `render_elements` from ScrollingSpace — DONE
- [x] TODO(TEAM_007): Port `columns_in_render_order` — DONE
- [x] TODO(TEAM_007): Port `update_render_elements` — DONE
- [x] `render_above_top_layer` — returns true when fullscreen and view stationary (TEAM_008)

### Interactive Resize
- [x] TODO(TEAM_007): Port `interactive_resize_begin` from ScrollingSpace — DONE
- [x] TODO(TEAM_007): Port `interactive_resize_update` from ScrollingSpace — DONE
- [x] TODO(TEAM_007): Port `interactive_resize_end` from ScrollingSpace — DONE

---

## src/layout/canvas/mod.rs

Canvas2D depends on Row completion. Additional work needed:

### Window Operations ✅ COMPLETE (TEAM_009)
- [x] `add_tile`, `add_tile_to_row` — done
- [x] `contains`, `find_window` — done
- [x] `add_window` — routes to correct layer (floating or tiled)
- [x] `remove_window` — finds window across all layers
- [x] `toggle_floating_window` — move window between layers
- [x] `toggle_floating_focus` — switch focus between layers

### Floating Layer ✅ COMPLETE (TEAM_009)
- [x] Integrate FloatingSpace into Canvas2D
- [x] Add floating layer rendering
- [x] Update animations to include floating

### Camera
- [x] TODO(TEAM_007): Animate camera_y when changing rows — DONE
- [ ] TODO(TEAM_007): Add vertical_view_movement config to niri-config (Phase 3)
- [ ] TODO(TEAM_006): Add camera_x tracking for horizontal scroll (Phase 3)

### Rendering ✅ COMPLETE
- [x] TODO(TEAM_007): Add `render_elements` method — DONE
- [x] TODO(TEAM_007): Add `update_render_elements` method — DONE
- [x] TEAM_009: Floating layer rendering integrated
- [ ] TODO(TEAM_007): Apply camera offset to render elements (`render.rs:25`) — Phase 3

### Floating Layer
- [ ] TODO(TEAM_009): Add close animation for tiled windows in rows (`floating.rs:126`)

### Navigation
- [ ] TODO(TEAM_007): Add vertical_view_movement config to niri-config (`navigation.rs:79`) — Phase 3

---

## How to Use This File

1. **Before starting work**: Check if your feature is already planned here
2. **When adding TODOs**: Use format `// TODO(TEAM_XXX): description`
3. **Before finishing**: Run `grep -rn "TODO(TEAM" src/layout/` and update this file
4. **When completing a TODO**: Mark it `[x]` here and remove from code

---

*Created by TEAM_006*
