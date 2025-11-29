//! Compositor subsystems.
//!
//! Each subsystem owns a domain of state and exposes a clean API.

mod cursor;
mod focus;
mod outputs;

pub use cursor::CursorSubsystem;
pub use focus::FocusState;
pub use outputs::OutputSubsystem;
