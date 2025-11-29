//! zwlr_screencopy protocol implementation for the Niri compositor.
//!
//! This module handles the zwlr_screencopy protocol for screen capture.

use std::cell::OnceCell;
use std::mem;

use anyhow::{bail, ensure, Context};
use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::backend::renderer::element::utils::{Relocate, RelocateRenderElement};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::sync::SyncPoint;
use smithay::output::{Output, OutputModeSource};
use smithay::utils::{Physical, Rectangle, Scale, Transform};

use smithay::reexports::wayland_protocols_wlr::screencopy::v1::server::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;

use crate::protocols::screencopy::{Screencopy, ScreencopyBuffer};
use crate::render_helpers::{render_to_dmabuf, render_to_shm, RenderTarget};

use super::{Niri, OutputRenderElements};

// =============================================================================
// Screencopy Methods
// =============================================================================

impl Niri {
    /// Renders for screencopy with damage tracking.
    pub fn render_for_screencopy_with_damage(
        &mut self,
        renderer: &mut GlesRenderer,
        output: &Output,
    ) {
        let _span = tracy_client::span!("Niri::render_for_screencopy_with_damage");

        let mut screencopy_state = mem::take(&mut self.screencopy_state);
        let elements = OnceCell::new();

        for queue in screencopy_state.queues_mut() {
            let (damage_tracker, screencopy) = queue.split();
            if let Some(screencopy) = screencopy {
                if screencopy.output() == output {
                    let elements = elements.get_or_init(|| {
                        self.render(renderer, output, true, RenderTarget::ScreenCapture)
                    });
                    // FIXME: skip elements if not including pointers
                    let render_result = Self::render_for_screencopy_internal(
                        renderer,
                        output,
                        elements,
                        true,
                        damage_tracker,
                        screencopy,
                    );
                    match render_result {
                        Ok((sync, damages)) => {
                            if let Some(damages) = damages {
                                // Convert from Physical coordinates back to Buffer coordinates.
                                let transform = output.current_transform();
                                let physical_size =
                                    transform.transform_size(screencopy.buffer_size());
                                let damages = damages.iter().map(|dmg| {
                                    dmg.to_logical(1).to_buffer(
                                        1,
                                        transform.invert(),
                                        &physical_size.to_logical(1),
                                    )
                                });

                                screencopy.damage(damages);
                                queue.pop().submit_after_sync(false, sync, &self.event_loop);
                            } else {
                                trace!("no damage found, waiting till next redraw");
                            }
                        }
                        Err(err) => {
                            // Recreate damage tracker to report full damage next check.
                            *damage_tracker =
                                OutputDamageTracker::new((0, 0), 1.0, Transform::Normal);
                            queue.pop();
                            warn!("error rendering for screencopy: {err:?}");
                        }
                    }
                };
            }
        }

        self.screencopy_state = screencopy_state;
    }

    /// Renders for screencopy without damage tracking.
    pub fn render_for_screencopy_without_damage(
        &mut self,
        renderer: &mut GlesRenderer,
        manager: &ZwlrScreencopyManagerV1,
        screencopy: Screencopy,
    ) -> anyhow::Result<()> {
        let _span = tracy_client::span!("Niri::render_for_screencopy");

        let output = screencopy.output();
        ensure!(
            self.output_state.contains_key(output),
            "screencopy output missing"
        );

        self.update_render_elements(Some(output));

        let elements = self.render(
            renderer,
            output,
            screencopy.overlay_cursor(),
            RenderTarget::ScreenCapture,
        );
        let Some(queue) = self.screencopy_state.get_queue_mut(manager) else {
            bail!("screencopy manager destroyed already");
        };
        let damage_tracker = queue.split().0;

        let render_result = Self::render_for_screencopy_internal(
            renderer,
            output,
            &elements,
            false,
            damage_tracker,
            &screencopy,
        );

        let res = render_result
            .map(|(sync, _damage)| screencopy.submit_after_sync(false, sync, &self.event_loop));

        if res.is_err() {
            // Recreate damage tracker to report full damage next check.
            *damage_tracker = OutputDamageTracker::new((0, 0), 1.0, Transform::Normal);
        }

        res
    }

    /// Internal helper for rendering to screencopy buffers.
    #[allow(clippy::type_complexity)]
    fn render_for_screencopy_internal<'a>(
        renderer: &mut GlesRenderer,
        output: &Output,
        elements: &[OutputRenderElements<GlesRenderer>],
        with_damage: bool,
        damage_tracker: &'a mut OutputDamageTracker,
        screencopy: &Screencopy,
    ) -> anyhow::Result<(Option<SyncPoint>, Option<&'a Vec<Rectangle<i32, Physical>>>)> {
        let OutputModeSource::Static {
            size: last_size,
            scale: last_scale,
            transform: last_transform,
        } = damage_tracker.mode().clone()
        else {
            unreachable!("damage tracker must have static mode");
        };

        let size = screencopy.buffer_size();
        let scale: Scale<f64> = output.current_scale().fractional_scale().into();
        let transform = output.current_transform();

        if size != last_size || scale != last_scale || transform != last_transform {
            *damage_tracker = OutputDamageTracker::new(size, scale, transform);
        }

        let region_loc = screencopy.region_loc();
        let elements = elements
            .iter()
            .map(|element| {
                RelocateRenderElement::from_element(
                    element,
                    region_loc.upscale(-1),
                    Relocate::Relative,
                )
            })
            .collect::<Vec<_>>();

        // Just checked damage tracker has static mode
        let damages = damage_tracker.damage_output(1, &elements).unwrap().0;
        if with_damage && damages.is_none() {
            return Ok((None, None));
        }

        let elements = elements.iter().rev();

        let sync = match screencopy.buffer() {
            ScreencopyBuffer::Dmabuf(dmabuf) => {
                let sync =
                    render_to_dmabuf(renderer, dmabuf.clone(), size, scale, transform, elements)
                        .context("error rendering to screencopy dmabuf")?;
                Some(sync)
            }
            ScreencopyBuffer::Shm(wl_buffer) => {
                render_to_shm(renderer, wl_buffer, size, scale, transform, elements)
                    .context("error rendering to screencopy shm buffer")?;
                None
            }
        };

        Ok((sync, damages))
    }

    /// Removes an output from all screencopy queues.
    pub fn remove_screencopy_output(&mut self, output: &Output) {
        let _span = tracy_client::span!("Niri::remove_screencopy_output");
        for queue in self.screencopy_state.queues_mut() {
            queue.remove_output(output);
        }
    }
}
