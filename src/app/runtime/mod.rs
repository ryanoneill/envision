//! Runtime for executing TEA applications.
//!
//! The runtime manages the main loop, event handling, and rendering.
//! It provides full async support via tokio including async commands,
//! subscriptions, and graceful shutdown via cancellation tokens.
//!
//! # Two Runtime Modes
//!
//! Envision supports two distinct runtime modes:
//!
//! ## Terminal Mode
//!
//! For running applications in a real terminal:
//!
//! ```ignore
//! #[tokio::main]
//! async fn main() -> std::io::Result<()> {
//!     Runtime::<MyApp>::new_terminal()?.run_terminal().await
//! }
//! ```
//!
//! This sets up raw mode, alternate screen, and mouse capture, then runs
//! an async event loop that handles terminal events, async messages,
//! subscriptions, and rendering.
//!
//! ## Virtual Terminal Mode
//!
//! For programmatic control (AI agents, automation, testing):
//!
//! ```ignore
//! let mut vt = Runtime::<MyApp>::virtual_terminal(80, 24)?;
//! vt.send(Event::key('j'));
//! vt.tick()?;
//! println!("{}", vt.display());
//! ```
//!
//! Events are injected programmatically and the display can be inspected.

use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEventKind,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::command::{BoxedError, CommandHandler};
use super::model::App;
use super::runtime_core::{ProcessEventResult, RuntimeCore};
use super::subscription::{BoxedSubscription, Subscription};
use crate::backend::CaptureBackend;
use crate::input::{Event, EventQueue};
use crate::overlay::{Overlay, OverlayAction, OverlayStack};
use crate::theme::Theme;

/// Configuration for the runtime.
#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    /// How often to poll for events (default: 50ms)
    pub tick_rate: Duration,

    /// How often to render (default: 16ms for ~60fps)
    pub frame_rate: Duration,

    /// Maximum number of messages to process per tick (prevents infinite loops)
    pub max_messages_per_tick: usize,

    /// Whether to capture frame history
    pub capture_history: bool,

    /// Number of frames to keep in history
    pub history_capacity: usize,

    /// Capacity of the async message channel
    pub message_channel_capacity: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(50),
            frame_rate: Duration::from_millis(16),
            max_messages_per_tick: 100,
            capture_history: false,
            history_capacity: 10,
            message_channel_capacity: 256,
        }
    }
}

impl RuntimeConfig {
    /// Creates a new runtime config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the tick rate.
    pub fn tick_rate(mut self, rate: Duration) -> Self {
        self.tick_rate = rate;
        self
    }

    /// Sets the frame rate.
    pub fn frame_rate(mut self, rate: Duration) -> Self {
        self.frame_rate = rate;
        self
    }

    /// Enables frame history capture.
    pub fn with_history(mut self, capacity: usize) -> Self {
        self.capture_history = true;
        self.history_capacity = capacity;
        self
    }

    /// Sets the maximum messages per tick.
    pub fn max_messages(mut self, max: usize) -> Self {
        self.max_messages_per_tick = max;
        self
    }

    /// Sets the message channel capacity.
    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.message_channel_capacity = capacity;
        self
    }
}

/// The runtime that executes a TEA application.
///
/// This manages the event loop, state updates, and rendering using tokio.
/// It supports both real terminal mode and virtual terminal mode for testing.
pub struct Runtime<A: App, B: Backend> {
    /// Shared runtime state (state, terminal, events, overlays, theme)
    core: RuntimeCore<A, B>,

    /// Command handler
    commands: CommandHandler<A::Message>,

    /// Configuration
    config: RuntimeConfig,

    /// Sender for async messages
    message_tx: mpsc::Sender<A::Message>,

    /// Receiver for async messages
    message_rx: mpsc::Receiver<A::Message>,

    /// Sender for errors from async operations
    error_tx: mpsc::Sender<BoxedError>,

    /// Receiver for errors from async operations
    error_rx: mpsc::Receiver<BoxedError>,

    /// Cancellation token for graceful shutdown
    cancel_token: CancellationToken,

    /// Active subscriptions as streams
    subscriptions: Vec<std::pin::Pin<Box<dyn tokio_stream::Stream<Item = A::Message> + Send>>>,
}

// =============================================================================
// Terminal Mode - for real terminal applications
// =============================================================================

impl<A: App> Runtime<A, CrosstermBackend<Stdout>> {
    /// Creates a new runtime connected to a real terminal.
    ///
    /// This sets up the terminal for TUI operation:
    /// - Enables raw mode (input is not line-buffered)
    /// - Enters alternate screen (preserves the original terminal content)
    /// - Enables mouse capture
    ///
    /// Call `run_terminal()` to start the interactive event loop.
    ///
    /// # Example
    ///
    /// ```ignore
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     Runtime::<MyApp>::new_terminal()?.run_terminal().await
    /// }
    /// ```
    pub fn new_terminal() -> io::Result<Self> {
        Self::terminal_with_config(RuntimeConfig::default())
    }

    /// Creates a terminal runtime with custom configuration.
    pub fn terminal_with_config(config: RuntimeConfig) -> io::Result<Self> {
        // Set up terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        Self::with_backend_and_config(backend, config)
    }

    /// Runs the interactive event loop until the application quits.
    ///
    /// This is the main entry point for terminal applications. It uses
    /// `crossterm::event::EventStream` for non-blocking event reading,
    /// and `tokio::select!` to multiplex between terminal events,
    /// async messages, tick intervals, and render intervals.
    ///
    /// # Example
    ///
    /// ```ignore
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     Runtime::<MyApp>::new_terminal()?.run_terminal().await
    /// }
    /// ```
    pub async fn run_terminal(mut self) -> io::Result<()> {
        use futures_util::StreamExt;

        let mut tick_interval = tokio::time::interval(self.config.tick_rate);
        let mut render_interval = tokio::time::interval(self.config.frame_rate);
        let mut event_stream = crossterm::event::EventStream::new();

        // Initial render
        self.render()?;

        let result = loop {
            tokio::select! {
                // Handle terminal events from crossterm
                maybe_event = event_stream.next() => {
                    match maybe_event {
                        Some(Ok(event)) => {
                            if let Some(envision_event) = Self::convert_crossterm_event(&event) {
                                match self.core.overlay_stack.handle_event(&envision_event) {
                                    OverlayAction::Consumed => {}
                                    OverlayAction::Message(msg) => self.dispatch(msg),
                                    OverlayAction::Dismiss => {
                                        self.core.overlay_stack.pop();
                                    }
                                    OverlayAction::DismissWithMessage(msg) => {
                                        self.core.overlay_stack.pop();
                                        self.dispatch(msg);
                                    }
                                    OverlayAction::Propagate => {
                                        if let Some(msg) =
                                            A::handle_event_with_state(&self.core.state, &envision_event)
                                        {
                                            self.dispatch(msg);
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => {
                            break Err(e);
                        }
                        None => {
                            // Event stream ended
                            break Ok(());
                        }
                    }
                }

                // Handle async messages from spawned tasks
                Some(msg) = self.message_rx.recv() => {
                    self.dispatch(msg);
                }

                // Handle tick interval
                _ = tick_interval.tick() => {
                    // Process sync commands
                    self.process_commands();

                    // Process events from the queue
                    let mut messages_processed = 0;
                    while self.process_event() && messages_processed < self.core.max_messages_per_tick {
                        messages_processed += 1;
                    }

                    // Handle tick
                    if let Some(msg) = A::on_tick(&self.core.state) {
                        self.dispatch(msg);
                    }

                    // Check if we should quit
                    if A::should_quit(&self.core.state) {
                        self.core.should_quit = true;
                    }
                }

                // Handle render interval
                _ = render_interval.tick() => {
                    if let Err(e) = self.render() {
                        break Err(e);
                    }
                }

                // Handle cancellation
                _ = self.cancel_token.cancelled() => {
                    self.core.should_quit = true;
                }
            }

            if self.core.should_quit {
                break Ok(());
            }
        };

        // Cleanup terminal - always attempt cleanup even on error
        let cleanup_result = self.cleanup_terminal();

        // Call on_exit
        A::on_exit(&self.core.state);

        // Return the first error if any
        result.and(cleanup_result)
    }

    /// Converts a crossterm event to our Event type.
    fn convert_crossterm_event(event: &CrosstermEvent) -> Option<Event> {
        match event {
            CrosstermEvent::Key(key_event) => {
                // Only handle key press events, not release or repeat
                if key_event.kind == KeyEventKind::Press {
                    Some(Event::Key(*key_event))
                } else {
                    None
                }
            }
            CrosstermEvent::Mouse(mouse_event) => Some(Event::Mouse(*mouse_event)),
            CrosstermEvent::Resize(width, height) => Some(Event::Resize(*width, *height)),
            CrosstermEvent::FocusGained => Some(Event::FocusGained),
            CrosstermEvent::FocusLost => Some(Event::FocusLost),
            CrosstermEvent::Paste(text) => Some(Event::Paste(text.clone())),
        }
    }

    /// Cleans up terminal state.
    fn cleanup_terminal(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        self.core
            .terminal
            .backend_mut()
            .execute(LeaveAlternateScreen)?;
        self.core
            .terminal
            .backend_mut()
            .execute(DisableMouseCapture)?;
        self.core.terminal.show_cursor()?;
        Ok(())
    }
}

// =============================================================================
// Virtual Terminal Mode - for programmatic control (agents, testing)
// =============================================================================

impl<A: App> Runtime<A, CaptureBackend> {
    /// Creates a virtual terminal for programmatic control.
    ///
    /// A virtual terminal is not connected to a physical terminal. Instead:
    /// - Events are injected via `send()`
    /// - The application is advanced via `tick()`
    /// - The display can be inspected via `display()`
    ///
    /// This is useful for:
    /// - AI agents driving the application
    /// - Automation and scripting
    /// - Testing
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut vt = Runtime::<MyApp>::virtual_terminal(80, 24)?;
    /// vt.send(Event::key(KeyCode::Char('j')));
    /// vt.tick()?;
    /// assert!(vt.display().contains("expected text"));
    /// ```
    pub fn virtual_terminal(width: u16, height: u16) -> io::Result<Self> {
        let backend = CaptureBackend::new(width, height);
        Self::with_backend(backend)
    }

    /// Creates a virtual terminal with custom configuration.
    pub fn virtual_terminal_with_config(
        width: u16,
        height: u16,
        config: RuntimeConfig,
    ) -> io::Result<Self> {
        let backend = if config.capture_history {
            CaptureBackend::with_history(width, height, config.history_capacity)
        } else {
            CaptureBackend::new(width, height)
        };
        Self::with_backend_and_config(backend, config)
    }

    /// Sends an event to the virtual terminal.
    ///
    /// The event is queued and will be processed on the next `tick()`.
    pub fn send(&mut self, event: Event) {
        self.core.events.push(event);
    }

    /// Returns the current display content as plain text.
    ///
    /// This is what would be shown on a terminal screen.
    pub fn display(&self) -> String {
        self.core.terminal.backend().to_string()
    }

    /// Returns the display content with ANSI color codes.
    pub fn display_ansi(&self) -> String {
        self.core.terminal.backend().to_ansi()
    }
}

// =============================================================================
// Common methods for all backends
// =============================================================================

impl<A: App, B: Backend> Runtime<A, B> {
    /// Creates a new runtime with the specified backend.
    pub fn with_backend(backend: B) -> io::Result<Self> {
        Self::with_backend_and_config(backend, RuntimeConfig::default())
    }

    /// Creates a new runtime with backend and config.
    pub fn with_backend_and_config(backend: B, config: RuntimeConfig) -> io::Result<Self> {
        let terminal = Terminal::new(backend)?;
        let (state, init_cmd) = A::init();

        let (message_tx, message_rx) = mpsc::channel(config.message_channel_capacity);
        let (error_tx, error_rx) = mpsc::channel(config.message_channel_capacity);
        let cancel_token = CancellationToken::new();

        let mut commands = CommandHandler::new();
        commands.execute(init_cmd);

        let mut runtime = Self {
            core: RuntimeCore {
                state,
                terminal,
                events: EventQueue::new(),
                overlay_stack: OverlayStack::new(),
                theme: Theme::default(),
                should_quit: false,
                max_messages_per_tick: config.max_messages_per_tick,
            },
            commands,
            config,
            message_tx,
            message_rx,
            error_tx,
            error_rx,
            cancel_token,
            subscriptions: Vec::new(),
        };

        // Spawn any async commands from init
        runtime.spawn_pending_commands();

        Ok(runtime)
    }

    /// Returns a reference to the current state.
    pub fn state(&self) -> &A::State {
        &self.core.state
    }

    /// Returns a mutable reference to the state.
    pub fn state_mut(&mut self) -> &mut A::State {
        &mut self.core.state
    }

    /// Returns a reference to the inner ratatui Terminal.
    pub fn terminal(&self) -> &Terminal<B> {
        &self.core.terminal
    }

    /// Returns a mutable reference to the inner ratatui Terminal.
    pub fn terminal_mut(&mut self) -> &mut Terminal<B> {
        &mut self.core.terminal
    }

    /// Returns a reference to the backend.
    pub fn backend(&self) -> &B {
        self.core.terminal.backend()
    }

    /// Returns a mutable reference to the backend.
    pub fn backend_mut(&mut self) -> &mut B {
        self.core.terminal.backend_mut()
    }

    /// Returns a mutable reference to the event queue.
    pub fn events(&mut self) -> &mut EventQueue {
        &mut self.core.events
    }

    /// Returns a clone of the cancellation token.
    ///
    /// Tasks can use this token to detect shutdown and cancel gracefully.
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// Returns a sender that can be used to send messages to the runtime.
    ///
    /// This is useful for sending messages from external async tasks.
    pub fn message_sender(&self) -> mpsc::Sender<A::Message> {
        self.message_tx.clone()
    }

    /// Returns a sender that can be used to report errors from async operations.
    ///
    /// This is useful for sending errors from external async tasks.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let error_tx = runtime.error_sender();
    /// tokio::spawn(async move {
    ///     if let Err(e) = some_fallible_operation().await {
    ///         let _ = error_tx.send(Box::new(e)).await;
    ///     }
    /// });
    /// ```
    pub fn error_sender(&self) -> mpsc::Sender<BoxedError> {
        self.error_tx.clone()
    }

    /// Takes all collected errors from async operations.
    ///
    /// Returns all errors that have been received since the last call.
    /// After calling this method, the error queue is emptied.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for error in runtime.take_errors() {
    ///     eprintln!("Async error: {}", error);
    /// }
    /// ```
    pub fn take_errors(&mut self) -> Vec<BoxedError> {
        let mut errors = Vec::new();
        while let Ok(err) = self.error_rx.try_recv() {
            errors.push(err);
        }
        errors
    }

    /// Returns true if there are pending errors.
    ///
    /// This is a quick check to see if any errors have been reported
    /// without consuming them.
    pub fn has_errors(&self) -> bool {
        !self.error_rx.is_empty()
    }

    /// Adds a subscription to the runtime.
    ///
    /// The subscription will produce messages until it ends or the runtime shuts down.
    pub fn subscribe(&mut self, subscription: impl Subscription<A::Message>) {
        let stream = Box::new(subscription).into_stream(self.cancel_token.clone());
        self.subscriptions.push(stream);
    }

    /// Adds multiple subscriptions to the runtime.
    pub fn subscribe_all(&mut self, subscriptions: Vec<BoxedSubscription<A::Message>>) {
        for sub in subscriptions {
            let stream = sub.into_stream(self.cancel_token.clone());
            self.subscriptions.push(stream);
        }
    }

    /// Dispatches a message to update the state.
    pub fn dispatch(&mut self, msg: A::Message) {
        let cmd = A::update(&mut self.core.state, msg);
        self.commands.execute(cmd);

        if self.commands.should_quit() {
            self.core.should_quit = true;
        }

        self.spawn_pending_commands();
    }

    /// Dispatches multiple messages.
    pub fn dispatch_all(&mut self, messages: impl IntoIterator<Item = A::Message>) {
        for msg in messages {
            self.dispatch(msg);
        }
    }

    /// Spawns any pending async commands.
    fn spawn_pending_commands(&mut self) {
        self.commands.spawn_pending(
            self.message_tx.clone(),
            self.error_tx.clone(),
            self.cancel_token.clone(),
        );
    }

    /// Processes any pending commands.
    pub fn process_commands(&mut self) {
        let messages = self.commands.take_messages();
        for msg in messages {
            self.dispatch(msg);
        }

        // Process overlay commands
        for overlay in self.commands.take_overlay_pushes() {
            self.core.overlay_stack.push(overlay);
        }
        for _ in 0..self.commands.take_overlay_pops() {
            self.core.overlay_stack.pop();
        }
    }

    /// Processes messages received from async tasks.
    fn process_async_messages(&mut self) {
        while let Ok(msg) = self.message_rx.try_recv() {
            self.dispatch(msg);
        }
    }

    /// Renders the current state to the terminal.
    ///
    /// Renders the main app view first, then any active overlays on top.
    pub fn render(&mut self) -> io::Result<()> {
        self.core.render()
    }

    /// Processes the next event from the queue.
    ///
    /// If the overlay stack is active, events are routed through it first.
    /// Only if the overlay propagates the event will it reach the app's
    /// `handle_event_with_state`.
    ///
    /// Returns true if an event was processed.
    pub fn process_event(&mut self) -> bool {
        match self.core.process_event() {
            ProcessEventResult::NoEvent => false,
            ProcessEventResult::Consumed => true,
            ProcessEventResult::Dispatch(msg) => {
                self.dispatch(msg);
                true
            }
        }
    }

    /// Processes all pending events.
    pub fn process_all_events(&mut self) {
        while self.process_event() {}
    }

    /// Runs a single tick of the application.
    ///
    /// This is the primary method for advancing the application. It performs
    /// a full cycle: process commands, drain events, call on_tick, check quit,
    /// and render.
    ///
    /// For more granular control:
    /// - [`process_all_events`](Runtime::process_all_events) — Drain the event queue only
    /// - [`process_event`](Runtime::process_event) — Process exactly one event
    /// - [`run_ticks`](Runtime::run_ticks) — Convenience: run N full tick cycles
    pub fn tick(&mut self) -> io::Result<()> {
        // Process pending commands
        self.process_commands();

        // Process async messages
        self.process_async_messages();

        // Process events
        let mut messages_processed = 0;
        while self.process_event() && messages_processed < self.core.max_messages_per_tick {
            messages_processed += 1;
        }

        // Handle tick
        if let Some(msg) = A::on_tick(&self.core.state) {
            self.dispatch(msg);
        }

        // Check if we should quit
        if A::should_quit(&self.core.state) {
            self.core.should_quit = true;
        }

        // Render
        self.render()?;

        Ok(())
    }

    /// Returns true if the runtime should quit.
    pub fn should_quit(&self) -> bool {
        self.core.should_quit
    }

    /// Sets the quit flag and cancels all async operations.
    pub fn quit(&mut self) {
        self.core.should_quit = true;
        self.cancel_token.cancel();
    }

    /// Runs the async event loop until the application quits.
    ///
    /// This is the main entry point for running a virtual terminal async loop.
    /// For terminal applications, use [`run_terminal`](Runtime::run_terminal) instead.
    pub async fn run(&mut self) -> io::Result<()> {
        let mut tick_interval = tokio::time::interval(self.config.tick_rate);
        let mut render_interval = tokio::time::interval(self.config.frame_rate);

        // Initial render
        self.render()?;

        loop {
            tokio::select! {
                // Handle async messages from spawned tasks
                Some(msg) = self.message_rx.recv() => {
                    self.dispatch(msg);
                }

                // Handle tick interval
                _ = tick_interval.tick() => {
                    // Process sync commands
                    self.process_commands();

                    // Process events
                    let mut messages_processed = 0;
                    while self.process_event() && messages_processed < self.core.max_messages_per_tick {
                        messages_processed += 1;
                    }

                    // Handle tick
                    if let Some(msg) = A::on_tick(&self.core.state) {
                        self.dispatch(msg);
                    }

                    // Check if we should quit
                    if A::should_quit(&self.core.state) {
                        self.core.should_quit = true;
                    }
                }

                // Handle render interval
                _ = render_interval.tick() => {
                    self.render()?;
                }

                // Handle cancellation
                _ = self.cancel_token.cancelled() => {
                    self.core.should_quit = true;
                }
            }

            if self.core.should_quit {
                break;
            }
        }

        // Final render
        self.render()?;

        A::on_exit(&self.core.state);
        Ok(())
    }

    /// Runs for a specified number of ticks.
    pub fn run_ticks(&mut self, ticks: usize) -> io::Result<()> {
        for _ in 0..ticks {
            if self.core.should_quit {
                break;
            }
            self.tick()?;
        }
        Ok(())
    }

    /// Processes all pending async work (for testing with paused time).
    ///
    /// This is useful in tests with `tokio::time::pause()` to process
    /// all pending messages without running the full event loop.
    pub fn process_pending(&mut self) {
        // Process commands
        self.process_commands();

        // Process async messages
        self.process_async_messages();
    }

    /// Pushes an overlay onto the stack.
    pub fn push_overlay(&mut self, overlay: Box<dyn Overlay<A::Message>>) {
        self.core.push_overlay(overlay);
    }

    /// Pops the topmost overlay from the stack.
    pub fn pop_overlay(&mut self) -> Option<Box<dyn Overlay<A::Message>>> {
        self.core.pop_overlay()
    }

    /// Clears all overlays from the stack.
    pub fn clear_overlays(&mut self) {
        self.core.clear_overlays();
    }

    /// Returns true if there are active overlays.
    pub fn has_overlays(&self) -> bool {
        self.core.has_overlays()
    }

    /// Returns the number of overlays on the stack.
    pub fn overlay_count(&self) -> usize {
        self.core.overlay_count()
    }

    /// Sets the theme.
    pub fn set_theme(&mut self, theme: Theme) {
        self.core.theme = theme;
    }

    /// Returns a reference to the current theme.
    pub fn theme(&self) -> &Theme {
        &self.core.theme
    }
}

// Additional convenience methods for CaptureBackend (virtual terminal)
impl<A: App> Runtime<A, CaptureBackend> {
    /// Returns the cell at the given position, or `None` if out of bounds.
    ///
    /// Use this to assert on cell styling:
    /// ```ignore
    /// let cell = vt.cell_at(5, 3).unwrap();
    /// assert_eq!(cell.fg, SerializableColor::Green);
    /// ```
    pub fn cell_at(&self, x: u16, y: u16) -> Option<&crate::backend::EnhancedCell> {
        self.core.terminal.backend().cell(x, y)
    }

    /// Returns true if the display contains the given text.
    pub fn contains_text(&self, needle: &str) -> bool {
        self.core.terminal.backend().contains_text(needle)
    }

    /// Finds all positions of the given text in the display.
    pub fn find_text(&self, needle: &str) -> Vec<ratatui::layout::Position> {
        self.core.terminal.backend().find_text(needle)
    }
}

#[cfg(test)]
mod tests;
