# Phase 1.5.3 Part 2: Replace Workspace with Row

> **Status**: PENDING — See detailed sub-parts below
> **Type**: BREAKING CHANGE — Workspace → Row transformation + Monitor refactor
> **Prerequisite**: Part 1 complete (Monitor methods migrated)

---

## Overview

**Key strategy**: Don't just remove workspace — **replace with Row equivalents**.

Also: **Refactor monitor.rs into modules** (following Row/Column pattern). Workspace code simply doesn't get migrated.

---

## Sub-Parts (alphabetical = execution order)

| Part | Description | Status |
|------|-------------|--------|
| [Part 2A](phase-1.5.3-part2a-config-workspace-actions.md) | Replace Config Actions (Workspace → Row) | ⏳ Pending |
| [Part 2B](phase-1.5.3-part2b-input-workspace-actions.md) | Replace Input Handlers (Workspace → Row) | ⏳ Pending |
| [Part 2C](phase-1.5.3-part2c-layout-workspace-switching.md) | Replace Layout Methods (Workspace → Row) | ⏳ Pending |
| [Part 2D](phase-1.5.3-part2d-monitor-workspace-switching.md) | Refactor Monitor into Modules + Row Navigation | ⏳ Pending |
| [Part 2E](phase-1.5.3-part2e-remove-workspace-tests.md) | Replace/Remove Workspace Tests | ⏳ Pending |

---

## Execution Order

1. **Part 2A** (Config) — Replace `FocusWorkspace*` → `FocusRow*`, etc.
2. **Part 2B** (Input) — Replace handlers to call new row methods
3. **Part 2C** (Layout) — Replace delegation methods
4. **Part 2D** (Monitor) — Refactor into `monitor/` modules, implement row navigation
5. **Part 2E** (Tests) — Transform tests, remove workspace-specific ones

---

## Transformation Summary

| Workspace Concept | Row Equivalent | Notes |
|-------------------|----------------|-------|
| `FocusWorkspaceDown/Up` | `FocusRowDown/Up` | Navigate between rows |
| `MoveWindowToWorkspaceDown/Up` | `MoveWindowToRowDown/Up` | Geometric placement |
| `MoveColumnToWorkspaceDown/Up` | `MoveColumnToRowDown/Up` | |
| `MoveWorkspaceDown/Up` | `MoveRowDown/Up` | Reorder rows |
| `FocusWorkspace(N)` | **REMOVE** | No jump to specific row |
| `FocusWorkspacePrevious` | `FocusPreviousPosition` | Browser-like back |
| `SetWorkspaceName` | `SetRowName` | Rows can be named |
| Workspace gestures | Row/Camera pan gestures | Vertical pan |
| Overview mode | **REMOVE** | Replaced by 2D zoom |

---

## Monitor Refactoring (Part 2D)

```
Current: monitor.rs (2255 lines monolith)
Target:  monitor/ (~800 lines modular)
         ├── mod.rs          - Core struct, canvas
         ├── operations.rs   - Window operations
         ├── navigation.rs   - Row navigation (NEW)
         ├── render.rs       - Rendering
         ├── hit_test.rs     - Geometry queries
         ├── config.rs       - Config updates
         └── insert_hint.rs  - Insert hint
```

---

## Principles

- **Replace, don't just remove**: Workspace → Row where applicable
- **Modularize**: Refactor monitor.rs following Row/Column pattern
- **Leave behind**: Workspace code doesn't get migrated
- **Breaking > Compatible**: Temporary breakage is fine

---

*TEAM_011: Phase 1.5.3 Part 2 — Workspace → Row + Monitor Refactor*
