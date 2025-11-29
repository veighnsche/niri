//! Compositor subsystems.
//!
//! Each subsystem owns a domain of state and exposes a clean API.

mod cursor;
mod focus;
mod input;
mod outputs;
mod streaming;
mod ui;

pub use cursor::CursorSubsystem;
pub use focus::{FocusContext, FocusState, LayerFocusCandidate};
pub use input::InputTracking;
pub use outputs::OutputSubsystem;
pub use streaming::StreamingSubsystem;
pub use ui::UiOverlays;
