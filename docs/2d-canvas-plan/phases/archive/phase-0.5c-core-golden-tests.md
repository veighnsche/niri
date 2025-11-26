# Phase 0.5.C: Core Golden Tests (Groups A-L)

> **Goal**: Create golden tests for core layout operations.
> **Estimated Time**: 4-6 hours
> **Dependency**: Phase 0.5.B complete

---

## Overview

Implement test groups A through L from [golden-test-scenarios.md](golden-test-scenarios.md).

These cover the **core operations** that most users rely on:
- Adding/focusing windows
- Column width presets
- Multi-tile columns
- Fullscreen/maximize
- Basic movement

---

## Tasks

### 1. Create Test File Structure

```rust
// src/layout/tests/golden.rs

use insta::assert_yaml_snapshot;
use crate::layout::tests::{TestWindow, TestWindowParams, Op, check_ops};

mod group_a_basic;
mod group_b_presets;
mod group_c_manual_width;
mod group_d_full_width;
mod group_e_centering;
mod group_f_multi_tile;
mod group_g_move_window;
mod group_h_move_column;
mod group_i_fullscreen;
mod group_j_heights;
mod group_k_expand;
mod group_l_tabbed;
```

---

### 2. Group A: Basic Window Management (~10 tests)

```rust
// src/layout/tests/golden/group_a_basic.rs

#[test]
fn a00_empty_workspace() {
    let layout = setup_empty();
    assert_yaml_snapshot!("a00_empty", layout.scrolling_snapshot());
}

#[test]
fn a01_add_first_window() {
    let ops = [Op::AddOutput(1), Op::AddWindow { params: TestWindowParams::new(0) }];
    let layout = check_ops(ops);
    assert_yaml_snapshot!("a01_add_first", layout.scrolling_snapshot());
}

// ... a02 through a09
```

---

### 3. Group B: Column Width Presets (~10 tests)

```rust
// src/layout/tests/golden/group_b_presets.rs

#[test]
fn b00_setup_three_columns() { ... }

#[test]
fn b01_preset_half_to_two_thirds() {
    // W1 starts at 1/2, switch to 2/3
    let ops = [..., Op::SwitchPresetColumnWidth];
    let layout = check_ops(ops);
    assert_yaml_snapshot!("b01_preset_switch", layout.scrolling_snapshot());
}
```

---

### 4. Groups C-L: Similar Pattern

Each group follows the same pattern:
1. Setup initial state
2. Apply operations
3. Snapshot after each operation
4. Use `assert_yaml_snapshot!` with descriptive names

---

## Test Count per Group

| Group | Tests | Operations Covered |
|-------|-------|-------------------|
| A | ~10 | add_window, focus_left/right/first/last |
| B | ~10 | switch_preset_column_width |
| C | ~8 | set_column_width (+/- percentages) |
| D | ~8 | maximize_column |
| E | ~6 | center_column, center_visible_columns |
| F | ~10 | consume/expel, multi-tile focus |
| G | ~6 | move_window_up/down |
| H | ~8 | move_column_left/right/first/last |
| I | ~6 | fullscreen_window |
| J | ~7 | window height presets |
| K | ~3 | expand_to_available_width |
| L | ~4 | toggle_column_tabbed_display |

**Total: ~86 tests**

---

## Running Tests

```bash
# Run all golden tests
cargo test golden --package niri

# Run specific group
cargo test golden::group_a --package niri

# Generate/update snapshots
cargo insta test --package niri

# Review changes
cargo insta review
```

---

## Success Criteria

- [ ] All 12 test group files created
- [ ] ~86 tests written and passing
- [ ] Snapshots generated in `src/layout/tests/snapshots/`
- [ ] `cargo insta test` passes

---

## Handoff

After completing this phase:
- Next: **Phase 0.5.D** (Advanced golden tests M-W)
