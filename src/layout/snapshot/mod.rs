//! Snapshot infrastructure for golden testing.
//!
//! This module is designed to be portable between branches.
//! It captures layout state for comparison between golden (main) and refactored code.
//!
//! ## Porting to Main Branch
//!
//! 1. Copy this entire `snapshot/` directory to main's `src/layout/`
//! 2. Add `pub mod snapshot;` to `src/layout/mod.rs`
//! 3. Add the accessor methods listed in `PORTING.md`
//! 4. Run `cargo insta test --accept` to generate golden snapshots
//!
//! ## History
//! - TEAM_004: Created initial infrastructure (on wrong branch)
//! - TEAM_010: Extended with animation timeline snapshots
//! - TEAM_011: Centralized for proper porting to main

mod types;

// Re-export all types
pub use types::{
    AnimationKindSnapshot, AnimationTimelineSnapshot, ColumnSnapshot, RectSnapshot,
    ScrollingSnapshot, SizeSnapshot, TileSnapshot,
};
