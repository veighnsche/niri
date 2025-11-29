# TEAM_084: Input Module Refactor Plan

## Status: Planning Complete (Revised) ✅

## Summary

Created refactor plan for `src/input/mod.rs` (5123 lines). **Critical self-review led to a more honest assessment** of what the refactor actually achieves.

## Critical Assessment

### What we're actually improving:

| Extraction | Benefit | Value |
|------------|---------|-------|
| **Bind resolution** | Pure functions, testable without compositor | ⭐⭐⭐ High |
| **Device settings** | Self-contained, isolated | ⭐⭐ Medium |
| **Helper predicates** | Pure functions, testable | ⭐⭐ Medium |
| **Event handlers** | Navigability only (not architecture) | ⭐ Low |

### What we're NOT improving:
- Handler abstractions (still need full State)
- Encapsulation (actions still reach into niri.layout)
- The `do_action` match - **complexity lives in Layout, not here**

### Honest truth about `do_action`:
The 1550-line match statement looks scary, but:
1. Actions are thin wrappers: `Action::X => self.niri.layout.x()`
2. Match statements navigate fine with IDE
3. Splitting doesn't improve architecture, just moves code

**User decision needed:** Split actions by category or keep as single file?

## Target Architecture

```
src/input/
├── mod.rs              # Dispatcher (~400 LOC)
├── binds.rs            # PURE, TESTABLE (~300 LOC)
├── helpers.rs          # PURE, TESTABLE (~150 LOC)
├── device.rs           # Self-contained (~350 LOC)
├── actions.rs          # do_action (~1600 LOC) - or split?
├── keyboard.rs         # Keyboard events (~250 LOC)
├── pointer.rs          # Pointer events (~600 LOC)
├── tablet.rs           # Tablet events (~250 LOC)
├── gesture.rs          # Gesture events (~300 LOC)
└── touch.rs            # Touch events (~250 LOC)
```

## Phases Created

| Phase | File | Focus |
|-------|------|-------|
| I1 | `phase-I1-input-refactor.md` | Overview & assessment |
| I1.1 | `phase-I1.1-extract-binds.md` | Bind resolution (high value) |
| I1.2 | `phase-I1.2-extract-device.md` | Device management |
| I1.3 | `phase-I1.3-extract-handlers.md` | Event handlers |
| I1.4 | `phase-I1.4-actions-decision.md` | **USER DECISION** on do_action |
| I1.5 | `phase-I1.5-extract-helpers.md` | Helper functions |

## Estimated Time: ~4-5 hours

## Files Created

- `phases/phase-I1-input-refactor.md`
- `phases/phase-I1.1-extract-binds.md`
- `phases/phase-I1.2-extract-device.md`
- `phases/phase-I1.3-extract-handlers.md`
- `phases/phase-I1.4-actions-decision.md`
- `phases/phase-I1.5-extract-helpers.md`

## Files Archived

- `phase-S1*.md` → `archive/`

## Handoff

- [x] Critical self-review done
- [x] Individual phase files created
- [x] README updated
- [x] User decision point identified (I1.4)

Ready for implementation. Start with I1.1 (highest value).
