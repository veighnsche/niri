# 2D Canvas Refactor — Clean TODO

> **Last Updated**: Nov 29, 2025 (TEAM_066 Audit)
> **Status**: Core refactor complete, feature work remaining

---

## 📊 Current Status Summary

### ✅ COMPLETED (Verified by TEAM_066)

| Phase | Description | Team |
|-------|-------------|------|
| Phase 0 | Dead code cleanup | TEAM_062 |
| Phase 1 | elements/ module creation | TEAM_062 |
| Phase 2 | FloatingSpace → canvas/floating/ | TEAM_063 |
| Phase 3 | tile.rs → tile/ module | TEAM_063 |
| Phase 5.1-5.8 | Layout implementation extraction | TEAM_063 |
| Phase 5.9 | interactive_move.rs extraction | TEAM_064 |
| Phase 5.10 | render.rs extraction | TEAM_064 |
| Phase 6 | canvas/operations/ split | TEAM_065 |
| Deferred 1 | row/mod.rs partial split | TEAM_064 |

### 📁 Current Module Architecture

```
src/layout/  (79 files)
├── mod.rs (1860 LOC)           # Core Layout struct
├── types.rs (66 LOC)           # Shared types
├── row_types.rs (66 LOC)       # RowId, etc.
├── snapshot.rs                  # Golden testing
├── tests.rs + tests/           # Tests
├── deprecated/                  # scrolling.rs (reference only)
│
├── animated_value/ (2 files)    # Animation values
├── canvas/ (10 files)           # Canvas2D struct
│   ├── floating/ (4 files)     # FloatingSpace
│   └── operations/ (5 files)   # Window/tile/row ops
├── column/ (12 files)           # Column struct
├── elements/ (7 files)          # Render elements
├── layout_impl/ (12 files)      # Layout methods (3831 LOC)
├── monitor/ (7 files)           # Monitor struct
├── row/ (16 files)              # Row struct
└── tile/ (3 files)              # Tile struct
```

### 🧪 Test Status

- **All tests pass**: 270 tests ✅
- **Layout tests**: 187 tests ✅
- **Golden tests**: 88 tests ✅
- **Warnings**: 48 (unused imports - minor cleanup)

---

## 🎯 REMAINING WORK

### Priority 1: Camera Zoom System (Critical - THE DIFFERENTIATOR)

Without zoom, Canvas2D is functionally identical to workspaces!

#### Step 1: Add Camera Struct (30 min)
Create `canvas/camera.rs` with `x`, `y`, `zoom` fields.

```rust
pub struct Camera {
    x: AnimatedValue,
    y: AnimatedValue,
    zoom: AnimatedValue,  // 1.0 = normal, 0.5 = see 2x area
}
```

#### Step 2: Integrate Camera (30 min)
Replace `camera_x`, `camera_y` fields in Canvas2D with Camera struct.

#### Step 3: Add visible_area() (20 min)
Calculate what area of canvas is visible at current zoom.

#### Step 4: Add visible_rows() (30 min)
Filter rows to only those visible at current zoom.

#### Step 5: Update Rendering (1 hour)
Scale render elements by zoom factor.

#### Step 6: Add Input Transform (30 min)
Convert screen coordinates to canvas coordinates at any zoom.

**Total**: ~3.5 hours

---

### Priority 2: Camera Bookmarks (After Zoom)

Camera bookmarks replace workspace switching.

#### Step 1: Add CameraBookmark Struct (15 min)
```rust
pub struct CameraBookmark {
    x: f64,
    y: f64,
    zoom: f64,
    name: Option<String>,
}
```

#### Step 2: Add Storage to Canvas2D (20 min)
```rust
bookmarks: HashMap<u8, CameraBookmark>,  // 0-9 for quick access
```

#### Step 3: Implement save_bookmark() (20 min)
#### Step 4: Implement goto_bookmark() (30 min)
#### Step 5: Add Layout Methods (20 min)
#### Step 6: Add Keybindings (30 min)
- `Mod+1-9` → goto bookmark
- `Mod+Shift+1-9` → save bookmark

#### Step 7: Add IPC Commands (30 min)
- `camera-bookmark-save <index>`
- `camera-bookmark-goto <index>`

**Total**: ~3 hours

---

### Priority 3: IPC/Protocol Migration (After Bookmarks)

Update IPC to reflect Canvas2D semantics.

#### Step 1: Rename IPC Events (30 min)
- `WorkspacesChanged` → `RowsChanged`
- `WorkspaceActivated` → `RowActivated`
- etc.

#### Step 2: Rename IPC Structures (30 min)
- `Workspace` → `Row`
- `WorkspacesState` → `RowsState`

#### Step 3: Update IPC Handlers (45 min)
#### Step 4: Update niri-msg (30 min)

**Total**: ~2.5 hours

---

### Priority 4: Row Spanning (Future)

Allow windows to span multiple rows.

#### Step 1: Add row_span to Tile (15 min)
#### Step 2: Update Tile Height Calculation (30 min)
#### Step 3: Track Cross-Row Occupancy (45 min)
#### Step 4: Update Navigation (30 min)
#### Step 5: Add Row Span Actions (20 min)

**Total**: ~2.5 hours

---

## 📋 Minor Cleanup Tasks

### Code Quality (30 min)
- [ ] Remove 48 unused import warnings
- [ ] Clean up internal `seen_workspace_*` variable names → `seen_row_*`

### Documentation (1 hour)
- [ ] Update README.md with Canvas2D architecture
- [ ] Update wiki examples for new row syntax

### Deprecated Code (After merge to main)
- [ ] Delete `deprecated/scrolling.rs` (currently kept for reference)

---

## 📊 Total Remaining Effort

| Area | Estimated Time |
|------|----------------|
| Camera Zoom | ~3.5 hours |
| Camera Bookmarks | ~3 hours |
| IPC Migration | ~2.5 hours |
| Row Spanning | ~2.5 hours |
| Cleanup | ~1.5 hours |
| **Total** | **~13 hours** |

---

## ✅ Success Criteria

The Canvas2D refactor is complete when:

1. **Zoom works**: User can zoom in/out to see more/fewer rows
2. **Bookmarks work**: User can save and restore camera positions
3. **IPC reflects reality**: Protocol exposes rows, not workspaces
4. **All tests pass**: No regressions from workspace behavior
5. **Code is clean**: No unused imports, clear module structure

---

## 🗂️ Reference

### Team Files
See `docs/2d-canvas-plan/.teams/TEAM_0XX_*.md` for detailed history.

### Key Teams
- TEAM_062: Architecture refactor plan
- TEAM_063: FloatingSpace + tile + layout_impl extraction
- TEAM_064: Interactive move + render + row state extraction
- TEAM_065: canvas/operations split
- TEAM_066: This audit/cleanup

### Original Docs
- `docs/2d-canvas-plan/README.md` - Original vision
- `docs/2d-canvas-plan/phases/` - Phase documentation
