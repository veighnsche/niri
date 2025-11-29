//! Compositor subsystems.
//!
//! Each subsystem owns a domain of state and exposes a clean API.

mod cursor;
mod focus;
mod outputs;

pub use cursor::CursorSubsystem;
pub use focus::{FocusContext, FocusState, LayerFocusCandidate};
pub use outputs::OutputSubsystem;
