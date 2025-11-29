# Phase S1.5: StreamingSubsystem Logic Migration

> **Goal**: Move all screencast and screencopy logic INTO `StreamingSubsystem`

## Current State

StreamingSubsystem has:
- ✅ Private fields (casts, pipewire, pw_sender, mapped_cast_outputs)
- ✅ Basic accessor methods
- ❌ No rendering logic - still in `screencast.rs`
- ❌ No message handling - still in `mod.rs`
- ❌ Screencopy not integrated

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    StreamingSubsystem                        │
├─────────────────────────────────────────────────────────────┤
│  Screencast (PipeWire)          │  Screencopy (wlr)         │
│  ├── casts: Vec<Cast>           │  ├── sessions: Vec<...>   │
│  ├── pipewire: Option<PipeWire> │  └── (to be added)        │
│  └── pw_sender                  │                           │
├─────────────────────────────────────────────────────────────┤
│  Shared                                                      │
│  ├── mapped_cast_outputs: HashMap<Window, Output>           │
│  └── render methods                                          │
└─────────────────────────────────────────────────────────────┘
```

---

## Methods to Move

### From `mod.rs` (impl State)

#### 1. `on_pw_msg` (mod.rs:1327-1342) - ~15 LOC

**Current:**
```rust
impl State {
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn on_pw_msg(&mut self, msg: PwToNiri) { ... }
}
```

**Target:**
```rust
impl StreamingSubsystem {
    /// Handles PipeWire message. Returns actions for caller.
    pub fn handle_pw_message(&mut self, msg: PwToNiri) -> StreamingAction {
        match msg {
            PwToNiri::StopCast { session_id } => StreamingAction::StopCast(session_id),
            PwToNiri::Redraw { stream_id } => StreamingAction::Redraw(stream_id),
            PwToNiri::FatalError => StreamingAction::Shutdown,
        }
    }
}

pub enum StreamingAction {
    None,
    StopCast(usize),
    Redraw(usize),
    Shutdown,
}
```

---

#### 2. `redraw_cast` (mod.rs:1345-1410) - ~65 LOC

**Current:**
```rust
impl State {
    #[cfg(feature = "xdp-gnome-screencast")]
    fn redraw_cast(&mut self, stream_id: usize) { ... }
}
```

**Target:**
```rust
impl StreamingSubsystem {
    /// Renders a cast frame. Returns output to queue redraw if needed.
    pub fn render_cast<R: NiriRenderer>(
        &mut self,
        stream_id: usize,
        renderer: &mut R,
        layout: &Layout<Mapped>,
    ) -> Option<Output> {
        // Handles CastTarget::Output, CastTarget::Window, CastTarget::Nothing
    }
}
```

---

#### 3. `set_dynamic_cast_target` (mod.rs:1416-1472) - ~56 LOC

**Current:**
```rust
impl State {
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn set_dynamic_cast_target(&mut self, target: CastTarget) { ... }
}
```

**Target:**
```rust
impl StreamingSubsystem {
    /// Updates dynamic cast targets. Returns outputs needing redraw.
    pub fn set_dynamic_target(
        &mut self,
        target: CastTarget,
        layout: &Layout<Mapped>,
    ) -> Vec<Output> {
        // Updates targets, returns what needs redraw
    }
}
```

---

#### 4. `on_screen_cast_msg` (mod.rs:1474-1609) - ~135 LOC

**Current:**
```rust
impl State {
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn on_screen_cast_msg(&mut self, msg: ScreenCastToNiri) { ... }
}
```

**Target:**
```rust
impl StreamingSubsystem {
    /// Handles screencast D-Bus message. Returns actions.
    pub fn handle_screencast_message(
        &mut self,
        msg: ScreenCastToNiri,
        outputs: &OutputSubsystem,
        layout: &Layout<Mapped>,
    ) -> Vec<ScreencastAction> {
        // Returns list of actions for caller to execute
    }
}

pub enum ScreencastAction {
    StartCast { cast: Cast, output: Option<Output> },
    StopCast { session_id: usize },
    Redraw { output: Output },
    Error { message: String },
}
```

---

### From `screencast.rs`

#### 5. `render_for_screen_cast` (screencast.rs:59-135) - ~76 LOC

**Current:**
```rust
impl Niri {
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn render_for_screen_cast<R: NiriRenderer>(...) -> impl Iterator<...> { ... }
}
```

**Target:**
```rust
impl StreamingSubsystem {
    /// Renders elements for screencast output.
    pub fn render_output<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        output: &Output,
        scale: Scale<f64>,
        target: RenderTarget,
        elements: &[OutputRenderElements<R>],
    ) -> Vec<ScreencastRenderElement<R>> {
        // Filters and transforms elements for screencast
    }
}
```

---

#### 6. `render_windows_for_screen_cast` (screencast.rs:137-192) - ~55 LOC

**Current:**
```rust
impl Niri {
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn render_windows_for_screen_cast<R: NiriRenderer>(...) -> impl Iterator<...> { ... }
}
```

**Target:**
```rust
impl StreamingSubsystem {
    /// Renders window cast elements.
    pub fn render_window<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        window: &Mapped,
        scale: Scale<f64>,
    ) -> Vec<ScreencastRenderElement<R>> {
        // Renders single window for cast
    }
}
```

---

### From `screencopy.rs`

#### 7. Integrate screencopy sessions - ~100 LOC

**Current:** `screencopy.rs` has separate session tracking.

**Target:** Unified in StreamingSubsystem:
```rust
pub struct StreamingSubsystem {
    // Screencast (PipeWire)
    casts: Vec<Cast>,
    pipewire: Option<PipeWire>,
    pw_sender: Option<Sender<PwToNiri>>,
    
    // Screencopy (wlr-screencopy)
    screencopy_sessions: Vec<ScreencopySession>,
    
    // Shared
    mapped_cast_outputs: HashMap<Window, Output>,
}
```

---

## New StreamingSubsystem API

```rust
impl StreamingSubsystem {
    // === Lifecycle ===
    pub fn new() -> Self;
    pub fn init_pipewire(&mut self, event_loop: &LoopHandle);
    pub fn shutdown(&mut self);
    
    // === Cast Management ===
    pub fn add_cast(&mut self, cast: Cast);
    pub fn remove_cast(&mut self, session_id: usize) -> Option<Cast>;
    pub fn find_cast(&self, stream_id: usize) -> Option<&Cast>;
    pub fn casts(&self) -> &[Cast];
    
    // === Message Handling ===
    pub fn handle_pw_message(&mut self, msg: PwToNiri) -> StreamingAction;
    pub fn handle_screencast_message(&mut self, msg: ScreenCastToNiri, ctx: &StreamingContext) -> Vec<ScreencastAction>;
    
    // === Rendering ===
    pub fn render_cast<R>(&mut self, stream_id: usize, renderer: &mut R, ctx: &RenderContext) -> Option<Output>;
    pub fn render_output<R>(&self, ...) -> Vec<ScreencastRenderElement<R>>;
    pub fn render_window<R>(&self, ...) -> Vec<ScreencastRenderElement<R>>;
    
    // === Dynamic Targets ===
    pub fn set_dynamic_target(&mut self, target: CastTarget, ctx: &StreamingContext) -> Vec<Output>;
    
    // === Output Tracking ===
    pub fn mapped_cast_output(&self, window: &Window) -> Option<&Output>;
    pub fn set_mapped_cast_output(&mut self, window: Window, output: Output);
    pub fn remove_mapped_cast_output(&mut self, window: &Window);
    
    // === Screencopy (future) ===
    pub fn add_screencopy_session(&mut self, session: ScreencopySession);
    pub fn remove_screencopy_session(&mut self, id: usize);
}
```

---

## Context Structs

To avoid passing many parameters:

```rust
/// Context for streaming operations that need read access.
pub struct StreamingContext<'a> {
    pub outputs: &'a OutputSubsystem,
    pub layout: &'a Layout<Mapped>,
    pub clock: &'a Clock,
}

/// Context for rendering operations.
pub struct StreamingRenderContext<'a, R: NiriRenderer> {
    pub renderer: &'a mut R,
    pub output: &'a Output,
    pub scale: Scale<f64>,
    pub elements: &'a [OutputRenderElements<R>],
}
```

---

## Implementation Order

1. **Add action types** - StreamingAction, ScreencastAction
2. **Move handle_pw_message** - simple, returns action
3. **Move render_cast** - medium, needs renderer access
4. **Move set_dynamic_target** - medium
5. **Move handle_screencast_message** - complex, largest method
6. **Move render_output/render_window** - rendering logic
7. **Integrate screencopy** - optional, can be Phase S1.5b

---

## File Changes

### `src/niri/subsystems/streaming.rs`
- Add `StreamingAction`, `ScreencastAction` enums
- Add `StreamingContext` struct
- Add `handle_pw_message()` (~15 LOC)
- Add `render_cast()` (~65 LOC)
- Add `set_dynamic_target()` (~56 LOC)
- Add `handle_screencast_message()` (~135 LOC)
- Add `render_output()` (~76 LOC)
- Add `render_window()` (~55 LOC)

### `src/niri/mod.rs`
- Simplify `on_pw_msg()` to call subsystem + handle action
- Simplify `redraw_cast()` to call subsystem
- Simplify `set_dynamic_cast_target()` to call subsystem
- Simplify `on_screen_cast_msg()` to call subsystem + handle actions

### `src/niri/screencast.rs`
- Remove `render_for_screen_cast()` (moved)
- Remove `render_windows_for_screen_cast()` (moved)
- Keep or remove file depending on what remains

---

## Success Criteria

- [ ] All screencast logic in StreamingSubsystem
- [ ] Action-based API (subsystem returns actions, caller executes)
- [ ] No direct cast manipulation outside subsystem
- [ ] Screencopy integrated (optional)
- [ ] `cargo check` succeeds

---

## Estimated Effort

| Task | LOC | Time |
|------|-----|------|
| Action types | 30 | 15 min |
| handle_pw_message | 15 | 10 min |
| render_cast | 65 | 30 min |
| set_dynamic_target | 56 | 25 min |
| handle_screencast_message | 135 | 45 min |
| render_output | 76 | 30 min |
| render_window | 55 | 20 min |
| Update mod.rs callers | 50 | 30 min |
| Testing & fixes | - | 30 min |
| **Total** | **~480** | **~4 hrs** |
