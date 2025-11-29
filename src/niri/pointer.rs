//! Pointer and cursor management for the Niri compositor.
//!
//! This module handles pointer constraints, inactivity timers, and focus-follows-mouse.

use std::time::Duration;

use calloop::timer::{TimeoutAction, Timer};
use smithay::desktop::LayerSurface;
use smithay::wayland::pointer_constraints::with_pointer_constraint;
use smithay::wayland::shell::wlr_layer;

use crate::layout::HitType;

use super::{Niri, PointContents, PointerVisibility};

// =============================================================================
// Pointer Methods
// =============================================================================

impl Niri {
    /// Activates the pointer constraint if necessary according to the current pointer contents.
    ///
    /// Make sure the pointer location and contents are up to date before calling this.
    pub fn maybe_activate_pointer_constraint(&self) {
        let Some((surface, surface_loc)) = &self.cursor.contents().surface else {
            return;
        };

        let pointer = self.seat.get_pointer().unwrap();
        if Some(surface) != pointer.current_focus().as_ref() {
            return;
        }

        with_pointer_constraint(surface, &pointer, |constraint| {
            let Some(constraint) = constraint else { return };

            if constraint.is_active() {
                return;
            }

            // Constraint does not apply if not within region.
            if let Some(region) = constraint.region() {
                let pointer_pos = pointer.current_location();
                let pos_within_surface = pointer_pos - *surface_loc;
                if !region.contains(pos_within_surface.to_i32_round()) {
                    return;
                }
            }

            constraint.activate();
        });
    }

    /// Focuses a layer surface if it has on-demand keyboard interactivity.
    pub fn focus_layer_surface_if_on_demand(&mut self, surface: Option<LayerSurface>) {
        if let Some(surface) = surface {
            if surface.cached_state().keyboard_interactivity
                == wlr_layer::KeyboardInteractivity::OnDemand
            {
                if self.focus.layer_on_demand() != Some(&surface) {
                    self.focus.set_layer_on_demand(Some(surface));

                    // FIXME: granular.
                    self.queue_redraw_all();
                }

                return;
            }
        }

        // Something else got clicked, clear on-demand layer-shell focus.
        if self.focus.layer_on_demand().is_some() {
            self.focus.set_layer_on_demand(None);

            // FIXME: granular.
            self.queue_redraw_all();
        }
    }

    /// Handles focus-follows-mouse behavior when the pointer moves to new contents.
    pub fn handle_focus_follows_mouse(&mut self, new_focus: &PointContents) {
        let Some(ffm) = self.config.borrow().input.focus_follows_mouse else {
            return;
        };

        let pointer = &self.seat.get_pointer().unwrap();
        if pointer.is_grabbed() {
            return;
        }

        if self.ui.mru.is_open() {
            return;
        }

        // Recompute the current pointer focus because we don't update it during animations.
        let current_focus = self.contents_under(pointer.current_location());

        if let Some(output) = &new_focus.output {
            if current_focus.output.as_ref() != Some(output) {
                self.layout.focus_output(output);
            }
        }

        if let Some(window) = &new_focus.window {
            // Overview mode has been removed, !self.layout.is_overview_open() is always true
            if current_focus.window.as_ref() != Some(window) {
                let (window, hit) = window;

                // Don't trigger focus-follows-mouse over the tab indicator.
                if matches!(
                    hit,
                    HitType::Activate {
                        is_tab_indicator: true
                    }
                ) {
                    return;
                }

                if !self.layout.should_trigger_focus_follows_mouse_on(window) {
                    return;
                }

                if let Some(threshold) = ffm.max_scroll_amount {
                    if self.layout.scroll_amount_to_activate(window) > threshold.0 {
                        return;
                    }
                }

                self.layout.activate_window_without_raising(window);
                self.focus.set_layer_on_demand(None);
            }
        }

        if let Some(layer) = &new_focus.layer {
            if current_focus.layer.as_ref() != Some(layer) {
                self.focus.set_layer_on_demand(Some(layer.clone()));
            }
        }
    }

    /// Resets the pointer inactivity timer.
    ///
    /// Called when the pointer moves to reset the hide-after-inactive timeout.
    pub fn reset_pointer_inactivity_timer(&mut self) {
        if self.cursor.timer_reset_this_iter() {
            return;
        }

        let _span = tracy_client::span!("Niri::reset_pointer_inactivity_timer");

        if let Some(token) = self.cursor.inactivity_timer() {
            self.event_loop.remove(token);
        }

        let Some(timeout_ms) = self.config.borrow().cursor.hide_after_inactive_ms else {
            return;
        };

        let duration = Duration::from_millis(timeout_ms as u64);
        let timer = Timer::from_duration(duration);
        let timer_token = self
            .event_loop
            .insert_source(timer, move |_, _, state| {
                state.niri.cursor.set_inactivity_timer(None);

                // If the pointer is already invisible, don't reset it back to Hidden causing one
                // frame of hover.
                if state.niri.cursor.visibility().is_visible() {
                    state.niri.cursor.hide_for_inactivity();
                    state.niri.queue_redraw_all();
                }

                TimeoutAction::Drop
            })
            .unwrap();
        self.cursor.set_inactivity_timer(Some(timer_token));

        self.cursor.mark_timer_reset();
    }

    /// Notifies that there has been user activity this iteration.
    pub fn notify_activity(&mut self) {
        if self.notified_activity_this_iteration {
            return;
        }

        let _span = tracy_client::span!("Niri::notify_activity");

        self.protocols.idle_notifier.notify_activity(&self.seat);

        self.notified_activity_this_iteration = true;
    }
}
