# Phase 0.5.A: Golden Infrastructure Setup

> **Goal**: Set up the foundation for golden snapshot testing.
> **Estimated Time**: 2-4 hours
> **Dependency**: None

---

## Tasks

### 1. Add `insta` Dependency

```toml
# Cargo.toml
[dev-dependencies]
insta = { version = "1.40", features = ["yaml"] }
```

Run `cargo check` to verify.

---

### 2. Create Snapshot Types

Create `src/layout/snapshot.rs`:

```rust
//! Snapshot types for golden testing.
//!
//! These types capture layout state for comparison between
//! golden (original) and refactored code.

use serde::Serialize;

/// Snapshot of scrolling layout state
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScrollingSnapshot {
    pub columns: Vec<ColumnSnapshot>,
    pub active_column_idx: usize,
    pub view_offset: f64,
    pub working_area: RectSnapshot,
    pub view_size: SizeSnapshot,
}

/// Snapshot of a single column
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ColumnSnapshot {
    pub x: f64,
    pub width: f64,
    pub tiles: Vec<TileSnapshot>,
    pub active_tile_idx: usize,
    pub is_full_width: bool,
    pub is_fullscreen: bool,
}

/// Snapshot of a single tile
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TileSnapshot {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub window_id: usize,
}

/// Rectangle snapshot
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RectSnapshot {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// Size snapshot
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SizeSnapshot {
    pub w: f64,
    pub h: f64,
}
```

Add to `src/layout/mod.rs`:
```rust
#[cfg(test)]
pub mod snapshot;
```

---

### 3. Create Golden Directory Structure

```bash
mkdir -p src/layout/golden
```

Create `src/layout/golden/mod.rs`:

```rust
//! Golden reference code from main branch.
//!
//! This module contains copies of the original layout code
//! used to generate baseline snapshots for regression testing.

// Files will be added in Phase 0.5.B
```

Add to `src/layout/mod.rs`:
```rust
#[cfg(test)]
mod golden;
```

---

## Success Criteria

- [ ] `cargo check` passes with `insta` dependency
- [ ] `src/layout/snapshot.rs` exists with all types
- [ ] `src/layout/golden/mod.rs` exists
- [ ] Both modules are declared in `src/layout/mod.rs`

---

## Handoff

After completing this phase:
- Next: **Phase 0.5.B** (Extract golden code and add snapshot methods)
