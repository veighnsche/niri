// TEAM_008: Operations module refactored into submodules
//!
//! This module handles adding, removing, moving, and consuming/expelling columns within a row.
//!
//! ## Module Structure
//!
//! ```text
//! operations/
//! ├── mod.rs      - This file (re-exports)
//! ├── add.rs      - Add tile/column operations
//! ├── remove.rs   - Remove tile/column operations
//! ├── move_col.rs - Move column left/right operations
//! └── consume.rs  - Consume/expel window operations
//! ```

mod add;
mod consume;
mod move_col;
mod remove;

// All methods are implemented directly on Row<W> via `impl` blocks in submodules.
// No re-exports needed - the methods are automatically available on Row.
