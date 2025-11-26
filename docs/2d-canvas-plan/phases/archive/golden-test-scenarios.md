# Golden Test Scenarios

> Comprehensive list of operations that affect layout state.
> Each step produces a snapshot that must match between golden and refactored code.

**Format**: YAML snapshots for easy LTR→RTL mirroring (negate X values)

---

## Goal Alignment: 2D Canvas Refactor

### What We're Doing
- Replacing multiple workspaces with a 2D canvas (rows + columns)
- Adding row-spanning windows and camera zoom

### What Must Stay The Same
- **Single-row LTR behavior** — all existing operations in a single row must produce identical results
- These golden tests capture the "single row" behavior that becomes the foundation for the 2D canvas

### Why Golden Tests Matter
1. **Golden code** = original main branch `scrolling.rs`, `workspace.rs`, etc.
2. **Refactored code** = our modular `column/`, `row/`, `canvas/` modules
3. **Both must produce identical snapshots** for single-row scenarios
4. When we add rows, we add NEW test groups, but existing groups must still pass

### LTR → RTL Mirroring
For RTL support, we:
1. Parse YAML snapshot
2. Negate all X values: `x_rtl = -x_ltr`
3. This gives us RTL golden snapshots automatically

---

## Test Group A: Basic Window Management

Tests: adding windows, focus movement, view offset changes

```
A00: empty_workspace
A01: add_window(W1, 1/2)           → 1 column
A02: add_window(W2, 1/2)           → 2 columns, W2 active
A03: add_window(W3, 1/2)           → 3 columns, W3 active, view scrolled
A04: focus_column_left             → W2 active, view offset changes
A05: focus_column_left             → W1 active, view offset changes
A06: focus_column_right            → W2 active
A07: focus_column_right            → W3 active
A08: focus_column_first            → W1 active
A09: focus_column_last             → W3 active
```

---

## Test Group B: Column Width Presets

Tests: cycling through preset widths (1/3, 1/2, 2/3)

```
# Starting state: [W1(1/2), W2(1/2), W3(1/2)], W1 active
B00: setup_three_columns
B01: switch_preset_column_width    → W1: 1/2 → 2/3
B02: switch_preset_column_width    → W1: 2/3 → 1/3
B03: switch_preset_column_width    → W1: 1/3 → 1/2 (cycles)
B04: focus_column_right            → W2 active
B05: switch_preset_column_width    → W2: 1/2 → 2/3
B06: switch_preset_column_width    → W2: 2/3 → 1/3
B07: focus_column_right            → W3 active
B08: switch_preset_column_width    → W3: 1/2 → 2/3
B09: switch_preset_column_width    → W3: 2/3 → 1/3
```

---

## Test Group C: Manual Width Adjustments

Tests: set-column-width with percentages and pixels

```
# Starting state: [W1(1/2)], W1 active
C00: setup_one_column
C01: set_column_width("+10%")      → W1 wider
C02: set_column_width("+10%")      → W1 wider again
C03: set_column_width("-10%")      → W1 narrower
C04: set_column_width("-10%")      → Back to ~original
C05: set_column_width("25%")       → W1 exactly 25%
C06: set_column_width("75%")       → W1 exactly 75%
C07: set_column_width("50%")       → W1 exactly 50%
```

---

## Test Group D: Full-Width (Maximize Column)

Tests: maximize-column toggle

```
# Starting state: [W1(1/2), W2(1/2), W3(1/2)], W2 active
D00: setup_three_columns_w2_active
D01: maximize_column               → W2 full-width
D02: maximize_column               → W2 back to 1/2
D03: focus_column_left             → W1 active
D04: maximize_column               → W1 full-width
D05: focus_column_right            → W2 active (scrolls)
D06: focus_column_right            → W3 active
D07: maximize_column               → W3 full-width
D08: maximize_column               → W3 back to 1/2
```

---

## Test Group E: Center Column

Tests: center-column and center-visible-columns

```
# Starting state: [W1(1/3), W2(1/3), W3(1/3), W4(1/3)], W1 active
E00: setup_four_narrow_columns
E01: center_column                 → View centers on W1
E02: focus_column_right            → W2 active
E03: center_column                 → View centers on W2
E04: focus_column_last             → W4 active
E05: center_column                 → View centers on W4
E06: center_visible_columns        → All visible columns centered
```

---

## Test Group F: Multi-Tile Columns (Consume/Expel)

Tests: consume-or-expel-window-left/right

```
# Starting state: [W1(1/2), W2(1/2), W3(1/2)], W2 active
F00: setup_three_columns_w2_active
F01: consume_or_expel_window_left  → W2 joins W1's column: [W1+W2, W3]
F02: focus_window_down             → W2 active (in column with W1)
F03: focus_window_up               → W1 active
F04: consume_or_expel_window_right → W1 joins right column: [W2, W1+W3]
F05: focus_column_left             → W2 active (alone)
F06: consume_or_expel_window_right → W2 joins W1+W3: [W2+W1+W3]
F07: focus_window_down             → W1 active
F08: focus_window_down             → W3 active
F09: expel_window_from_column      → W3 expelled: [W2+W1, W3]
F10: consume_window_into_column    → Consumes W3 back: [W2+W1+W3]
```

---

## Test Group G: Move Window Within Column

Tests: move-window-up/down

```
# Starting state: [W1+W2+W3 in column], W2 active (middle)
G00: setup_three_tiles_in_column
G01: move_window_up                → Order: W2, W1, W3 (W2 now top)
G02: move_window_down              → Order: W1, W2, W3 (back to original)
G03: move_window_down              → Order: W1, W3, W2 (W2 now bottom)
G04: focus_window_up               → W3 active
G05: move_window_up                → Order: W3, W1, W2
G06: move_window_up                → Order: W3, W1, W2 (no change, W3 at top)
```

---

## Test Group H: Move Column

Tests: move-column-left/right/to-first/to-last

```
# Starting state: [W1, W2, W3], W2 active
H00: setup_three_columns_w2_active
H01: move_column_left              → [W2, W1, W3], W2 still active
H02: move_column_left              → [W2, W1, W3] (no change, W2 at left)
H03: move_column_right             → [W1, W2, W3] (back to original)
H04: move_column_right             → [W1, W3, W2]
H05: move_column_to_first          → [W2, W1, W3]
H06: move_column_to_last           → [W1, W3, W2]
H07: focus_column_first            → W1 active
H08: move_column_to_last           → [W3, W2, W1]
```

---

## Test Group I: Fullscreen

Tests: fullscreen-window

```
# Starting state: [W1+W2 in column, W3], W1 active
I00: setup_two_tiles_and_one_column
I01: fullscreen_window             → W1 fullscreen, covers everything
I02: fullscreen_window             → W1 exits fullscreen
I03: focus_column_right            → W3 active
I04: fullscreen_window             → W3 fullscreen
I05: focus_column_left             → W1 active (while W3 fullscreen? or exits?)
I06: fullscreen_window             → Toggle back
```

---

## Test Group J: Window Height Presets

Tests: switch-preset-window-height, set-window-height, reset-window-height

```
# Starting state: [W1+W2 in column], W1 active
J00: setup_two_tiles_in_column
J01: switch_preset_window_height   → W1 height changes
J02: switch_preset_window_height   → W1 height cycles
J03: set_window_height("+10%")     → W1 taller
J04: set_window_height("-10%")     → W1 shorter
J05: focus_window_down             → W2 active
J06: set_window_height("+20%")     → W2 taller (W1 shorter)
J07: reset_window_height           → W2 back to default
```

---

## Test Group K: Expand to Available Width

Tests: expand-column-to-available-width

```
# Starting state: [W1(1/3), W2(1/3)], W1 active, both visible
K00: setup_two_narrow_columns
K01: expand_column_to_available_width → W1 expands to fill remaining space
K02: focus_column_right               → W2 active
K03: expand_column_to_available_width → W2 expands
```

---

## Test Group L: Tabbed Display Mode

Tests: toggle-column-tabbed-display

```
# Starting state: [W1+W2+W3 in column], W1 active
L00: setup_three_tiles_in_column
L01: toggle_column_tabbed_display  → Column shows as tabs
L02: focus_window_down             → W2 active (tab switches)
L03: focus_window_down             → W3 active
L04: toggle_column_tabbed_display  → Back to stacked
```

---

## Test Group M: Insert Position (New Window Placement)

Tests: where new windows appear relative to active

```
# Starting state: [W1, W2, W3], W1 active
M00: setup_three_columns_w1_active
M01: add_window(W4)                → [W1, W4, W2, W3], W4 active
M02: focus_column_right            → W2 active
M03: add_window(W5)                → [W1, W4, W2, W5, W3], W5 active
M04: focus_column_last             → W3 active
M05: add_window(W6)                → [W1, W4, W2, W5, W3, W6], W6 active
```

---

## Test Group N: Close Window

Tests: close-window affects layout

```
# Starting state: [W1, W2, W3], W2 active
N00: setup_three_columns_w2_active
N01: close_window                  → [W1, W3], which becomes active?
N02: add_window(W4)                → [W1, W4, W3] or [W1, W3, W4]?
N03: focus_column_first            → W1 active
N04: close_window                  → [W4, W3] or [W3, W4]?
```

---

## Test Group O: Edge Cases

Tests: boundary conditions

```
O00: single_window_focus_left      → No change (nowhere to go)
O01: single_window_focus_right     → No change
O02: single_window_move_left       → No change
O03: single_window_move_right      → No change
O04: empty_workspace_operations    → All ops should be no-ops
O05: single_tile_move_up           → No change
O06: single_tile_move_down         → No change
```

---

## Test Group P: Interactive Resize

Tests: interactive_resize_begin, update, end (CRITICAL for golden parity)

```
# Starting state: [W1(1/2), W2(1/2)], W1 active
P00: setup_two_columns
P01: interactive_resize_begin(W1, right_edge)   → Resize started
P02: interactive_resize_update(+100px)          → W1 wider, W2 position shifts
P03: interactive_resize_update(+100px)          → W1 even wider
P04: interactive_resize_update(-150px)          → W1 narrower
P05: interactive_resize_end                     → Resize committed
P06: interactive_resize_begin(W2, left_edge)    → Resize W2's left edge
P07: interactive_resize_update(-50px)           → W2 wider (left edge moves left)
P08: interactive_resize_end                     → Committed

# Multi-tile column resize
P09: setup_two_tiles_in_column
P10: interactive_resize_begin(W1, bottom_edge)  → Resize height
P11: interactive_resize_update(+50px)           → W1 taller, W2 shorter
P12: interactive_resize_end                     → Committed
```

---

## Test Group Q: Swap Window

Tests: swap_window_in_direction

```
# Starting state: [W1+W2 in column], W1 active (top)
Q00: setup_two_tiles_in_column
Q01: swap_window_in_direction(right)  → W1 becomes its own column to right
Q02: swap_window_in_direction(left)   → W1 swaps back
Q03: focus_window_down                → W2 active
Q04: swap_window_in_direction(left)   → W2 becomes column to left
```

---

## Test Group R: Focus Wrap-Around

Tests: focus_column_right_or_first, focus_column_left_or_last, etc.

```
# Starting state: [W1, W2, W3], W3 active (rightmost)
R00: setup_three_columns_w3_active
R01: focus_column_right_or_first      → W1 active (wraps to first)
R02: focus_column_left_or_last        → W3 active (wraps to last)
R03: focus_column_left                → W2 active
R04: focus_column_left                → W1 active
R05: focus_column_left_or_last        → W3 active (wraps)

# Vertical wrap
R06: setup_three_tiles_in_column_w3_active
R07: focus_window_down_or_top         → W1 active (wraps to top)
R08: focus_window_up_or_bottom        → W3 active (wraps to bottom)
```

---

## Test Group S: Focus Specific Index

Tests: focus_column, focus_window_in_column

```
# Starting state: [W1, W2, W3, W4], W1 active
S00: setup_four_columns
S01: focus_column(2)                  → W3 active (0-indexed)
S02: focus_column(0)                  → W1 active
S03: focus_column(3)                  → W4 active

# Focus window in column by index
S04: setup_three_tiles_in_column
S05: focus_window_in_column(1)        → Second tile active
S06: focus_window_in_column(2)        → Third tile active
S07: focus_window_in_column(0)        → First tile active
```

---

## Test Group T: Focus Top/Bottom

Tests: focus_window_top, focus_window_bottom

```
# Starting state: [W1+W2+W3 in column], W2 active (middle)
T00: setup_three_tiles_w2_active
T01: focus_window_top                 → W1 active
T02: focus_window_bottom              → W3 active
T03: focus_window_top                 → W1 active (already at top, no change? or wraps?)
```

---

## Test Group U: Floating Windows

Tests: toggle_window_floating, focus_floating, switch_focus_floating_tiling

```
# Starting state: [W1, W2] tiled, W1 active
U00: setup_two_columns_tiled
U01: toggle_window_floating           → W1 becomes floating, W2 is now only tiled column
U02: focus_tiling                     → W2 active (tiled)
U03: focus_floating                   → W1 active (floating)
U04: switch_focus_floating_tiling     → W2 active (toggles)
U05: switch_focus_floating_tiling     → W1 active (toggles back)
U06: toggle_window_floating           → W1 back to tiled: [W1, W2]
```

---

## Test Group V: View Offset Gestures

Tests: view_offset_gesture (touchpad scrolling)

```
# Starting state: [W1, W2, W3], W2 active
V00: setup_three_columns_w2_active
V01: view_offset_gesture_begin(touchpad)
V02: view_offset_gesture_update(delta=-100)  → View scrolls left
V03: view_offset_gesture_update(delta=-100)  → More left
V04: view_offset_gesture_update(delta=+50)   → Slightly right
V05: view_offset_gesture_end                 → Settles to nearest column
```

---

## Test Group W: Combined Focus Movement

Tests: focus_down_or_left, focus_down_or_right, focus_up_or_left, focus_up_or_right

```
# Starting state: [W1+W2, W3], W2 active (bottom of first column)
W00: setup_two_and_one_columns_w2_active
W01: focus_down_or_right              → W3 active (no down, goes right)
W02: focus_up_or_left                 → W2 active (no up in W3, goes left to bottom)
W03: focus_up                         → W1 active
W04: focus_down_or_right              → W2 active (goes down)
W05: focus_down_or_left               → W1 active (no down, goes left... but already in leftmost, so maybe W1?)
```

---

## Summary: All Layout-Affecting Operations

| Operation | Affects | Test Groups |
|-----------|---------|-------------|
| **Window Lifecycle** | | |
| add_window | columns, active | A, M |
| close_window | columns, active | N |
| **Focus Movement** | | |
| focus_column_left/right | active, view_offset | A, B, D, E, F, H |
| focus_column_first/last | active, view_offset | A, E, H, M |
| focus_column_right_or_first | active, view_offset (wrap) | R |
| focus_column_left_or_last | active, view_offset (wrap) | R |
| focus_column(index) | active, view_offset | S |
| focus_window_up/down | active_tile | F, G, J, L |
| focus_window_top/bottom | active_tile | T |
| focus_window_down_or_top | active_tile (wrap) | R |
| focus_window_up_or_bottom | active_tile (wrap) | R |
| focus_window_in_column(index) | active_tile | S |
| focus_down_or_left/right | active, view_offset | W |
| focus_up_or_left/right | active, view_offset | W |
| **Column Movement** | | |
| move_column_left/right | column order | H |
| move_column_to_first/last | column order | H |
| move_column_to_index | column order | H |
| **Window Movement** | | |
| move_window_up/down | tile order | G |
| swap_window_in_direction | column membership | Q |
| **Column/Window Grouping** | | |
| consume_or_expel_window_left/right | column membership | F |
| consume_window_into_column | column membership | F |
| expel_window_from_column | column membership | F |
| **Sizing** | | |
| switch_preset_column_width | column width | B |
| set_column_width | column width | C |
| maximize_column | column width | D |
| expand_column_to_available_width | column width | K |
| switch_preset_window_height | tile height | J |
| set_window_height | tile height | J |
| reset_window_height | tile height | J |
| interactive_resize_begin/update/end | column/tile size | P |
| **View Control** | | |
| center_column | view_offset | E |
| center_visible_columns | view_offset | E |
| view_offset_gesture_begin/update/end | view_offset | V |
| **Display Modes** | | |
| fullscreen_window | fullscreen state, view | I |
| toggle_column_tabbed_display | display mode | L |
| **Floating** | | |
| toggle_window_floating | floating state | U |
| focus_floating/tiling | active space | U |
| switch_focus_floating_tiling | active space | U |

---

## Total: ~150 snapshots across 23 test groups (A-W)

Each snapshot captures:
- Column positions (x, width)
- Tile positions (x, y, width, height) 
- Active column index
- Active tile index per column
- View offset
- Working area
- Fullscreen state
- Tabbed display state
- Floating window positions (if any)
