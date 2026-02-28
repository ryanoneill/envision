//! Async runtime for executing TEA applications with tokio.
//!
//! This runtime provides full async support including:
//! - Async commands that spawn futures
//! - Subscriptions for long-running async streams
//! - Graceful shutdown via cancellation tokens
//! - Error collection from async operations

use std::io;
use std::time::Duration;

use ratatui::backend::Backend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// A boxed error that is Send + Sync.
///
/// This type is used for collecting errors from async operations that can
/// fail. It allows any error type that implements the standard error traits.
pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

use super::async_command::AsyncCommandHandler;
use super::model::App;
use super::subscription::{BoxedSubscription, Subscription};
use crate::backend::CaptureBackend;
use crate::input::EventQueue;
use crate::overlay::{Overlay, OverlayAction, OverlayStack};
use crate::theme::Theme;

/// Configuration for the async runtime.
#[derive(Clone, Debug)]
pub struct AsyncRuntimeConfig {
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

impl Default for AsyncRuntimeConfig {
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

impl AsyncRuntimeConfig {
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

/// The async runtime that executes a TEA application.
///
/// This manages the async event loop, state updates, and rendering using tokio.
pub struct AsyncRuntime<A: App, B: Backend>
where
    A::Message: Send + 'static,
{
    /// The application state
    state: A::State,

    /// The terminal
    terminal: Terminal<B>,

    /// Event queue for input
    events: EventQueue,

    /// Async command handler
    commands: AsyncCommandHandler<A::Message>,

    /// Configuration
    config: AsyncRuntimeConfig,

    /// Whether the runtime should quit
    should_quit: bool,

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

    /// The overlay stack for modal UI layers
    overlay_stack: OverlayStack<A::Message>,

    /// The current theme
    theme: Theme,
}

// =============================================================================
// Virtual Terminal Mode - for programmatic control (agents, testing)
// =============================================================================

impl<A: App> AsyncRuntime<A, CaptureBackend>
where
    A::Message: Send + 'static,
{
    /// Creates a virtual terminal for programmatic async control.
    ///
    /// A virtual terminal is not connected to a physical terminal. Instead:
    /// - Events are injected via `send()`
    /// - The application is advanced via `tick()` or run async with `run()`
    /// - The display can be inspected via `display()`
    ///
    /// This is useful for:
    /// - AI agents driving the application
    /// - Automation and scripting
    /// - Testing async applications
    pub fn virtual_terminal(width: u16, height: u16) -> io::Result<Self> {
        let backend = CaptureBackend::new(width, height);
        Self::with_backend(backend)
    }

    /// Creates a virtual terminal with custom configuration.
    pub fn virtual_terminal_with_config(
        width: u16,
        height: u16,
        config: AsyncRuntimeConfig,
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
    /// The event is queued and will be processed on the next `tick()` or `run()` cycle.
    pub fn send(&mut self, event: crate::input::Event) {
        self.events.push(event);
    }

    /// Returns the current display content as plain text.
    pub fn display(&self) -> String {
        self.terminal.backend().to_string()
    }

    /// Returns the display content with ANSI color codes.
    pub fn display_ansi(&self) -> String {
        self.terminal.backend().to_ansi()
    }
}

impl<A: App, B: Backend> AsyncRuntime<A, B>
where
    A::Message: Send + 'static,
{
    /// Creates a new async runtime with the specified backend.
    pub fn with_backend(backend: B) -> io::Result<Self> {
        Self::with_backend_and_config(backend, AsyncRuntimeConfig::default())
    }

    /// Creates a new async runtime with backend and config.
    pub fn with_backend_and_config(backend: B, config: AsyncRuntimeConfig) -> io::Result<Self> {
        let terminal = Terminal::new(backend)?;
        let (state, init_cmd) = A::init();

        let (message_tx, message_rx) = mpsc::channel(config.message_channel_capacity);
        let (error_tx, error_rx) = mpsc::channel(config.message_channel_capacity);
        let cancel_token = CancellationToken::new();

        let mut commands = AsyncCommandHandler::new();
        commands.execute(init_cmd);

        let mut runtime = Self {
            state,
            terminal,
            events: EventQueue::new(),
            commands,
            config,
            should_quit: false,
            message_tx,
            message_rx,
            error_tx,
            error_rx,
            cancel_token,
            subscriptions: Vec::new(),
            overlay_stack: OverlayStack::new(),
            theme: Theme::default(),
        };

        // Spawn any async commands from init
        runtime.spawn_pending_commands();

        Ok(runtime)
    }

    /// Returns a reference to the current state.
    pub fn state(&self) -> &A::State {
        &self.state
    }

    /// Returns a mutable reference to the state.
    pub fn state_mut(&mut self) -> &mut A::State {
        &mut self.state
    }

    /// Returns a reference to the terminal.
    pub fn terminal(&self) -> &Terminal<B> {
        &self.terminal
    }

    /// Returns a mutable reference to the terminal.
    pub fn terminal_mut(&mut self) -> &mut Terminal<B> {
        &mut self.terminal
    }

    /// Returns a reference to the backend.
    pub fn backend(&self) -> &B {
        self.terminal.backend()
    }

    /// Returns a mutable reference to the backend.
    pub fn backend_mut(&mut self) -> &mut B {
        self.terminal.backend_mut()
    }

    /// Returns a mutable reference to the event queue.
    pub fn events(&mut self) -> &mut EventQueue {
        &mut self.events
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
        // Note: We can't check is_empty() directly, but we can check the sender count
        // We use a workaround by checking if the receiver is closed
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
        let cmd = A::update(&mut self.state, msg);
        self.commands.execute(cmd);

        if self.commands.should_quit() {
            self.should_quit = true;
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

    /// Processes any pending sync commands and returns resulting messages.
    fn process_sync_commands(&mut self) {
        let messages = self.commands.take_messages();
        for msg in messages {
            self.dispatch(msg);
        }

        // Process overlay commands
        for overlay in self.commands.take_overlay_pushes() {
            self.overlay_stack.push(overlay);
        }
        for _ in 0..self.commands.take_overlay_pops() {
            self.overlay_stack.pop();
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
        let theme = &self.theme;
        let overlay_stack = &self.overlay_stack;
        self.terminal.draw(|frame| {
            A::view(&self.state, frame);
            overlay_stack.render(frame, frame.area(), theme);
        })?;
        Ok(())
    }

    /// Processes the next event from the queue.
    ///
    /// If the overlay stack is active, events are routed through it first.
    /// Only if the overlay propagates the event will it reach the app's
    /// `handle_event_with_state`.
    ///
    /// Returns true if an event was processed.
    pub fn process_event(&mut self) -> bool {
        if let Some(event) = self.events.pop() {
            match self.overlay_stack.handle_event(&event) {
                OverlayAction::Consumed => {}
                OverlayAction::Message(msg) => self.dispatch(msg),
                OverlayAction::Dismiss => {
                    self.overlay_stack.pop();
                }
                OverlayAction::DismissWithMessage(msg) => {
                    self.overlay_stack.pop();
                    self.dispatch(msg);
                }
                OverlayAction::Propagate => {
                    if let Some(msg) = A::handle_event_with_state(&self.state, &event) {
                        self.dispatch(msg);
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Processes all pending events.
    pub fn process_all_events(&mut self) {
        while self.process_event() {}
    }

    /// Runs a single tick of the application.
    ///
    /// This processes events, updates state, and renders.
    pub fn tick(&mut self) -> io::Result<()> {
        // Process pending sync commands
        self.process_sync_commands();

        // Process async messages
        self.process_async_messages();

        // Process events
        let mut messages_processed = 0;
        while self.process_event() && messages_processed < self.config.max_messages_per_tick {
            messages_processed += 1;
        }

        // Handle tick
        if let Some(msg) = A::on_tick(&self.state) {
            self.dispatch(msg);
        }

        // Check if we should quit
        if A::should_quit(&self.state) {
            self.should_quit = true;
        }

        // Render
        self.render()?;

        Ok(())
    }

    /// Returns true if the runtime should quit.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Sets the quit flag and cancels all async operations.
    pub fn quit(&mut self) {
        self.should_quit = true;
        self.cancel_token.cancel();
    }

    /// Runs the async event loop until the application quits.
    ///
    /// This is the main entry point for running an async application.
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
                    self.process_sync_commands();

                    // Process events
                    let mut messages_processed = 0;
                    while self.process_event() && messages_processed < self.config.max_messages_per_tick {
                        messages_processed += 1;
                    }

                    // Handle tick
                    if let Some(msg) = A::on_tick(&self.state) {
                        self.dispatch(msg);
                    }

                    // Check if we should quit
                    if A::should_quit(&self.state) {
                        self.should_quit = true;
                    }
                }

                // Handle render interval
                _ = render_interval.tick() => {
                    self.render()?;
                }

                // Handle cancellation
                _ = self.cancel_token.cancelled() => {
                    self.should_quit = true;
                }
            }

            if self.should_quit {
                break;
            }
        }

        // Final render
        self.render()?;

        A::on_exit(&self.state);
        Ok(())
    }

    /// Runs for a specified number of ticks (for testing).
    pub fn run_ticks(&mut self, ticks: usize) -> io::Result<()> {
        for _ in 0..ticks {
            if self.should_quit {
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
        // Process sync commands
        self.process_sync_commands();

        // Process async messages
        self.process_async_messages();
    }

    /// Pushes an overlay onto the stack.
    pub fn push_overlay(&mut self, overlay: Box<dyn Overlay<A::Message>>) {
        self.overlay_stack.push(overlay);
    }

    /// Pops the topmost overlay from the stack.
    pub fn pop_overlay(&mut self) -> Option<Box<dyn Overlay<A::Message>>> {
        self.overlay_stack.pop()
    }

    /// Clears all overlays from the stack.
    pub fn clear_overlays(&mut self) {
        self.overlay_stack.clear();
    }

    /// Returns true if there are active overlays.
    pub fn has_overlays(&self) -> bool {
        self.overlay_stack.is_active()
    }

    /// Returns the number of overlays on the stack.
    pub fn overlay_count(&self) -> usize {
        self.overlay_stack.len()
    }

    /// Sets the theme.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Returns a reference to the current theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }
}

// Convenience methods for CaptureBackend
impl<A: App> AsyncRuntime<A, CaptureBackend>
where
    A::Message: Send + 'static,
{
    /// Returns the cell at the given position, or `None` if out of bounds.
    ///
    /// Use this to assert on cell styling:
    /// ```ignore
    /// let cell = runtime.cell_at(5, 3).unwrap();
    /// assert_eq!(cell.fg, SerializableColor::Green);
    /// ```
    pub fn cell_at(&self, x: u16, y: u16) -> Option<&crate::backend::EnhancedCell> {
        self.terminal.backend().cell(x, y)
    }

    /// Returns true if the captured output contains the given text.
    pub fn contains_text(&self, needle: &str) -> bool {
        self.terminal.backend().contains_text(needle)
    }

    /// Finds all positions of the given text.
    pub fn find_text(&self, needle: &str) -> Vec<ratatui::layout::Position> {
        self.terminal.backend().find_text(needle)
    }
}

#[cfg(test)]
mod tests;
