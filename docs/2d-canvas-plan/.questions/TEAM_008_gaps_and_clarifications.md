# TEAM_008: Gaps and Clarifications

## Questions for USER

### 1. Row Module Testing
The Row module now has ~2300 lines of code across 8 files, but there are no dedicated unit tests for it. The existing 251 tests and 58 golden tests all test ScrollingSpace, not Row.

**Question**: Should we add unit tests for Row before proceeding to Phase 1.5.2? Or is the plan to test Row only after Monitor integration (when it's actually used)?

### 2. Canvas2D Module Structure
The README shows Canvas2D should have multiple submodules:
```
├── canvas/
│   ├── mod.rs
│   ├── layout.rs
│   ├── navigation.rs
│   ├── spanning.rs
│   └── operations.rs
```

But currently Canvas2D is a single `mod.rs` (426 lines). 

**Question**: Should Canvas2D be split into submodules like Row was? Or is 426 lines acceptable for now?

### 3. Row vs ScrollingSpace Parity
I ported many methods from ScrollingSpace to Row, but I noticed some differences:

1. **Animation config parameters**: ScrollingSpace methods often take `anim_config: Option<niri_config::Animation>`, but Row methods use `self.options.animations.*` directly. This simplifies the API but reduces flexibility.

2. **Transaction handling**: `remove_tile_by_idx` takes a `Transaction` parameter, but I'm not sure if Row will use transactions the same way ScrollingSpace does.

**Question**: Are these differences intentional simplifications, or should Row match ScrollingSpace's API exactly for future compatibility?

### 4. FloatingSpace Integration Priority
Phase 1.5.2 lists FloatingSpace integration, but the 2D canvas vision seems focused on tiled windows. 

**Question**: Is FloatingSpace integration critical for the MVP, or can it be deferred to a later phase?

### 5. Remaining TODO(TEAM_006) Items
There's still one TODO from TEAM_006:
- `TODO(TEAM_006): Animate column movement during add/remove/move`

This appears in `operations.rs` at lines 166, 192, and 447.

**Question**: Is this animation important for Phase 1.5, or can it be deferred? The current code works but doesn't animate column sliding when adding/removing.

---

## Observations (Not Questions)

### Row Module is Feature-Complete for Phase 1.5.1
All methods from the Phase 1.5.1 checklist have been ported:
- Gesture handling ✅
- Interactive resize ✅
- Window operations (add/remove) ✅
- Consume/expel operations ✅
- render_above_top_layer ✅

### Code Quality
- All 251 tests pass
- All 58 golden tests pass
- Only warnings are for unused methods (expected for WIP)

---

*Created by TEAM_008*
