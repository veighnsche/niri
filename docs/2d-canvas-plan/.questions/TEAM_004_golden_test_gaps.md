# TEAM_004: Golden Test Gap Analysis

## Current Coverage (30 tests, Groups A-L partial)

### What We Have

| Group | Scenario Doc | Our Tests | Coverage |
|-------|--------------|-----------|----------|
| A | 10 tests | 4 | 40% - Missing A04-A09 (focus_left/right/first/last sequences) |
| B | 10 tests | 4 | 40% - We have focus, doc has preset width cycling |
| C | 8 tests | 0 | 0% - set_column_width (+/-/exact %) |
| D | 8 tests | 0 | 0% - maximize_column toggle sequences |
| E | 6 tests | 0 | 0% - center_column, center_visible_columns |
| F | 10 tests | 4 | 40% - consume/expel (we have E group for this) |
| G | 6 tests | 2 | 33% - move_window_up/down (we have H group) |
| H | 8 tests | 2 | 25% - move_column (we have G group) |
| I | 6 tests | 1 | 17% - fullscreen (we have F group) |
| J | 7 tests | 2 | 29% - window heights (we have I group) |
| K | 3 tests | 0 | 0% - expand_to_available_width |
| L | 4 tests | 2 | 50% - tabbed (we have J group) |

### Missing Groups (M-W) - 0% coverage

| Group | Tests | Operations |
|-------|-------|------------|
| M | ~6 | Insert position (new window placement) |
| N | ~5 | Close window effects |
| O | ~7 | Edge cases (single window, empty workspace) |
| P | ~12 | **Interactive resize** (CRITICAL) |
| Q | ~4 | Swap window in direction |
| R | ~8 | Focus wrap-around |
| S | ~7 | Focus specific index |
| T | ~4 | Focus top/bottom |
| U | ~6 | Floating windows |
| V | ~6 | View offset gestures |
| W | ~6 | Combined focus movement |

## Critical Gaps

### 1. Interactive Resize (Group P) - HIGH PRIORITY
This is critical for golden parity. Tests needed:
- `interactive_resize_begin` (column width)
- `interactive_resize_update` (multiple deltas)
- `interactive_resize_end`
- Height resize in multi-tile columns

### 2. set_column_width (Group C) - MEDIUM
Manual width adjustments with percentages:
- `set_column_width("+10%")`
- `set_column_width("-10%")`
- `set_column_width("50%")` (exact)

### 3. expand_column_to_available_width (Group K) - MEDIUM
No tests for this operation.

### 4. Floating Windows (Group U) - MEDIUM
No tests for floating window operations.

### 5. View Offset Gestures (Group V) - LOW
Touchpad scrolling gestures.

## Naming Mismatch

Our groups don't match the scenario doc:
- Our B = Focus → Doc B = Preset widths
- Our C = Presets → Doc C = Manual widths
- Our D = Centering → Doc D = Full-width
- etc.

**Recommendation**: Rename our tests to match the scenario doc exactly.

## Uniqueness Check

Looking at our current tests, some may produce identical snapshots:
- `golden_e3_expel_from_column` — consume then expel = back to original?
- `golden_h2_move_window_down` — move up then down = back to original?
- `golden_i2_reset_window_height` — switch then reset = back to original?

These "round-trip" tests are still valuable (verify operations are reversible), but we should verify they're not duplicating other snapshots.

## Action Items

1. **Rename tests** to match scenario doc groups (A-W)
2. **Add missing operations**:
   - set_column_width (C)
   - expand_column_to_available_width (K)
   - interactive_resize (P)
   - swap_window_in_direction (Q)
   - focus wrap-around (R)
   - focus by index (S)
   - focus top/bottom (T)
   - floating (U)
   - gestures (V)
   - combined focus (W)
3. **Verify uniqueness** — no duplicate snapshots
4. **Target**: ~150 tests as specified in scenario doc

## Question for USER

Should we:
- (A) Add all ~150 tests now before proceeding to Phase 1
- (B) Add critical tests (interactive resize, set_column_width) now, rest later
- (C) Current 30 tests are sufficient for Phase 0.5, add more incrementally

---

## Status: AWAITING USER INPUT
