//! TEAM_062: Render elements for layout visualization.
//!
//! This module groups all render elements used by the layout system:
//! - Window decorations (focus ring, shadow)
//! - Animation elements (closing/opening windows)
//! - UI indicators (tab indicator, insert hint)

pub mod closing_window;
pub mod focus_ring;
pub mod insert_hint;
pub mod opening_window;
pub mod shadow;
pub mod tab_indicator;

pub use closing_window::ClosingWindow;
pub use focus_ring::FocusRing;
pub use insert_hint::{InsertHintElement, InsertHintRenderElement};
pub use opening_window::OpenAnimation;
pub use shadow::Shadow;
pub use tab_indicator::{TabIndicator, TabInfo};
