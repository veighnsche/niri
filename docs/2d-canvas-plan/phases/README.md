# Modular Refactor Phases

> **Goal**: Break monolithic files into focused, maintainable modules.

---

## Current Focus: Input Module Refactor

The `src/input/mod.rs` file is 5123 lines.

### Overview
- [I1: Input Refactor Overview](phase-I1-input-refactor.md) — Summary & critical assessment

### Sub-phases (in order)

| Phase | Focus | Benefit | Time |
|-------|-------|---------|------|
| [I1.1](phase-I1.1-extract-binds.md) | Bind resolution | ⭐⭐⭐ Pure/testable | 1h |
| [I1.2](phase-I1.2-extract-device.md) | Device management | ⭐⭐ Isolated | 45m |
| [I1.3](phase-I1.3-extract-handlers.md) | Event handlers | ⭐ Navigability | 2h |
| [I1.4](phase-I1.4-actions-decision.md) | Actions split? | ❓ **USER DECISION** | - |
| [I1.5](phase-I1.5-extract-helpers.md) | Helper functions | ⭐⭐ Pure/testable | 30m |

**Total: ~4-5 hours**

---

## Completed: niri/ Module Refactor

The `src/niri/` refactor is complete. See [src/niri/README.md](/src/niri/README.md) for architecture.

**Results:**
- Reduced from 6604 LOC monolith to modular structure
- 75% reduction in Niri struct fields
- Clear subsystem boundaries

---

## Archive

Previous phase documents are in [archive/](archive/):
- **S1.x**: Subsystem encapsulation phases (completed)
- **P1-P9**: Original niri refactor plan (completed)
- **Phase 0-5**: 2D Canvas implementation phases