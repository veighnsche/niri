# 2D Canvas Implementation Plan

> Transform niri from a 1D scrolling tiler to a 2D canvas with rows, row-spanning windows, and dynamic camera zoom — built from modular, well-encapsulated components.

**AI Teams: Read [ai-teams-rules.md](ai-teams-rules.md) first. Check [.teams/](.teams/) for recent activity.**

⚠️ **Before ANY layout refactor**: Run `cargo insta test` to verify golden tests pass. See Rule 8.

---

## Design Principles

### 1. Modular Architecture
Every component owns its own state and exposes a clean interface. No monolithic files.

```
Current:  scrolling.rs (5586 lines, everything in one file)

Target:
  layout/
  ├── column/           (Column extracted from scrolling.rs)
  │   ├── mod.rs        (struct + public interface)
  │   ├── layout.rs     (tile positioning)
  │   ├── operations.rs (add/remove/focus)
  │   └── sizing.rs     (width/height)
  ├── scrolling.rs      (ScrollingSpace only, uses column/)
  ├── row/              (NEW in Phase 1)
  └── canvas/           (NEW in Phase 1)
```

### 2. Composition Over Inheritance
Build complex behavior by composing simple, single-purpose modules.

### 3. State Ownership
Each module owns its data. No reaching into other modules' internals.

### 4. Incremental Progress
Each phase produces working, testable code. No big bang rewrites.

### 5. Golden Snapshot Testing
Refactored code must produce identical positions as original main branch code. Every layout change must pass `cargo insta test`.

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
| **Navigation** | Geometric — finds nearest window in direction (crosses rows) |

---

## Keyboard Shortcuts (2D Canvas)

### Navigation
| Shortcut | Action |
|----------|--------|
| `Mod+Left/Right` | Focus column left/right (within row) |
| `Mod+Up/Down` | Navigate up/down (geometric, crosses rows at edges) |
| `Mod+Shift+Up/Down` | Move window/column to row above/below |

### Row Spanning
| Shortcut | Action |
|----------|--------|
| `Mod+Ctrl+Plus/Minus` | Increase/decrease row span |
| `Mod+Ctrl+1/2/3` | Set row span to 1/2/3 rows |

### Camera/Zoom
| Shortcut | Action |
|----------|--------|
| `Mod+Scroll` | Zoom in/out |
| `Mod+0` | Reset zoom to 100% |
| `Mod+=` | Zoom to fit focused window |
| `Mod+Shift+=` | Zoom to fit all visible |

### Camera Bookmarks (replaces workspaces)
| Shortcut | Action |
|----------|--------|
| `Mod+1/2/3...` | Jump to saved camera position |
| `Mod+Shift+1/2/3...` | Save current camera position |

### Preserved from 1D
| Shortcut | Action |
|----------|--------|
| `Mod+Shift+Left/Right` | Move column left/right |
| `Mod+R` | Cycle preset widths |
| `Mod+F` | Maximize column |
| `Mod+Shift+F` | Fullscreen |
| `Mod+C` | Center column |
| `Mod+[/]` | Consume/expel window |

> **Note**: Workspaces are **removed**. One infinite canvas per output. Camera bookmarks replace workspace switching.

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
├── golden/                   # Original main branch code (reference)
│   ├── mod.rs                # Test-only module
│   ├── scrolling.rs          # Original scrolling.rs
│   └── ...                   # Other original files
│
├── snapshot.rs               # Snapshot types for golden testing
│
├── tests/
│   └── golden.rs             # Golden snapshot comparison tests
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

### Phase 0: Preparation (~1 week)
- [x] Step 0.1: Extract Column from scrolling.rs → layout/column/ module (TEAM_002 - COMPLETE)
- [x] Step 0.3: Clean up ScrollingSpace dependencies (TEAM_003 - COMPLETE)
- [x] **Step 0.5: Golden snapshot infrastructure** (TEAM_004 - COMPLETE)
  - [x] 0.5.A: Infrastructure setup (insta, types)
  - [x] 0.5.B: Golden code extraction, snapshot() methods
  - [x] 0.5.C: Core golden tests (Groups A-L) — 30 tests
  - [x] 0.5.D: Advanced golden tests (Groups M-W) — 28 tests (58 total)
- [x] **Step 0.2: Create AnimatedValue abstraction** (TEAM_005 - COMPLETE)
  - [x] AnimatedValue enum (Static/Animation/Gesture)
  - [x] AnimatedPoint for 2D (Camera prep)
  - [x] ViewGesture extracted to animated_value/gesture.rs

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
- [ ] Step 5.1: Replace workspaces with infinite canvas (BREAKING CHANGE)
- [ ] Step 5.2: Replace overview with zoom-out view
- [ ] Step 5.3: Update IPC (remove workspace APIs, add canvas APIs)
- [ ] Step 5.4: Implement camera bookmarks (`Mod+1/2/3` to jump, `Mod+Shift+1/2/3` to save)
- [ ] Step 5.5: Testing
- [ ] Step 5.6: Documentation

---

## Quick Links

- [Phase 0: Preparation](phases/phase-0-preparation.md)
- [Phase 0.5: Golden Snapshots](phases/phase-0.5-golden-snapshots.md) ⚠️ **REQUIRED**
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
