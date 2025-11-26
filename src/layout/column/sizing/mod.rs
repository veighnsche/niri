// TEAM_008: Sizing module refactored into submodules
//!
//! This module handles column and tile sizing operations.
//!
//! ## Module Structure
//!
//! ```text
//! sizing/
//! ├── mod.rs       - This file (submodule declarations)
//! ├── tile_sizes.rs - Core tile size computation
//! ├── width.rs     - Column width operations
//! ├── height.rs    - Window height operations
//! └── display.rs   - Fullscreen, maximize, display mode
//! ```

mod display;
mod height;
mod tile_sizes;
mod width;

// All methods are implemented directly on Column<W> via `impl` blocks in submodules.
// No re-exports needed - the methods are automatically available on Column.
