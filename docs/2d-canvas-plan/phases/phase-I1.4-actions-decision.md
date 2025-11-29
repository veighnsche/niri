# Phase I1.4: Actions - Keep or Split?

> **Status**: ⏳ PENDING - DECISION REQUIRED  
> **Architectural Benefit**: ❓ Debatable

---

## The Question

The `do_action` method is 1550 lines. Should we split it?

---

## Option A: Don't Split

**Argument:**
- It's a match statement - IDE navigation works fine
- The logic lives in `Layout`, not here
- Actions are thin wrappers: `Action::FocusLeft => self.niri.layout.focus_left()`
- Splitting spreads trivial code across many files
- More files = more imports, more boilerplate

**Result:**
- Move `do_action` + `handle_bind` to `src/input/actions.rs` (~1600 lines)
- Single file, but focused on action dispatch
- Easy to search with `Action::FocusColumnLeft`

---

## Option B: Split by Category

**Argument:**
- 1600 lines is still too long
- Related actions together (all window actions in window.rs)
- Easier to review PRs that touch one category
- Matches how users think ("I want to change window actions")

**Categories:**
| File | Actions | Est. Lines |
|------|---------|------------|
| `system.rs` | Quit, Suspend, Spawn, VT, Debug | ~100 |
| `screenshot.rs` | Screenshot*, Confirm, Cancel | ~100 |
| `window.rs` | Close, Focus, Fullscreen, Toggle*, Move* | ~300 |
| `column.rs` | Focus, Move, Consume, Swap, Center, Width | ~250 |
| `row.rs` | Focus, Move, Name | ~150 |
| `monitor.rs` | Focus, MoveTo | ~300 |
| `layout.rs` | Width, Height, Display, Keyboard | ~200 |
| `mru.rs` | Advance, Confirm, Cancel, etc. | ~100 |

**Result:**
- `src/input/actions/mod.rs` - router
- `src/input/actions/*.rs` - category handlers
- ~10 files, 100-300 lines each

---

## Recommendation

**Option A: Don't over-split.**

Reasons:
1. The complexity is in Layout, not in action dispatch
2. Match statements are easy to navigate
3. More files ≠ better architecture
4. Keep it simple

Move to `actions.rs`, add section comments:
```rust
// === Window Actions ===
Action::CloseWindow => { ... }
Action::FocusWindow(id) => { ... }
// ...

// === Column Actions ===
Action::MoveColumnLeft => { ... }
// ...
```

---

## Decision

**USER should decide:**
- [X] Option A: Single `actions.rs` with section comments
- [ ] Option B: Split into `actions/*.rs` by category
