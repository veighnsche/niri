# TEAM_106 — Critical Bug Fixes

## Status: INCOMPLETE - Bugs NOT Fixed, Handoff to Next Team

## Summary
Attempted fixes for floating window bugs. Some fixes were added but **NOT VERIFIED TO WORK**.
User reports bugs still persist. Next team must verify and properly fix.

## Bugs Investigated

### BUG-001: Floating windows not rendering on top ✅ FIXED
- **Fix**: Reversed render order in `canvas/render.rs` (floating FIRST = on top in Smithay)
- **Fix**: Added `FloatingSpace::window_under()` for hit testing
- **Fix**: Modified `Monitor::window_under()` to check floating first
- **Status**: User confirmed FIXED

### BUG-002: Mod+drag causes tiles to fly off-screen
- **Status**: NOT FIXED - Still investigating
- **Bug file**: `.bugs/BUG_mod_drag_tiles.md` (needs creation)

### BUG-003: Cannot drag floating windows by title bar
- **Status**: NOT FIXED
- **Bug file**: `.bugs/BUG_floating_drag.md`
- **What was tried**: 
  - Fixed `window_under` to use `HitType::hit_tile()` - this fixed buttons but not drag
  - Added logging to `interactive_move_begin()` - needs testing

### BUG-004: Mod+R/Mod+F affects wrong window ✅ FIXED
- **Fix**: Modified `toggle_width()` and `toggle_full_width()` to check `floating_is_active`
- **Status**: User confirmed FIXED

### BUG-005: Floating close animation missing
- **Status**: ATTEMPTED FIX - NOT VERIFIED
- **Bug file**: `.bugs/BUG_floating_close_animation.md`
- **What was tried**: Added floating check to `Layout::start_close_animation_for_window()`

### BUG-006: No focus after floating closes
- **Status**: ATTEMPTED FIX - NOT VERIFIED  
- **Bug file**: `.bugs/BUG_floating_close_no_focus.md`
- **What was tried**: Added `update_focus_after_removing()` to Canvas2D

## Code Changes Made (may be incomplete/broken)

### src/layout/canvas/render.rs
- Reversed render order: floating FIRST, then active row, then inactive rows

### src/layout/canvas/floating/mod.rs
- Added `window_under()` using `HitType::hit_tile()`

### src/layout/monitor/hit_test.rs
- Modified `window_under()` to check floating first

### src/layout/layout_impl/resize.rs
- Modified `toggle_width()` to handle floating
- Modified `toggle_full_width()` to return early if floating active

### src/layout/canvas/canvas_floating.rs
- Added `update_focus_after_removing()` 
- Modified `remove_window()` to call it

### src/layout/mod.rs
- Modified `start_close_animation_for_window()` to check floating

### src/layout/layout_impl/interactive_move.rs
- Added BUG003 debug logging (TEMP - remove after fix)

## Files to Check for Debug Logging
```bash
grep -rn "BUG00" src/
grep -rn "TEAM_106" src/
```

## Handoff Checklist
- [x] Code compiles (`cargo check`)
- [ ] Tests pass - NOT VERIFIED
- [ ] Bugs actually fixed - **NO, USER SAYS BUGS PERSIST**
- [x] Bug files created in `.bugs/`
- [x] Team file updated

## CRITICAL FOR NEXT TEAM

**User explicitly stated bugs still persist:**
1. Floating window drag still doesn't work
2. Close animation still missing
3. Focus still doesn't return to tiled window after floating closes

**My fixes may be wrong or incomplete. Do not trust them.**

## Recommended Next Steps

1. **Test each fix individually** - verify what actually works
2. **Add proper debug logging** per debugging rules
3. **Check main branch** for correct behavior patterns
4. **Focus on BUG_floating_close_no_focus first** - this is most visible to user
