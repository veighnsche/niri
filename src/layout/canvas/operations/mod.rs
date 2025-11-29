// TEAM_065: Canvas operations module - split from monolithic operations.rs
//!
//! This module provides all operations for Canvas2D organized into focused submodules:
//!
//! - [`row`] - Row management (ensure, cleanup, renumber)
//! - [`window`] - Window finding, activation, and queries
//! - [`tile`] - Tile addition and iteration
//! - [`state`] - Configuration, state changes, and compatibility methods
//!
//! All methods are implemented directly on [`Canvas2D`](super::Canvas2D).

mod row;
mod state;
mod tile;
mod window;

// Re-export Canvas2D for convenience (the impl blocks are in submodules)
// No actual types are exported from here - just impl blocks
