//! Streaming subsystem (screencast, screencopy, PipeWire).
//!
//! Handles screen capture streams for portals and protocols.

use std::collections::HashMap;

use smithay::desktop::Window;
use smithay::output::Output;

use crate::pw_utils::{Cast, PipeWire};
#[cfg(feature = "xdp-gnome-screencast")]
use crate::pw_utils::PwToNiri;
use crate::window::mapped::MappedId;

/// Streaming subsystem for screencast and screencopy.
///
/// Manages PipeWire streams, cast sessions, and the mapping between
/// windows and their screencast outputs.
pub struct StreamingSubsystem {
    /// Active screen cast sessions.
    casts: Vec<Cast>,
    
    /// PipeWire connection (if initialized).
    pipewire: Option<PipeWire>,
    
    /// Channel to send messages from PipeWire thread.
    #[cfg(feature = "xdp-gnome-screencast")]
    pw_sender: Option<calloop::channel::Sender<PwToNiri>>,
    
    /// Screencast output for each mapped window.
    #[cfg(feature = "xdp-gnome-screencast")]
    mapped_cast_output: HashMap<Window, Output>,
    
    /// Window ID for the "dynamic cast" special window for the xdp-gnome picker.
    #[cfg(feature = "xdp-gnome-screencast")]
    dynamic_cast_id: Option<MappedId>,
}

impl StreamingSubsystem {
    /// Creates a new streaming subsystem.
    pub fn new() -> Self {
        Self {
            casts: Vec::new(),
            pipewire: None,
            #[cfg(feature = "xdp-gnome-screencast")]
            pw_sender: None,
            #[cfg(feature = "xdp-gnome-screencast")]
            mapped_cast_output: HashMap::new(),
            #[cfg(feature = "xdp-gnome-screencast")]
            dynamic_cast_id: Some(MappedId::next()),
        }
    }
    
    /// Initializes the PipeWire channel and event loop source.
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn init_pipewire_channel(&mut self, event_loop: &calloop::LoopHandle<'static, super::super::State>) {
        let (pw_sender, from_pipewire) = calloop::channel::channel();
        event_loop
            .insert_source(from_pipewire, move |event, _, state| match event {
                calloop::channel::Event::Msg(msg) => state.on_pw_msg(msg),
                calloop::channel::Event::Closed => (),
            })
            .unwrap();
        self.pw_sender = Some(pw_sender);
    }
    
    // =========================================================================
    // Cast Management
    // =========================================================================
    
    /// Returns the active casts.
    pub fn casts(&self) -> &[Cast] {
        &self.casts
    }
    
    /// Returns mutable access to active casts.
    pub fn casts_mut(&mut self) -> &mut Vec<Cast> {
        &mut self.casts
    }
    
    /// Adds a new cast session.
    pub fn add_cast(&mut self, cast: Cast) {
        self.casts.push(cast);
    }
    
    /// Removes a cast session by stream ID.
    pub fn remove_cast(&mut self, stream_id: usize) -> Option<Cast> {
        self.casts
            .iter()
            .position(|c| c.stream_id == stream_id)
            .map(|idx| self.casts.remove(idx))
    }
    
    /// Finds a cast by stream ID.
    pub fn find_cast(&self, stream_id: usize) -> Option<&Cast> {
        self.casts.iter().find(|c| c.stream_id == stream_id)
    }
    
    /// Finds a cast by stream ID (mutable).
    pub fn find_cast_mut(&mut self, stream_id: usize) -> Option<&mut Cast> {
        self.casts.iter_mut().find(|c| c.stream_id == stream_id)
    }
    
    // =========================================================================
    // PipeWire
    // =========================================================================
    
    /// Returns the PipeWire connection.
    pub fn pipewire(&self) -> Option<&PipeWire> {
        self.pipewire.as_ref()
    }
    
    /// Returns mutable access to PipeWire.
    pub fn pipewire_mut(&mut self) -> Option<&mut PipeWire> {
        self.pipewire.as_mut()
    }
    
    /// Sets the PipeWire connection.
    pub fn set_pipewire(&mut self, pw: Option<PipeWire>) {
        self.pipewire = pw;
    }
    
    /// Returns whether PipeWire is initialized.
    pub fn has_pipewire(&self) -> bool {
        self.pipewire.is_some()
    }
    
    // =========================================================================
    // Mapped Cast Outputs (feature-gated)
    // =========================================================================
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn pw_sender(&self) -> Option<&calloop::channel::Sender<PwToNiri>> {
        self.pw_sender.as_ref()
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn set_pw_sender(&mut self, sender: calloop::channel::Sender<PwToNiri>) {
        self.pw_sender = Some(sender);
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn mapped_cast_output(&self, window: &Window) -> Option<&Output> {
        self.mapped_cast_output.get(window)
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn set_mapped_cast_output(&mut self, window: Window, output: Output) {
        self.mapped_cast_output.insert(window, output);
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn remove_mapped_cast_output(&mut self, window: &Window) {
        self.mapped_cast_output.remove(window);
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn mapped_cast_outputs(&self) -> &HashMap<Window, Output> {
        &self.mapped_cast_output
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn mapped_cast_outputs_mut(&mut self) -> &mut HashMap<Window, Output> {
        &mut self.mapped_cast_output
    }
    
    // =========================================================================
    // Dynamic Cast
    // =========================================================================
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn dynamic_cast_id(&self) -> Option<MappedId> {
        self.dynamic_cast_id
    }
    
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn set_dynamic_cast_id(&mut self, id: Option<MappedId>) {
        self.dynamic_cast_id = id;
    }
    
    // =========================================================================
    // Behavior Methods (encapsulated logic, not raw accessors)
    // =========================================================================
    
    /// Iterates over casts with a closure (avoids exposing iterator).
    pub fn for_each_cast(&self, mut f: impl FnMut(&Cast)) {
        for cast in &self.casts {
            f(cast);
        }
    }
    
    /// Iterates over casts mutably with a closure.
    pub fn for_each_cast_mut(&mut self, mut f: impl FnMut(&mut Cast)) {
        for cast in &mut self.casts {
            f(cast);
        }
    }
    
    /// Collects session IDs of all casts (for iteration without borrow).
    pub fn collect_session_ids(&self) -> Vec<usize> {
        self.casts.iter().map(|c| c.session_id).collect()
    }
    
    /// Collects stream IDs of all casts (for iteration without borrow).
    pub fn collect_stream_ids(&self) -> Vec<usize> {
        self.casts.iter().map(|c| c.stream_id).collect()
    }
    
    /// Takes the PipeWire connection out (for shutdown).
    pub fn take_pipewire(&mut self) -> Option<PipeWire> {
        self.pipewire.take()
    }
    
    /// Removes a cast by session ID and returns it.
    pub fn remove_cast_by_session(&mut self, session_id: usize) -> Option<Cast> {
        self.casts
            .iter()
            .position(|c| c.session_id == session_id)
            .map(|idx| self.casts.remove(idx))
    }
    
    /// Finds a cast by session ID.
    pub fn find_cast_by_session(&self, session_id: usize) -> Option<&Cast> {
        self.casts.iter().find(|c| c.session_id == session_id)
    }
    
    /// Finds a cast by session ID (mutable).
    pub fn find_cast_by_session_mut(&mut self, session_id: usize) -> Option<&mut Cast> {
        self.casts.iter_mut().find(|c| c.session_id == session_id)
    }
    
    /// Returns true if there are any active casts.
    pub fn has_casts(&self) -> bool {
        !self.casts.is_empty()
    }
    
    /// Returns the number of active casts.
    pub fn cast_count(&self) -> usize {
        self.casts.len()
    }
}

impl Default for StreamingSubsystem {
    fn default() -> Self {
        Self::new()
    }
}
