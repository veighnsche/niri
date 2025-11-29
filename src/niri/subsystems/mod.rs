//! Compositor subsystems.
//!
//! Each subsystem owns a domain of state and exposes a clean API.

mod outputs;

pub use outputs::OutputSubsystem;
