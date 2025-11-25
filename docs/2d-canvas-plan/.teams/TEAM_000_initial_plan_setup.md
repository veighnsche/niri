# TEAM_000: Initial Plan Setup

## Status: COMPLETED

## Mission
Establish the 2D canvas implementation plan, team coordination rules, and project structure.

## Context Read
- `src/layout/scrolling.rs` — understood current monolithic structure (5586 lines)
- `src/layout/workspace.rs` — understood workspace/scrolling relationship
- `src/layout/monitor.rs` — understood monitor/workspace relationship
- `src/layout/mod.rs` — understood layout module structure
- `niri-config/src/animations.rs` — understood animation configuration
- `gutter-bar/src/niri.rs` — understood external IPC integration

## Changes Made
- `docs/2d-canvas-plan/README.md` — created master plan with 6 phases
- `docs/2d-canvas-plan/AI_TEAM_RULES.md` — created team coordination rules
- `docs/2d-canvas-plan/phases/phase-0-preparation.md` — modular refactoring plan
- `docs/2d-canvas-plan/phases/phase-1-row-and-canvas.md` — Row + Canvas2D creation
- `docs/2d-canvas-plan/phases/phase-2-row-spanning.md` — row spanning implementation
- `docs/2d-canvas-plan/phases/phase-3-camera.md` — camera system with zoom
- `docs/2d-canvas-plan/phases/phase-4-navigation.md` — geometric navigation
- `docs/2d-canvas-plan/phases/phase-5-integration.md` — final integration
- `docs/2d-canvas-plan/.teams/` — created team folder structure

## Decisions

### Architecture: Rows + Row-Spanning (not per-window pixel scale)
User clarified that "200% window" means spanning 2 rows, not different pixel density.
Camera zooms out to fit the focused window's row span.
This avoids the tiling paradox entirely.

### Branch Strategy: Fresh from main
Created `2d-canvas` branch from latest `main` instead of continuing from `LTR-insta-tests-extended`.
The old branch had a flawed scrolling/ refactor that would need to be deleted anyway.

### Modular First (Phase 0)
Before building new features, refactor existing code into clean modules.
This creates a solid foundation and fulfills user's original modularity goal.

### Team Coordination
Established numbered team system so multiple AI sessions can coordinate without conflicts.
Each team owns a file, leaves breadcrumbs for next team.

## For Next Team

### Start Here
1. Read `AI_TEAM_RULES.md` (you probably already did)
2. Read `phases/phase-0-preparation.md`
3. Your first task: Step 0.1 — Extract Column into its own module

### Key Files to Study
- `src/layout/scrolling.rs` — the monolith you're breaking up
- `src/layout/tile.rs` — Column depends on this

### Warnings
- Don't create `scrolling/` folder — that was the failed attempt on old branch
- Create `column/` as a sibling to `scrolling.rs`, not inside it
- Keep `scrolling.rs` working throughout — incremental refactor

### What's Left
- [ ] Phase 0: Modular foundation (Column, AnimatedValue, clean ScrollingSpace)
- [ ] Phase 1-5: Actual 2D canvas implementation

## Handoff
- [x] Code compiles (no code changes, just docs)
- [x] Tests pass (no code changes)
- [x] Team file complete
- [x] Branch created: `2d-canvas` from latest `main`
- [x] Planning docs committed: NO (untracked, user can commit when ready)

---

*TEAM_000 signing off. Good luck, future teams.*
