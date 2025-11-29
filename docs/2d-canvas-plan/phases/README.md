# niri Refactor — Masterplan

> **Status**: 🔴 **REFACTOR IN PROGRESS**
> **Priority**: Complete modularization before ANY new features
> **Goal**: Clean architecture with <500 LOC per file

---

## 🚨 START HERE

New to the codebase? Follow these steps:

1. **Read this file** — Understand the overall plan
2. **Check current phase** — See which phase is 🔄 CURRENT below
3. **Read that phase file** — Get detailed work units
4. **Claim your team number** — See `.teams/` folder
5. **Start working** — Follow the phase instructions

---

## Phase Overview

### Part A: Modularization (BLOCKING)

| Phase | Status | File | Description |
|-------|--------|------|-------------|
| [Phase A1](phase-A1-niri-types.md) | ✅ **DONE** | niri/types.rs | Extract pure data types |
| [**Phase A2**](phase-A2-niri-output.md) | 🔄 **CURRENT** | niri/output.rs | Output management |
| [Phase A3](phase-A3-niri-hit-test.md) | ⏳ PENDING | niri/hit_test.rs | Hit testing queries |
| [Phase A4](phase-A4-niri-lock.md) | ⏳ PENDING | niri/lock.rs | Session lock |
| [Phase A5](phase-A5-niri-render.md) | ⏳ PENDING | niri/render.rs + frame_callbacks.rs | Rendering |
| [Phase A6](phase-A6-niri-capture.md) | ⏳ PENDING | screenshot.rs + screencopy.rs + screencast.rs | Screen capture |
| [Phase A7](phase-A7-niri-input.md) | ⏳ PENDING | niri/pointer.rs + rules.rs | Input & rules |
| [Phase A8](phase-A8-niri-init.md) | ⏳ PENDING | niri/init.rs | Constructor extraction |

### Part B: Features (BLOCKED until Part A complete)

| Phase | Status | Description |
|-------|--------|-------------|
| Phase B1 | ⏸️ BLOCKED | Camera zoom system |
| Phase B2 | ⏸️ BLOCKED | Camera bookmarks |
| Phase B3 | ⏸️ BLOCKED | IPC/Protocol migration |
| Phase B4 | ⏸️ BLOCKED | Row spanning |

---

## Target Architecture

### Before (niri.rs — 6604 LOC monolith)
```
src/niri.rs  ← Everything in one file!
```

### After (niri/ module — 12 focused files)
```
src/
├── niri.rs (~600 LOC)      # Core Niri + State structs only
└── niri/
    ├── mod.rs              # Re-exports
    ├── types.rs (~200)     # KeyboardFocus, PointContents, LockState, etc.
    ├── output.rs (~500)    # add_output, remove_output, output_*_of
    ├── hit_test.rs (~450)  # contents_under, window_under, row_under
    ├── lock.rs (~350)      # lock, unlock, session lock
    ├── render.rs (~400)    # render(), pointer_element()
    ├── frame_callbacks.rs (~350)  # send_frame_callbacks, feedback
    ├── screenshot.rs (~400)       # screenshot, save_screenshot
    ├── screencopy.rs (~300)       # zwlr_screencopy protocol
    ├── screencast.rs (~300)       # PipeWire (#[cfg(xdp-gnome-screencast)])
    ├── pointer.rs (~350)   # pointer constraint, inactivity
    ├── rules.rs (~150)     # recompute_window_rules
    └── init.rs (~500)      # Niri::new() extracted
```

---

## Key Principles

### 1. Follow `layout_impl/` Pattern
The layout module already demonstrates good modularization:
```rust
// In niri/output.rs
impl Niri {
    pub fn add_output(&mut self, ...) { ... }
    pub fn remove_output(&mut self, ...) { ... }
}
```

### 2. No `pub(super)` — Proper Encapsulation
If something needs to be accessed externally, make a proper getter method.

### 3. Feature Gates in Separate Files
All `#[cfg(feature = "xdp-gnome-screencast")]` code goes in `screencast.rs`.

### 4. Pure Queries vs Mutations
- `hit_test.rs` — Read-only queries (`&self`)
- `output.rs` — Mutations (`&mut self`)

---

## Effort Estimates

| Part | Phases | Time | Status |
|------|--------|------|--------|
| **Part A** | A1-A8 | ~8 hours | 🔴 BLOCKING |
| **Part B** | B1-B4 | ~11.5 hours | ⏸️ BLOCKED |
| **Total** | | ~19.5 hours | |

---

## Success Criteria

### Part A Complete ✓
- [ ] niri.rs < 700 LOC
- [ ] Each module < 500 LOC
- [ ] No `pub(super)`
- [ ] All tests pass
- [ ] No unused imports

### Part B Complete ✓
- [ ] Zoom works
- [ ] Bookmarks work
- [ ] IPC reflects rows

---

## Archived Phases

See `archive/` folder for completed work:
- Canvas2D layout refactor (TEAM_062-066)
- Workspace → Row migration
- Golden test infrastructure

---

## Quick Commands

```bash
# Verify compilation
cargo check

# Run tests
cargo test

# Run golden tests
cargo insta test

# Find your team number
ls docs/2d-canvas-plan/.teams/ | tail -1
```
