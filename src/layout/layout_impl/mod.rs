// TEAM_063: Layout implementation split into submodules
//!
//! This module contains the implementation of Layout methods, split by category:
//!
//! - `queries.rs` - Read-only query methods (is_*, has_*, should_*)
//! - `fullscreen.rs` - Fullscreen/maximize operations
//! - `resize.rs` - Width/height operations
//! - `row_management.rs` - Row finding and naming
//! - `focus.rs` - Activation and focus methods
//! - `output_ops.rs` - Output management
//! - `window_ops.rs` - Window lifecycle
//! - `navigation.rs` - Movement and scrolling
//! - `interactive_move.rs` - DnD operations
//! - `render.rs` - Rendering methods

mod queries;
mod fullscreen;
mod resize;
mod row_management;
mod focus;
mod output_ops;
mod window_ops;
mod navigation;
