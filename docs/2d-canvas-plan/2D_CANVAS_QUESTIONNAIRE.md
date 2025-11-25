# 2D Canvas Window Manager — Vision Questionnaire

> **Purpose**: Align on requirements before any implementation work.
> **How to use**: Answer each question below. Add notes, sketches, or "skip" if unsure.

---

## Part 1: Core Mental Model

### 1.1 What is the "canvas"?

Currently niri has:
- **ScrollingSpace**: A 1D horizontal strip of columns (infinite left/right, fixed height)
- **FloatingSpace**: Free-positioned windows within the working area

**Question**: In your 2D vision, is the canvas:

- [X] A. An infinite 2D plane (windows can be placed anywhere)
- [ ] B. A finite grid (e.g., 5×5 cells, each can hold a window)
- [ ] C. A combination — infinite but with a grid snapping
- [ ] D. Something else? _______________

### 1.2 The Origin Point (0,0)

You mentioned "a very clear 0,0 point" where:
- Everything LEFT of 0 is RTL
- Everything RIGHT of 0 is LTR

**Questions**:

a) What does RTL/LTR mean in this context?
- [ ] Text direction for window content
- [X] Which direction new windows are added
- [ ] Which direction focus moves when pressing "right"
- [X] Something else? So in animations !! You forgot to check for animations !! for example in preset-column-widths thing... and you press mod+r the leading edge of the window animates and the trailing edge is pinned into place... Please look at the animations. (in LTR the leading edge is right and trailing is left edge and in RTL it's the other way around)

b) Is there also a vertical component?
- [ ] Above 0,0 behaves differently than below
- [ ] Only horizontal matters
- [X] Other? Yes, it does behave differently... because next to column width there is also a window height that has presets... so above the 0,0 the leading edge is the top and the trailing edge is the bottom, and below the 0,0 the leading edge is the bottom and the trailing edge is the top

c) Is the origin fixed, or can the user move it?
- [X] Fixed at workspace creation
- [ ] User can redefine origin
- [ ] Origin follows the focused window
- [ ] Other? _______________

### 1.3 What lives at a "position"?

In current niri, a position holds a **Column** (which contains 1+ tiles stacked vertically).

In your 2D model:
- [ ] A position holds a single window
- [ ] A position holds a column (vertical stack)
- [ ] A position holds a "cell" that can be split (like i3)
- [ ] Windows can overlap (floating style)
- [X] Other? I know what I do not want. I don't want floating windows. Windows never overlap, the default zoom level is where a browser would normally be full screen on a normal desktop.... I hope that makes sense... so yes it can go up and down. 

---

## Part 2: Navigation

### 2.1 Focus Movement

You want to go "left, right, up, down."

**Questions**:

a) What determines "the window to the left"?
- [ ] Geometric position (nearest center/edge)
- [ ] Grid position (like a spreadsheet: move one cell left)
- [ ] Order of window creation
- [X] Other? So we need to absolutely be careful with semantics... there is physical (Left/Right/Up/Down) and there is Logical (Next/Previous/First/Last/Leading/Trailing) And we need to use physical most of the time. but for animation using leading and trailing makes more sense. 

b) What if there's no window in that direction?
- [ ] Wrap around to the other side
- [X] Do nothing (stay focused)
- [ ] Create an empty space
- [ ] Other? _______________

c) Should focus movement be animated (smooth pan)?
- [X] Yes, always
- [X] Yes, with option to disable (So there is already a config for that we will honour. Please look at the config possibilities... for user)
- [ ] No, instant jump
- [ ] Other? _______________

### 2.2 Scrolling/Panning

Current niri scrolls horizontally to keep focused column visible.

**Questions**:

a) In 2D, should the "camera" follow focus?
- [ ] Yes, auto-center on focused window
- [X] Yes, but with configurable behavior (edge vs center)
- [ ] No, user controls camera separately
- [ ] Other? _______________

b) What about touchpad/gesture panning?
- [X] Pan in any direction (2D gestures)
- [ ] Only horizontal (like current)
- [ ] Configurable axis
- [ ] Other? _______________

---

## Part 3: Zoom

### 3.1 Zoom Out Behavior

You want `Mod+ScrollDown` to zoom out and see tiles above/below/left/right.

**Questions**:

a) What does "zoom out" reveal?
- [X] More windows on the same workspace
- [i] Adjacent workspaces (overview mode) (Niri already has a overview mode,... please look for it. because I want to completely replace that with this 2d thing.)
- [ ] Both, depending on zoom level
- [ ] Other? _______________

b) How far can you zoom?
- [X] Until all windows on workspace fit
- [ ] Until multiple workspaces are visible
- [ ] Infinite zoom (windows become dots)
- [ ] Discrete zoom levels (1x, 0.5x, 0.25x)
- [ ] Other? _______________

c) Can you interact (click, drag) while zoomed out?
- [X] Yes, fully interactive
- [ ] Click to focus only
- [ ] No, zoom is view-only
- [ ] Other? _______________

### 3.2 Per-Window Scale

You mentioned controlling "scale or zoom or size of each window."

**Questions**:

a) Can individual windows have different sizes?
- [X] Yes, fully resizable (like floating) (Not really... nothing overlaps... )
- [ ] Yes, but snapped to a grid
- [ ] No, all windows same size
- [ ] Other? _______________

b) Can individual windows have different *zoom/scale*?
- [X] Yes, window A at 100%, window B at 50% (this might be a bigger challenge)
- [ ] No, zoom is global (camera-level)
- [ ] Other? _______________

---

## Part 4: Window Placement

### 4.1 Where do new windows go?

**Questions**:

a) When a new window spawns, where is it placed?
- [X] Next to the focused window (which direction? Depends on where the origin point is. if we are on the left of the 0,0 then we spawn a new windows left of the active one (like RTL) and when we're at the right of the 0,0 then we spawn a new window right of the active one (like LTR))
- [ ] At the origin (0,0)
- [ ] At a specific coordinate (configured per app)
- [X] User chooses position interactively (Look what I want is gutter-bars... uuuh it's another project entirely... but let me actually copy it into here so we can start making is a first class citezen) look at here /home/vince/Projects/niri/gutter-bar <- study this
- [ ] Other? _______________

b) Do windows tile automatically, or can they overlap?
- [X] Always tiled (no overlap)
- [ ] Can overlap if user places them that way
- [X] Overlap only in certain modes, like some windows can be floating. like currently available in niri that is for UX
- [ ] Other? _______________

### 4.2 Gaps and Spacing

- [ ] Uniform gaps between all windows
- [X] Different gaps horizontal vs vertical
- [ ] No gaps (windows touch)
- [ ] Configurable per-window
- [ ] Other? _______________

---

## Part 5: Relationship to Existing Concepts

### 5.1 Workspaces

Currently, each monitor has multiple workspaces (vertical switching).

**Questions**:

a) In 2D mode, are workspaces still needed?
- [ ] Yes, keep workspaces as separate 2D canvases
- [X] No, one giant 2D canvas replaces workspaces
- [ ] Workspaces become "regions" within the canvas
- [ ] Other? _______________

b) How does workspace switching interact with 2D navigation?
- [ ] Completely separate (mod+1/2/3 vs arrows)
- [ ] Zoom out far enough = see workspaces
- [X] Other? I want to completely eliminate the concept of having multiple workspaces per monitor... or multiple desktops per monitor... let's just have one infinite 2d workspace per monitor and users can go up and down to seperate their workflows...

### 5.2 Columns

Current niri's core unit is the **Column** (vertically stacked tiles).

**Questions**:

a) In 2D mode, do columns still exist?
- [X] Yes, keep columns as a grouping mechanism, Yes let keep the column mechanism... because that seems important... and lets say there are multiple tiles in a column (tiles cannot overflow to other columns that is fine let's keep it like that... but each row of columns... aaaah what do I really want... and what is the feasibility... So I want different zoom and scales for each window. WHICH MEANS that tiles are trenscending columns and rows... which is a complete fucking overhaul of the current architecture... but... I want to do it... so... let's do it... )
- [ ] No, each window is independent
- [ ] Optional — user can group windows into columns
- [ ] Other? _______________

### 5.3 Floating Windows

Currently there's a separate floating layer.

**Questions**:

a) In 2D mode:
- [ ] Everything is effectively floating (free position)
- [X] Keep floating as a separate layer on top
- [ ] Remove floating, it's redundant
- [ ] Other? _______________

---

## Part 6: Feasibility Context

### 6.1 Current Architecture Constraints

Based on my analysis, here's what would need to change:

| Component | Current State | 2D Impact |
|-----------|---------------|-----------|
| `ScrollingSpace` | 1D strip, X-only positioning | Needs Y positioning |
| `Column` | Vertical stack of tiles | May not make sense in 2D |
| `ViewOffset` | Single f64 (X offset) | Needs `(x, y)` + zoom factor |
| `column_x()` | Returns X position | Needs `window_pos()` returning `(x, y)` |
| Workspaces | Separate, vertical switching | Unclear role |
| Rendering | Assumed single zoom level | Needs scale transform |
| Input | Horizontal gestures | Needs 2D gestures |

### 6.2 Estimated Scope

- **Small change**: If 2D means "columns can also be arranged vertically" → ~1-2 weeks
- **Medium change**: If 2D means "free positioning + zoom" → ~1-2 months  
- **Large change**: If 2D means "complete rethink with workspaces as regions" → ~3+ months

---

## Part 7: Open Questions from You

Please add any questions, clarifications, or ideas you have:

```
(Your notes here)

This isn't as straightforward as I thought... Please make a second questionaire with follow up questions


```

---

## Part 8: Priority

What's most important to you? Rank 1-5 (1 = must have, 5 = nice to have):

- [2] __ 2D navigation (up/down/left/right focus)
- [1] __ Zoom out to see more windows
- [4] __ Origin-based RTL/LTR behavior
- [3] __ Per-window size control
- [5] __ Smooth animations during all of the above
- [ ] __ Other: _______________

---

*After you fill this out, I'll analyze feasibility and propose a concrete architecture.*
