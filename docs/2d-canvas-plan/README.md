# 2D Canvas Implementation Plan

> Transform niri from a 1D scrolling tiler to a 2D canvas with rows, row-spanning windows, and dynamic camera zoom — built from modular, well-encapsulated components.

---

## Design Principles

### 1. Modular Architecture
Every component owns its own state and exposes a clean interface. No monolithic files.

```
Bad:  scrolling.rs (4000+ lines, everything in one file)
Good: scrolling/
      ├── mod.rs          (public interface only)
      ├── column.rs       (Column owns its tiles)
      ├── navigation.rs   (focus movement logic)
      ├── positioning.rs  (layout calculations)
      └── ...
```

### 2. Composition Over Inheritance
Build complex behavior by composing simple, single-purpose modules.

### 3. State Ownership
Each module owns its data. No reaching into other modules' internals.

### 4. Incremental Progress
Each phase produces working, testable code. No big bang rewrites.

---

## Vision Recap

```
                        x (origin 0,0)
                        |
─────────────────────────────────────────────────────────────────────
ROW 0   | Column A    | Column B  |        Column C                 |
        | (1× span)   | TileC     |                                 |
        |             |-----------|     "Important App"             |
        |   App A     | TileD     |       spans 2 rows              |
        |             |           |                                 |
─────────────────────────────────────────────────────────────────────
ROW 1   |             | Column B' |                                 |
        |   App B     | TileE     |      (same window,              |
        |  (1× span)  | TileF     |       still here)               |
        |             |           |                                 |
─────────────────────────────────────────────────────────────────────
```

### Core Concepts

| Concept | Definition |
|---------|------------|
| **Row** | Horizontal strip of columns (modular, owns its columns) |
| **Row Span** | Window spans 1, 2, or more rows vertically |
| **Camera** | Position `(x, y)` + `zoom` factor (modular, owns its state) |
| **Origin** | Fixed (0,0) determines leading/trailing edge behavior |
| **Navigation** | Geometric — finds nearest window in direction |

---

## Phases Overview

| Phase | Focus | Key Deliverable |
|-------|-------|-----------------|
| [Phase 0](phases/phase-0-preparation.md) | Modular foundation | Refactor existing code into clean modules |
| [Phase 1](phases/phase-1-row-and-canvas.md) | Row + Canvas2D | Multi-row canvas with basic navigation |
| [Phase 2](phases/phase-2-row-spanning.md) | Row spanning | Windows can span multiple rows |
| [Phase 3](phases/phase-3-camera.md) | Camera system | Dynamic zoom based on focused window |
| [Phase 4](phases/phase-4-navigation.md) | Navigation + polish | Geometric nav, origin-based behavior |
| [Phase 5](phases/phase-5-integration.md) | Integration | Replace workspaces, IPC, docs |

---

## File Structure Target

```
src/layout/
├── mod.rs                    # Top-level exports
├── tile.rs                   # Tile (mostly unchanged)
├── floating.rs               # FloatingSpace (unchanged)
│
├── column/                   # Column module (refactored)
│   ├── mod.rs                # Column struct + public interface
│   ├── layout.rs             # Tile positioning within column
│   ├── resize.rs             # Interactive resize
│   └── operations.rs         # Add/remove tiles
│
├── row/                      # Row module (NEW)
│   ├── mod.rs                # Row struct + public interface
│   ├── layout.rs             # Column positioning within row
│   ├── navigation.rs         # Left/right focus movement
│   └── operations.rs         # Add/remove columns
│
├── canvas/                   # Canvas2D module (NEW)
│   ├── mod.rs                # Canvas2D struct + public interface
│   ├── layout.rs             # Row positioning
│   ├── navigation.rs         # Up/down/geometric focus
│   ├── spanning.rs           # Row-span coordination
│   └── operations.rs         # Add/remove rows/windows
│
├── camera/                   # Camera module (NEW)
│   ├── mod.rs                # Camera struct + public interface
│   ├── position.rs           # X/Y offset with animation
│   ├── zoom.rs               # Zoom level with animation
│   └── following.rs          # Auto-follow focused tile
│
└── monitor.rs                # Monitor (updated to use Canvas2D)
```

---

## Progress Tracking

### Phase 0: Preparation
- [ ] Step 0.1: Create modular column structure
- [ ] Step 0.2: Extract view offset into reusable component
- [ ] Step 0.3: Clean up scrolling dependencies

### Phase 1: Row + Canvas2D
- [ ] Step 1.1: Create Row module
- [ ] Step 1.2: Create Canvas2D module
- [ ] Step 1.3: Basic vertical navigation
- [ ] Step 1.4: Feature flag integration

### Phase 2: Row Spanning
- [ ] Step 2.1: Add row_span to Tile
- [ ] Step 2.2: Cross-row coordination
- [ ] Step 2.3: Row span commands

### Phase 3: Camera System
- [ ] Step 3.1: Create Camera module
- [ ] Step 3.2: Auto-zoom for row span
- [ ] Step 3.3: Zoom gestures
- [ ] Step 3.4: Render with zoom

### Phase 4: Navigation + Polish
- [ ] Step 4.1: Geometric navigation
- [ ] Step 4.2: Origin-based leading edge
- [ ] Step 4.3: Spawn direction
- [ ] Step 4.4: Animation configs

### Phase 5: Integration
- [ ] Step 5.1: Replace workspaces
- [ ] Step 5.2: Replace overview
- [ ] Step 5.3: Update IPC
- [ ] Step 5.4: Testing
- [ ] Step 5.5: Documentation

---

## Quick Links

- [Phase 0: Preparation](phases/phase-0-preparation.md)
- [Phase 1: Row + Canvas2D](phases/phase-1-row-and-canvas.md)
- [Phase 2: Row Spanning](phases/phase-2-row-spanning.md)
- [Phase 3: Camera System](phases/phase-3-camera.md)
- [Phase 4: Navigation + Polish](phases/phase-4-navigation.md)
- [Phase 5: Integration](phases/phase-5-integration.md)

---

## Context Documents

- [Initial Questionnaire](2D_CANVAS_QUESTIONNAIRE.md) — First round of requirements gathering
- [Follow-up Questionnaire](2D_CANVAS_QUESTIONNAIRE_2.md) — Refined requirements with answers
- [Feasibility Study](2D_CANVAS_FEASIBILITY.md) — Initial architectural analysis
