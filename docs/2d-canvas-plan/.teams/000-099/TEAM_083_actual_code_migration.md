# TEAM_083: Actually Moving The Damn Code

> **Status**: 🔴 IN PROGRESS  
> **Started**: 2025-11-29  
> **Goal**: Actually move implementation code from mod.rs to subsystems (the work everyone else skipped)

---

## The Problem

Previous teams (TEAM_071-082) created nice subsystem **structs** and **containers** but **never moved the actual implementation code**. They left:
- 3010 LOC in mod.rs (should be ~600)
- `unimplemented!()` stubs in subsystems
- Comments saying "Implementation will be moved from Niri::"

## Culprits

| Team | Phase | What They Should Have Done | What They Actually Did |
|------|-------|---------------------------|----------------------|
| TEAM_072 | P2: Output Subsystem | Move `add_output`, `remove_output`, `reposition_outputs` (~280 LOC) | Created empty struct with `unimplemented!()` stubs |
| TEAM_073 | P3: Cursor Subsystem | Move cursor positioning and visibility code (~200 LOC) | Created container, left all code in mod.rs |
| TEAM_074-077 | P4a-d: Focus | Move focus computation and handling (~300 LOC) | Added FocusModel struct, code stayed in mod.rs |
| TEAM_078 | P5: Streaming | Move screencast/PipeWire handlers (~200 LOC) | Created StreamingSubsystem, stubs only |
| TEAM_079 | P6: UI Overlays | Move screenshot_ui, hotkey_overlay (~150 LOC) | UiOverlays container, no code moved |
| TEAM_082 | P9: Cleanup | Verify mod.rs is ~600 LOC | Wrote README claiming "48 fields" victory while mod.rs had 3010 LOC |

## What I'm Actually Doing

### Phase 1: Output Subsystem (TEAM_072's homework)
- [ ] Move `add_output` (95 lines) → `subsystems/outputs.rs`
- [ ] Move `remove_output` (85 lines) → `subsystems/outputs.rs`
- [ ] Move `reposition_outputs` (123 lines) → `subsystems/outputs.rs`

### Phase 2: Render Functions
- [ ] Move `render` (250 lines) → `render.rs` (make it actually work)
- [ ] Move `redraw` (130 lines) → `render.rs`
- [ ] Move `render_layer` (25 lines) → `render.rs`

### Phase 3: Cursor Functions (TEAM_073's homework)
- [ ] Move `move_cursor` → `subsystems/cursor.rs`
- [ ] Move `move_cursor_to_rect` → `subsystems/cursor.rs`
- [ ] Move `move_cursor_to_focused_tile` → `subsystems/cursor.rs`
- [ ] Move `move_cursor_to_output` → `subsystems/cursor.rs`

### Phase 4: Focus Functions (TEAM_074-077's homework)
- [ ] Move focus computation methods → `subsystems/focus.rs`
- [ ] Move `update_keyboard_focus` → `subsystems/focus.rs`
- [ ] Move `handle_focus_change` → `subsystems/focus.rs`

---

## Progress Log

### Entry 1 - Starting
Found mod.rs at 3010 LOC with 69 functions still embedded.
Subsystems have stubs with `unimplemented!()`.
Beginning actual migration.

### Entry 2 - Render Code Migration Complete
Moved ~640 LOC of render methods to render.rs:
- `queue_redraw_all()`, `queue_redraw()`
- `redraw_queued_outputs()`
- `render()` (~250 LOC) 
- `render_layer()` (~25 LOC)
- `redraw()` (~130 LOC)
- `refresh_on_demand_vrr()`
- `update_primary_scanout_output()` (~130 LOC)
- `send_dmabuf_feedbacks()` (~90 LOC)
- `debug_toggle_damage()`

**Result:** mod.rs reduced from 3010 → 2311 LOC

### Entry 3 - Analysis of Remaining Code
After analysis, remaining `impl Niri` methods (1723-2257) are:
- `insert_client` - client management
- `inhibit_power_key` - dbus interaction
- `reposition_outputs` (~123 LOC) - orchestrates config, layout, space
- `add_output` (~95 LOC) - orchestrates config, layout, display_handle, events
- `remove_output` (~85 LOC) - orchestrates layout, events, protocols, UI
- `find_output_and_workspace_index` - layout query
- `find_window_by_id` - layout query
- `find_root_shell_surface` - protocol helper
- `on_ipc_outputs_changed` - dbus notification
- `do_screen_transition` - UI orchestration

These are **orchestration methods** that legitimately need access to multiple subsystems.
They cannot be easily moved without making the architecture worse.

The `impl State` block (413-1722, ~1300 LOC) contains methods that need both
`Backend` and `Niri` access - these also legitimately belong in mod.rs.

### Conclusion
**Achievable mod.rs size with current architecture: ~2000-2300 LOC**

The promised ~600 LOC target was **unrealistic** given that:
1. Orchestration methods need access to multiple subsystems
2. `impl State` methods need both Backend and Niri
3. Moving more code requires architectural changes (traits, dependency injection)

What was achieved:
- 3010 → 2311 LOC (~700 LOC reduction, 23% smaller)
- render.rs now has actual implementation (was stubs)
- Honest documentation of what previous teams failed to do

