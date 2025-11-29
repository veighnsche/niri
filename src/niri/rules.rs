//! Window and layer rule recomputation for the Niri compositor.
//!
//! This module handles recomputing window and layer rules when configuration changes.

use crate::window::{InitialConfigureState, ResolvedWindowRules, WindowRef};

use super::Niri;

// =============================================================================
// Rules Methods
// =============================================================================

impl Niri {
    /// Recomputes window rules for all windows.
    ///
    /// Called when the window rules configuration changes.
    pub fn recompute_window_rules(&mut self) {
        let _span = tracy_client::span!("Niri::recompute_window_rules");

        let changed = {
            let window_rules = &self.config.borrow().window_rules;

            for unmapped in self.unmapped_windows.values_mut() {
                let new_rules = ResolvedWindowRules::compute(
                    window_rules,
                    WindowRef::Unmapped(unmapped),
                    self.is_at_startup,
                );
                if let InitialConfigureState::Configured { rules, .. } = &mut unmapped.state {
                    *rules = new_rules;
                }
            }

            let mut windows = vec![];
            self.layout.with_windows_mut(|mapped, _| {
                if mapped.recompute_window_rules(window_rules, self.is_at_startup) {
                    windows.push(mapped.window.clone());
                }
            });
            let changed = !windows.is_empty();
            for win in windows {
                self.layout.update_window(&win, None);
            }
            changed
        };

        if changed {
            // FIXME: granular.
            self.queue_redraw_all();
        }
    }

    /// Recomputes layer rules for all layer surfaces.
    ///
    /// Called when the layer rules configuration changes.
    pub fn recompute_layer_rules(&mut self) {
        let _span = tracy_client::span!("Niri::recompute_layer_rules");

        let mut changed = false;
        {
            let config = self.config.borrow();
            let rules = &config.layer_rules;

            for mapped in self.mapped_layer_surfaces.values_mut() {
                if mapped.recompute_layer_rules(rules, self.is_at_startup) {
                    changed = true;
                    mapped.update_config(&config);
                }
            }
        }

        if changed {
            // FIXME: granular.
            self.queue_redraw_all();
        }
    }
}
