//! Action handler for keyboard/mouse bindings.
//!
//! This module contains the `do_action` method which handles all user-triggered actions
//! from keybindings, mouse bindings, and IPC commands.

use niri_config::{Action, MruDirection};
use niri_ipc::LayoutSwitchTarget;
use smithay::input::keyboard::Layout;
use smithay::input::pointer::CursorImageStatus;

use super::helpers::allowed_when_locked;
use crate::layout::types::ScrollDirection;
use crate::layout::{ActivateWindow, LayoutElement as _};
use crate::niri::{CastTarget, State};
use crate::ui::mru::WindowMru;
use crate::utils::spawning::{spawn, spawn_sh};

// TEAM_087: Action handler trait for State
pub trait ActionHandler {
    fn do_action(&mut self, action: Action, allow_when_locked: bool);
}

impl ActionHandler for State {
    fn do_action(&mut self, action: Action, allow_when_locked: bool) {
        if self.niri.is_locked() && !(allow_when_locked || allowed_when_locked(&action)) {
            return;
        }

        if let Some(touch) = self.niri.seat.get_touch() {
            touch.cancel(self);
        }

        match action {
            Action::Quit(skip_confirmation) => {
                if !skip_confirmation && self.niri.ui.exit_dialog.show() {
                    self.niri.queue_redraw_all();
                    return;
                }

                info!("quitting as requested");
                self.niri.stop_signal.stop()
            }
            Action::ChangeVt(vt) => {
                self.backend.change_vt(vt);
                // Changing VT may not deliver the key releases, so clear the state.
                self.niri.suppressed_keys.clear();
            }
            Action::Suspend => {
                self.backend.suspend();
                // Suspend may not deliver the key releases, so clear the state.
                self.niri.suppressed_keys.clear();
            }
            Action::PowerOffMonitors => {
                self.niri.deactivate_monitors(&mut self.backend);
            }
            Action::PowerOnMonitors => {
                self.niri.activate_monitors(&mut self.backend);
            }
            Action::ToggleDebugTint => {
                self.backend.toggle_debug_tint();
                self.niri.queue_redraw_all();
            }
            Action::DebugToggleOpaqueRegions => {
                self.niri.debug_draw_opaque_regions = !self.niri.debug_draw_opaque_regions;
                self.niri.queue_redraw_all();
            }
            Action::DebugToggleDamage => {
                self.niri.debug_toggle_damage();
            }
            Action::Spawn(command) => {
                info!("Spawn action triggered with command: {:?}", command);
                let (token, _) = self.niri.protocols.activation.create_external_token(None);
                spawn(command, Some(token.clone()));
            }
            Action::SpawnSh(command) => {
                let (token, _) = self.niri.protocols.activation.create_external_token(None);
                spawn_sh(command, Some(token.clone()));
            }
            Action::DoScreenTransition(delay_ms) => {
                self.backend.with_primary_renderer(|renderer| {
                    self.niri.do_screen_transition(renderer, delay_ms);
                });
            }
            Action::ScreenshotScreen(write_to_disk, show_pointer, path) => {
                let active = self.niri.layout.active_output().cloned();
                if let Some(active) = active {
                    self.backend.with_primary_renderer(|renderer| {
                        if let Err(err) = self.niri.screenshot(
                            renderer,
                            &active,
                            write_to_disk,
                            show_pointer,
                            path,
                        ) {
                            warn!("error taking screenshot: {err:?}");
                        }
                    });
                }
            }
            Action::ConfirmScreenshot { write_to_disk } => {
                self.confirm_screenshot(write_to_disk);
            }
            Action::CancelScreenshot => {
                if !self.niri.ui.screenshot.is_open() {
                    return;
                }

                self.niri.ui.screenshot.close();
                self.niri
                    .cursor
                    .manager
                    .set_cursor_image(CursorImageStatus::default_named());
                self.niri.queue_redraw_all();
            }
            Action::ScreenshotTogglePointer => {
                self.niri.ui.screenshot.toggle_pointer();
                self.niri.queue_redraw_all();
            }
            Action::Screenshot(show_cursor, path) => {
                self.open_screenshot_ui(show_cursor, path);
                self.niri.cancel_mru();
            }
            Action::ScreenshotWindow(write_to_disk, path) => {
                let focus = self.niri.layout.focus_with_output();
                if let Some((mapped, output)) = focus {
                    self.backend.with_primary_renderer(|renderer| {
                        if let Err(err) = self.niri.screenshot_window(
                            renderer,
                            output,
                            mapped,
                            write_to_disk,
                            path,
                        ) {
                            warn!("error taking screenshot: {err:?}");
                        }
                    });
                }
            }
            Action::ScreenshotWindowById {
                id,
                write_to_disk,
                path,
            } => {
                let mut windows = self.niri.layout.windows();
                let window = windows.find(|(_, m)| m.id().get() == id);
                if let Some((Some(monitor), mapped)) = window {
                    let output = monitor.output();
                    self.backend.with_primary_renderer(|renderer| {
                        if let Err(err) = self.niri.screenshot_window(
                            renderer,
                            output,
                            mapped,
                            write_to_disk,
                            path,
                        ) {
                            warn!("error taking screenshot: {err:?}");
                        }
                    });
                }
            }
            Action::ToggleKeyboardShortcutsInhibit => {
                if let Some(inhibitor) = self
                    .niri
                    .focus
                    .current
                    .surface()
                    .and_then(|surface| self.niri.focus.shortcut_inhibitors.get(surface))
                {
                    if inhibitor.is_active() {
                        inhibitor.inactivate();
                    } else {
                        inhibitor.activate();
                    }
                }
            }
            Action::CloseWindow => {
                if let Some(mapped) = self.niri.layout.focus() {
                    mapped.toplevel().send_close();
                }
            }
            Action::CloseWindowById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                if let Some((_, mapped)) = window {
                    mapped.toplevel().send_close();
                }
            }
            Action::FullscreenWindow => {
                let focus = self.niri.layout.focus().map(|m| m.window.clone());
                if let Some(window) = focus {
                    self.niri.layout.toggle_fullscreen(&window);
                    if let Some(output) = self.niri.layout.active_output().cloned() {
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            Action::FullscreenWindowById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let output = window.as_ref().and_then(|(mon, _)| mon.map(|m| m.output().clone()));
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.toggle_fullscreen(&window);
                    if let Some(output) = output {
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            Action::ToggleWindowedFullscreen => {
                let focus = self.niri.layout.focus().map(|m| m.window.clone());
                if let Some(window) = focus {
                    self.niri.layout.toggle_windowed_fullscreen(&window);
                    if let Some(output) = self.niri.layout.active_output().cloned() {
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            Action::ToggleWindowedFullscreenById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let output = window.as_ref().and_then(|(mon, _)| mon.map(|m| m.output().clone()));
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.toggle_windowed_fullscreen(&window);
                    if let Some(output) = output {
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            Action::FocusWindow(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.focus_window(&window);
                }
            }
            Action::FocusWindowInColumn(index) => {
                self.niri.layout.focus_window_in_column(index);
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusWindowPrevious => {
                let current = self.niri.layout.focus().map(|win| win.id());
                if let Some(window) = self
                    .niri
                    .layout
                    .windows()
                    .map(|(_, win)| win)
                    .filter(|win| Some(win.id()) != current)
                    .max_by_key(|win| win.get_focus_timestamp())
                    .map(|win| win.window.clone())
                {
                    // Commit current focus so repeated focus-window-previous works as expected.
                    self.niri.mru_apply_keyboard_commit();

                    self.focus_window(&window);
                }
            }
            Action::SwitchLayout(action) => {
                let keyboard = &self.niri.seat.get_keyboard().unwrap();
                keyboard.with_xkb_state(self, |mut state| match action {
                    LayoutSwitchTarget::Next => state.cycle_next_layout(),
                    LayoutSwitchTarget::Prev => state.cycle_prev_layout(),
                    LayoutSwitchTarget::Index(layout) => {
                        let num_layouts = state.xkb().lock().unwrap().layouts().count();
                        if usize::from(layout) >= num_layouts {
                            warn!("requested layout doesn't exist")
                        } else {
                            state.set_layout(Layout(layout.into()))
                        }
                    }
                });
            }
            Action::MoveColumnLeft => {
                if self.niri.ui.screenshot.is_open() {
                    self.niri.ui.screenshot.move_left();
                } else {
                    self.niri.layout.move_left();
                    self.maybe_warp_cursor_to_focus();
                }

                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::MoveColumnRight => {
                if self.niri.ui.screenshot.is_open() {
                    self.niri.ui.screenshot.move_right();
                } else {
                    self.niri.layout.move_right();
                    self.maybe_warp_cursor_to_focus();
                }

                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::MoveColumnToFirst => {
                self.niri.layout.move_column_to_first();
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::MoveColumnToLast => {
                self.niri.layout.move_column_to_last();
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::MoveColumnLeftOrToMonitorLeft => {
                let source_output = self.niri.layout.active_output().cloned();
                if self.niri.ui.screenshot.is_open() {
                    self.niri.ui.screenshot.move_left();
                } else if let Some(output) = self.niri.output_left() {
                    if self.niri.layout.move_column_left_or_to_output(&output)
                        && !self.maybe_warp_cursor_to_focus_centered()
                    {
                        self.move_cursor_to_output(&output);
                    } else {
                        self.maybe_warp_cursor_to_focus();
                    }
                } else {
                    self.niri.layout.move_left();
                    self.maybe_warp_cursor_to_focus();
                }

                // Redraw source output
                if let Some(src) = &source_output {
                    self.niri.queue_redraw(src);
                }
                // Redraw target output if different
                if let Some(dst) = self.niri.layout.active_output().cloned() {
                    if source_output.as_ref() != Some(&dst) {
                        self.niri.queue_redraw(&dst);
                    }
                }
            }
            Action::MoveColumnRightOrToMonitorRight => {
                let source_output = self.niri.layout.active_output().cloned();
                if self.niri.ui.screenshot.is_open() {
                    self.niri.ui.screenshot.move_right();
                } else if let Some(output) = self.niri.output_right() {
                    if self.niri.layout.move_column_right_or_to_output(&output)
                        && !self.maybe_warp_cursor_to_focus_centered()
                    {
                        self.move_cursor_to_output(&output);
                    } else {
                        self.maybe_warp_cursor_to_focus();
                    }
                } else {
                    self.niri.layout.move_right();
                    self.maybe_warp_cursor_to_focus();
                }

                // Redraw source output
                if let Some(src) = &source_output {
                    self.niri.queue_redraw(src);
                }
                // Redraw target output if different
                if let Some(dst) = self.niri.layout.active_output().cloned() {
                    if source_output.as_ref() != Some(&dst) {
                        self.niri.queue_redraw(&dst);
                    }
                }
            }
            Action::MoveWindowDown => {
                if self.niri.ui.screenshot.is_open() {
                    self.niri.ui.screenshot.move_down();
                } else {
                    self.niri.layout.move_down();
                    self.maybe_warp_cursor_to_focus();
                }

                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::MoveWindowUp => {
                if self.niri.ui.screenshot.is_open() {
                    self.niri.ui.screenshot.move_up();
                } else {
                    self.niri.layout.move_up();
                    self.maybe_warp_cursor_to_focus();
                }

                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced MoveWindowDownOrToWorkspaceDown with MoveWindowDownOrToRowDown
            Action::MoveWindowDownOrToRowDown => {
                if self.niri.ui.screenshot.is_open() {
                    self.niri.ui.screenshot.move_down();
                } else {
                    self.niri.layout.move_down_or_to_row_down();
                }
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced MoveWindowUpOrToWorkspaceUp with MoveWindowUpOrToRowUp
            Action::MoveWindowUpOrToRowUp => {
                if self.niri.ui.screenshot.is_open() {
                    self.niri.ui.screenshot.move_up();
                } else {
                    self.niri.layout.move_up_or_to_row_up();
                }
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::ConsumeOrExpelWindowLeft => {
                self.niri.layout.consume_or_expel_window_left(None);
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::ConsumeOrExpelWindowLeftById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let output = window.as_ref().and_then(|(mon, _)| mon.map(|m| m.output().clone()));
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.consume_or_expel_window_left(Some(&window));
                    self.maybe_warp_cursor_to_focus();
                    if let Some(output) = output {
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            Action::ConsumeOrExpelWindowRight => {
                self.niri.layout.consume_or_expel_window_right(None);
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::ConsumeOrExpelWindowRightById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let output = window.as_ref().and_then(|(mon, _)| mon.map(|m| m.output().clone()));
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri
                        .layout
                        .consume_or_expel_window_right(Some(&window));
                    self.maybe_warp_cursor_to_focus();
                    if let Some(output) = output {
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            Action::FocusColumnLeft => {
                self.niri.layout.focus_left();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusColumnLeftUnderMouse => {
                if let Some((output, ws)) = self.niri.row_under_cursor(true) {
                    let ws_id = ws.id();
                    let ws = {
                        let mut workspaces = self.niri.layout.workspaces_mut();
                        workspaces.find(|ws| ws.id() == ws_id).unwrap()
                    };
                    ws.focus_left();
                    self.maybe_warp_cursor_to_focus();
                    self.niri.focus.layer_on_demand = None;
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusColumnRight => {
                self.niri.layout.focus_right();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusColumnRightUnderMouse => {
                if let Some((output, ws)) = self.niri.row_under_cursor(true) {
                    let ws_id = ws.id();
                    let ws = {
                        let mut workspaces = self.niri.layout.workspaces_mut();
                        workspaces.find(|ws| ws.id() == ws_id).unwrap()
                    };
                    ws.focus_right();
                    self.maybe_warp_cursor_to_focus();
                    self.niri.focus.layer_on_demand = None;
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusColumnFirst => {
                self.niri.layout.focus_column_first();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusColumnLast => {
                self.niri.layout.focus_column_last();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusColumnRightOrFirst => {
                self.niri.layout.focus_column_right_or_first();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusColumnLeftOrLast => {
                self.niri.layout.focus_column_left_or_last();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusColumn(index) => {
                self.niri.layout.focus_column(index);
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusWindowOrMonitorUp => {
                let source_output = self.niri.layout.active_output().cloned();
                if let Some(output) = self.niri.output_up() {
                    if self.niri.layout.focus_window_up_or_output(&output)
                        && !self.maybe_warp_cursor_to_focus_centered()
                    {
                        self.move_cursor_to_output(&output);
                    } else {
                        self.maybe_warp_cursor_to_focus();
                    }
                } else {
                    self.niri.layout.focus_up();
                    self.maybe_warp_cursor_to_focus();
                }
                self.niri.focus.layer_on_demand = None;

                // Redraw source output
                if let Some(src) = &source_output {
                    self.niri.queue_redraw(src);
                }
                // Redraw target output if different
                if let Some(dst) = self.niri.layout.active_output().cloned() {
                    if source_output.as_ref() != Some(&dst) {
                        self.niri.queue_redraw(&dst);
                    }
                }
            }
            Action::FocusWindowOrMonitorDown => {
                let source_output = self.niri.layout.active_output().cloned();
                if let Some(output) = self.niri.output_down() {
                    if self.niri.layout.focus_window_down_or_output(&output)
                        && !self.maybe_warp_cursor_to_focus_centered()
                    {
                        self.move_cursor_to_output(&output);
                    } else {
                        self.maybe_warp_cursor_to_focus();
                    }
                } else {
                    self.niri.layout.focus_down();
                    self.maybe_warp_cursor_to_focus();
                }
                self.niri.focus.layer_on_demand = None;

                // Redraw source output
                if let Some(src) = &source_output {
                    self.niri.queue_redraw(src);
                }
                // Redraw target output if different
                if let Some(dst) = self.niri.layout.active_output().cloned() {
                    if source_output.as_ref() != Some(&dst) {
                        self.niri.queue_redraw(&dst);
                    }
                }
            }
            Action::FocusColumnOrMonitorLeft => {
                let source_output = self.niri.layout.active_output().cloned();
                if let Some(output) = self.niri.output_left() {
                    if self.niri.layout.focus_column_left_or_output(&output)
                        && !self.maybe_warp_cursor_to_focus_centered()
                    {
                        self.move_cursor_to_output(&output);
                    } else {
                        self.maybe_warp_cursor_to_focus();
                    }
                } else {
                    self.niri.layout.focus_left();
                    self.maybe_warp_cursor_to_focus();
                }
                self.niri.focus.layer_on_demand = None;

                // Redraw source output
                if let Some(src) = &source_output {
                    self.niri.queue_redraw(src);
                }
                // Redraw target output if different
                if let Some(dst) = self.niri.layout.active_output().cloned() {
                    if source_output.as_ref() != Some(&dst) {
                        self.niri.queue_redraw(&dst);
                    }
                }
            }
            Action::FocusColumnOrMonitorRight => {
                let source_output = self.niri.layout.active_output().cloned();
                if let Some(output) = self.niri.output_right() {
                    if self.niri.layout.focus_column_right_or_output(&output)
                        && !self.maybe_warp_cursor_to_focus_centered()
                    {
                        self.move_cursor_to_output(&output);
                    } else {
                        self.maybe_warp_cursor_to_focus();
                    }
                } else {
                    self.niri.layout.focus_right();
                    self.maybe_warp_cursor_to_focus();
                }
                self.niri.focus.layer_on_demand = None;

                // Redraw source output
                if let Some(src) = &source_output {
                    self.niri.queue_redraw(src);
                }
                // Redraw target output if different
                if let Some(dst) = self.niri.layout.active_output().cloned() {
                    if source_output.as_ref() != Some(&dst) {
                        self.niri.queue_redraw(&dst);
                    }
                }
            }
            Action::FocusWindowDown => {
                self.niri.layout.focus_down();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusWindowUp => {
                self.niri.layout.focus_up();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusWindowDownOrColumnLeft => {
                self.niri.layout.focus_down_or_left();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusWindowDownOrColumnRight => {
                self.niri.layout.focus_down_or_right();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusWindowUpOrColumnLeft => {
                self.niri.layout.focus_up_or_left();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusWindowUpOrColumnRight => {
                self.niri.layout.focus_up_or_right();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced FocusWindowOrWorkspaceDown with FocusWindowOrRowDown
            Action::FocusWindowOrRowDown => {
                self.niri.layout.focus_window_or_row_down();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced FocusWindowOrWorkspaceUp with FocusWindowOrRowUp
            Action::FocusWindowOrRowUp => {
                self.niri.layout.focus_window_or_row_up();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusWindowTop => {
                self.niri.layout.focus_window_top();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusWindowBottom => {
                self.niri.layout.focus_window_bottom();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusWindowDownOrTop => {
                self.niri.layout.focus_window_down_or_top();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusWindowUpOrBottom => {
                self.niri.layout.focus_window_up_or_bottom();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced MoveWindowToWorkspaceDown with MoveWindowToRowDown
            Action::MoveWindowToRowDown(focus) => {
                self.niri.layout.move_to_row_down(focus);
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced MoveWindowToWorkspaceUp with MoveWindowToRowUp
            Action::MoveWindowToRowUp(focus) => {
                self.niri.layout.move_to_row_up(focus);
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced MoveColumnToWorkspaceDown with MoveColumnToRowDown
            Action::MoveColumnToRowDown(focus) => {
                self.niri.layout.move_column_to_row_down(focus);
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced MoveColumnToWorkspaceUp with MoveColumnToRowUp
            Action::MoveColumnToRowUp(focus) => {
                self.niri.layout.move_column_to_row_up(focus);
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::MoveColumnToIndex(idx) => {
                self.niri.layout.move_column_to_index(idx);
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced FocusWorkspaceDown with FocusRowDown
            Action::FocusRowDown => {
                self.niri.layout.focus_row_down();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced FocusWorkspaceDownUnderMouse with FocusRowDownUnderMouse
            Action::FocusRowDownUnderMouse => {
                if let Some(output) = self.niri.output_under_cursor() {
                    if let Some(mon) = self.niri.layout.monitor_for_output_mut(&output) {
                        mon.switch_row_down();
                        self.maybe_warp_cursor_to_focus();
                        self.niri.focus.layer_on_demand = None;
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            // TEAM_012: Replaced FocusWorkspaceUp with FocusRowUp
            Action::FocusRowUp => {
                self.niri.layout.focus_row_up();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced FocusWorkspaceUpUnderMouse with FocusRowUpUnderMouse
            Action::FocusRowUpUnderMouse => {
                if let Some(output) = self.niri.output_under_cursor() {
                    if let Some(mon) = self.niri.layout.monitor_for_output_mut(&output) {
                        mon.switch_row_up();
                        self.maybe_warp_cursor_to_focus();
                        self.niri.focus.layer_on_demand = None;
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            // TEAM_012: Replaced FocusWorkspacePrevious with FocusPreviousPosition
            Action::FocusPreviousPosition => {
                self.niri.layout.focus_previous_position();
                self.maybe_warp_cursor_to_focus();
                self.niri.focus.layer_on_demand = None;
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced MoveWorkspaceDown with MoveRowDown
            Action::MoveRowDown => {
                self.niri.layout.move_row_down();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced MoveWorkspaceUp with MoveRowUp
            Action::MoveRowUp => {
                self.niri.layout.move_row_up();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced MoveWorkspaceToIndex with MoveRowToIndex
            Action::MoveRowToIndex(new_idx) => {
                let new_idx = new_idx.saturating_sub(1);
                self.niri.layout.move_row_to_index(None, new_idx);
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            // TEAM_012: Replaced SetWorkspaceName with SetRowName
            Action::SetRowName(name) => {
                self.niri.layout.set_row_name(name);
            }
            // TEAM_012: Replaced UnsetWorkspaceName with UnsetRowName
            Action::UnsetRowName => {
                self.niri.layout.unset_row_name();
            }
            Action::ConsumeWindowIntoColumn => {
                self.niri.layout.consume_into_column();
                // This does not cause immediate focus or window size change, so warping mouse to
                // focus won't do anything here.
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::ExpelWindowFromColumn => {
                self.niri.layout.expel_from_column();
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::SwapWindowRight => {
                self.niri
                    .layout
                    .swap_window_in_direction(ScrollDirection::Right);
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::SwapWindowLeft => {
                self.niri
                    .layout
                    .swap_window_in_direction(ScrollDirection::Left);
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::ToggleColumnTabbedDisplay => {
                self.niri.layout.toggle_column_tabbed_display();
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::SetColumnDisplay(display) => {
                self.niri.layout.set_column_display(display);
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::SwitchPresetColumnWidth => {
                self.niri.layout.toggle_width(true);
            }
            Action::SwitchPresetColumnWidthBack => {
                self.niri.layout.toggle_width(false);
            }
            Action::SwitchPresetWindowWidth => {
                self.niri.layout.toggle_window_width(None, true);
            }
            Action::SwitchPresetWindowWidthBack => {
                self.niri.layout.toggle_window_width(None, false);
            }
            Action::SwitchPresetWindowWidthById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.toggle_window_width(Some(&window), true);
                }
            }
            Action::SwitchPresetWindowWidthBackById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.toggle_window_width(Some(&window), false);
                }
            }
            Action::SwitchPresetWindowHeight => {
                self.niri.layout.toggle_window_height(None, true);
            }
            Action::SwitchPresetWindowHeightBack => {
                self.niri.layout.toggle_window_height(None, false);
            }
            Action::SwitchPresetWindowHeightById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.toggle_window_height(Some(&window), true);
                }
            }
            Action::SwitchPresetWindowHeightBackById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.toggle_window_height(Some(&window), false);
                }
            }
            Action::CenterColumn => {
                self.niri.layout.center_column();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::CenterWindow => {
                self.niri.layout.center_window(None);
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::CenterWindowById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let output = window.as_ref().and_then(|(mon, _)| mon.map(|m| m.output().clone()));
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.center_window(Some(&window));
                    if let Some(output) = output {
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            Action::CenterVisibleColumns => {
                self.niri.layout.center_visible_columns();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::MaximizeColumn => {
                self.niri.layout.toggle_full_width();
            }
            Action::MaximizeWindowToEdges => {
                let focus = self.niri.layout.focus().map(|m| m.window.clone());
                if let Some(window) = focus {
                    self.niri.layout.toggle_maximized(&window);
                    if let Some(output) = self.niri.layout.active_output().cloned() {
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            Action::MaximizeWindowToEdgesById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let output = window.as_ref().and_then(|(mon, _)| mon.map(|m| m.output().clone()));
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.toggle_maximized(&window);
                    if let Some(output) = output {
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            Action::FocusMonitorLeft => {
                if let Some(output) = self.niri.output_left() {
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                    self.niri.focus.layer_on_demand = None;
                }
            }
            Action::FocusMonitorRight => {
                if let Some(output) = self.niri.output_right() {
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                    self.niri.focus.layer_on_demand = None;
                }
            }
            Action::FocusMonitorDown => {
                if let Some(output) = self.niri.output_down() {
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                    self.niri.focus.layer_on_demand = None;
                }
            }
            Action::FocusMonitorUp => {
                if let Some(output) = self.niri.output_up() {
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                    self.niri.focus.layer_on_demand = None;
                }
            }
            Action::FocusMonitorPrevious => {
                if let Some(output) = self.niri.output_previous() {
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                    self.niri.focus.layer_on_demand = None;
                }
            }
            Action::FocusMonitorNext => {
                if let Some(output) = self.niri.output_next() {
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                    self.niri.focus.layer_on_demand = None;
                }
            }
            Action::FocusMonitor(output) => {
                if let Some(output) = self.niri.output_by_name_match(&output).cloned() {
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                    self.niri.focus.layer_on_demand = None;
                }
            }
            Action::MoveWindowToMonitorLeft => {
                if let Some(current_output) = self.niri.ui.screenshot.selection_output() {
                    if let Some(target_output) = self.niri.output_left_of(current_output) {
                        self.move_cursor_to_output(&target_output);
                        self.niri.ui.screenshot.move_to_output(target_output);
                    }
                } else if let Some(output) = self.niri.output_left() {
                    self.niri
                        .layout
                        .move_to_output(None, &output, None, ActivateWindow::Smart);
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            Action::MoveWindowToMonitorRight => {
                if let Some(current_output) = self.niri.ui.screenshot.selection_output() {
                    if let Some(target_output) = self.niri.output_right_of(current_output) {
                        self.move_cursor_to_output(&target_output);
                        self.niri.ui.screenshot.move_to_output(target_output);
                    }
                } else if let Some(output) = self.niri.output_right() {
                    self.niri
                        .layout
                        .move_to_output(None, &output, None, ActivateWindow::Smart);
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            Action::MoveWindowToMonitorDown => {
                if let Some(current_output) = self.niri.ui.screenshot.selection_output() {
                    if let Some(target_output) = self.niri.output_down_of(current_output) {
                        self.move_cursor_to_output(&target_output);
                        self.niri.ui.screenshot.move_to_output(target_output);
                    }
                } else if let Some(output) = self.niri.output_down() {
                    self.niri
                        .layout
                        .move_to_output(None, &output, None, ActivateWindow::Smart);
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            Action::MoveWindowToMonitorUp => {
                if let Some(current_output) = self.niri.ui.screenshot.selection_output() {
                    if let Some(target_output) = self.niri.output_up_of(current_output) {
                        self.move_cursor_to_output(&target_output);
                        self.niri.ui.screenshot.move_to_output(target_output);
                    }
                } else if let Some(output) = self.niri.output_up() {
                    self.niri
                        .layout
                        .move_to_output(None, &output, None, ActivateWindow::Smart);
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            Action::MoveWindowToMonitorPrevious => {
                if let Some(current_output) = self.niri.ui.screenshot.selection_output() {
                    if let Some(target_output) = self.niri.output_previous_of(current_output) {
                        self.move_cursor_to_output(&target_output);
                        self.niri.ui.screenshot.move_to_output(target_output);
                    }
                } else if let Some(output) = self.niri.output_previous() {
                    self.niri
                        .layout
                        .move_to_output(None, &output, None, ActivateWindow::Smart);
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            Action::MoveWindowToMonitorNext => {
                if let Some(current_output) = self.niri.ui.screenshot.selection_output() {
                    if let Some(target_output) = self.niri.output_next_of(current_output) {
                        self.move_cursor_to_output(&target_output);
                        self.niri.ui.screenshot.move_to_output(target_output);
                    }
                } else if let Some(output) = self.niri.output_next() {
                    self.niri
                        .layout
                        .move_to_output(None, &output, None, ActivateWindow::Smart);
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            Action::MoveWindowToMonitor(output) => {
                if let Some(output) = self.niri.output_by_name_match(&output).cloned() {
                    if self.niri.ui.screenshot.is_open() {
                        self.move_cursor_to_output(&output);
                        self.niri.ui.screenshot.move_to_output(output);
                    } else {
                        self.niri
                            .layout
                            .move_to_output(None, &output, None, ActivateWindow::Smart);
                        self.niri.layout.focus_output(&output);
                        if !self.maybe_warp_cursor_to_focus_centered() {
                            self.move_cursor_to_output(&output);
                        }
                    }
                }
            }
            Action::MoveWindowToMonitorById { id, output } => {
                if let Some(output) = self.niri.output_by_name_match(&output).cloned() {
                    let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                    let window = window.map(|(_, m)| m.window.clone());

                    if let Some(window) = window {
                        let target_was_active = self
                            .niri
                            .layout
                            .active_output()
                            .is_some_and(|active| output == *active);

                        self.niri.layout.move_to_output(
                            Some(&window),
                            &output,
                            None,
                            ActivateWindow::Smart,
                        );

                        // If the active output changed (window was moved and focused).
                        #[allow(clippy::collapsible_if)]
                        if !target_was_active && self.niri.layout.active_output() == Some(&output) {
                            if !self.maybe_warp_cursor_to_focus_centered() {
                                self.move_cursor_to_output(&output);
                            }
                        }
                    }
                }
            }
            Action::MoveColumnToMonitorLeft => {
                if let Some(current_output) = self.niri.ui.screenshot.selection_output() {
                    if let Some(target_output) = self.niri.output_left_of(current_output) {
                        self.move_cursor_to_output(&target_output);
                        self.niri.ui.screenshot.move_to_output(target_output);
                    }
                } else if let Some(output) = self.niri.output_left() {
                    self.niri.layout.move_column_to_output(&output, None, true);
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            Action::MoveColumnToMonitorRight => {
                if let Some(current_output) = self.niri.ui.screenshot.selection_output() {
                    if let Some(target_output) = self.niri.output_right_of(current_output) {
                        self.move_cursor_to_output(&target_output);
                        self.niri.ui.screenshot.move_to_output(target_output);
                    }
                } else if let Some(output) = self.niri.output_right() {
                    self.niri.layout.move_column_to_output(&output, None, true);
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            Action::MoveColumnToMonitorDown => {
                if let Some(current_output) = self.niri.ui.screenshot.selection_output() {
                    if let Some(target_output) = self.niri.output_down_of(current_output) {
                        self.move_cursor_to_output(&target_output);
                        self.niri.ui.screenshot.move_to_output(target_output);
                    }
                } else if let Some(output) = self.niri.output_down() {
                    self.niri.layout.move_column_to_output(&output, None, true);
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            Action::MoveColumnToMonitorUp => {
                if let Some(current_output) = self.niri.ui.screenshot.selection_output() {
                    if let Some(target_output) = self.niri.output_up_of(current_output) {
                        self.move_cursor_to_output(&target_output);
                        self.niri.ui.screenshot.move_to_output(target_output);
                    }
                } else if let Some(output) = self.niri.output_up() {
                    self.niri.layout.move_column_to_output(&output, None, true);
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            Action::MoveColumnToMonitorPrevious => {
                if let Some(current_output) = self.niri.ui.screenshot.selection_output() {
                    if let Some(target_output) = self.niri.output_previous_of(current_output) {
                        self.move_cursor_to_output(&target_output);
                        self.niri.ui.screenshot.move_to_output(target_output);
                    }
                } else if let Some(output) = self.niri.output_previous() {
                    self.niri.layout.move_column_to_output(&output, None, true);
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            Action::MoveColumnToMonitorNext => {
                if let Some(current_output) = self.niri.ui.screenshot.selection_output() {
                    if let Some(target_output) = self.niri.output_next_of(current_output) {
                        self.move_cursor_to_output(&target_output);
                        self.niri.ui.screenshot.move_to_output(target_output);
                    }
                } else if let Some(output) = self.niri.output_next() {
                    self.niri.layout.move_column_to_output(&output, None, true);
                    self.niri.layout.focus_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            Action::MoveColumnToMonitor(output) => {
                if let Some(output) = self.niri.output_by_name_match(&output).cloned() {
                    if self.niri.ui.screenshot.is_open() {
                        self.move_cursor_to_output(&output);
                        self.niri.ui.screenshot.move_to_output(output);
                    } else {
                        self.niri.layout.move_column_to_output(&output, None, true);
                        self.niri.layout.focus_output(&output);
                        if !self.maybe_warp_cursor_to_focus_centered() {
                            self.move_cursor_to_output(&output);
                        }
                    }
                }
            }
            Action::SetColumnWidth(change) => {
                if self.niri.ui.screenshot.is_open() {
                    self.niri.ui.screenshot.set_width(change);
                    if let Some(output) = self.niri.ui.screenshot.selection_output().cloned() {
                        self.niri.queue_redraw(&output);
                    }
                } else {
                    self.niri.layout.set_column_width(change);
                }
            }
            Action::SetWindowWidth(change) => {
                if self.niri.ui.screenshot.is_open() {
                    self.niri.ui.screenshot.set_width(change);
                    if let Some(output) = self.niri.ui.screenshot.selection_output().cloned() {
                        self.niri.queue_redraw(&output);
                    }
                } else {
                    self.niri.layout.set_window_width(None, change);
                }
            }
            Action::SetWindowWidthById { id, change } => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.set_window_width(Some(&window), change);
                }
            }
            Action::SetWindowHeight(change) => {
                if self.niri.ui.screenshot.is_open() {
                    self.niri.ui.screenshot.set_height(change);
                    if let Some(output) = self.niri.ui.screenshot.selection_output().cloned() {
                        self.niri.queue_redraw(&output);
                    }
                } else {
                    self.niri.layout.set_window_height(None, change);
                }
            }
            Action::SetWindowHeightById { id, change } => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.set_window_height(Some(&window), change);
                }
            }
            Action::ResetWindowHeight => {
                self.niri.layout.reset_window_height(None);
            }
            Action::ResetWindowHeightById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.reset_window_height(Some(&window));
                }
            }
            Action::ExpandColumnToAvailableWidth => {
                self.niri.layout.expand_column_to_available_width();
            }
            Action::ShowHotkeyOverlay => {
                if self.niri.ui.hotkey.show() {
                    self.niri.queue_redraw_all();

                    #[cfg(feature = "dbus")]
                    self.niri.a11y_announce_hotkey_overlay();
                }
            }
            // TEAM_012: Replaced MoveWorkspaceToMonitorLeft with MoveRowToMonitorLeft
            Action::MoveRowToMonitorLeft => {
                if let Some(output) = self.niri.output_left() {
                    self.niri.layout.move_workspace_to_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            // TEAM_012: Replaced MoveWorkspaceToMonitorRight with MoveRowToMonitorRight
            Action::MoveRowToMonitorRight => {
                if let Some(output) = self.niri.output_right() {
                    self.niri.layout.move_workspace_to_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            // TEAM_012: Replaced MoveWorkspaceToMonitorDown with MoveRowToMonitorDown
            Action::MoveRowToMonitorDown => {
                if let Some(output) = self.niri.output_down() {
                    self.niri.layout.move_workspace_to_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            // TEAM_012: Replaced MoveWorkspaceToMonitorUp with MoveRowToMonitorUp
            Action::MoveRowToMonitorUp => {
                if let Some(output) = self.niri.output_up() {
                    self.niri.layout.move_workspace_to_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            // TEAM_012: Replaced MoveWorkspaceToMonitorPrevious with MoveRowToMonitorPrevious
            Action::MoveRowToMonitorPrevious => {
                if let Some(output) = self.niri.output_previous() {
                    self.niri.layout.move_workspace_to_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            // TEAM_012: Replaced MoveWorkspaceToMonitorNext with MoveRowToMonitorNext
            Action::MoveRowToMonitorNext => {
                if let Some(output) = self.niri.output_next() {
                    self.niri.layout.move_workspace_to_output(&output);
                    if !self.maybe_warp_cursor_to_focus_centered() {
                        self.move_cursor_to_output(&output);
                    }
                }
            }
            // TEAM_012: Replaced MoveWorkspaceToMonitor with MoveRowToMonitor
            Action::MoveRowToMonitor(new_output) => {
                if let Some(new_output) = self.niri.output_by_name_match(&new_output).cloned() {
                    if self.niri.layout.move_workspace_to_output(&new_output)
                        && !self.maybe_warp_cursor_to_focus_centered()
                    {
                        self.move_cursor_to_output(&new_output);
                    }
                }
            }
            Action::ToggleWindowFloating => {
                self.niri.layout.toggle_window_floating(None);
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::ToggleWindowFloatingById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let output = window.as_ref().and_then(|(mon, _)| mon.map(|m| m.output().clone()));
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.toggle_window_floating(Some(&window));
                    if let Some(output) = output {
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            Action::MoveWindowToFloating => {
                self.niri.layout.set_window_floating(None, true);
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::MoveWindowToFloatingById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let output = window.as_ref().and_then(|(mon, _)| mon.map(|m| m.output().clone()));
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.set_window_floating(Some(&window), true);
                    if let Some(output) = output {
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            Action::MoveWindowToTiling => {
                self.niri.layout.set_window_floating(None, false);
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::MoveWindowToTilingById(id) => {
                let window = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                let output = window.as_ref().and_then(|(mon, _)| mon.map(|m| m.output().clone()));
                let window = window.map(|(_, m)| m.window.clone());
                if let Some(window) = window {
                    self.niri.layout.set_window_floating(Some(&window), false);
                    if let Some(output) = output {
                        self.niri.queue_redraw(&output);
                    }
                }
            }
            Action::FocusFloating => {
                self.niri.layout.focus_floating();
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::FocusTiling => {
                self.niri.layout.focus_tiling();
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::SwitchFocusBetweenFloatingAndTiling => {
                self.niri.layout.switch_focus_floating_tiling();
                self.maybe_warp_cursor_to_focus();
                if let Some(output) = self.niri.layout.active_output().cloned() {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::MoveFloatingWindowById { id, x, y } => {
                let (window, output) = if let Some(id) = id {
                    let found = self.niri.layout.windows().find(|(_, m)| m.id().get() == id);
                    let output = found.as_ref().and_then(|(mon, _)| mon.map(|m| m.output().clone()));
                    let window = found.map(|(_, m)| m.window.clone());
                    if window.is_none() {
                        return;
                    }
                    (window, output)
                } else {
                    (None, self.niri.layout.active_output().cloned())
                };

                self.niri
                    .layout
                    .move_floating_window(window.as_ref(), x, y, true);
                if let Some(output) = output {
                    self.niri.queue_redraw(&output);
                }
            }
            Action::ToggleWindowRuleOpacity => {
                let active_window = self
                    .niri
                    .layout
                    .active_row_mut()
                    .and_then(|ws| ws.active_window_mut());
                if let Some(window) = active_window {
                    if window.rules().opacity.is_some_and(|o| o != 1.) {
                        window.toggle_ignore_opacity_window_rule();
                        if let Some(output) = self.niri.layout.active_output().cloned() {
                            self.niri.queue_redraw(&output);
                        }
                    }
                }
            }
            Action::ToggleWindowRuleOpacityById(id) => {
                // Find the output first (immutable borrow)
                let output = self
                    .niri
                    .layout
                    .windows()
                    .find(|(_, m)| m.id().get() == id)
                    .and_then(|(mon, _)| mon.map(|m| m.output().clone()));
                // Now do the mutable operation
                let window = self
                    .niri
                    .layout
                    .workspaces_mut()
                    .find_map(|ws| ws.windows_mut().find(|w| w.id().get() == id));
                if let Some(window) = window {
                    if window.rules().opacity.is_some_and(|o| o != 1.) {
                        window.toggle_ignore_opacity_window_rule();
                        if let Some(output) = output {
                            self.niri.queue_redraw(&output);
                        }
                    }
                }
            }
            Action::SetDynamicCastWindow => {
                let id = self
                    .niri
                    .layout
                    .active_row()
                    .and_then(|ws| ws.active_window())
                    .map(|mapped| mapped.id().get());
                if let Some(id) = id {
                    self.set_dynamic_cast_target(CastTarget::Window { id });
                }
            }
            Action::SetDynamicCastWindowById(id) => {
                let layout = &self.niri.layout;
                if layout.windows().any(|(_, mapped)| mapped.id().get() == id) {
                    self.set_dynamic_cast_target(CastTarget::Window { id });
                }
            }
            Action::SetDynamicCastMonitor(output) => {
                let output = match output {
                    None => self.niri.layout.active_output(),
                    Some(name) => self.niri.output_by_name_match(&name),
                };
                if let Some(output) = output {
                    let output = output.downgrade();
                    self.set_dynamic_cast_target(CastTarget::Output(output));
                }
            }
            Action::ClearDynamicCastTarget => {
                self.set_dynamic_cast_target(CastTarget::Nothing);
            }
            Action::ToggleWindowUrgent(id) => {
                let window = self
                    .niri
                    .layout
                    .workspaces_mut()
                    .find_map(|ws| ws.windows_mut().find(|w| w.id().get() == id));
                if let Some(window) = window {
                    let urgent = window.is_urgent();
                    window.set_urgent(!urgent);
                }
                self.niri.queue_redraw_all();
            }
            Action::SetWindowUrgent(id) => {
                let window = self
                    .niri
                    .layout
                    .workspaces_mut()
                    .find_map(|ws| ws.windows_mut().find(|w| w.id().get() == id));
                if let Some(window) = window {
                    window.set_urgent(true);
                }
                self.niri.queue_redraw_all();
            }
            Action::UnsetWindowUrgent(id) => {
                let window = self
                    .niri
                    .layout
                    .workspaces_mut()
                    .find_map(|ws| ws.windows_mut().find(|w| w.id().get() == id));
                if let Some(window) = window {
                    window.set_urgent(false);
                }
                self.niri.queue_redraw_all();
            }
            Action::LoadConfigFile => {
                if let Some(watcher) = &self.niri.config_file_watcher {
                    watcher.load_config();
                }
            }
            Action::MruConfirm => {
                self.confirm_mru();
            }
            Action::MruCancel => {
                self.niri.cancel_mru();
            }
            Action::MruAdvance {
                direction,
                scope,
                filter,
            } => {
                if self.niri.ui.mru.is_open() {
                    self.niri.ui.mru.advance(direction, filter);
                    self.niri.queue_redraw_mru_output();
                } else if self.niri.config.borrow().recent_windows.on {
                    self.niri.mru_apply_keyboard_commit();

                    let config = self.niri.config.borrow();
                    let scope = scope.unwrap_or(self.niri.ui.mru.scope());

                    let mut wmru = WindowMru::new(&self.niri);
                    if !wmru.is_empty() {
                        wmru.set_scope(scope);
                        if let Some(filter) = filter {
                            wmru.set_filter(filter);
                        }

                        if let Some(output) = self.niri.layout.active_output().cloned() {
                            self.niri
                                .ui
                                .mru
                                .open(self.niri.clock.clone(), wmru, output.clone());

                            // Only select the *next* window if some window (which should be the
                            // first one) is already focused. If nothing is focused, keep the first
                            // window (which is logically the "previously selected" one).
                            let keep_first = direction == MruDirection::Forward
                                && self.niri.layout.focus().is_none();
                            if !keep_first {
                                self.niri.ui.mru.advance(direction, None);
                            }

                            drop(config);
                            self.niri.queue_redraw_all();
                        }
                    }
                }
            }
            Action::MruCloseCurrentWindow => {
                if self.niri.ui.mru.is_open() {
                    if let Some(id) = self.niri.ui.mru.current_window_id() {
                        if let Some(w) = self.niri.find_window_by_id(id) {
                            if let Some(tl) = w.toplevel() {
                                tl.send_close();
                            }
                        }
                    }
                }
            }
            Action::MruFirst => {
                if self.niri.ui.mru.is_open() {
                    self.niri.ui.mru.first();
                    self.niri.queue_redraw_mru_output();
                }
            }
            Action::MruLast => {
                if self.niri.ui.mru.is_open() {
                    self.niri.ui.mru.last();
                    self.niri.queue_redraw_mru_output();
                }
            }
            Action::MruSetScope(scope) => {
                if self.niri.ui.mru.is_open() {
                    self.niri.ui.mru.set_scope(scope);
                    self.niri.queue_redraw_mru_output();
                }
            }
            Action::MruCycleScope => {
                if self.niri.ui.mru.is_open() {
                    self.niri.ui.mru.cycle_scope();
                    self.niri.queue_redraw_mru_output();
                }
            }
        }
    }
}
