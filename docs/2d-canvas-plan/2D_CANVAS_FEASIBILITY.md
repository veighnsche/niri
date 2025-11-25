# 2D Canvas Feasibility Study (v2)

> **Updated**: Based on fresh `2d-canvas` branch from `main`. All line numbers and file sizes verified.

---

## Current Architecture Overview

### File Inventory

| File | Lines | Purpose |
|------|-------|---------|
| `scrolling.rs` | 5587 | ScrollingSpace + Column + all methods |
| `mod.rs` | 4927 | Layout struct, LayoutElement trait, overview |
| `tests.rs` | 3873 | Layout tests |
| `monitor.rs` | 2181 | Monitor, workspace switching, overview |
| `workspace.rs` | 2065 | Workspace (scrolling + floating) |
| `tile.rs` | 1428 | Tile wrapper for windows |
| `floating.rs` | 1414 | FloatingSpace |
| `tab_indicator.rs` | 412 | Tab UI for tabbed columns |
| `focus_ring.rs` | 280 | Focus ring rendering |
| `closing_window.rs` | 275 | Close animation |
| `shadow.rs` | 184 | Window shadows |
| `opening_window.rs` | 143 | Open animation |
| **Total** | **22,828** | |

---

## Key Structures Analysis

### 1. ScrollingSpace (scrolling.rs:34-94)

```rust
pub struct ScrollingSpace<W: LayoutElement> {
    columns: Vec<Column<W>>,           // Horizontal strip of columns
    data: Vec<ColumnData>,             // Cached column widths
    active_column_idx: usize,          // Currently focused column
    interactive_resize: Option<InteractiveResize<W>>,
    view_offset: ViewOffset,           // 1D HORIZONTAL ONLY (f64)
    activate_prev_column_on_removal: Option<f64>,
    view_offset_to_restore: Option<f64>,
    closing_windows: Vec<ClosingWindow>,
    view_size: Size<f64, Logical>,
    working_area: Rectangle<f64, Logical>,
    parent_area: Rectangle<f64, Logical>,
    scale: f64,
    clock: Clock,
    options: Rc<Options>,
}
```

**Key Observation**: `view_offset` is a single `f64` — purely horizontal scrolling.

### 2. ViewOffset (scrolling.rs:112-142)

```rust
pub(super) enum ViewOffset {
    Static(f64),                    // Fixed position
    Animation(Animation),           // Animating to target
    Gesture(ViewGesture),           // User dragging
}

pub(super) struct ViewGesture {
    current_view_offset: f64,
    animation: Option<Animation>,   // For DnD scroll adjustments
    tracker: SwipeTracker,          // 1D gesture tracking
    delta_from_tracker: f64,
    stationary_view_offset: f64,
    is_touchpad: bool,
    dnd_last_event_time: Option<Duration>,
    dnd_nonzero_start_time: Option<Duration>,
}
```

**Key Observation**: `SwipeTracker` tracks a single `f64` position. For 2D, we need 2D tracking.

### 3. Column (scrolling.rs:145-216)

```rust
pub struct Column<W: LayoutElement> {
    tiles: Vec<Tile<W>>,            // Vertical stack of windows
    data: Vec<TileData>,            // Per-tile cached data
    active_tile_idx: usize,
    width: ColumnWidth,             // Proportion or Fixed
    preset_width_idx: Option<usize>,
    is_full_width: bool,
    is_pending_fullscreen: bool,
    is_pending_maximized: bool,
    display_mode: ColumnDisplay,    // Normal or Tabbed
    tab_indicator: TabIndicator,
    move_animation: Option<MoveAnimation>,
    view_size: Size<f64, Logical>,
    working_area: Rectangle<f64, Logical>,
    parent_area: Rectangle<f64, Logical>,
    scale: f64,
    clock: Clock,
    options: Rc<Options>,
}
```

**Key Observation**: Column is self-contained. Can be extracted as-is.

### 4. Tile (tile.rs:38-100)

```rust
pub struct Tile<W: LayoutElement> {
    window: W,
    border: FocusRing,
    focus_ring: FocusRing,
    shadow: Shadow,
    sizing_mode: SizingMode,
    fullscreen_backdrop: SolidColorBuffer,
    restore_to_floating: bool,
    floating_window_size: Option<Size<i32, Logical>>,
    floating_pos: Option<Point<f64, SizeFrac>>,
    floating_preset_width_idx: Option<usize>,
    floating_preset_height_idx: Option<usize>,
    open_animation: Option<OpenAnimation>,
    resize_animation: Option<ResizeAnimation>,
    move_x_animation: Option<MoveAnimation>,
    move_y_animation: Option<MoveAnimation>,
    alpha_animation: Option<AlphaAnimation>,
    interactive_move_offset: Point<f64, Logical>,
    // ...
}
```

**Key Observation**: Tile already has `move_x_animation` and `move_y_animation` — supports 2D movement. **No `row_span` field exists yet** — needs to be added.

### 5. Workspace (workspace.rs:45-113)

```rust
pub struct Workspace<W: LayoutElement> {
    scrolling: ScrollingSpace<W>,   // The tiling layout
    floating: FloatingSpace<W>,     // Floating windows
    floating_is_active: FloatingActive,
    original_output: OutputId,
    output: Option<Output>,
    scale: smithay::output::Scale,
    transform: Transform,
    view_size: Size<f64, Logical>,
    working_area: Rectangle<f64, Logical>,
    shadow: Shadow,
    background_buffer: SolidColorBuffer,
    clock: Clock,
    base_options: Rc<Options>,
    options: Rc<Options>,
    name: Option<String>,
    layout_config: Option<niri_config::LayoutPart>,
    id: WorkspaceId,
}
```

**Key Observation**: Workspace composes `ScrollingSpace` + `FloatingSpace`. Canvas2D will replace this.

### 6. Monitor (monitor.rs:48-90)

```rust
pub struct Monitor<W: LayoutElement> {
    output: Output,
    output_name: String,
    scale: smithay::output::Scale,
    view_size: Size<f64, Logical>,
    working_area: Rectangle<f64, Logical>,
    workspaces: Vec<Workspace<W>>,      // Multiple workspaces
    active_workspace_idx: usize,
    previous_workspace_id: Option<WorkspaceId>,
    workspace_switch: Option<WorkspaceSwitch>,
    insert_hint: Option<InsertHint>,
    insert_hint_element: InsertHintElement,
    insert_hint_render_loc: Option<InsertHintRenderLoc>,
    overview_open: bool,                // Overview state
    overview_progress: Option<OverviewProgress>,
    clock: Clock,
    base_options: Rc<Options>,
    options: Rc<Options>,
    layout_config: Option<niri_config::LayoutPart>,
}
```

**Key Observation**: Monitor manages multiple workspaces. In 2D mode, this becomes a single Canvas2D.

### 7. Animation System (animation/mod.rs)

```rust
pub struct Animation {
    from: f64,
    to: f64,
    initial_velocity: f64,
    is_off: bool,
    duration: Duration,
    clamped_duration: Duration,
    start_time: Duration,
    clock: Clock,
    kind: Kind,  // Easing, Spring, or Deceleration
}
```

**Key Observation**: Animation is already generic over `f64`. Can be reused for X, Y, and zoom independently.

### 8. SwipeTracker (input/swipe_tracker.rs)

```rust
pub struct SwipeTracker {
    history: VecDeque<Event>,
    pos: f64,  // Single dimension
}
```

**Key Observation**: 1D only. For 2D pan gestures, we need a 2D variant or two trackers.

---

## What Needs to Change for 2D Canvas

### Changes Required

| Component | Current | Required Change | Difficulty |
|-----------|---------|-----------------|------------|
| **ViewOffset** | `f64` (X only) | `Point<f64>` (X, Y) + zoom | Medium |
| **SwipeTracker** | 1D | 2D variant or paired trackers | Medium |
| **Tile** | No row_span | Add `row_span: u8` field | Easy |
| **Column** | Embedded in scrolling.rs | Extract to `column/` module | Medium |
| **ScrollingSpace** | 1D horizontal | Becomes `Row` (unchanged logic) | Easy |
| **Workspace** | Single ScrollingSpace | Becomes `Canvas2D` with `Row[]` | Medium |
| **Monitor** | `Vec<Workspace>` | Single `Canvas2D` (feature-flagged) | Medium |
| **Overview** | Separate zoom mode | Continuous zoom (replaces overview) | Medium |
| **Animations config** | `horizontal_view_movement` | Add `vertical_view_movement`, `camera_zoom` | Easy |

### What Can Stay Unchanged

| Component | Why It's Reusable |
|-----------|-------------------|
| **Tile** | Position-agnostic, just add `row_span` |
| **Column** | Self-contained vertical stack |
| **Animation** | Already generic `f64` |
| **Clock** | Timing system, no changes |
| **Options** | Just add new config fields |
| **LayoutElement trait** | Core abstraction, unchanged |
| **FloatingSpace** | Already has 2D positioning |
| **Focus ring, shadow, borders** | Rendering primitives |

---

## Detailed Impact Analysis

### 1. View Offset → Camera

**Current** (scrolling.rs:112-142):
```rust
enum ViewOffset {
    Static(f64),
    Animation(Animation),
    Gesture(ViewGesture),
}
```

**Required**:
```rust
struct Camera {
    x: AnimatedValue<f64>,      // Reuse ViewOffset pattern
    y: AnimatedValue<f64>,      // New
    zoom: AnimatedValue<f64>,   // New
    manual_override: bool,      // For user zoom control
}

enum AnimatedValue<T> {
    Static(T),
    Animation(Animation),
    Gesture(GestureTracker),
}
```

**Files affected**: New `camera/` module, `scrolling.rs` updated to use it.

### 2. Row Spanning

**Current Tile** (tile.rs:38):
```rust
pub struct Tile<W: LayoutElement> {
    window: W,
    // ... no row_span
}
```

**Required**:
```rust
pub struct Tile<W: LayoutElement> {
    window: W,
    row_span: u8,  // NEW: 1 = normal, 2 = spans two rows
    // ...
}
```

**Impact**: 
- Add field + getter/setter
- Layout calculation uses `row_span * row_height` for tile height
- Canvas2D tracks which positions are "occupied" by spanning tiles

### 3. Gesture Tracking

**Current** (input/swipe_tracker.rs):
```rust
pub struct SwipeTracker {
    history: VecDeque<Event>,
    pos: f64,  // 1D
}

struct Event {
    delta: f64,  // 1D
    timestamp: Duration,
}
```

**Options**:
1. Create `SwipeTracker2D` with `Point<f64>` position
2. Use two `SwipeTracker` instances (X and Y)
3. Keep 1D for horizontal, add separate Y tracker

**Recommendation**: Option 2 — minimal changes, reuses existing code.

### 4. Animation Configuration

**Current** (niri-config/src/animations.rs):
```rust
pub struct Animations {
    pub horizontal_view_movement: HorizontalViewMovementAnim,
    pub overview_open_close: OverviewOpenCloseAnim,
    // ...
}
```

**Required**:
```rust
pub struct Animations {
    pub horizontal_view_movement: HorizontalViewMovementAnim,
    pub vertical_view_movement: VerticalViewMovementAnim,  // NEW
    pub camera_zoom: CameraZoomAnim,                       // NEW
    // overview_open_close may be removed or repurposed
}
```

### 5. Overview → Continuous Zoom

**Current** (monitor.rs:79-81):
```rust
pub(super) overview_open: bool,
overview_progress: Option<OverviewProgress>,
```

**Current zoom calculation** (mod.rs:4941-4950):
```rust
fn compute_overview_zoom(options: &Options, overview_progress: Option<f64>) -> f64 {
    let zoom = options.overview.zoom.clamp(0.0001, 0.75);
    if let Some(p) = overview_progress {
        (1. - p * (1. - zoom)).max(0.0001)
    } else {
        1.
    }
}
```

**Required**: 
- Remove `overview_open` bool
- Camera zoom is continuous (0.1 to 2.0)
- `toggle-overview` action zooms to fit all tiles
- Clicking while zoomed focuses + zooms in

---

## Dependency Graph

```
Monitor
└── Canvas2D (replaces Workspace)
    ├── Row[] (each Row ≈ ScrollingSpace)
    │   └── Column[]
    │       └── Tile[]
    ├── FloatingSpace (unchanged)
    └── Camera
        ├── AnimatedValue<f64> x
        ├── AnimatedValue<f64> y
        └── AnimatedValue<f64> zoom
```

**Build order**:
1. Extract `Column` to `column/` module (no dependencies)
2. Create `AnimatedValue` (generalizes ViewOffset)
3. Create `Camera` (uses AnimatedValue)
4. Create `Row` (uses Column, AnimatedValue)
5. Create `Canvas2D` (uses Row, Camera, FloatingSpace)
6. Update `Monitor` (feature-flagged Canvas2D vs Workspace)

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking existing users | Medium | High | Feature flag, keep old code path |
| Performance with many rows | Low | Medium | Only render visible rows |
| Complex navigation edge cases | Medium | Medium | Extensive testing, fallback behavior |
| Animation jank during zoom | Medium | Low | Tune animation curves |
| Gesture conflicts (pan vs zoom) | Medium | Medium | Clear modifier key separation |

---

## Feasibility Verdict

### ✅ FEASIBLE

The 2D canvas is achievable with the current architecture because:

1. **Tile is position-agnostic** — Just add `row_span` field
2. **Column is self-contained** — Can be extracted and reused as-is
3. **Animation system is generic** — Already handles `f64`, works for X, Y, zoom
4. **FloatingSpace already has 2D** — Proves the rendering pipeline supports it
5. **Overview zoom exists** — Shows zoom rendering already works

### Estimated Effort

| Phase | Effort | Notes |
|-------|--------|-------|
| Phase 0: Modular foundation | 1 week | Extract Column, create AnimatedValue |
| Phase 1: Row + Canvas2D | 1-2 weeks | Core 2D structure |
| Phase 2: Row spanning | 1 week | Tile field + layout logic |
| Phase 3: Camera system | 1-2 weeks | Zoom + auto-follow |
| Phase 4: Navigation | 1 week | Geometric nav, origin edges |
| Phase 5: Integration | 1-2 weeks | Replace workspaces, IPC, docs |
| **Total** | **5-7 weeks** | |

---

## Recommended Approach

**Incremental refactor with feature flag**:

1. Keep `scrolling.rs` working throughout
2. Extract Column first (no feature flag needed)
3. Build Row/Canvas2D behind `canvas-2d` feature
4. Test both modes in parallel
5. Eventually make 2D the default

This approach minimizes risk while allowing steady progress.
