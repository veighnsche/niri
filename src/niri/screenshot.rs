//! Screenshot capture and saving for the Niri compositor.
//!
//! This module handles capturing screenshots and saving them to disk.

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use anyhow::Context;
use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::utils::{Relocate, RelocateRenderElement};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;
use smithay::utils::{Physical, Scale, Size, Transform};

use crate::render_helpers::{
    encompassing_geo, render_to_encompassing_texture, render_to_texture, render_to_vec, RenderTarget,
};
use crate::ui::screenshot_ui::OutputScreenshot;
use crate::utils::{make_screenshot_path, write_png_rgba8};
use crate::window::Mapped;

use super::Niri;

// =============================================================================
// Screenshot Methods
// =============================================================================

impl Niri {
    /// Captures screenshots of all outputs for the screenshot UI.
    pub fn capture_screenshots<'a>(
        &'a self,
        renderer: &'a mut GlesRenderer,
    ) -> impl Iterator<Item = (Output, [OutputScreenshot; 3])> + 'a {
        self.global_space.outputs().cloned().filter_map(|output| {
            let size = output.current_mode().unwrap().size;
            let transform = output.current_transform();
            let size = transform.transform_size(size);

            let scale = Scale::from(output.current_scale().fractional_scale());
            let targets = [
                RenderTarget::Output,
                RenderTarget::Screencast,
                RenderTarget::ScreenCapture,
            ];
            let screenshot = targets.map(|target| {
                let elements = self.render::<GlesRenderer>(renderer, &output, false, target);
                let elements = elements.iter().rev();

                let res = render_to_texture(
                    renderer,
                    size,
                    scale,
                    Transform::Normal,
                    Fourcc::Abgr8888,
                    elements,
                );
                if let Err(err) = &res {
                    warn!("error rendering output {}: {err:?}", output.name());
                }
                let res_output = res.ok();

                let pointer = self.pointer_element(renderer, &output);
                let res_pointer = if pointer.is_empty() {
                    None
                } else {
                    let res = render_to_encompassing_texture(
                        renderer,
                        scale,
                        Transform::Normal,
                        Fourcc::Abgr8888,
                        &pointer,
                    );
                    if let Err(err) = &res {
                        warn!("error rendering pointer for {}: {err:?}", output.name());
                    }
                    res.ok()
                };

                res_output.map(|(texture, _)| {
                    OutputScreenshot::from_textures(
                        renderer,
                        scale,
                        texture,
                        res_pointer.map(|(texture, _, geo)| (texture, geo)),
                    )
                })
            });

            if screenshot.iter().any(|res| res.is_none()) {
                return None;
            }

            let screenshot = screenshot.map(|res| res.unwrap());
            Some((output, screenshot))
        })
    }

    /// Takes a screenshot of a single output.
    pub fn screenshot(
        &mut self,
        renderer: &mut GlesRenderer,
        output: &Output,
        write_to_disk: bool,
        include_pointer: bool,
        path: Option<String>,
    ) -> anyhow::Result<()> {
        let _span = tracy_client::span!("Niri::screenshot");

        self.update_render_elements(Some(output));

        let size = output.current_mode().unwrap().size;
        let transform = output.current_transform();
        let size = transform.transform_size(size);

        let scale = Scale::from(output.current_scale().fractional_scale());
        let elements = self.render::<GlesRenderer>(
            renderer,
            output,
            include_pointer,
            RenderTarget::ScreenCapture,
        );
        let elements = elements.iter().rev();
        let pixels = render_to_vec(
            renderer,
            size,
            scale,
            Transform::Normal,
            Fourcc::Abgr8888,
            elements,
        )?;

        self.save_screenshot(size, pixels, write_to_disk, path)
            .context("error saving screenshot")
    }

    // screenshot_window remains in mod.rs due to Mapped private method access

    /// Saves a screenshot to disk and/or clipboard.
    pub fn save_screenshot(
        &self,
        size: Size<i32, Physical>,
        pixels: Vec<u8>,
        write_to_disk: bool,
        path_arg: Option<String>,
    ) -> anyhow::Result<()> {
        use smithay::wayland::selection::data_device::set_data_device_selection;

        let path = write_to_disk
            .then(|| {
                // When given an explicit path, don't try to strftime it or create parents.
                path_arg.map(|p| (PathBuf::from(p), false)).or_else(|| {
                    match make_screenshot_path(&self.config.borrow()) {
                        Ok(path) => path.map(|p| (p, true)),
                        Err(err) => {
                            warn!("error making screenshot path: {err:?}");
                            None
                        }
                    }
                })
            })
            .flatten();

        // Prepare to set the encoded image as our clipboard selection. This must be done from the
        // main thread.
        let (tx, rx) = calloop::channel::sync_channel::<Arc<[u8]>>(1);
        self.event_loop
            .insert_source(rx, move |event, _, state| match event {
                calloop::channel::Event::Msg(buf) => {
                    set_data_device_selection(
                        &state.niri.display_handle,
                        &state.niri.seat,
                        vec![String::from("image/png")],
                        buf.clone(),
                    );
                }
                calloop::channel::Event::Closed => (),
            })
            .unwrap();

        // Prepare to send screenshot completion event back to main thread.
        let (event_tx, event_rx) = calloop::channel::sync_channel::<Option<String>>(1);
        self.event_loop
            .insert_source(event_rx, move |event, _, state| match event {
                calloop::channel::Event::Msg(path) => {
                    state.ipc_screenshot_taken(path);
                }
                calloop::channel::Event::Closed => (),
            })
            .unwrap();

        // Encode and save the image in a thread as it's slow.
        thread::spawn(move || {
            let mut buf = vec![];

            let w = std::io::Cursor::new(&mut buf);
            if let Err(err) = write_png_rgba8(w, size.w as u32, size.h as u32, &pixels) {
                warn!("error encoding screenshot image: {err:?}");
                return;
            }

            let buf: Arc<[u8]> = Arc::from(buf.into_boxed_slice());
            let _ = tx.send(buf.clone());

            let mut image_path = None;

            if let Some((path, create_parent)) = path {
                debug!("saving screenshot to {path:?}");

                if create_parent {
                    if let Some(parent) = path.parent() {
                        // Relative paths with one component, i.e. "test.png", have Some("") parent.
                        if !parent.as_os_str().is_empty() {
                            if let Err(err) = std::fs::create_dir_all(parent) {
                                if err.kind() != std::io::ErrorKind::AlreadyExists {
                                    warn!("error creating screenshot directory: {err:?}");
                                }
                            }
                        }
                    }
                }

                match std::fs::write(&path, buf) {
                    Ok(()) => image_path = Some(path),
                    Err(err) => {
                        warn!("error saving screenshot image: {err:?}");
                    }
                }
            } else {
                debug!("not saving screenshot to disk");
            }

            #[cfg(feature = "dbus")]
            if let Err(err) = crate::utils::show_screenshot_notification(image_path.as_deref()) {
                warn!("error showing screenshot notification: {err:?}");
            }

            // Send screenshot completion event.
            let path_string = image_path
                .as_ref()
                .and_then(|p| p.to_str())
                .map(|s| s.to_owned());
            let _ = event_tx.send(path_string);
        });

        Ok(())
    }

    /// Takes a screenshot of all outputs (for D-Bus screenshot interface).
    #[cfg(feature = "dbus")]
    pub fn screenshot_all_outputs(
        &mut self,
        renderer: &mut GlesRenderer,
        include_pointer: bool,
        on_done: impl FnOnce(PathBuf) + Send + 'static,
    ) -> anyhow::Result<()> {
        use anyhow::ensure;
        use std::env;

        let _span = tracy_client::span!("Niri::screenshot_all_outputs");

        self.update_render_elements(None);

        let outputs: Vec<_> = self.global_space.outputs().cloned().collect();

        // FIXME: support multiple outputs, needs fixing multi-scale handling and cropping.
        ensure!(outputs.len() == 1);

        let output = outputs.into_iter().next().unwrap();
        let geom = self.global_space.output_geometry(&output).unwrap();

        let output_scale = output.current_scale().integer_scale();
        let geom = geom.to_physical(output_scale);

        let size = geom.size;
        let transform = output.current_transform();
        let size = transform.transform_size(size);

        let elements = self.render::<GlesRenderer>(
            renderer,
            &output,
            include_pointer,
            RenderTarget::ScreenCapture,
        );
        let elements = elements.iter().rev();
        let pixels = render_to_vec(
            renderer,
            size,
            Scale::from(f64::from(output_scale)),
            Transform::Normal,
            Fourcc::Abgr8888,
            elements,
        )?;

        let path = make_screenshot_path(&self.config.borrow())
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                let mut path = env::temp_dir();
                path.push("screenshot.png");
                path
            });
        debug!("saving screenshot to {path:?}");

        thread::spawn(move || {
            let file = match std::fs::File::create(&path) {
                Ok(file) => file,
                Err(err) => {
                    warn!("error creating file: {err:?}");
                    return;
                }
            };

            let w = std::io::BufWriter::new(file);
            if let Err(err) = write_png_rgba8(w, size.w as u32, size.h as u32, &pixels) {
                warn!("error encoding screenshot image: {err:?}");
                return;
            }

            on_done(path);
        });

        Ok(())
    }
}
