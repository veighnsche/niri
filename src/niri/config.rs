//! Configuration reloading for the Niri compositor.
//!
//! This module contains the config reload logic, broken into
//! focused helper methods for each config section.

use std::mem;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use niri_config::{Config, Xkb};
use smithay::input::keyboard::XkbConfig;
use smithay::utils::Transform;
use tracing::{info, trace, warn};

use super::{
    closest_representable_scale, guess_monitor_scale, ipc_transform_to_smithay, panel_orientation,
    OutputScale,
};

/// Extension trait for State to provide configuration reloading methods.
pub trait StateConfigExt {
    /// Main config reload entry point.
    fn reload_config(&mut self, config: Result<Config, ()>);

    /// Reloads output configuration for all outputs.
    fn reload_output_config(&mut self);

    /// Loads XKB configuration from file.
    fn load_xkb_file(&mut self);

    /// Sets XKB configuration.
    fn set_xkb_config(&mut self, xkb: smithay::input::keyboard::XkbConfig);
}

impl StateConfigExt for super::State {
    /// Main config reload entry point.
    fn reload_config(&mut self, config: Result<Config, ()>) {
        let _span = tracy_client::span!("State::reload_config");

        // Handle config error
        let mut config = match self.handle_config_result(config) {
            Some(c) => c,
            None => return,
        };

        // Apply config sections
        self.apply_named_rows_config(&config);
        self.apply_layout_config(&config);
        self.apply_animation_config(&config);
        self.apply_environment_config(&mut config);
        self.apply_cursor_config(&config);
        self.apply_keyboard_config(&config);
        self.apply_input_device_config(&config);
        self.apply_output_config_changes(&config);
        self.apply_binding_config(&config);
        self.apply_window_rules(&config);
        self.apply_shader_config(&config);
        self.apply_misc_config(&config);

        // Store the new config
        *self.niri.config.borrow_mut() = config;
    }

    /// Reloads output configuration for all outputs.
    fn reload_output_config(&mut self) {
        let mut resized_outputs = vec![];
        let mut recolored_outputs = vec![];

        // Collect outputs first to avoid borrow conflicts.
        let outputs: Vec<_> = self.niri.outputs.collect_outputs();

        for output in outputs {
            let name = output.user_data().get::<niri_config::OutputName>().unwrap();
            let full_config = self.niri.config.borrow_mut();
            let config = full_config.outputs.find(name);

            let scale = config
                .and_then(|c| c.scale)
                .map(|s| s.0)
                .unwrap_or_else(|| {
                    let size_mm = output.physical_properties().size;
                    let resolution = output.current_mode().unwrap().size;
                    guess_monitor_scale(size_mm, resolution)
                });
            let scale = closest_representable_scale(scale.clamp(0.1, 10.));

            let mut transform = panel_orientation(&output)
                + config
                    .map(|c| ipc_transform_to_smithay(c.transform))
                    .unwrap_or(Transform::Normal);
            // FIXME: fix winit damage on other transforms.
            if name.connector == "winit" {
                transform = Transform::Flipped180;
            }

            if output.current_scale().fractional_scale() != scale
                || output.current_transform() != transform
            {
                output.change_current_state(
                    None,
                    Some(transform),
                    Some(OutputScale::Fractional(scale)),
                    None,
                );
                self.niri.ipc_outputs_changed = true;
                resized_outputs.push(output.clone());
            }

            let mut backdrop_color = config
                .and_then(|c| c.backdrop_color)
                .unwrap_or(niri_config::appearance::DEFAULT_BACKDROP_COLOR)
                .to_array_unpremul();
            backdrop_color[3] = 1.;
            let backdrop_color = niri_config::Color::from_array_unpremul(backdrop_color);
            let backdrop_color32f: smithay::backend::renderer::Color32F = backdrop_color.into();

            if self
                .niri
                .outputs
                .update_backdrop_color(&output, backdrop_color32f)
            {
                recolored_outputs.push(output.clone());
            }

            for mon in self.niri.layout.monitors_mut() {
                if mon.output() != &output {
                    continue;
                }

                let mut layout_config = config.and_then(|c| c.layout.clone());
                // Support the deprecated non-layout background-color key.
                if let Some(layout) = &mut layout_config {
                    if layout.background_color.is_none() {
                        layout.background_color = config.and_then(|c| c.background_color);
                    }
                }

                if mon.update_layout_config(layout_config) {
                    // Also redraw these; if anything, the background color could've changed.
                    recolored_outputs.push(output.clone());
                }
                break;
            }
        }

        for output in resized_outputs {
            self.niri.output_resized(&output);
        }

        for output in recolored_outputs {
            self.niri.queue_redraw(&output);
        }

        self.backend.on_output_config_changed(&mut self.niri);

        self.niri.reposition_outputs(None);

        if let Some(touch) = self.niri.seat.get_touch() {
            touch.cancel(self);
        }

        let config = self.niri.config.borrow().outputs.clone();
        self.niri
            .protocols
            .output_management
            .on_config_changed(config);
    }

    /// Loads XKB configuration from file.
    fn load_xkb_file(&mut self) {
        let xkb_file = self.niri.config.borrow().input.keyboard.xkb.file.clone();
        if let Some(xkb_file) = xkb_file {
            if let Err(err) = self.set_xkb_file(xkb_file) {
                warn!("error loading xkb_file: {err:?}");
            }
        }
    }

    /// Sets XKB configuration.
    fn set_xkb_config(&mut self, xkb: XkbConfig) {
        let keyboard = self.niri.seat.get_keyboard().unwrap();
        keyboard.set_xkb_config(self, xkb);
    }
}

/// Private helper methods for configuration reloading.
impl super::State {
    /// Handles config parse result, shows error notification if needed.
    fn handle_config_result(&mut self, config: Result<Config, ()>) -> Option<Config> {
        match config {
            Ok(config) => {
                self.niri.ui.config_error.hide();
                Some(config)
            }
            Err(()) => {
                self.niri.ui.config_error.show();
                self.niri.queue_redraw_all();

                #[cfg(feature = "dbus")]
                self.niri.a11y_announce_config_error();

                None
            }
        }
    }

    /// Applies named rows configuration changes.
    fn apply_named_rows_config(&mut self, config: &Config) {
        // Find & orphan removed named rows.
        let mut removed_rows: Vec<String> = vec![];
        for row in &self.niri.config.borrow().rows {
            if !config.rows.iter().any(|r| r.name == row.name) {
                removed_rows.push(row.name.0.clone());
            }
        }
        for name in removed_rows {
            self.niri.layout.unname_workspace(&name);
        }

        // Create new named rows.
        for row_config in &config.rows {
            self.niri.layout.ensure_named_row(row_config);
        }
    }

    /// Applies layout configuration changes.
    fn apply_layout_config(&mut self, config: &Config) {
        self.niri.layout.update_config(config);
        for mapped in self.niri.mapped_layer_surfaces.values_mut() {
            mapped.update_config(config);
        }
    }

    /// Applies animation configuration changes.
    fn apply_animation_config(&mut self, config: &Config) {
        let rate = 1.0 / config.animations.slowdown.max(0.001);
        self.niri.clock.set_rate(rate);
        self.niri
            .clock
            .set_complete_instantly(config.animations.off);
    }

    /// Applies environment configuration changes.
    fn apply_environment_config(&mut self, config: &mut Config) {
        use crate::utils::spawning::CHILD_ENV;

        *CHILD_ENV.write().unwrap() = mem::take(&mut config.environment);
    }

    /// Applies cursor configuration changes.
    fn apply_cursor_config(&mut self, config: &Config) {
        let cursor_config = &config.cursor;
        if cursor_config != &self.niri.config.borrow().cursor {
            self.niri
                .cursor
                .manager
                .reload(&cursor_config.xcursor_theme, cursor_config.xcursor_size);
            self.niri.cursor.texture_cache.clear();
        }
    }

    /// Applies keyboard configuration changes.
    fn apply_keyboard_config(&mut self, config: &Config) {
        let old_config = self.niri.config.borrow();

        // We need &mut self to reload the xkb config, so just store it here.
        let reload_xkb = if config.input.keyboard.xkb != old_config.input.keyboard.xkb {
            Some(config.input.keyboard.xkb.clone())
        } else {
            None
        };

        // Reload the repeat info.
        if config.input.keyboard.repeat_rate != old_config.input.keyboard.repeat_rate
            || config.input.keyboard.repeat_delay != old_config.input.keyboard.repeat_delay
        {
            let keyboard = self.niri.seat.get_keyboard().unwrap();
            keyboard.change_repeat_info(
                config.input.keyboard.repeat_rate.into(),
                config.input.keyboard.repeat_delay.into(),
            );
        }

        // Release the borrow before applying XKB changes.
        drop(old_config);

        // Now with a &mut self we can reload the xkb config.
        if let Some(xkb) = reload_xkb {
            self.apply_xkb_config(xkb);
            self.ipc_keyboard_layouts_changed();
        }
    }

    /// Applies XKB configuration.
    fn apply_xkb_config(&mut self, mut xkb: Xkb) {
        let mut set_xkb_config = true;

        // It's fine to .take() the xkb file, as this is a
        // clone and the file field is not used in the XkbConfig.
        if let Some(xkb_file) = xkb.file.take() {
            if let Err(err) = self.set_xkb_file(xkb_file) {
                warn!("error reloading xkb_file: {err:?}");
            } else {
                // We successfully set xkb file so we don't need to fallback to XkbConfig.
                set_xkb_config = false;
            }
        }

        if set_xkb_config {
            // If xkb is unset in the niri config, use settings from locale1.
            if xkb == Xkb::default() {
                trace!("using xkb from locale1");
                xkb = self.niri.xkb_from_locale1.clone().unwrap_or_default();
            }

            self.set_xkb_config(xkb.to_xkb_config());
        }
    }

    /// Applies input device configuration.
    fn apply_input_device_config(&mut self, config: &Config) {
        use crate::input::apply_libinput_settings;

        let old_config = self.niri.config.borrow();

        let libinput_config_changed = config.input.touchpad != old_config.input.touchpad
            || config.input.mouse != old_config.input.mouse
            || config.input.trackball != old_config.input.trackball
            || config.input.trackpoint != old_config.input.trackpoint
            || config.input.tablet != old_config.input.tablet
            || config.input.touch != old_config.input.touch;

        // Release the borrow.
        drop(old_config);

        if libinput_config_changed {
            let config = self.niri.config.borrow();
            for mut device in self.niri.devices.iter().cloned() {
                apply_libinput_settings(&config.input, &mut device);
            }
        }
    }

    /// Detects and applies output configuration changes.
    fn apply_output_config_changes(&mut self, config: &Config) {
        let old_config = self.niri.config.borrow();
        let mut output_config_changed = false;
        let mut preserved_output_config = None;

        let ignored_nodes_changed =
            config.debug.ignored_drm_devices != old_config.debug.ignored_drm_devices;

        if config.outputs != self.niri.config_file_output_config {
            output_config_changed = true;
            self.niri
                .config_file_output_config
                .clone_from(&config.outputs);
        } else {
            // Output config did not change from the last disk load, so we need to preserve the
            // transient changes.
            preserved_output_config = Some(old_config.outputs.clone());
        }

        if config.debug.keep_laptop_panel_on_when_lid_is_closed
            != old_config.debug.keep_laptop_panel_on_when_lid_is_closed
        {
            output_config_changed = true;
        }

        if config.debug.ignored_drm_devices != old_config.debug.ignored_drm_devices {
            output_config_changed = true;
        }

        // FIXME: move backdrop rendering into layout::Monitor, then this will become unnecessary.
        // Overview mode has been removed, no backdrop color to check.
        if config.layout.background_color != old_config.layout.background_color {
            output_config_changed = true;
        }

        // Release the borrow.
        drop(old_config);

        if ignored_nodes_changed {
            self.backend.update_ignored_nodes_config(&mut self.niri);
        }

        if output_config_changed {
            self.reload_output_config();
        }

        // Restore preserved output config if needed.
        if let Some(outputs) = preserved_output_config {
            self.niri.config.borrow_mut().outputs = outputs;
        }
    }

    /// Applies binding/hotkey configuration.
    fn apply_binding_config(&mut self, config: &Config) {
        let old_config = self.niri.config.borrow();
        let binds_changed = config.binds != old_config.binds;
        let new_mod_key = self.backend.mod_key(config);

        if new_mod_key != self.backend.mod_key(&old_config) || binds_changed {
            self.niri.ui.hotkey.on_hotkey_config_updated(new_mod_key);
            self.niri.input.update_from_config(config);
        }

        // Release the borrow.
        drop(old_config);

        if binds_changed {
            self.niri.ui.mru.update_binds();
        }
    }

    /// Applies window and layer rules.
    fn apply_window_rules(&mut self, config: &Config) {
        let old_config = self.niri.config.borrow();

        let window_rules_changed = config.window_rules != old_config.window_rules;
        let layer_rules_changed = config.layer_rules != old_config.layer_rules;

        // Release the borrow.
        drop(old_config);

        if window_rules_changed {
            self.niri.recompute_window_rules();
        }

        if layer_rules_changed {
            self.niri.recompute_layer_rules();
        }
    }

    /// Applies custom shader configuration.
    fn apply_shader_config(&mut self, config: &Config) {
        use crate::render_helpers::shaders;

        let old_config = self.niri.config.borrow();
        let mut shaders_changed = false;

        if config.animations.window_resize.custom_shader
            != old_config.animations.window_resize.custom_shader
        {
            let src = config.animations.window_resize.custom_shader.as_deref();
            self.backend.with_primary_renderer(|renderer| {
                shaders::set_custom_resize_program(renderer, src);
            });
            shaders_changed = true;
        }

        if config.animations.window_close.custom_shader
            != old_config.animations.window_close.custom_shader
        {
            let src = config.animations.window_close.custom_shader.as_deref();
            self.backend.with_primary_renderer(|renderer| {
                shaders::set_custom_close_program(renderer, src);
            });
            shaders_changed = true;
        }

        if config.animations.window_open.custom_shader
            != old_config.animations.window_open.custom_shader
        {
            let src = config.animations.window_open.custom_shader.as_deref();
            self.backend.with_primary_renderer(|renderer| {
                shaders::set_custom_open_program(renderer, src);
            });
            shaders_changed = true;
        }

        // Release the borrow.
        drop(old_config);

        if shaders_changed {
            self.niri.update_shaders();
        }
    }

    /// Applies miscellaneous configuration.
    fn apply_misc_config(&mut self, config: &Config) {
        use crate::utils::spawning::CHILD_DISPLAY;
        use crate::utils::xwayland::satellite;

        let old_config = self.niri.config.borrow();

        let cursor_inactivity_timeout_changed =
            config.cursor.hide_after_inactive_ms != old_config.cursor.hide_after_inactive_ms;
        let recent_windows_changed = config.recent_windows != old_config.recent_windows;
        let xwls_changed = config.xwayland_satellite != old_config.xwayland_satellite;

        // Release the borrow.
        drop(old_config);

        if cursor_inactivity_timeout_changed {
            // Force reset due to timeout change.
            self.niri.cursor.clear_timer_reset_flag();
            self.niri.reset_pointer_inactivity_timer();
        }

        if recent_windows_changed {
            self.niri.ui.mru.update_config();
        }

        if xwls_changed {
            // If xwl-s was previously working and is now off, we don't try to kill it or stop
            // watching the sockets, for simplicity's sake.
            let was_working = self.niri.satellite.is_some();

            // Try to start, or restart in case the user corrected the path or something.
            satellite::setup(self);

            let config = self.niri.config.borrow();
            let display_name = (!config.xwayland_satellite.off)
                .then_some(self.niri.satellite.as_ref())
                .flatten()
                .map(|satellite| satellite.display_name().to_owned());

            if let Some(name) = &display_name {
                if !was_working {
                    info!("listening on X11 socket: {name}");
                }
            }

            // This won't change the systemd environment, but oh well.
            *CHILD_DISPLAY.write().unwrap() = display_name;
        }

        // Can't really update xdg-decoration settings since we have to hide the globals for CSD
        // due to the SDL2 bug... I don't imagine clients are prepared for the xdg-decoration
        // global suddenly appearing? Either way, right now it's live-reloaded in a sense that new
        // clients will use the new xdg-decoration setting.
    }

    // =========================================================================
    // XKB Helper Methods
    // =========================================================================

    /// Sets XKB configuration from a file.
    fn set_xkb_file(&mut self, xkb_file: String) -> anyhow::Result<()> {
        use crate::utils::expand_home;

        let xkb_file = PathBuf::from(xkb_file);
        let xkb_file = expand_home(&xkb_file)
            .context("failed to expand ~")?
            .unwrap_or(xkb_file);

        let keymap = std::fs::read_to_string(xkb_file).context("failed to read xkb_file")?;

        let xkb = self.niri.seat.get_keyboard().unwrap();
        xkb.set_keymap_from_string(self, keymap)
            .context("failed to set keymap")?;

        Ok(())
    }
}
