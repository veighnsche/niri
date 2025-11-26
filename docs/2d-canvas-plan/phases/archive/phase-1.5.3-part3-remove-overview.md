# Phase 1.5.3 Part 3: Remove Overview Mode

> **Status**: PENDING
> **Prerequisite**: Part 2 complete (workspace switching removed)

---

## Overview

Overview mode shows all workspaces in a zoomed-out view.
Since we're removing workspaces, overview must be removed too.

**Note**: A new "zoom-out" view will be added in Phase 5 using the camera system.

---

## Step 3.1: Remove Overview Triggers

| Trigger | File | Change |
|---------|------|--------|
| Overview keybind | `src/input/` | Remove handler |
| Overview gesture | `src/input/` | Remove handler |
| Hot corner (top-right) | `src/input/` | Remove handler |

---

## Step 3.2: Remove Overview State from Monitor

| Item | File | Change |
|------|------|--------|
| `overview_open: bool` | `src/layout/monitor.rs` | Remove |
| `overview_progress: Option<OverviewProgress>` | `src/layout/monitor.rs` | Remove |
| `OverviewProgress` enum | `src/layout/monitor.rs` | Remove |

---

## Step 3.3: Remove Overview Rendering

| Item | File | Change |
|------|------|--------|
| Overview render logic | `src/layout/monitor.rs` | Remove |
| `compute_overview_zoom()` | `src/layout/mod.rs` | Remove or keep for Phase 5 |
| Overview-specific element positioning | Various | Remove |

---

## Step 3.4: Remove Overview Animations

| Item | File | Change |
|------|------|--------|
| Overview open/close animation | `src/layout/monitor.rs` | Remove |
| Overview gesture animation | `src/layout/monitor.rs` | Remove |

---

## Verification

After each step:
1. `cargo check` — must compile
2. `cargo test --lib` — fix any failing tests
3. Hot corner should do nothing (manual test)
4. Overview keybind should do nothing (manual test)

---

## Notes for Phase 5

The camera system will provide a new "zoom-out" view:
- User can zoom out to see more of the canvas
- Not workspace-based, just camera zoom
- Will use `compute_overview_zoom()` or similar

---

*TEAM_010: Phase 1.5.3 Part 3*
