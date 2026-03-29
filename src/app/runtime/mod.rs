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
//! ```rust,no_run
//! # use envision::prelude::*;
//! # struct MyApp;
//! # #[derive(Default, Clone)]
//! # struct MyState;
//! # #[derive(Clone)]
//! # enum MyMsg {}
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
//! #[tokio::main]
//! async fn main() -> envision::Result<()> {
//!     let _final_state = Runtime::<MyApp, _>::new_terminal()?.run_terminal().await?;
//!     Ok(())
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
//! ```rust
//! # use envision::prelude::*;
//! # struct MyApp;
//! # #[derive(Default, Clone)]
//! # struct MyState;
//! # #[derive(Clone)]
//! # enum MyMsg {}
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
//! let mut vt = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
//! vt.send(Event::key(KeyCode::Char('j')));
//! vt.tick()?;
//! println!("{}", vt.display());
//! # Ok::<(), envision::EnvisionError>(())
//! ```
//!
//! Events are injected programmatically and the display can be inspected.

mod config;
mod terminal;
mod virtual_terminal;
pub use config::{RuntimeConfig, TerminalHook};

use std::io::Stdout;

use crate::error;
use std::pin::Pin;

use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

use super::command::{BoxedError, Command, CommandHandler};
use super::model::App;
use super::runtime_core::{ProcessEventResult, RuntimeCore};
use super::subscription::{BoxedSubscription, Subscription};
use crate::backend::CaptureBackend;
use crate::input::EventQueue;
use crate::overlay::{Overlay, OverlayStack};
use crate::theme::Theme;

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
}

/// Alias for a runtime using the crossterm terminal backend (production).
///
/// This is the type returned by [`Runtime::new_terminal()`] and
/// [`Runtime::terminal_with_config()`]. Use this alias when you need to
/// store or pass a terminal-mode runtime without spelling out the backend type.
///
/// # Example
///
/// ```rust,no_run
/// # use envision::prelude::*;
/// # use envision::TerminalRuntime;
/// # struct MyApp;
/// # #[derive(Default, Clone)]
/// # struct MyState;
/// # #[derive(Clone)]
/// # enum MyMsg {}
/// # impl App for MyApp {
/// #     type State = MyState;
/// #     type Message = MyMsg;
/// #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
/// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
/// #     fn view(state: &MyState, frame: &mut Frame) {}
/// # }
/// let runtime: TerminalRuntime<MyApp> = Runtime::new_terminal()?;
/// # Ok::<(), envision::EnvisionError>(())
/// ```
pub type TerminalRuntime<A> = Runtime<A, CrosstermBackend<Stdout>>;

/// Alias for a runtime using the virtual capture backend (testing/automation).
///
/// This is the type returned by [`Runtime::virtual_terminal()`] and
/// [`Runtime::virtual_terminal_with_config()`]. Use this alias when you need
/// to store or pass a virtual-terminal runtime without spelling out the backend type.
///
/// # Example
///
/// ```rust
/// # use envision::prelude::*;
/// # use envision::VirtualRuntime;
/// # struct MyApp;
/// # #[derive(Default, Clone)]
/// # struct MyState;
/// # #[derive(Clone)]
/// # enum MyMsg {}
/// # impl App for MyApp {
/// #     type State = MyState;
/// #     type Message = MyMsg;
/// #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
/// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
/// #     fn view(state: &MyState, frame: &mut Frame) {}
/// # }
/// let vt: VirtualRuntime<MyApp> = Runtime::virtual_terminal(80, 24)?;
/// # Ok::<(), envision::EnvisionError>(())
/// ```
pub type VirtualRuntime<A> = Runtime<A, CaptureBackend>;

// Common methods for all backends
// =============================================================================

impl<A: App, B: Backend> Runtime<A, B> {
    /// Creates a new runtime with the specified backend.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the ratatui `Terminal` with the
    /// provided backend fails.
    pub fn with_backend(backend: B) -> error::Result<Self> {
        Self::with_backend_and_config(backend, RuntimeConfig::default())
    }

    /// Creates a new runtime with a pre-built state, bypassing [`App::init()`].
    ///
    /// [`App::init()`] is **not called** — the provided `state` and `init_cmd`
    /// are used instead. This is the simplest way to construct a runtime with
    /// external state. Uses default configuration and executes the provided
    /// startup command.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the ratatui `Terminal` with the
    /// provided backend fails.
    pub fn with_backend_and_state(
        backend: B,
        state: A::State,
        init_cmd: Command<A::Message>,
    ) -> error::Result<Self> {
        Self::with_backend_state_and_config(backend, state, init_cmd, RuntimeConfig::default())
    }

    /// Creates a new runtime with backend and config.
    ///
    /// Calls [`App::init()`] to obtain the initial state and startup command.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the ratatui `Terminal` with the
    /// provided backend fails.
    pub fn with_backend_and_config(backend: B, config: RuntimeConfig) -> error::Result<Self> {
        let (state, init_cmd) = A::init();
        Self::with_backend_state_and_config(backend, state, init_cmd, config)
    }

    /// Creates a new runtime with a pre-built state and startup command.
    ///
    /// [`App::init()`] is **not called**. This allows callers to construct
    /// the initial state from external sources (CLI arguments, config files,
    /// databases, etc.) and pass it directly.
    ///
    /// The `init_cmd` parameter mirrors the command returned by `init()`.
    /// Pass [`Command::none()`] if no startup command is needed.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the ratatui `Terminal` with the
    /// provided backend fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState { count: i32 }
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState::default(), Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// // Build state from external data instead of App::init()
    /// let state = MyState { count: 42 };
    /// let backend = CaptureBackend::new(80, 24);
    /// let vt = Runtime::<MyApp, _>::with_backend_state_and_config(
    ///     backend, state, Command::none(), RuntimeConfig::default(),
    /// )?;
    /// assert_eq!(vt.state().count, 42);
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn with_backend_state_and_config(
        backend: B,
        state: A::State,
        init_cmd: Command<A::Message>,
        config: RuntimeConfig,
    ) -> error::Result<Self> {
        let terminal = Terminal::new(backend)?;

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
        };

        // Spawn any async commands from init
        runtime.spawn_pending_commands();

        Ok(runtime)
    }

    /// Returns a reference to the current state.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState { count: i32 }
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState::default(), Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let vt = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
    /// assert_eq!(vt.state().count, 0);
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn state(&self) -> &A::State {
        &self.core.state
    }

    /// Returns a mutable reference to the state.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState { count: i32 }
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState::default(), Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let mut vt = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
    /// vt.state_mut().count = 42;
    /// assert_eq!(vt.state().count, 42);
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
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
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState;
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// # let runtime = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
    /// let error_tx = runtime.error_sender();
    /// // error_tx can be sent to async tasks to report errors
    /// # Ok::<(), envision::EnvisionError>(())
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
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState;
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// # let mut runtime = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
    /// for error in runtime.take_errors() {
    ///     eprintln!("Async error: {}", error);
    /// }
    /// # Ok::<(), envision::EnvisionError>(())
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
    /// The subscription is converted to a stream and spawned as a tokio task
    /// that forwards messages through the runtime's message channel. Messages
    /// are automatically picked up by `tick()`, `process_pending()`, and `run()`.
    ///
    /// The subscription stops when it ends naturally, when the runtime's
    /// cancellation token is triggered, or when the message channel is closed.
    pub fn subscribe(&mut self, subscription: impl Subscription<A::Message>) {
        #[cfg(feature = "tracing")]
        tracing::info!("registering subscription");

        let stream = Box::new(subscription).into_stream(self.cancel_token.clone());
        Self::spawn_subscription(stream, self.message_tx.clone(), self.cancel_token.clone());
    }

    /// Adds multiple subscriptions to the runtime.
    pub fn subscribe_all(&mut self, subscriptions: Vec<BoxedSubscription<A::Message>>) {
        #[cfg(feature = "tracing")]
        tracing::info!(count = subscriptions.len(), "registering subscriptions");

        for sub in subscriptions {
            let stream = sub.into_stream(self.cancel_token.clone());
            Self::spawn_subscription(stream, self.message_tx.clone(), self.cancel_token.clone());
        }
    }

    /// Spawns a tokio task that reads from a subscription stream and forwards
    /// messages through the message channel.
    fn spawn_subscription(
        stream: Pin<Box<dyn tokio_stream::Stream<Item = A::Message> + Send>>,
        msg_tx: mpsc::Sender<A::Message>,
        cancel: CancellationToken,
    ) {
        tokio::spawn(async move {
            let mut stream = stream;
            loop {
                tokio::select! {
                    item = stream.next() => {
                        match item {
                            Some(msg) => {
                                if msg_tx.send(msg).await.is_err() {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    _ = cancel.cancelled() => break,
                }
            }
        });
    }

    /// Dispatches a message to update the state.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState { count: i32 }
    /// # #[derive(Clone)]
    /// # enum MyMsg { Increment }
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState::default(), Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> {
    /// #         match msg { MyMsg::Increment => state.count += 1 }
    /// #         Command::none()
    /// #     }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let mut vt = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
    /// vt.dispatch(MyMsg::Increment);
    /// assert_eq!(vt.state().count, 1);
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn dispatch(&mut self, msg: A::Message) {
        #[cfg(feature = "tracing")]
        let _span = tracing::debug_span!("dispatch").entered();

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

        // Process cancel token requests
        for cb in self.commands.take_cancel_token_requests() {
            let msg = cb(self.cancel_token.clone());
            self.dispatch(msg);
        }
    }

    /// Processes messages received from async tasks.
    fn process_async_messages(&mut self) {
        #[cfg(feature = "tracing")]
        let _span = tracing::debug_span!("process_async_messages").entered();

        while let Ok(msg) = self.message_rx.try_recv() {
            self.dispatch(msg);
        }
    }

    /// Renders the current state to the terminal.
    ///
    /// Renders the main app view first, then any active overlays on top.
    ///
    /// # Errors
    ///
    /// Returns an error if drawing to the terminal backend fails.
    pub fn render(&mut self) -> error::Result<()> {
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
    ///
    /// # Errors
    ///
    /// Returns an error if rendering to the terminal backend fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState;
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let mut vt = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
    /// vt.send(Event::key(KeyCode::Char('j')));
    /// vt.tick()?; // processes the 'j' event and re-renders
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn tick(&mut self) -> error::Result<()> {
        #[cfg(feature = "tracing")]
        let _span = tracing::debug_span!("tick").entered();

        // Process pending commands
        self.process_commands();

        // Process async messages
        self.process_async_messages();

        // Process events
        let mut messages_processed = 0;
        while self.process_event() && messages_processed < self.core.max_messages_per_tick {
            messages_processed += 1;
        }

        #[cfg(feature = "tracing")]
        if messages_processed > 0 {
            tracing::debug!(messages_processed, "tick: processed events");
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
        #[cfg(feature = "tracing")]
        tracing::info!("runtime quit requested");

        self.core.should_quit = true;
        self.cancel_token.cancel();
    }

    /// Runs the async event loop until the application quits.
    ///
    /// This is the main entry point for running a virtual terminal async loop.
    /// For terminal applications, use [`run_terminal`](Runtime::run_terminal) instead.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering to the terminal backend fails.
    pub async fn run(&mut self) -> error::Result<()> {
        #[cfg(feature = "tracing")]
        tracing::info!("starting runtime event loop");

        let mut tick_interval = tokio::time::interval(self.config.tick_rate);
        let mut render_interval = tokio::time::interval(self.config.frame_rate);

        // Initial render
        self.render()?;

        loop {
            tokio::select! {
                // Handle async messages from spawned tasks
                Some(msg) = self.message_rx.recv() => {
                    #[cfg(feature = "tracing")]
                    tracing::debug!("runtime received async message");

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
                    #[cfg(feature = "tracing")]
                    tracing::info!("runtime received cancellation");

                    self.core.should_quit = true;
                }
            }

            if self.core.should_quit {
                #[cfg(feature = "tracing")]
                tracing::info!("runtime shutting down");

                break;
            }
        }

        // Final render
        self.render()?;

        A::on_exit(&self.core.state);
        Ok(())
    }

    /// Runs for a specified number of ticks.
    ///
    /// # Errors
    ///
    /// Returns an error if any individual tick fails to render.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState;
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let mut vt = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
    /// vt.run_ticks(5)?;
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn run_ticks(&mut self, ticks: usize) -> error::Result<()> {
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
        #[cfg(feature = "tracing")]
        let _span = tracing::debug_span!("process_pending").entered();

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

#[cfg(test)]
mod tests;
