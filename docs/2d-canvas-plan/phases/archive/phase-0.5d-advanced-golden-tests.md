# Phase 0.5.D: Advanced Golden Tests (Groups M-W)

> **Goal**: Create golden tests for advanced/edge-case operations.
> **Estimated Time**: 4-6 hours
> **Dependency**: Phase 0.5.C complete

---

## Overview

Implement test groups M through W from [golden-test-scenarios.md](golden-test-scenarios.md).

These cover **advanced operations** and edge cases:
- Interactive resize (CRITICAL)
- Floating windows
- Wrap-around navigation
- Gestures

---

## Tasks

### 1. Add Test Group Files

```rust
// Add to src/layout/tests/golden.rs

mod group_m_insert;
mod group_n_close;
mod group_o_edge_cases;
mod group_p_interactive_resize;
mod group_q_swap;
mod group_r_wrap_around;
mod group_s_focus_index;
mod group_t_focus_top_bottom;
mod group_u_floating;
mod group_v_gestures;
mod group_w_combined_focus;
```

---

### 2. Group P: Interactive Resize (CRITICAL)

This is the most complex group — interactive resize affects multiple columns.

```rust
// src/layout/tests/golden/group_p_interactive_resize.rs

#[test]
fn p00_setup_two_columns() {
    let ops = [Op::AddOutput(1), Op::AddWindow {...}, Op::AddWindow {...}];
    let layout = check_ops(ops);
    assert_yaml_snapshot!("p00_setup", layout.scrolling_snapshot());
}

#[test]
fn p01_resize_begin_right_edge() {
    let ops = [..., Op::InteractiveResizeBegin { window: 0, edges: ResizeEdge::RIGHT }];
    let layout = check_ops(ops);
    assert_yaml_snapshot!("p01_resize_begin", layout.scrolling_snapshot());
}

#[test]
fn p02_resize_update_expand() {
    let ops = [..., Op::InteractiveResizeUpdate { dx: 100, dy: 0 }];
    let layout = check_ops(ops);
    assert_yaml_snapshot!("p02_resize_expand", layout.scrolling_snapshot());
}

// ... continue through p12
```

---

### 3. Group U: Floating Windows

Tests floating ↔ tiled transitions.

```rust
// src/layout/tests/golden/group_u_floating.rs

#[test]
fn u01_toggle_to_floating() {
    let ops = [..., Op::ToggleWindowFloating];
    let layout = check_ops(ops);
    // Snapshot includes both scrolling AND floating state
    assert_yaml_snapshot!("u01_floating", layout.full_snapshot());
}
```

**Note**: May need `FloatingSnapshot` type for floating window positions.

---

### 4. Group V: View Offset Gestures

Tests touchpad scrolling gestures.

```rust
// src/layout/tests/golden/group_v_gestures.rs

#[test]
fn v01_gesture_begin() {
    let ops = [..., Op::ViewOffsetGestureBegin { is_touchpad: true }];
    let layout = check_ops(ops);
    assert_yaml_snapshot!("v01_gesture_begin", layout.scrolling_snapshot());
}

#[test]
fn v02_gesture_scroll_left() {
    let ops = [..., Op::ViewOffsetGestureUpdate { delta: -100.0 }];
    let layout = check_ops(ops);
    assert_yaml_snapshot!("v02_scroll_left", layout.scrolling_snapshot());
}
```

---

## Test Count per Group

| Group | Tests | Operations Covered |
|-------|-------|-------------------|
| M | ~6 | Insert position (new window placement) |
| N | ~4 | close_window |
| O | ~7 | Edge cases (empty, single window) |
| **P** | ~13 | Interactive resize (CRITICAL) |
| Q | ~5 | swap_window_in_direction |
| R | ~8 | Wrap-around focus |
| S | ~8 | Focus by index |
| T | ~3 | focus_top/bottom |
| U | ~6 | Floating windows |
| V | ~5 | View gestures |
| W | ~6 | Combined focus |

**Total: ~71 tests**

---

## Additional Snapshot Types Needed

For Group U (Floating), add to `src/layout/snapshot.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FloatingSnapshot {
    pub tiles: Vec<FloatingTileSnapshot>,
    pub active_idx: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FloatingTileSnapshot {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub window_id: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WorkspaceSnapshot {
    pub scrolling: ScrollingSnapshot,
    pub floating: FloatingSnapshot,
    pub floating_active: bool,
}
```

---

## Success Criteria

- [ ] All 11 test group files created (M-W)
- [ ] ~71 tests written and passing
- [ ] `FloatingSnapshot` type added
- [ ] `cargo insta test` passes for all groups

---

## Final Verification

After Phase 0.5.D is complete:

```bash
# Run ALL golden tests
cargo insta test --package niri

# Should show ~157 tests (86 + 71) passing
# All snapshots should be committed
```

---

## Handoff

After completing this phase:
- Golden snapshot infrastructure is COMPLETE
- Next: **Phase 0.2** (AnimatedValue extraction)
- All future layout changes must pass `cargo insta test`
