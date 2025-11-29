// TEAM_063: Tile animation methods
//!
//! This module handles animation-related methods for tiles including
//! open, resize, move, and alpha animations.

use smithay::utils::{Logical, Point};

use super::{MoveAnimation, Tile};
use crate::animation::Animation;
use crate::layout::LayoutElement;
use crate::render_helpers::offscreen::OffscreenBuffer;

impl<W: LayoutElement> Tile<W> {
    pub fn advance_animations(&mut self) {
        if let Some(open) = &mut self.open_animation {
            if open.is_done() {
                self.open_animation = None;
            }
        }

        if let Some(resize) = &mut self.resize_animation {
            if resize.anim.is_done() {
                self.resize_animation = None;
            }
        }

        if let Some(move_) = &mut self.move_x_animation {
            if move_.anim.is_done() {
                self.move_x_animation = None;
            }
        }
        if let Some(move_) = &mut self.move_y_animation {
            if move_.anim.is_done() {
                self.move_y_animation = None;
            }
        }

        if let Some(alpha) = &mut self.alpha_animation {
            if !alpha.hold_after_done && alpha.anim.is_done() {
                self.alpha_animation = None;
            }
        }
    }

    pub fn are_animations_ongoing(&self) -> bool {
        self.are_transitions_ongoing() || self.window.rules().baba_is_float == Some(true)
    }

    pub fn are_transitions_ongoing(&self) -> bool {
        self.open_animation.is_some()
            || self.resize_animation.is_some()
            || self.move_x_animation.is_some()
            || self.move_y_animation.is_some()
            || self
                .alpha_animation
                .as_ref()
                .is_some_and(|alpha| !alpha.anim.is_done())
    }

    pub fn render_offset(&self) -> Point<f64, Logical> {
        let mut offset = Point::from((0., 0.));

        if let Some(move_) = &self.move_x_animation {
            offset.x += move_.from * move_.anim.value();
        }
        if let Some(move_) = &self.move_y_animation {
            offset.y += move_.from * move_.anim.value();
        }

        offset += self.interactive_move_offset;

        offset
    }

    pub fn start_open_animation(&mut self) {
        use crate::layout::elements::opening_window::OpenAnimation;
        self.open_animation = Some(OpenAnimation::new(Animation::new(
            self.clock.clone(),
            0.,
            1.,
            0.,
            self.options.animations.window_open.anim,
        )));
    }

    pub fn resize_animation(&self) -> Option<&Animation> {
        self.resize_animation.as_ref().map(|resize| &resize.anim)
    }

    pub fn animate_move_from(&mut self, from: Point<f64, Logical>) {
        self.animate_move_x_from(from.x);
        self.animate_move_y_from(from.y);
    }

    pub fn animate_move_x_from(&mut self, from: f64) {
        self.animate_move_x_from_with_config(from, self.options.animations.window_movement.0);
    }

    pub fn animate_move_x_from_with_config(&mut self, from: f64, config: niri_config::Animation) {
        let current_offset = self.render_offset().x;

        // Preserve the previous config if ongoing.
        let anim = self.move_x_animation.take().map(|move_| move_.anim);
        let anim = anim
            .map(|anim| anim.restarted(1., 0., 0.))
            .unwrap_or_else(|| Animation::new(self.clock.clone(), 1., 0., 0., config));

        self.move_x_animation = Some(MoveAnimation {
            anim,
            from: from + current_offset,
        });
    }

    pub fn animate_move_y_from(&mut self, from: f64) {
        self.animate_move_y_from_with_config(from, self.options.animations.window_movement.0);
    }

    pub fn animate_move_y_from_with_config(&mut self, from: f64, config: niri_config::Animation) {
        let current_offset = self.render_offset().y;

        // Preserve the previous config if ongoing.
        let anim = self.move_y_animation.take().map(|move_| move_.anim);
        let anim = anim
            .map(|anim| anim.restarted(1., 0., 0.))
            .unwrap_or_else(|| Animation::new(self.clock.clone(), 1., 0., 0., config));

        let final_from = from + current_offset;

        self.move_y_animation = Some(MoveAnimation {
            anim,
            from: final_from,
        });
    }

    pub fn offset_move_y_anim_current(&mut self, offset: f64) {
        if let Some(move_) = self.move_y_animation.as_mut() {
            // If the anim is almost done, there's little point trying to offset it; we can let
            // things jump. If it turns out like a bad idea, we could restart the anim instead.
            let value = move_.anim.value();
            if value > 0.001 {
                move_.from += offset / value;
            }
        }
    }

    pub fn stop_move_animations(&mut self) {
        self.move_x_animation = None;
        self.move_y_animation = None;
    }

    pub fn animate_alpha(&mut self, from: f64, to: f64, config: niri_config::Animation) {
        use super::AlphaAnimation;

        let from = from.clamp(0., 1.);
        let to = to.clamp(0., 1.);

        let (current, offscreen) = if let Some(alpha) = self.alpha_animation.take() {
            (alpha.anim.clamped_value(), alpha.offscreen)
        } else {
            (from, OffscreenBuffer::default())
        };

        self.alpha_animation = Some(AlphaAnimation {
            anim: Animation::new(self.clock.clone(), current, to, 0., config),
            hold_after_done: false,
            offscreen,
        });
    }

    pub fn ensure_alpha_animates_to_1(&mut self) {
        if let Some(alpha) = &self.alpha_animation {
            if alpha.anim.to() != 1. {
                // Cancel animation instead of starting a new one because the user likely wants to
                // see the tile right away.
                self.alpha_animation = None;
            }
        }
    }

    pub fn hold_alpha_animation_after_done(&mut self) {
        if let Some(alpha) = &mut self.alpha_animation {
            alpha.hold_after_done = true;
        }
    }
}
