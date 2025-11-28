# TEAM_051: Bottom-up TODO Completion

## Team Number
TEAM_051

## Mission
Complete high-priority TODOs working bottom-up (row → canvas → monitor → mod) to maintain compilation stability and maximize progress.

## Strategy
Following smart friend recommendation: work bottom-up to avoid breaking compilation as higher layers depend on lower ones.

## Work Plan

### ✅ Phase 1: Row Layer (src/layout/row/) - COMPLETED
- [x] Line 1004: Implement column width expansion (TEAM_024)
- [x] Line 1881: Implement proper close animation with snapshot (TEAM_033)

### ✅ Phase 2: Canvas Layer (src/layout/canvas/) - COMPLETED
- [x] operations.rs Line 108: Implement proper active window handling (TEAM_019)
- [x] operations.rs Line 123: Implement proper active window handling (TEAM_019)
- [x] operations.rs Line 194: Implement layout_config for Row (TEAM_019)
- [x] operations.rs Line 217: Implement start_open_animation for Row (TEAM_019)
- [x] operations.rs Line 240: Implement proper centering for tiled windows (TEAM_019)
- [x] operations.rs Line 324: Implement actual column reordering (TEAM_019)
- [x] operations.rs Line 340: Implement actual column reordering (TEAM_019)
- [x] operations.rs Line 448: Implement proper window update (TEAM_020)
- [x] operations.rs Line 468: Properly activate in row (TEAM_020)
- [x] operations.rs Line 513: Implement proper scroll calculation (TEAM_020)
- [x] operations.rs Line 520: Implement proper popup positioning (TEAM_021)
- [x] render.rs Line 25: Apply camera offset to render elements (TEAM_007)
- [x] mod.rs Line 312: Implement proper row removal (TEAM_025)
- [x] floating.rs Line 163: Add close animation for tiled windows (TEAM_009)
- [x] navigation.rs Line 79: Add vertical_view_movement config (TEAM_007)
- [x] navigation.rs Line 324: Implement back-and-forth logic (TEAM_018)
- [x] navigation.rs Line 331: Implement previous row tracking (TEAM_018)

### Phase 3: Monitor Layer (src/layout/monitor/)
- [ ] Geometry calculation fixes (TEAM_022, TEAM_023)
- [ ] Navigation between rows
- [ ] Gesture support (TEAM_024)
- [ ] Hit testing improvements

### Phase 4: Layout Mod Layer (src/layout/mod.rs)
- [ ] Workspace config removal (TEAM_020)
- [ ] Window operations implementation
- [ ] ID mapping fixes (TEAM_023)

## Handoff
- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] Golden tests pass (`cargo insta test`) — if touching layout logic
- [ ] Team file complete
