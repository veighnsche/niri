# TEAM_009: Canvas2D FloatingSpace Integration + Planning Update

## Status: COMPLETE

## Objectives
1. Complete Phase 1.5.2: Integrate FloatingSpace into Canvas2D module
2. Conduct comprehensive contradiction sweep and alignment with USER
3. Create planning update masterplan for Phase 1.5.3

## Starting Point
Per TEAM_008 handoff:
- Row module is feature-complete (Phase 1.5.1 ✅)
- Canvas2D has basic row operations and rendering
- FloatingSpace NOT YET integrated into Canvas2D
- All 251 tests pass, 58 golden tests pass

## Completed Work

### Phase 1.5.2.1-2: FloatingSpace Integration ✅
- [x] Added `floating: FloatingSpace<W>` field
- [x] Added `floating_is_active: bool` field
- [x] Added FloatingSpace constructor in `Canvas2D::new`

### Phase 1.5.2.3-5: Floating Operations ✅
- [x] Implemented `toggle_floating_window` - moves window between layers
- [x] Implemented `toggle_floating_focus` - switches focus between layers
- [x] Updated `render_elements` for floating layer
- [x] Updated `update_render_elements` for floating layer
- [x] Updated `advance_animations` to include floating
- [x] Updated `are_animations_ongoing` to include floating
- [x] Added accessor methods: `floating()`, `floating_mut()`, `floating_is_active()`, `has_floating_windows()`

### Phase 1.5.2.6-8: Camera System
- [ ] Apply camera offset in `render_elements` — Deferred to Phase 3
- [ ] Add `camera_x` tracking for horizontal scroll — Deferred to Phase 3
- [ ] Add `vertical_view_movement` config to niri-config — Deferred to Phase 3

**Decision**: Camera offset and vertical_view_movement are part of the full camera system (Phase 3). The current implementation uses `horizontal_view_movement` config for Y animations as a reasonable fallback.

### Phase 1.5.2.9-10: Window Operations ✅
- [x] Added `add_window` - routes to floating or tiled layer
- [x] Added `remove_window` - finds window across all layers
- [x] Added `contains_any` - checks both floating and tiled
- [x] Added `start_close_animation_for_window` - handles floating close animations
- [x] Added `start_close_animation_for_tile` - wrapper for floating close animations

### Row Module Addition ✅
- [x] Added `remove_active_tile` method to Row (operations/remove.rs)

## Changes Made

### Modified Files
- `src/layout/canvas/mod.rs` — Full FloatingSpace integration (426 → 608 lines)
  - Added FloatingSpace and related imports
  - Added `floating` and `floating_is_active` fields
  - Added accessor methods
  - Updated `render_elements` with floating layer rendering
  - Updated `update_render_elements` for floating
  - Updated animation methods
  - Added toggle_floating methods
  - Added add_window and remove_window methods
  - Added close animation methods
- `src/layout/row/operations/remove.rs` — Added `remove_active_tile` method

### Documentation Updated
- `docs/2d-canvas-plan/TODO.md` — Marked FloatingSpace integration complete, added all TODOs/FIXMEs
- `docs/2d-canvas-plan/phases/phase-1.5-integration.md` — Updated with TEAM_009 progress
- `docs/2d-canvas-plan/README.md` — Fixed Phase 1.5.3 to say "Replace Workspace" not "Feature flag"
- `docs/2d-canvas-plan/phases/phase-5-integration.md` — Removed feature flag references, marked 5.1 done early
- `docs/2d-canvas-plan/ai-teams-rules.md` — Updated current status and removed feature flag references
- `src/layout/canvas/mod.rs` — Fixed comment to say "replaces Workspace entirely"

### Questionnaires Created
- `.questions/TEAM_009_contradiction_sweep.md` — Comprehensive contradiction sweep with USER answers
- `.questions/TEAM_009_followup_planning_gaps.md` — Follow-up questionnaire for major planning gaps

### Planning Documents Created (Masterplan)
- `MASTERPLAN.md` — Single source of truth for Phase 1.5.3 requirements
- `phases/animation-regression-checklist.md` — Audit of all animations to preserve
- `phases/phase-1.5.3-removal-checklist.md` — Detailed workspace removal steps

## Remaining Work for Next Team

### CRITICAL: Read MASTERPLAN.md First!
The masterplan (`MASTERPLAN.md`) is now the single source of truth. It contains:
- Pre-Phase 1.5.3 requirements (testing, animation audit)
- Links to all planning documents
- Success criteria

### USER Has Answered Follow-up Questions
See `.questions/TEAM_009_followup_planning_gaps.md` — USER answers are marked with `<-`

### Phase 1.5.3: Replace Workspace with Canvas2D (BREAKING CHANGE)
- [ ] Remove `workspaces: Vec<Workspace<W>>` from Monitor
- [ ] Add `canvas: Canvas2D<W>` to Monitor
- [ ] Remove workspace switching logic (`Mod+1/2/3`, etc.)
- [ ] Remove overview mode entirely
- [ ] Remove hot corner (top right)
- [ ] Update all Monitor methods to use Canvas2D

### Phase 1.5.4: Monitor Integration
- [ ] Wire window operations through Canvas2D
- [ ] Wire navigation through Canvas2D
- [ ] Wire rendering through Canvas2D
- [ ] Update IPC to work with Canvas2D

### ⚠️ Animation Gap (USER flagged as emergency)
See TODO.md for animation-related TODOs that need planning.

### Phase 3: Camera System
- [ ] Apply camera offset to render elements
- [ ] Add camera_x tracking for horizontal scroll
- [ ] Add vertical_view_movement config

## Handoff
- [x] Code compiles (`cargo check`) — 1 dead code warning (expected)
- [x] Tests pass (`cargo test`) — 251 tests
- [x] Golden tests pass (`cargo insta test`) — 58 tests
- [x] Team file complete
- [x] Contradiction sweep complete
- [x] Follow-up questionnaire created
