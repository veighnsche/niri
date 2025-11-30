# Phase I1.3: Extract Event Handlers

> **Status**: ✅ COMPLETE (TEAM_085)  
> **Time Estimate**: ~2 hours  
> **Risk Level**: 🟡 Medium (many methods to move)  
> **Architectural Benefit**: ⭐ Low-Medium - navigability, not architecture

---

## Honest Assessment

This phase is primarily about **code organization**, not architectural improvement.

**What we gain:**
- Smaller files (easier to navigate)
- Device-specific code grouped together
- Easier to find handler logic

**What we don't gain:**
- Better abstractions
- Improved encapsulation
- Testability (handlers need full State)

---

## What Moves

### `keyboard.rs` (~250 lines)
```rust
impl State {
    fn on_keyboard<I>(&mut self, event: I::KeyboardKeyEvent, consumed_by_a11y: &mut bool)
    fn start_key_repeat(&mut self, bind: Bind)
    fn hide_cursor_if_needed(&mut self)
}
```

### `pointer.rs` (~600 lines)
```rust
impl State {
    fn on_pointer_motion<I>(&mut self, event: I::PointerMotionEvent)
    fn on_pointer_motion_absolute<I>(&mut self, event: I::PointerMotionAbsoluteEvent)
    fn on_pointer_button<I>(&mut self, event: I::PointerButtonEvent)
    fn on_pointer_axis<I>(&mut self, event: I::PointerAxisEvent)
}
```

### `tablet.rs` (~250 lines)
```rust
impl State {
    fn compute_tablet_position<I>(&self, event: &...) -> Option<Point<f64, Logical>>
    fn on_tablet_tool_axis<I>(&mut self, event: I::TabletToolAxisEvent)
    fn on_tablet_tool_tip<I>(&mut self, event: I::TabletToolTipEvent)
    fn on_tablet_tool_proximity<I>(&mut self, event: I::TabletToolProximityEvent)
    fn on_tablet_tool_button<I>(&mut self, event: I::TabletToolButtonEvent)
}
```

### `gesture.rs` (~300 lines)
```rust
impl State {
    fn on_gesture_swipe_begin<I>(&mut self, event: I::GestureSwipeBeginEvent)
    fn on_gesture_swipe_update<I>(&mut self, event: I::GestureSwipeUpdateEvent)
    fn on_gesture_swipe_end<I>(&mut self, event: I::GestureSwipeEndEvent)
    fn on_gesture_pinch_begin<I>(&mut self, event: I::GesturePinchBeginEvent)
    fn on_gesture_pinch_update<I>(&mut self, event: I::GesturePinchUpdateEvent)
    fn on_gesture_pinch_end<I>(&mut self, event: I::GesturePinchEndEvent)
    fn on_gesture_hold_begin<I>(&mut self, event: I::GestureHoldBeginEvent)
    fn on_gesture_hold_end<I>(&mut self, event: I::GestureHoldEndEvent)
}
```

### `touch.rs` (~250 lines)
```rust
impl State {
    fn compute_touch_location<I>(&self, evt: &...) -> Option<Point<f64, Logical>>
    fn on_touch_down<I>(&mut self, evt: I::TouchDownEvent)
    fn on_touch_up<I>(&mut self, evt: I::TouchUpEvent)
    fn on_touch_motion<I>(&mut self, evt: I::TouchMotionEvent)
    fn on_touch_frame<I>(&mut self, _evt: I::TouchFrameEvent)
    fn on_touch_cancel<I>(&mut self, _evt: I::TouchCancelEvent)
}
```

---

## Why Do This At All?

1. **5123 lines is too long** - Hard to navigate, hard to review PRs
2. **Locality** - Pointer bugs? Look in pointer.rs
3. **Consistency** - Matches niri/ module structure

**Why NOT split further?**
- Creating actions/window.rs, actions/column.rs etc. doesn't improve architecture
- The actions are just delegating to Layout methods
- Splitting the match statement just spreads thin code across many files

---

## Verification

- [ ] All event types still handled
- [ ] `cargo check` passes
- [ ] No behavior changes
