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
}

impl<A: App> AsyncRuntime<A, CaptureBackend>
where
    A::Message: Send + 'static,
{
    /// Creates a new async runtime with a capture backend for headless operation.
    pub fn headless(width: u16, height: u16) -> io::Result<Self> {
        let backend = CaptureBackend::new(width, height);
        Self::with_backend(backend)
    }

    /// Creates a new async runtime with history tracking.
    pub fn headless_with_config(
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
    }

    /// Processes messages received from async tasks.
    fn process_async_messages(&mut self) {
        while let Ok(msg) = self.message_rx.try_recv() {
            self.dispatch(msg);
        }
    }

    /// Renders the current state to the terminal.
    pub fn render(&mut self) -> io::Result<()> {
        self.terminal.draw(|frame| {
            A::view(&self.state, frame);
        })?;
        Ok(())
    }

    /// Processes the next event from the queue.
    ///
    /// Returns true if an event was processed.
    pub fn process_event(&mut self) -> bool {
        if let Some(event) = self.events.pop() {
            if let Some(msg) = A::handle_event(&self.state, &event) {
                self.dispatch(msg);
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
}

// Convenience methods for CaptureBackend
impl<A: App> AsyncRuntime<A, CaptureBackend>
where
    A::Message: Send + 'static,
{
    /// Returns the captured output as a string.
    pub fn captured_output(&self) -> String {
        self.terminal.backend().to_string()
    }

    /// Returns the captured output with ANSI colors.
    pub fn captured_ansi(&self) -> String {
        self.terminal.backend().to_ansi()
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
mod tests {
    use super::*;
    use crate::app::Command;
    use ratatui::widgets::Paragraph;
    use std::time::Duration;

    struct CounterApp;

    #[derive(Clone, Default)]
    struct CounterState {
        count: i32,
        quit: bool,
    }

    #[derive(Clone, Debug)]
    enum CounterMsg {
        Increment,
        Decrement,
        IncrementBy(i32),
        Quit,
    }

    impl App for CounterApp {
        type State = CounterState;
        type Message = CounterMsg;

        fn init() -> (Self::State, Command<Self::Message>) {
            (CounterState::default(), Command::none())
        }

        fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
            match msg {
                CounterMsg::Increment => state.count += 1,
                CounterMsg::Decrement => state.count -= 1,
                CounterMsg::IncrementBy(n) => state.count += n,
                CounterMsg::Quit => state.quit = true,
            }
            Command::none()
        }

        fn view(state: &Self::State, frame: &mut ratatui::Frame) {
            let text = format!("Count: {}", state.count);
            frame.render_widget(Paragraph::new(text), frame.area());
        }

        fn should_quit(state: &Self::State) -> bool {
            state.quit
        }
    }

    #[test]
    fn test_async_runtime_headless() {
        let runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_async_runtime_dispatch() {
        let mut runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();

        runtime.dispatch(CounterMsg::Increment);
        assert_eq!(runtime.state().count, 1);

        runtime.dispatch(CounterMsg::Increment);
        runtime.dispatch(CounterMsg::Increment);
        assert_eq!(runtime.state().count, 3);

        runtime.dispatch(CounterMsg::Decrement);
        assert_eq!(runtime.state().count, 2);
    }

    #[test]
    fn test_async_runtime_render() {
        let mut runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Increment);
        runtime.dispatch(CounterMsg::Increment);
        runtime.render().unwrap();

        assert!(runtime.contains_text("Count: 2"));
    }

    #[test]
    fn test_async_runtime_quit() {
        let mut runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        assert!(!runtime.should_quit());

        runtime.dispatch(CounterMsg::Quit);
        runtime.tick().unwrap();

        assert!(runtime.should_quit());
    }

    #[test]
    fn test_async_runtime_config() {
        let config = AsyncRuntimeConfig::new()
            .tick_rate(Duration::from_millis(100))
            .frame_rate(Duration::from_millis(32))
            .with_history(5)
            .max_messages(50)
            .channel_capacity(512);

        assert_eq!(config.tick_rate, Duration::from_millis(100));
        assert_eq!(config.frame_rate, Duration::from_millis(32));
        assert!(config.capture_history);
        assert_eq!(config.history_capacity, 5);
        assert_eq!(config.max_messages_per_tick, 50);
        assert_eq!(config.message_channel_capacity, 512);
    }

    #[test]
    fn test_async_runtime_config_default() {
        let config = AsyncRuntimeConfig::default();
        assert_eq!(config.tick_rate, Duration::from_millis(50));
        assert_eq!(config.frame_rate, Duration::from_millis(16));
        assert_eq!(config.max_messages_per_tick, 100);
        assert!(!config.capture_history);
    }

    #[test]
    fn test_async_runtime_cancellation_token() {
        let runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        let token = runtime.cancellation_token();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_async_runtime_message_sender() {
        let runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        let _sender = runtime.message_sender();
        // Just verify we can get a sender
    }

    #[tokio::test]
    async fn test_async_runtime_async_command() {
        let mut runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();

        // Create an async command
        let cmd = Command::perform_async(async { Some(CounterMsg::IncrementBy(5)) });

        // Execute the command
        runtime.commands.execute(cmd);
        runtime.spawn_pending_commands();

        // Wait for the message
        tokio::time::sleep(Duration::from_millis(10)).await;
        runtime.process_pending();

        assert_eq!(runtime.state().count, 5);
    }

    #[tokio::test]
    async fn test_async_runtime_message_channel() {
        let mut runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        let sender = runtime.message_sender();

        // Send a message via the channel
        sender.send(CounterMsg::Increment).await.unwrap();
        sender.send(CounterMsg::Increment).await.unwrap();

        // Process the messages
        runtime.process_pending();
        assert_eq!(runtime.state().count, 2);
    }

    #[test]
    fn test_async_runtime_dispatch_all() {
        let mut runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();

        runtime.dispatch_all(vec![
            CounterMsg::Increment,
            CounterMsg::Increment,
            CounterMsg::Decrement,
        ]);

        assert_eq!(runtime.state().count, 1);
    }

    #[test]
    fn test_async_runtime_tick() {
        let mut runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Increment);
        runtime.tick().unwrap();

        assert!(runtime.contains_text("Count: 1"));
    }

    #[test]
    fn test_async_runtime_run_ticks() {
        let mut runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Increment);
        runtime.run_ticks(3).unwrap();

        assert!(runtime.contains_text("Count: 1"));
    }

    #[test]
    fn test_async_runtime_manual_quit() {
        let mut runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        assert!(!runtime.should_quit());
        assert!(!runtime.cancellation_token().is_cancelled());

        runtime.quit();
        assert!(runtime.should_quit());
        assert!(runtime.cancellation_token().is_cancelled());
    }

    #[test]
    fn test_async_runtime_error_sender() {
        let runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        let _error_tx = runtime.error_sender();
        // Just verify we can get an error sender
    }

    #[tokio::test]
    async fn test_async_runtime_take_errors() {
        let mut runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        let error_tx = runtime.error_sender();

        // No errors initially
        let errors = runtime.take_errors();
        assert!(errors.is_empty());

        // Send an error
        let err: BoxedError = Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test error",
        ));
        error_tx.send(err).await.unwrap();

        // Should have one error
        let errors = runtime.take_errors();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("test error"));

        // Errors are consumed
        let errors = runtime.take_errors();
        assert!(errors.is_empty());
    }

    #[tokio::test]
    async fn test_async_runtime_multiple_errors() {
        let mut runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        let error_tx = runtime.error_sender();

        // Send multiple errors
        for i in 0..3 {
            let err: BoxedError = Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("error {}", i),
            ));
            error_tx.send(err).await.unwrap();
        }

        // Should have all three errors
        let errors = runtime.take_errors();
        assert_eq!(errors.len(), 3);
    }

    #[tokio::test]
    async fn test_async_runtime_has_errors() {
        let mut runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        let error_tx = runtime.error_sender();

        // No errors initially
        assert!(!runtime.has_errors());

        // Send an error
        let err: BoxedError = Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test error",
        ));
        error_tx.send(err).await.unwrap();

        // Give the channel a moment to process
        tokio::time::sleep(Duration::from_millis(1)).await;

        // Should have errors now
        assert!(runtime.has_errors());

        // Consume the errors
        let _ = runtime.take_errors();

        // No more errors
        assert!(!runtime.has_errors());
    }

    #[tokio::test]
    async fn test_async_runtime_error_from_spawned_task() {
        let mut runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        let error_tx = runtime.error_sender();

        // Spawn a task that reports an error
        tokio::spawn(async move {
            let err: BoxedError = Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "resource not found",
            ));
            let _ = error_tx.send(err).await;
        });

        // Wait for the task to complete
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should have the error
        let errors = runtime.take_errors();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("resource not found"));
    }
}
