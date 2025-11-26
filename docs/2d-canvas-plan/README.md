# 2D Canvas Implementation Plan

> Transform niri from a 1D scrolling tiler to a 2D canvas with rows, row-spanning windows, and dynamic camera zoom — built from modular, well-encapsulated components.

---

## ⚠️ **CRITICAL: GOLDEN TEST RULES**

**LEARNED FROM TEAM_018's MISTAKES - READ BEFORE STARTING**

🚨 **NEVER accept golden snapshot changes** - they represent MAIN BRANCH behavior  
🚨 **NEVER remove workspace-related golden tests** - they must continue working  
🚨 **If golden tests fail, fix YOUR CODE** - not the tests  

**FULL GUIDELINES**: See [GOLDEN_TEST_RULES.md](GOLDEN_TEST_RULES.md)  
**COMPLIANCE CHECKLIST**: See [GOLDEN_TEST_CHECKLIST.md](GOLDEN_TEST_CHECKLIST.md)

*TEAM_018 violated these rules and had to revert all work. Don't make their mistake.*

---

## 🚀 CURRENT WORK: Phase 1.5.3 — Replace Workspace with Canvas2D

**Status**: IN PROGRESS (Part 1)  
**Latest Team**: Check [.teams/](.teams/) for the most recent `TEAM_XXX_*.md` file

### Phase 1.5.3 Parts

| Part | Description | Status |
|------|-------------|--------|
| [Part 1](phases/phase-1.5.3-part1-monitor-methods.md) | Migrate Monitor methods to Canvas2D | ✅ Complete |
| **[Part 2: Workspace → Row](phases/phase-1.5.3-part2-remove-workspace-switching.md)** | Replace + Refactor (alphabetical = execution order) | ✅ Complete |
| [Part 2A](phases/phase-1.5.3-part2a-config-workspace-actions.md) | Replace Config Actions (Workspace → Row) | ✅ Complete (TEAM_012) |
| [Part 2B](phases/phase-1.5.3-part2b-input-workspace-actions.md) | Replace Input Handlers | ✅ Complete (TEAM_012) |
| [Part 2C](phases/phase-1.5.3-part2c-layout-workspace-switching.md) | Replace Layout Methods | ✅ Complete (TEAM_012) |
| [Part 2D](phases/phase-1.5.3-part2d-monitor-workspace-switching.md) | Refactor Monitor → Modules + Row Nav | ✅ Complete (TEAM_013) |
| [Part 2E](phases/phase-1.5.3-part2e-remove-workspace-tests.md) | Replace/Remove Tests | ✅ Complete (TEAM_014) |
| [Part 3](phases/phase-1.5.3-part3-remove-overview.md) | Remove overview mode | ⚠️ DISABLED (TEAM_014) |
| [Part 3 Cleanup](phases/phase-1.5.3-part3-overview-removal-guide.md) | Delete all `DEPRECATED(overview)` code | 🔄 NEXT |
| [Part 4](phases/phase-1.5.3-part4-remove-workspace-fields.md) | Remove workspace fields from Monitor | ⏳ Pending |
| [Part 5](phases/phase-1.5.3-part5-config-and-ipc.md) | Remove workspace config and IPC | ⏳ Pending |

### Pre-Work Complete
- ✅ Canvas2D field added to Monitor struct
- ✅ `canvas()` and `canvas_mut()` accessors added
- ✅ `snapshot()` methods added to Row and Canvas2D
- ✅ `tiles()`, `windows()` methods added to Canvas2D and Row
- ✅ Golden snapshot infrastructure fixed
- ✅ Planning docs reorganized

### Quick Start for New Teams
```bash
# 1. Read the rules
cat docs/2d-canvas-plan/ai-teams-rules.md

# 2. Check latest team file
ls -la docs/2d-canvas-plan/.teams/

# 3. Verify golden tests pass BEFORE starting
./scripts/verify-golden.sh

# 4. Claim your team number and create team file
# 5. Continue Phase 1.5.3 work
```

---

## AI Teams

**Read [ai-teams-rules.md](ai-teams-rules.md) first.**

⚠️ **Golden tests**: Run `./scripts/verify-golden.sh` before AND after changes.  
⚠️ **Never accept**: `cargo insta accept` on golden tests — snapshots come from `golden-snapshots` branch.

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
  │   └── sizing/       (width/height - refactored by TEAM_008)
  │       ├── mod.rs
  │       ├── tile_sizes.rs
  │       ├── width.rs
  │       ├── height.rs
  │       └── display.rs
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
| [Phase 1](phases/phase-1-row-and-canvas.md) | Row + Canvas2D | Create core modules with basic functionality |
| [Phase 1.5](phases/phase-1.5-integration.md) | Integration | Complete modules, wire into compositor |
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
├── animated_value/           # Animation abstraction ✅ COMPLETE
│   ├── mod.rs                # AnimatedValue enum (Static/Animation/Gesture)
│   └── gesture.rs            # ViewGesture struct
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
├── row/                      # Row module (NEW) ✅ COMPLETE
│   ├── mod.rs                # Row struct + public interface
│   ├── layout.rs             # Column positioning within row
│   ├── navigation.rs         # Left/right focus movement + activate_column
│   ├── operations/           # Column operations (refactored by TEAM_008)
│   │   ├── mod.rs            # Submodule declarations
│   │   ├── add.rs            # Add tile/column
│   │   ├── remove.rs         # Remove tile/column
│   │   ├── move_col.rs       # Move column left/right
│   │   └── consume.rs        # Consume/expel window
│   ├── view_offset.rs        # View offset calculation & animation
│   ├── gesture.rs            # Gesture-based scrolling
│   ├── resize.rs             # Interactive resize
│   └── render.rs             # Rendering
│
├── canvas/                   # Canvas2D module (NEW) ✅ COMPLETE (TEAM_009)
│   ├── mod.rs                # Canvas2D struct + accessors
│   ├── navigation.rs         # Up/down/left/right focus (refactored by TEAM_008)
│   ├── operations.rs         # Add/remove/find windows
│   ├── render.rs             # Rendering
│   └── floating.rs           # Floating window operations
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

### Phase 0: Preparation ✅ COMPLETE
- [x] Step 0.1: Extract Column from scrolling.rs → layout/column/ module (TEAM_002)
- [x] Step 0.3: Clean up ScrollingSpace dependencies (TEAM_003)
- [x] **Step 0.5: Golden snapshot infrastructure** (TEAM_004)
  - [x] 0.5.A: Infrastructure setup (insta, types)
  - [x] 0.5.B: Golden code extraction, snapshot() methods
  - [x] 0.5.C: Core golden tests (Groups A-L) — 30 tests
  - [x] 0.5.D: Advanced golden tests (Groups M-W) — 28 tests (58 total)
- [x] **Step 0.2: Create AnimatedValue abstraction** (TEAM_005)
  - [x] AnimatedValue enum (Static/Animation/Gesture)
  - [x] AnimatedPoint for 2D (Camera prep)
  - [x] ViewGesture extracted to animated_value/gesture.rs

### Phase 1: Row + Canvas2D ✅ CORE COMPLETE
- [x] Step 1.1: Create Row module (TEAM_006, TEAM_007)
  - [x] Core struct and accessors
  - [x] Navigation (focus_left/right/column)
  - [x] Operations (add/remove/move columns)
  - [x] View offset animation logic
  - [x] Rendering (render_elements, update_render_elements)
- [x] Step 1.2: Create Canvas2D module (TEAM_006, TEAM_007)
  - [x] BTreeMap-based row storage
  - [x] Window operations (add_tile, contains, find)
  - [x] Rendering (render_elements, update_render_elements)
- [x] Step 1.3: Basic vertical navigation (TEAM_006, TEAM_007)
  - [x] focus_up, focus_down
  - [x] Camera Y animation
- [ ] Step 1.4: Feature flag integration → moved to Phase 1.5

### Phase 1.5: Integration (IN PROGRESS)
- [x] Step 1.5.1: Complete Row module (TEAM_007, TEAM_008)
  - [x] Gesture handling (gesture.rs)
  - [x] Interactive resize (resize.rs)
  - [x] Window operations (add/remove/consume/expel)
  - [x] render_above_top_layer
  - [x] Refactored operations.rs into submodules (TEAM_008)
- [x] Step 1.5.2: Complete Canvas2D (TEAM_009, TEAM_008)
  - [x] FloatingSpace integration
  - [x] Window operations (add_window, remove_window, toggle_floating)
  - [x] Refactored mod.rs into submodules (TEAM_008)
  - [ ] Camera offset (deferred to Phase 3)
- [ ] Step 1.5.3: Replace Workspace with Canvas2D (BREAKING CHANGE)
- [ ] Step 1.5.4: Monitor integration (wire Canvas2D)

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

### Current Phase
- [**Phase 1.5.3: Workspace Removal**](phases/phase-1.5.3-removal-checklist.md) ⬅️ **CURRENT** — Detailed removal steps
- [Phase 1.5: Integration](phases/phase-1.5-integration.md) — Parent phase context

### Future Phases
- [Phase 2: Row Spanning](phases/phase-2-row-spanning.md)
- [Phase 3: Camera System](phases/phase-3-camera.md)
- [Phase 4: Navigation + Polish](phases/phase-4-navigation.md)
- [Phase 5: Final Integration](phases/phase-5-integration.md)

### Completed Phases (Archived)
- [Phase 0: Preparation](phases/archive/phase-0-preparation.md) ✅
- [Phase 1: Row + Canvas2D](phases/archive/phase-1-row-and-canvas.md) ✅
- [Phase 0.5: Golden Snapshots](phases/archive/) ✅

### Reference Documents
- [Animation Regression Checklist](phases/animation-regression-checklist.md) — Ensure no animation regressions
- [Golden Snapshot Testing](ai-teams-rules.md#rule-4-golden-snapshot-testing) — In ai-teams-rules.md

---

## Archive

These documents are kept for historical reference but are no longer actively used:

- [2D_CANVAS_FEASIBILITY.md](2D_CANVAS_FEASIBILITY.md) — Initial architectural analysis
- [.questions/](.questions/) — Historical questionnaires and USER answers
