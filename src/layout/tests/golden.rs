//! TEAM_004: Golden snapshot tests for layout regression testing.
//!
//! ## ⚠️ READ BEFORE MODIFYING
//!
//! These tests compare current code against **locked baseline snapshots**.
//! The snapshots were created from commit `75d5e3b0` (original main branch code).
//!
//! ### If tests fail:
//! 1. **DO NOT** run `cargo insta accept`
//! 2. **FIX YOUR CODE** — your refactor changed behavior
//! 3. See `src/layout/tests/snapshots/README.md` for details
//!
//! ### To view original code:
//! ```bash
//! git show 75d5e3b0:src/layout/scrolling.rs
//! ```
//!
//! ### Verification:
//! ```bash
//! ./scripts/verify-golden.sh
//! ```

use super::{check_ops, Op, TestWindowParams};
use crate::layout::snapshot::ScrollingSnapshot;

/// Helper to run ops and get scrolling snapshot.
fn snapshot(ops: impl IntoIterator<Item = Op>) -> ScrollingSnapshot {
    check_ops(ops).active_workspace().unwrap().scrolling().snapshot()
}

/// Helper to add a window with given id.
fn win(id: usize) -> Op {
    Op::AddWindow { params: TestWindowParams::new(id) }
}

/// Helper to consume window to the left.
fn consume_left() -> Op {
    Op::ConsumeOrExpelWindowLeft { id: None }
}

/// Helper to expel window to the right.
fn expel_right() -> Op {
    Op::ConsumeOrExpelWindowRight { id: None }
}

// ============================================================================
// Group A: Basic Window Management
// ============================================================================

#[test]
fn golden_a1_empty_workspace() {
    insta::assert_yaml_snapshot!(snapshot([Op::AddOutput(1)]));
}

#[test]
fn golden_a2_single_window() {
    insta::assert_yaml_snapshot!(snapshot([Op::AddOutput(1), win(1)]));
}

#[test]
fn golden_a3_two_windows() {
    insta::assert_yaml_snapshot!(snapshot([Op::AddOutput(1), win(1), win(2)]));
}

#[test]
fn golden_a4_three_windows() {
    insta::assert_yaml_snapshot!(snapshot([Op::AddOutput(1), win(1), win(2), win(3)]));
}

// ============================================================================
// Group B: Focus Changes
// ============================================================================

#[test]
fn golden_b1_focus_left() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnLeft,
    ]));
}

#[test]
fn golden_b2_focus_left_twice() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnLeft, Op::FocusColumnLeft,
    ]));
}

#[test]
fn golden_b3_focus_first() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnFirst,
    ]));
}

#[test]
fn golden_b4_focus_right_from_first() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnFirst, Op::FocusColumnRight,
    ]));
}

// ============================================================================
// Group C: Column Width Presets
// ============================================================================

#[test]
fn golden_c1_switch_preset_width() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1),
        Op::SwitchPresetColumnWidth,
    ]));
}

#[test]
fn golden_c2_switch_preset_width_twice() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1),
        Op::SwitchPresetColumnWidth, Op::SwitchPresetColumnWidth,
    ]));
}

#[test]
fn golden_c3_maximize_column() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::FocusColumnFirst, Op::MaximizeColumn,
    ]));
}

// ============================================================================
// Group D: Centering
// ============================================================================

#[test]
fn golden_d1_center_column() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnFirst, Op::CenterColumn,
    ]));
}

#[test]
fn golden_d2_center_middle_column() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnLeft, Op::CenterColumn,
    ]));
}

// ============================================================================
// Group E: Multi-tile Columns (Consume/Expel)
// ============================================================================

#[test]
fn golden_e1_consume_left() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
    ]));
}

#[test]
fn golden_e2_consume_two_into_column() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        consume_left(), consume_left(),
    ]));
}

#[test]
fn golden_e3_expel_from_column() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(), expel_right(),
    ]));
}

#[test]
fn golden_e4_focus_window_in_multi_tile() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(), Op::FocusWindowUp,
    ]));
}

// ============================================================================
// Group F: Fullscreen
// ============================================================================

#[test]
fn golden_f1_fullscreen_window() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::FocusColumnFirst, Op::FullscreenWindow(1),
    ]));
}

// ============================================================================
// Group G: Move Column
// ============================================================================

#[test]
fn golden_g1_move_column_left() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::MoveColumnLeft,
    ]));
}

#[test]
fn golden_g2_move_column_to_first() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::MoveColumnToFirst,
    ]));
}

// ============================================================================
// Group H: Move Window Within Column
// ============================================================================

#[test]
fn golden_h1_move_window_up() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(), Op::MoveWindowUp,
    ]));
}

#[test]
fn golden_h2_move_window_down() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(), Op::FocusWindowUp, Op::MoveWindowDown,
    ]));
}

// ============================================================================
// Group I: Window Heights
// ============================================================================

#[test]
fn golden_i1_switch_preset_height() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(), Op::SwitchPresetWindowHeight { id: None },
    ]));
}

#[test]
fn golden_i2_reset_window_height() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::SwitchPresetWindowHeight { id: None },
        Op::ResetWindowHeight { id: None },
    ]));
}

// ============================================================================
// Group J: Tabbed Display
// ============================================================================

#[test]
fn golden_j1_toggle_tabbed() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(), Op::ToggleColumnTabbedDisplay,
    ]));
}

#[test]
fn golden_j2_tabbed_focus_change() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(), Op::ToggleColumnTabbedDisplay, Op::FocusWindowUp,
    ]));
}

// ============================================================================
// Group K: Close Window
// ============================================================================

#[test]
fn golden_k1_close_middle_window() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnLeft, Op::CloseWindow(2),
    ]));
}

#[test]
fn golden_k2_close_active_window() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::CloseWindow(3),
    ]));
}

// ============================================================================
// Group L: Edge Cases
// ============================================================================

#[test]
fn golden_l1_single_window_focus_left() {
    insta::assert_yaml_snapshot!(snapshot([Op::AddOutput(1), win(1), Op::FocusColumnLeft]));
}

#[test]
fn golden_l2_single_window_move_right() {
    insta::assert_yaml_snapshot!(snapshot([Op::AddOutput(1), win(1), Op::MoveColumnRight]));
}

// ============================================================================
// Group M: Insert Position (New Window Placement)
// ============================================================================

#[test]
fn golden_m1_insert_after_first() {
    // W1 active, add W2 → W2 inserted after W1
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnFirst, win(4),
    ]));
}

#[test]
fn golden_m2_insert_after_middle() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnLeft, win(4),
    ]));
}

// ============================================================================
// Group N: Close Window Effects
// ============================================================================

#[test]
fn golden_n1_close_first_window() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnFirst, Op::CloseWindow(1),
    ]));
}

#[test]
fn golden_n2_close_last_window() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::CloseWindow(3),
    ]));
}

#[test]
fn golden_n3_close_only_window() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1),
        Op::CloseWindow(1),
    ]));
}

// ============================================================================
// Group O: Edge Cases
// ============================================================================

#[test]
fn golden_o1_empty_workspace_focus() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1),
        Op::FocusColumnLeft,
    ]));
}

#[test]
fn golden_o2_single_tile_move_up() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1),
        Op::MoveWindowUp,
    ]));
}

#[test]
fn golden_o3_single_tile_move_down() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1),
        Op::MoveWindowDown,
    ]));
}

// ============================================================================
// Group P: Interactive Resize
// ============================================================================

#[test]
fn golden_p1_resize_column_right_edge() {
    use crate::utils::ResizeEdge;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::FocusColumnFirst,
        Op::InteractiveResizeBegin { window: 1, edges: ResizeEdge::RIGHT },
        Op::InteractiveResizeUpdate { window: 1, dx: 100., dy: 0. },
        Op::InteractiveResizeEnd { window: 1 },
    ]));
}

#[test]
fn golden_p2_resize_column_left_edge() {
    use crate::utils::ResizeEdge;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::InteractiveResizeBegin { window: 2, edges: ResizeEdge::LEFT },
        Op::InteractiveResizeUpdate { window: 2, dx: -50., dy: 0. },
        Op::InteractiveResizeEnd { window: 2 },
    ]));
}

#[test]
fn golden_p3_resize_tile_bottom_edge() {
    use crate::utils::ResizeEdge;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::FocusWindowUp,
        Op::InteractiveResizeBegin { window: 1, edges: ResizeEdge::BOTTOM },
        Op::InteractiveResizeUpdate { window: 1, dx: 0., dy: 50. },
        Op::InteractiveResizeEnd { window: 1 },
    ]));
}

// ============================================================================
// Group Q: Swap Window
// ============================================================================

#[test]
fn golden_q1_swap_right() {
    use crate::layout::ScrollDirection;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::SwapWindowInDirection(ScrollDirection::Right),
    ]));
}

#[test]
fn golden_q2_swap_left() {
    use crate::layout::ScrollDirection;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::FocusWindowUp,
        Op::SwapWindowInDirection(ScrollDirection::Left),
    ]));
}

// ============================================================================
// Group R: Focus Wrap-Around
// ============================================================================

#[test]
fn golden_r1_focus_right_or_first_wraps() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnRightOrFirst,
    ]));
}

#[test]
fn golden_r2_focus_left_or_last_wraps() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnFirst,
        Op::FocusColumnLeftOrLast,
    ]));
}

#[test]
fn golden_r3_focus_down_or_top_wraps() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::FocusWindowDownOrTop,
    ]));
}

#[test]
fn golden_r4_focus_up_or_bottom_wraps() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::FocusWindowUp,
        Op::FocusWindowUpOrBottom,
    ]));
}

// ============================================================================
// Group S: Focus Specific Index
// ============================================================================

#[test]
fn golden_s1_focus_column_by_index() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3), win(4),
        Op::FocusColumn(1), // 0-indexed, so column 2
    ]));
}

#[test]
fn golden_s2_focus_window_in_column_by_index() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        consume_left(), consume_left(),
        Op::FocusWindowInColumn(1),
    ]));
}

// ============================================================================
// Group T: Focus Top/Bottom
// ============================================================================

#[test]
fn golden_t1_focus_window_top() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        consume_left(), consume_left(),
        Op::FocusWindowTop,
    ]));
}

#[test]
fn golden_t2_focus_window_bottom() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        consume_left(), consume_left(),
        Op::FocusWindowUp, Op::FocusWindowUp,
        Op::FocusWindowBottom,
    ]));
}

// ============================================================================
// Group U: Floating Windows
// ============================================================================

#[test]
fn golden_u1_toggle_floating() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::ToggleWindowFloating { id: None },
    ]));
}

#[test]
fn golden_u2_focus_tiling_from_floating() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::ToggleWindowFloating { id: None },
        Op::FocusTiling,
    ]));
}

#[test]
fn golden_u3_switch_focus_floating_tiling() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::ToggleWindowFloating { id: None },
        Op::FocusTiling,
        Op::SwitchFocusFloatingTiling,
    ]));
}

// ============================================================================
// Group V: View Offset Gestures
// ============================================================================

#[test]
fn golden_v1_gesture_scroll_left() {
    use std::time::Duration;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::ViewOffsetGestureBegin { output_idx: 1, workspace_idx: None, is_touchpad: true },
        Op::ViewOffsetGestureUpdate { delta: -100., timestamp: Duration::from_millis(100), is_touchpad: true },
        Op::ViewOffsetGestureEnd { is_touchpad: Some(true) },
        Op::CompleteAnimations,
    ]));
}

// ============================================================================
// TEAM_010: Animation Timeline Tests
// These capture snapshots DURING animation (before CompleteAnimations)
// to verify animation parameters match between golden and refactored code.
// ============================================================================

/// Captures view_offset animation triggered by gesture end.
/// This test does NOT call CompleteAnimations, so animations[] will contain the timeline.
#[test]
fn golden_anim_v1_gesture_triggers_animation() {
    use std::time::Duration;
    // Note: No CompleteAnimations - we want to capture the animation in progress
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::ViewOffsetGestureBegin { output_idx: 1, workspace_idx: None, is_touchpad: true },
        Op::ViewOffsetGestureUpdate { delta: -100., timestamp: Duration::from_millis(100), is_touchpad: true },
        Op::ViewOffsetGestureEnd { is_touchpad: Some(true) },
        // No CompleteAnimations - snapshot captures animation in progress!
    ]));
}

// ============================================================================
// Group W: Combined Focus Movement
// ============================================================================

#[test]
fn golden_w1_focus_down_or_column_right() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::FocusWindowDownOrColumnRight,
    ]));
}

#[test]
fn golden_w2_focus_up_or_column_left() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::FocusWindowUp,
        Op::FocusWindowUpOrColumnLeft,
    ]));
}

#[test]
fn golden_w3_focus_window_or_workspace_down() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::FocusWindowOrWorkspaceDown,
    ]));
}

// ============================================================================
// Group X: Manual Width Adjustments (SetColumnWidth)
// ============================================================================

#[test]
fn golden_x1_set_column_width_increase() {
    use crate::layout::SizeChange;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::FocusColumnFirst,
        Op::SetColumnWidth(SizeChange::AdjustProportion(0.1)),
    ]));
}

#[test]
fn golden_x2_set_column_width_decrease() {
    use crate::layout::SizeChange;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::FocusColumnFirst,
        Op::SetColumnWidth(SizeChange::AdjustProportion(-0.1)),
    ]));
}

#[test]
fn golden_x3_set_column_width_fixed() {
    use crate::layout::SizeChange;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::FocusColumnFirst,
        Op::SetColumnWidth(SizeChange::SetProportion(0.25)),
    ]));
}

// ============================================================================
// Group Y: Expand to Available Width
// ============================================================================

#[test]
fn golden_y1_expand_column_to_available_width() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::FocusColumnFirst,
        Op::ExpandColumnToAvailableWidth,
    ]));
}

#[test]
fn golden_y2_expand_second_column() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::ExpandColumnToAvailableWidth,
    ]));
}

// ============================================================================
// Group Z: Move Column to Index
// ============================================================================

#[test]
fn golden_z1_move_column_right() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnFirst,
        Op::MoveColumnRight,
    ]));
}

#[test]
fn golden_z2_move_column_to_last() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnFirst,
        Op::MoveColumnToLast,
    ]));
}

// ============================================================================
// ANIMATION TESTS: Capture animations DURING execution (no CompleteAnimations)
// These verify animation parameters match between golden and refactored code.
// ============================================================================

// --- Animation: Focus Changes ---

#[test]
fn golden_anim_focus_left() {
    // Focus change triggers view_offset animation
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnLeft,
        // No CompleteAnimations - captures animation
    ]));
}

#[test]
fn golden_anim_focus_first() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnFirst,
    ]));
}

// --- Animation: Column Width Changes ---

#[test]
fn golden_anim_switch_preset_width() {
    // Preset width change triggers resize animation
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::FocusColumnFirst,
        Op::SwitchPresetColumnWidth,
    ]));
}

#[test]
fn golden_anim_maximize_column() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::FocusColumnFirst,
        Op::MaximizeColumn,
    ]));
}

// --- Animation: Consume/Expel ---

#[test]
fn golden_anim_consume_left() {
    // Consume triggers tile move animations
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
    ]));
}

#[test]
fn golden_anim_expel_right() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        expel_right(),
    ]));
}

// --- Animation: Move Column ---

#[test]
fn golden_anim_move_column_left() {
    // Move column triggers column_move_x animations
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::MoveColumnLeft,
    ]));
}

#[test]
fn golden_anim_move_column_to_first() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::MoveColumnToFirst,
    ]));
}

// --- Animation: Move Window Within Column ---

#[test]
fn golden_anim_move_window_up() {
    // Move window triggers tile move_y animations
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::MoveWindowUp,
    ]));
}

#[test]
fn golden_anim_move_window_down() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::FocusWindowUp,
        Op::MoveWindowDown,
    ]));
}

// --- Animation: Height Changes ---

#[test]
fn golden_anim_switch_preset_height() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::SwitchPresetWindowHeight { id: None },
    ]));
}

// --- Animation: Swap Window ---

#[test]
fn golden_anim_swap_right() {
    use crate::layout::ScrollDirection;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::SwapWindowInDirection(ScrollDirection::Right),
    ]));
}

#[test]
fn golden_anim_swap_left() {
    use crate::layout::ScrollDirection;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::FocusWindowUp,
        Op::SwapWindowInDirection(ScrollDirection::Left),
    ]));
}

// --- Animation: Center Column ---

#[test]
fn golden_anim_center_column() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnFirst,
        Op::CenterColumn,
    ]));
}

// --- Animation: Add Window ---

#[test]
fn golden_anim_add_window() {
    // Adding window may trigger view_offset animation
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        win(3), // This triggers animation
    ]));
}

// --- Animation: Close Window ---

#[test]
fn golden_anim_close_window() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnLeft,
        Op::CloseWindow(2),
    ]));
}

// --- Animation: Interactive Resize ---

#[test]
fn golden_anim_resize_right_edge() {
    use crate::utils::ResizeEdge;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::FocusColumnFirst,
        Op::InteractiveResizeBegin { window: 1, edges: ResizeEdge::RIGHT },
        Op::InteractiveResizeUpdate { window: 1, dx: 100., dy: 0. },
        Op::InteractiveResizeEnd { window: 1 },
    ]));
}

#[test]
fn golden_anim_resize_bottom_edge() {
    use crate::utils::ResizeEdge;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::FocusWindowUp,
        Op::InteractiveResizeBegin { window: 1, edges: ResizeEdge::BOTTOM },
        Op::InteractiveResizeUpdate { window: 1, dx: 0., dy: 50. },
        Op::InteractiveResizeEnd { window: 1 },
    ]));
}

// --- Animation: Tabbed Display ---

#[test]
fn golden_anim_toggle_tabbed() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        consume_left(),
        Op::ToggleColumnTabbedDisplay,
    ]));
}

// --- Animation: Focus Wrap ---

#[test]
fn golden_anim_focus_right_or_first() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnRightOrFirst,
    ]));
}

#[test]
fn golden_anim_focus_left_or_last() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3),
        Op::FocusColumnFirst,
        Op::FocusColumnLeftOrLast,
    ]));
}

// --- Animation: Set Width ---

#[test]
fn golden_anim_set_column_width() {
    use crate::layout::SizeChange;
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::FocusColumnFirst,
        Op::SetColumnWidth(SizeChange::AdjustProportion(0.2)),
    ]));
}

// --- Animation: Expand Width ---

#[test]
fn golden_anim_expand_to_available() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2),
        Op::FocusColumnFirst,
        Op::ExpandColumnToAvailableWidth,
    ]));
}

// --- Animation: Four Windows ---

#[test]
fn golden_anim_four_windows_focus_first() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3), win(4),
        Op::FocusColumnFirst,
    ]));
}

#[test]
fn golden_anim_four_windows_center() {
    insta::assert_yaml_snapshot!(snapshot([
        Op::AddOutput(1), win(1), win(2), win(3), win(4),
        Op::FocusColumn(1),
        Op::CenterColumn,
    ]));
}
