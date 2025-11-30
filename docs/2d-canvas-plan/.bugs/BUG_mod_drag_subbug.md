# BUG-002a: Insert Hint Bar Not Showing During Drag

## Summary
When Mod+dragging a tiled window, the blue insert hint bar (showing where the window will be placed) does not appear like it does in the main branch.

## Priority
Medium - UX regression, drag still works but user has no visual feedback

## Expected Behavior
A blue bar should appear between columns to indicate where the dragged window will be inserted when dropped.

## Actual Behavior
No insert hint bar appears during the drag operation.

## Related
- Parent bug: BUG-002 (now fixed)
- Main branch reference: Check how `InsertHint` is rendered in the original scrolling layout

---

# BUG-002b: Cannot Drag Window From Right to Left

## Summary
When you have two tiled windows side by side [left | right], you can drag the left window to the right position, but you CANNOT drag the right window to the left position.

## Priority
High - Core functionality broken

## Steps to Reproduce
1. Open 2 terminal windows (tiled side by side)
2. Try to Mod+drag the RIGHT window to the LEFT position
3. Observe that it cannot be placed on the left

## Expected Behavior
Should be able to swap window positions in both directions.

## Actual Behavior
Can only drag left→right, not right→left.

## Investigation Hints
- Check `interactive_move_update` for how drop targets are computed
- Check if `insert_position` calculation has a directional bias
- Compare with main branch behavior

## Related
- Parent bug: BUG-002 (now fixed)
