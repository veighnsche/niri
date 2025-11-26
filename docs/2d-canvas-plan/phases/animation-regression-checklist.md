# Animation Regression Checklist

> **Purpose**: Ensure refactored code triggers the SAME animations in the SAME scenarios as the original code.
> **This is NOT about implementing animations — they already exist. This is about NOT BREAKING them.**

---

## How to Use This Checklist

1. **Before refactoring**: Verify animation works in original code
2. **After refactoring**: Verify same trigger produces same animation
3. **Mark each item**: ✅ Works | ❌ Broken | ⏳ Not yet tested

---

## Animation Categories

### 1. View Offset Animations (Horizontal Scroll)

These animations move the "camera" horizontally to keep the focused column visible.

| Trigger | Method | Config | Status |
|---------|--------|--------|--------|
| Focus column left/right | `animate_view_offset_to_column()` | `horizontal_view_movement` | ⏳ |
| Add new column | `animate_view_offset_to_column()` | `horizontal_view_movement` | ⏳ |
| Remove column | `animate_view_offset_to_column()` | `horizontal_view_movement` | ⏳ |
| Activate column | `animate_view_offset_to_column()` | `horizontal_view_movement` | ⏳ |
| Consume/expel window | `animate_view_offset_to_column()` | `horizontal_view_movement` | ⏳ |

### 2. Column Width Animations

These animations change column width smoothly.

| Trigger | Method | Config | Status |
|---------|--------|--------|--------|
| Cycle preset width (Mod+R) | Column width animation | `window_resize` | ⏳ |
| Set specific width | Column width animation | `window_resize` | ⏳ |
| Interactive resize | Column width animation | `window_resize` | ⏳ |
| Maximize column (Mod+F) | Column width animation | `window_resize` | ⏳ |

### 3. Tile Height Animations

These animations change tile height within a column.

| Trigger | Method | Config | Status |
|---------|--------|--------|--------|
| Cycle preset height | Tile height animation | `window_resize` | ⏳ |
| Set specific height | Tile height animation | `window_resize` | ⏳ |
| Interactive resize | Tile height animation | `window_resize` | ⏳ |
| Add tile to column | Tile height redistribution | `window_resize` | ⏳ |
| Remove tile from column | Tile height redistribution | `window_resize` | ⏳ |

### 4. Window Open/Close Animations

| Trigger | Method | Config | Status |
|---------|--------|--------|--------|
| Window opens | Open animation | `window_open` | ⏳ |
| Window closes | Close animation | `window_close` | ⏳ |
| Window closes (with animation) | `start_close_animation_for_tile()` | `window_close` | ⏳ |

### 5. Gesture Animations

| Trigger | Method | Config | Status |
|---------|--------|--------|--------|
| Touchpad scroll begin | `view_offset_gesture_begin()` | N/A | ⏳ |
| Touchpad scroll update | `view_offset_gesture_update()` | N/A | ⏳ |
| Touchpad scroll end | `view_offset_gesture_end()` | `horizontal_view_movement` | ⏳ |

### 6. Focus Ring Animations

| Trigger | Method | Config | Status |
|---------|--------|--------|--------|
| Focus changes | Focus ring animation | `focus_ring` | ⏳ |
| Focus ring appears | Focus ring animation | `focus_ring` | ⏳ |

### 7. Workspace Animations (TO BE REMOVED)

These will be removed in Phase 1.5.3. Document for reference only.

| Trigger | Method | Config | Status |
|---------|--------|--------|--------|
| Switch workspace | Workspace switch animation | `workspace_switch` | 🗑️ Remove |
| Overview open | Overview animation | N/A | 🗑️ Remove |
| Overview close | Overview animation | N/A | 🗑️ Remove |

---

## Verification Process

### Manual Testing

For each animation:
1. Trigger the animation in original code (main branch)
2. Observe: Does it animate? How long? What easing?
3. Trigger the same action in refactored code
4. Compare: Same animation behavior?

### Automated Testing (Future)

Currently, golden tests only verify positions, not animations. Future work could:
- Snapshot animation state (is_animating, target_value)
- Compare animation triggers between implementations

---

## Animation Config Reference

From `niri-config/src/animations.rs`:

```rust
pub struct Animations {
    pub workspace_switch: WorkspaceSwitchAnim,      // 🗑️ Remove
    pub horizontal_view_movement: HorizontalViewMovementAnim,
    pub window_open: WindowOpenAnim,
    pub window_close: WindowCloseAnim,
    pub window_resize: WindowResizeAnim,
    pub window_movement: WindowMovementAnim,
    pub config_notification_open_close: ConfigNotificationOpenCloseAnim,
    pub screenshot_ui_open: ScreenshotUiOpenAnim,
}
```

### Key Animation Types

| Config | Used For |
|--------|----------|
| `horizontal_view_movement` | View offset (camera follow) |
| `window_resize` | Column width, tile height changes |
| `window_open` | New window appearing |
| `window_close` | Window disappearing |
| `window_movement` | Window position changes |

---

## Row Module Animation Status

### Ported from ScrollingSpace ✅

| Method | Status |
|--------|--------|
| `animate_view_offset()` | ✅ Ported |
| `animate_view_offset_to_column()` | ✅ Ported |
| `animate_view_offset_to_column_centered()` | ✅ Ported |
| `view_offset_gesture_begin()` | ✅ Ported |
| `view_offset_gesture_update()` | ✅ Ported |
| `view_offset_gesture_end()` | ✅ Ported |

### Known Issues

| Issue | Location | Status |
|-------|----------|--------|
| Column movement not animated | `operations/add.rs:157` | TODO(TEAM_006) |
| Column movement not animated | `operations/move_col.rs:48` | TODO(TEAM_006) |

---

## Canvas2D Animation Status

### Implemented ✅

| Method | Status |
|--------|--------|
| `camera_y` animation on row change | ✅ Working |
| FloatingSpace animations | ✅ Delegated |

### Known Issues

| Issue | Location | Status |
|-------|----------|--------|
| Camera offset not applied to render | `render.rs:25` | TODO (Phase 3) |
| Close animation for tiled windows | `floating.rs:126` | TODO(TEAM_009) |

---

## Phase 1.5.3 Animation Checklist

Before completing Phase 1.5.3, verify:

- [ ] View offset animations work when focusing columns
- [ ] Column width animations work on Mod+R
- [ ] Tile height animations work on resize
- [ ] Window open animations work
- [ ] Window close animations work
- [ ] Gesture scrolling animates smoothly
- [ ] Focus ring animates on focus change

---

*Created by TEAM_009 — Animation Regression Audit*
