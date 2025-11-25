# Phase 0.5: Golden Snapshot Infrastructure

> **Goal**: Create snapshot testing to prevent behavioral regressions.
> **Total Estimated Time**: 2-3 days (split across 4 sub-phases)

---

## Why Golden Tests?

1. **Golden code** = original main branch `scrolling.rs`, `workspace.rs`, etc.
2. **Refactored code** = our modular `column/`, `row/`, `canvas/` modules
3. **Both must produce identical snapshots** for single-row scenarios
4. Prevents silent regressions during refactoring

---

## Sub-Phases

| Phase | Focus | Time | Dependency |
|-------|-------|------|------------|
| [0.5.A](phase-0.5a-golden-infrastructure.md) | Infrastructure (insta, types, dirs) | 2-4 hrs | None |
| [0.5.B](phase-0.5b-golden-code-extraction.md) | Extract golden code, add snapshot() | 4-6 hrs | 0.5.A |
| [0.5.C](phase-0.5c-core-golden-tests.md) | Core tests (Groups A-L, ~86 tests) | 4-6 hrs | 0.5.B |
| [0.5.D](phase-0.5d-advanced-golden-tests.md) | Advanced tests (Groups M-W, ~71 tests) | 4-6 hrs | 0.5.C |

**Total: ~157 golden tests**

---

## Test Groups Overview

See [golden-test-scenarios.md](golden-test-scenarios.md) for detailed scenarios.

| Groups | Focus |
|--------|-------|
| A-E | Basic ops, presets, centering |
| F-H | Multi-tile, move column/window |
| I-L | Fullscreen, heights, tabbed |
| M-O | Insert, close, edge cases |
| P | **Interactive resize (CRITICAL)** |
| Q-W | Swap, wrap-around, floating, gestures |

---

## Quick Start for Teams

### Before ANY layout refactor:
```bash
cargo insta test  # Must pass BEFORE you start
```

### After making changes:
```bash
cargo insta test  # Must STILL pass
```

### If tests fail:
1. **DO NOT** accept new snapshots without USER approval
2. Fix your code to match golden behavior
3. Or ask USER if change is intentional

---

## Files That Trigger Golden Testing

Any changes to these files require `cargo insta test`:
- `src/layout/scrolling.rs`
- `src/layout/column/*.rs`
- `src/layout/workspace.rs`
- `src/layout/floating.rs`
- `src/layout/types.rs`
- Any new layout modules

---

## Next Phase

After 0.5.D is complete → [Phase 0.2: AnimatedValue](phase-0-preparation.md#step-02-extract-view-offset-into-reusable-component)
