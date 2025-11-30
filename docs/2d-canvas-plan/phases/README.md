# Modular Refactor Phases

> **Goal**: Break monolithic files into focused, maintainable modules.

---

## Current Focus: TTY Backend Refactor

The `src/backend/tty.rs` file is 3473 lines — the largest remaining monolithic file.

### Overview
- [T1: TTY Refactor Overview](phase-T1-tty-refactor.md) — Summary & critical assessment

### Sub-phases (in order)

| Phase | Focus | Benefit | Time |
|-------|-------|---------|------|
| [T1.1](phase-T1.1-extract-types.md) | Types & module structure | Foundation | 30m |
| [T1.2](phase-T1.2-extract-device.md) | OutputDevice | ⭐⭐ Isolated | 45m |
| [T1.3](phase-T1.3-extract-helpers.md) | Helper functions | ⭐⭐⭐ Testable | 45m |
| [T1.4](phase-T1.4-extract-lifecycle.md) | Device lifecycle | ⭐⭐⭐ Focused | 1h |
| [T1.5](phase-T1.5-extract-connectors.md) | Connector handling | ⭐⭐ Isolated | 45m |
| [T1.6](phase-T1.6-extract-render.md) | Render pipeline | ⭐⭐ Clear | 45m |
| [T1.7](phase-T1.7-extract-output.md) | Output management | ⭐⭐ Organized | 30m |

**Total: ~4-5 hours**

---

## Completed Refactors

### niri/ Module Refactor ✅
The `src/niri/` refactor is complete. See [src/niri/README.md](/src/niri/README.md) for architecture.

**Results:**
- Reduced from 6604 LOC monolith to 2349 LOC mod.rs
- Clear subsystem boundaries (render.rs, config.rs, etc.)

### input/ Module Refactor ✅
The `src/input/` refactor is complete.

**Results:**
- Reduced from 5109 LOC monolith to 942 LOC mod.rs
- Extracted: actions.rs, pointer.rs, binds.rs, device.rs

### layout/ Module Refactor ✅
The `src/layout/` refactor is ongoing with 2D Canvas work.

**Results:**
- Reduced from 5353 LOC to 1875 LOC mod.rs
- Extracted: layout_impl/ (navigation, render, interactive_move)

---

## Archive

Previous phase documents are in [archive/](archive/):
- **I1.x**: Input module refactor phases (completed)
- **S1.x**: Subsystem encapsulation phases (completed)
- **P1-P9**: Original niri refactor plan (completed)
- **Phase 0-5**: 2D Canvas implementation phases