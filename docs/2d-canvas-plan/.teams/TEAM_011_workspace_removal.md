# TEAM_011: Workspace Removal — Phase 1.5.3 Planning

## Status: PLANNING COMPLETE

## Team Assignment
- **Team Number**: 011
- **Task**: Create detailed breakdown for Phase 1.5.3 Part 2 (Workspace Removal)
- **Previous Team**: TEAM_010 (Phase 1.5.3 Part 1 — Monitor methods migration)

## Starting Point
Per TEAM_010 handoff:
- Phase 1.5.3 Part 1: IN PROGRESS (not complete)
- 284 tests pass
- 91 golden tests pass

## Work Completed

### Created Detailed Phase Documentation

The user correctly identified that my initial approach (no-ops + #[ignore]) was creating technical debt.

**Correct approach**: Complete removal, fix all call sites, no backwards compatibility.

Created 5 new phase documents breaking down Part 2 (alphabetical = execution order):

| Document | Purpose |
|----------|---------|
| `phase-1.5.3-part2a-config-workspace-actions.md` | Remove Action variants from Config |
| `phase-1.5.3-part2b-input-workspace-actions.md` | Remove Action handlers from Input |
| `phase-1.5.3-part2c-layout-workspace-switching.md` | Remove methods from Layout |
| `phase-1.5.3-part2d-monitor-workspace-switching.md` | Remove types/methods from Monitor |
| `phase-1.5.3-part2e-remove-workspace-tests.md` | Remove workspace tests |

### Updated Existing Documentation
- Updated `README.md` with detailed Part 2 breakdown
- Updated `phase-1.5.3-part2-remove-workspace-switching.md` to reference sub-parts
- Renamed files so **alphabetical = execution order**

### Execution Order
1. Part 2A (Config) — Remove Action variants
2. Part 2B (Input) — Remove handlers
3. Part 2C (Layout) — Remove methods
4. Part 2D (Monitor) — Remove types/methods
5. Part 2E (Tests) — Remove tests

This ensures the compiler guides you to all call sites.

### Reverted Technical Debt
Reverted my initial no-op changes to `monitor.rs` and `tests.rs`.

## Scope Analysis

| Area | Matches | Notes |
|------|---------|-------|
| `src/` workspace switching | 192 | Methods, handlers, tests |
| `niri-config/` Workspace | 200 | Actions, config parsing |
| `niri-ipc/` Workspace | 188 | IPC commands, state |

This is a large task requiring multiple teams to complete.

### Created Replacement Questionnaire

Created `.questions/TEAM_011_workspace_action_replacements.md` with:
- Pre-answered questions based on README vision
- Open questions for USER about:
  - Camera bookmark behavior
  - Row reordering (yes/no?)
  - Cross-monitor row movement
  - Row/bookmark naming
  - Essential vs later shortcuts

### Questionnaire Complete (USER answered)

Key decisions from USER:
- **Rows replace workspaces** for vertical navigation
- **No camera bookmarks** — `FocusWorkspace(N)` is REMOVED entirely
- **Previous position** — Yes, like browser back button
- **Move to row placement** — Geometrically nearest window
- **Row reordering** — Yes, `MoveRowDown/Up`
- **Cross-monitor** — Yes, `MoveRowToMonitor` (shrinks spanning windows)
- **Row naming** — Yes, rows can be named
- **Row-spanning edge case** — Window's 0,0 determines home row; shrinks to 1 span when row moves

## Handoff Notes

**Next team should**:
1. Read questionnaire answers in `.questions/TEAM_011_workspace_action_replacements.md`
2. Start implementation with Part 2A (Config)
3. Follow alphabetical order: 2A → 2B → 2C → 2D → 2E

## Handoff
- [x] Code compiles (`cargo check`)
- [x] Tests pass (`cargo test`)
- [x] Golden tests pass (`./scripts/verify-golden.sh`)
- [x] Team file complete
