# Phase 1.5.3 Part 2: Remove Workspace Switching

> **Status**: PENDING — See detailed sub-parts below
> **Type**: BREAKING CHANGE — Complete removal, no backwards compatibility
> **Prerequisite**: Part 1 complete (Monitor methods migrated)

---

## Overview

This is a large task broken into sub-parts. **Complete removal, no no-ops or #[ignore]**.

---

## Sub-Parts (in execution order)

| Part | Description | Status |
|------|-------------|--------|
| [Part 2A](phase-1.5.3-part2a-config-workspace-actions.md) | Remove from Config | ⏳ Pending |
| [Part 2B](phase-1.5.3-part2b-input-workspace-actions.md) | Remove from Input handlers | ⏳ Pending |
| [Part 2C](phase-1.5.3-part2c-layout-workspace-switching.md) | Remove from Layout | ⏳ Pending |
| [Part 2D](phase-1.5.3-part2d-monitor-workspace-switching.md) | Remove from Monitor | ⏳ Pending |
| [Part 2E](phase-1.5.3-part2e-remove-workspace-tests.md) | Remove workspace tests | ⏳ Pending |

---

## Execution Order

**Alphabetical = Execution order** (reverse dependency):

1. **Part 2A** (Config) — Remove Action variants from niri-config
2. **Part 2B** (Input) — Remove Action handlers from input
3. **Part 2C** (Layout) — Remove methods from Layout
4. **Part 2D** (Monitor) — Remove methods and types from Monitor
5. **Part 2E** (Tests) — Remove workspace tests

This order ensures the compiler guides you to all call sites.

---

## Principles

- **No no-ops**: Don't make functions return early — remove them entirely
- **No #[ignore]**: Don't ignore tests — remove them entirely
- **Fix call sites**: When removing a function, fix all callers
- **Breaking > Compatible**: Temporary breakage is fine; backwards compat is forever

---

*TEAM_011: Phase 1.5.3 Part 2 — Detailed breakdown*
