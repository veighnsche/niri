# Phase 5: Integration

> **Goal**: Update overview, IPC, testing, documentation.
> 
> ⚠️ **NOTE**: Workspace replacement moved to Phase 1.5.3 (done early as breaking change).

---

## Prerequisites

- Phase 4 complete (navigation and polish done)
- **Phase 1.5.3-4 complete** (workspaces already removed, Canvas2D integrated)

### Starting Point After Phase 4
```
src/layout/
├── column/              # Modular column (Phase 0)
├── animated_value/      # Reusable animations (Phase 0)
├── row/                 # Row = horizontal strip (Phase 1)
├── canvas/              # Canvas2D with spanning + navigation (Phase 1-4)
├── camera/              # Camera with zoom (Phase 3)
├── tile.rs              # Has row_span field
└── monitor.rs           # Uses Canvas2D (workspaces removed in Phase 1.5.3)
```

All 2D canvas features work. This phase polishes integration and adds documentation.

---

## Step 5.1: ~~Replace Workspaces~~ ✅ DONE EARLY (Phase 1.5.3)

> **Moved to Phase 1.5.3** — Workspaces are removed as a breaking change, not feature-flagged.
> See `phase-1.5-integration.md` for the actual implementation.

### What Was Done in Phase 1.5.3
- [x] Removed `workspaces: Vec<Workspace<W>>` from Monitor
- [x] Added `canvas: Canvas2D<W>` to Monitor
- [x] Removed workspace switching logic
- [x] Removed overview mode
- [x] Removed hot corner
- [x] Disabled `Mod+1/2/3` (will be repurposed for camera bookmarks later)

### Keybind Repurposing (Future — Phase 5.4)
The `Mod+1/2/3` keybinds are currently disabled. They will be repurposed for camera bookmarks:
```kdl
// Future keybinds (not implemented yet)
binds {
    Mod+1 { jump-to-bookmark 1; }
    Mod+2 { jump-to-bookmark 2; }
    Mod+Shift+1 { save-bookmark 1; }
    // etc.
}
```

---

## Step 5.2: Replace Overview

### Current Overview

- Triggered by keybind or gesture
- Zooms out to show all workspaces
- Click to switch workspace

### 2D Overview

- Triggered by same keybind/gesture
- Zooms out to show entire canvas (all rows)
- Click to focus any tile
- Stays interactive while zoomed

### Tasks

- [ ] **5.2.1**: Remove `overview_open` bool — zoom is continuous
- [ ] **5.2.2**: Add `toggle-overview` action that zooms to fit all
- [ ] **5.2.3**: Compute zoom level to fit all tiles
- [ ] **5.2.4**: Keep full interactivity while zoomed out
- [ ] **5.2.5**: Click on tile → focus + zoom in
- [ ] **5.2.6**: Test overview at various canvas sizes

### Implementation

```rust
impl<W: LayoutElement> Canvas2D<W> {
    /// Zoom out to fit all tiles on canvas.
    pub fn zoom_to_fit_all(&mut self) {
        let bounds = self.compute_all_tiles_bounds();
        let viewport = self.viewport_size();
        
        // Compute zoom to fit bounds in viewport
        let zoom_x = viewport.w / bounds.size.w;
        let zoom_y = viewport.h / bounds.size.h;
        let zoom = zoom_x.min(zoom_y).min(1.0); // Don't zoom in past 1.0
        
        // Center on bounds
        let center = Point::from((
            bounds.loc.x + bounds.size.w / 2.0,
            bounds.loc.y + bounds.size.h / 2.0,
        ));
        
        let camera_x = center.x - (viewport.w / zoom) / 2.0;
        let camera_y = center.y - (viewport.h / zoom) / 2.0;
        
        self.camera.animate_to(camera_x, camera_y, zoom, &self.options.animations.camera);
        self.camera.manual_override = true;
    }
    
    /// Toggle between overview (fit all) and focused view.
    pub fn toggle_overview(&mut self) {
        if self.camera.manual_override {
            // Currently in overview, return to focused
            self.camera.manual_override = false;
            self.update_camera_for_focus();
        } else {
            // Go to overview
            self.zoom_to_fit_all();
        }
    }
}
```

---

## Step 5.3: Update IPC

### New IPC Fields

```rust
// niri-ipc/src/lib.rs

pub struct Window {
    // ... existing fields
    
    /// Row index in 2D canvas (0 = origin row)
    pub row: i32,
    
    /// Number of rows this window spans
    pub row_span: u8,
}

pub struct CanvasState {
    /// Active row index
    pub active_row: i32,
    
    /// Camera position
    pub camera_x: f64,
    pub camera_y: f64,
    pub camera_zoom: f64,
    
    /// All rows with their window counts
    pub rows: Vec<RowInfo>,
}

pub struct RowInfo {
    pub index: i32,
    pub column_count: usize,
    pub window_count: usize,
}
```

### Tasks

- [ ] **5.3.1**: Add `row` and `row_span` to Window struct
- [ ] **5.3.2**: Add CanvasState for `niri msg canvas`
- [ ] **5.3.3**: Update event stream for 2D events
- [ ] **5.3.4**: Graceful degradation: workspace IPC returns empty in 2D mode
- [ ] **5.3.5**: Update `niri msg` CLI for new commands
- [ ] **5.3.6**: Test IPC with gutter-bar

### New IPC Commands

| Command | Description |
|---------|-------------|
| `niri msg canvas` | Get canvas state (rows, camera) |
| `niri msg focus-row N` | Focus row N |
| `niri msg set-row-span N` | Set focused window's row span |
| `niri msg zoom LEVEL` | Set camera zoom |

---

## Step 5.4: Testing

### Test Categories

1. **Unit Tests**: Individual module tests (Column, Row, Canvas2D, Camera)
2. **Integration Tests**: End-to-end behavior
3. **Visual Tests**: Using niri-visual-tests
4. **Manual Tests**: Interactive testing

### Tasks

- [ ] **5.4.1**: Port existing layout tests to Canvas2D
- [ ] **5.4.2**: Add tests for row spanning
- [ ] **5.4.3**: Add tests for geometric navigation
- [ ] **5.4.4**: Add tests for camera behavior
- [ ] **5.4.5**: Add tests for origin-based edges
- [ ] **5.4.6**: Create visual test cases
- [ ] **5.4.7**: Manual testing checklist

### Test Cases

```rust
#[cfg(test)]
mod tests {
    // Row spanning
    #[test]
    fn tile_spans_two_rows() { ... }
    
    #[test]
    fn cannot_place_in_occupied_span() { ... }
    
    #[test]
    fn spanning_tile_renders_once() { ... }
    
    // Navigation
    #[test]
    fn focus_down_finds_nearest() { ... }
    
    #[test]
    fn focus_right_wraps_correctly() { ... }
    
    // Camera
    #[test]
    fn camera_zooms_for_row_span() { ... }
    
    #[test]
    fn manual_zoom_overrides_auto() { ... }
    
    // Origin
    #[test]
    fn quadrant_determines_leading_edge() { ... }
    
    #[test]
    fn spawn_direction_follows_quadrant() { ... }
}
```

### Manual Testing Checklist

```markdown
## 2D Canvas Manual Testing

### Basic Operations
- [ ] Create new window → appears in correct position
- [ ] Close window → layout adjusts
- [ ] Move window left/right → works
- [ ] Move window up/down (to new row) → works

### Row Spanning
- [ ] Increase row span → window grows to 2 rows
- [ ] Decrease row span → window shrinks
- [ ] Camera zooms out when focusing 2-row window
- [ ] Camera zooms in when focusing 1-row window

### Navigation
- [ ] Focus down → goes to nearest below
- [ ] Focus up → goes to nearest above
- [ ] Focus left/right → works within row
- [ ] No tile in direction → focus stays

### Camera
- [ ] Mod+ScrollDown → zooms out
- [ ] Mod+ScrollUp → zooms in
- [ ] Click while zoomed → focuses and zooms to fit
- [ ] Overview toggle → shows all tiles

### Origin Behavior
- [ ] Resize right of origin → right edge leads
- [ ] Resize left of origin → left edge leads
- [ ] Spawn right of origin → new window to right
- [ ] Spawn left of origin → new window to left
```

---

## Step 5.5: Documentation

### Tasks

- [ ] **5.5.1**: Update wiki with 2D canvas concepts
- [ ] **5.5.2**: Document new keybinds
- [ ] **5.5.3**: Document new config options
- [ ] **5.5.4**: Create migration guide
- [ ] **5.5.5**: Update README
- [ ] **5.5.6**: Add examples to default-config.kdl

### Documentation Structure

```
docs/wiki/
├── 2D-Canvas.md           # NEW: Overview of 2D canvas
├── 2D-Canvas-Keybinds.md  # NEW: Keybind reference
├── 2D-Canvas-Config.md    # NEW: Configuration options
├── Migration-to-2D.md     # NEW: Migration guide
└── ... existing docs
```

### Key Documentation Topics

1. **Concept Overview**
   - What is the 2D canvas?
   - Rows, columns, tiles
   - Row spanning
   - Camera and zoom

2. **Keybind Reference**
   - Navigation (up/down/left/right)
   - Row span controls
   - Zoom controls
   - Row jumping

3. **Configuration**
   - Animation settings
   - Default row span
   - Zoom limits
   - Origin position (if configurable)

4. **Migration Guide**
   - What changes from workspaces
   - How to update config
   - Breaking changes

---

## Checklist Summary

### Step 5.1: Replace Workspaces
- [ ] Remove workspace switching
- [ ] Repurpose keybinds
- [ ] Add move-window-to-row
- [ ] Handle IPC gracefully
- [ ] Test migration

### Step 5.2: Replace Overview
- [ ] Remove overview_open
- [ ] toggle-overview zooms to fit
- [ ] Interactive while zoomed
- [ ] Click to focus
- [ ] Test at various sizes

### Step 5.3: IPC
- [ ] Add row, row_span to Window
- [ ] Add CanvasState
- [ ] Update event stream
- [ ] New CLI commands
- [ ] Test with gutter-bar

### Step 5.4: Testing
- [ ] Port existing tests
- [ ] Add spanning tests
- [ ] Add navigation tests
- [ ] Add camera tests
- [ ] Visual tests
- [ ] Manual checklist

### Step 5.5: Documentation
- [ ] Wiki pages
- [ ] Keybind docs
- [ ] Config docs
- [ ] Migration guide
- [ ] Update README

---

## Estimated Time: 1-2 Weeks

---

## Success Criteria

- [x] Workspaces fully replaced by 2D canvas (Phase 1.5.3)
- [ ] All keybinds work as documented
- [ ] IPC updated and working
- [ ] All tests pass
- [ ] Documentation complete
- [ ] Breaking changes documented in migration guide

---

## Post-Launch

### Follow-up Items

- [ ] Gather user feedback
- [ ] Performance optimization if needed
- [ ] Gutter-bar integration improvements
- [ ] Additional row span limits (3+)

---

## Project Complete! 🎉

Total estimated time: **5-7 weeks**

| Phase | Weeks |
|-------|-------|
| Phase 0: Preparation | 1 |
| Phase 1: Row + Canvas2D | 1-2 |
| Phase 2: Row Spanning | 1 |
| Phase 3: Camera System | 1-2 |
| Phase 4: Navigation | 1 |
| Phase 5: Integration | 1-2 |
