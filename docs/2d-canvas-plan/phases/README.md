# Subsystem Encapsulation Phases

> **Goal**: Reduce public API surface by moving logic INTO subsystems, achieving true encapsulation.

## Overview

The `src/niri/` module suffers from the "god object" anti-pattern:
- 143 public functions scattered across files
- Subsystems are hollow shells (data only, no logic)
- Inconsistent patterns between subsystems

These phases systematically fix this by:
1. Making subsystem fields private
2. Moving logic INTO subsystems
3. Exposing minimal, focused public APIs

---

## Phase Documents

| Phase | Subsystem | Status | Est. Time |
|-------|-----------|--------|-----------|
| [S1](phase-S1-subsystem-encapsulation.md) | Overview & Principles | ✅ Complete | - |
| [S1.2](phase-S1.2-output-subsystem.md) | OutputSubsystem | 🔄 In Progress | 3 hrs |
| [S1.3](phase-S1.3-cursor-subsystem.md) | CursorSubsystem | ⏳ Pending | 2.5 hrs |
| [S1.4](phase-S1.4-focus-state.md) | FocusState | ✅ Already Good | 35 min |
| [S1.5](phase-S1.5-streaming-subsystem.md) | StreamingSubsystem | ⏳ Pending | 4 hrs |

**Total estimated time: ~10 hours**

---

## Dependency Order

```
S1.2 OutputSubsystem (no dependencies)
  │
  ├── S1.3 CursorSubsystem (depends on output space)
  │
  └── S1.5 StreamingSubsystem (depends on outputs, layout)

S1.4 FocusState (independent, already complete)
```

---

## Target Metrics

| Metric | Before | After |
|--------|--------|-------|
| `pub fn` in mod.rs | 58 | <20 |
| `pub fn` in niri/*.rs | 143 | <60 |
| LOC in mod.rs | 2427 | <800 |
| Subsystem logic % | 0% | 100% |

---

## Archive

Previous phase documents (P1-P9) are in [archive/](archive/). These were the original refactor plan that created the subsystem *structure* but not the *content*.