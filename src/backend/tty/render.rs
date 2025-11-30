//! Rendering subsystem for TTY backend.
//!
//! Handles frame rendering state and debug settings.
//! The actual rendering logic remains in mod.rs due to tight coupling
//! with device management and configuration.

use smithay::backend::renderer::DebugFlags;

use super::devices::DeviceManager;

/// Rendering subsystem.
///
/// Manages render-related state:
/// - Debug tint toggle
/// - Future: render statistics, frame pacing hints
pub struct RenderManager {
    /// Whether debug tinting is enabled.
    debug_tint: bool,
}

impl RenderManager {
    /// Create a new RenderManager.
    pub fn new() -> Self {
        Self { debug_tint: false }
    }

    /// Check if debug tint is enabled.
    pub fn debug_tint(&self) -> bool {
        self.debug_tint
    }

    /// Toggle debug tint on all surfaces.
    pub fn toggle_debug_tint(&mut self, devices: &mut DeviceManager) {
        self.debug_tint = !self.debug_tint;

        for device in devices.values_mut() {
            for surface in device.surfaces.values_mut() {
                let compositor = &mut surface.compositor;

                let mut flags = compositor.debug_flags();
                flags.set(DebugFlags::TINT, self.debug_tint);
                compositor.set_debug_flags(flags);
            }
        }
    }

    /// Set debug tint to a specific value on all surfaces.
    pub fn set_debug_tint(&mut self, enabled: bool, devices: &mut DeviceManager) {
        if self.debug_tint == enabled {
            return;
        }
        self.debug_tint = enabled;

        for device in devices.values_mut() {
            for surface in device.surfaces.values_mut() {
                let compositor = &mut surface.compositor;

                let mut flags = compositor.debug_flags();
                flags.set(DebugFlags::TINT, self.debug_tint);
                compositor.set_debug_flags(flags);
            }
        }
    }
}

impl Default for RenderManager {
    fn default() -> Self {
        Self::new()
    }
}
