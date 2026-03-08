//! Terminal mode runtime implementation.
//!
//! Provides the `Runtime` methods for running applications in a real terminal
//! using crossterm for input and alternate screen management.

use std::io::{self, Stdout};

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEventKind,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;

use super::config::RuntimeConfig;
use super::Runtime;
use crate::app::command::Command;
use crate::app::model::App;
use crate::input::Event;
use crate::overlay::OverlayAction;

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
    /// # Errors
    ///
    /// Returns an error if enabling raw mode, entering alternate screen,
    /// enabling mouse capture, or creating the terminal fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // requires real terminal
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let _final_state = Runtime::<MyApp>::new_terminal()?.run_terminal().await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new_terminal() -> io::Result<Self> {
        Self::terminal_with_config(RuntimeConfig::default())
    }

    /// Creates a terminal runtime with custom configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if enabling raw mode, entering alternate screen,
    /// enabling mouse capture, or creating the terminal fails.
    pub fn terminal_with_config(config: RuntimeConfig) -> io::Result<Self> {
        // Set up terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(EnableMouseCapture)?;

        // Run the on_setup hook if configured
        if let Some(ref hook) = config.on_setup {
            hook()?;
        }

        let backend = CrosstermBackend::new(stdout);
        Self::with_backend_and_config(backend, config)
    }

    /// Creates a terminal runtime with a pre-built state, bypassing [`App::init()`].
    ///
    /// This allows constructing the initial state from external sources
    /// (CLI arguments, config files, databases, etc.) and passing it directly.
    ///
    /// # Errors
    ///
    /// Returns an error if enabling raw mode, entering alternate screen,
    /// enabling mouse capture, or creating the terminal fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // requires real terminal
    /// let state = MyState::from_config("config.toml")?;
    /// let runtime = Runtime::<MyApp>::new_terminal_with_state(state, Command::none())?;
    /// runtime.run_terminal().await?;
    /// ```
    pub fn new_terminal_with_state(
        state: A::State,
        init_cmd: Command<A::Message>,
    ) -> io::Result<Self> {
        Self::terminal_with_state_and_config(state, init_cmd, RuntimeConfig::default())
    }

    /// Creates a terminal runtime with a pre-built state and custom configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if enabling raw mode, entering alternate screen,
    /// enabling mouse capture, or creating the terminal fails.
    pub fn terminal_with_state_and_config(
        state: A::State,
        init_cmd: Command<A::Message>,
        config: RuntimeConfig,
    ) -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        Self::with_backend_state_and_config(backend, state, init_cmd, config)
    }

    /// Runs the interactive event loop until the application quits.
    ///
    /// This is the main entry point for terminal applications. It uses
    /// `crossterm::event::EventStream` for non-blocking event reading,
    /// and `tokio::select!` to multiplex between terminal events,
    /// async messages, tick intervals, and render intervals.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the crossterm event stream fails,
    /// if rendering to the terminal fails, or if terminal cleanup
    /// (disabling raw mode, leaving alternate screen, disabling mouse
    /// capture) fails on shutdown.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // requires real terminal
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let final_state = Runtime::<MyApp>::new_terminal()?.run_terminal().await?;
    ///     println!("Final count: {}", final_state.count);
    ///     Ok(())
    /// }
    /// ```
    pub async fn run_terminal(mut self) -> io::Result<A::State> {
        use futures_util::StreamExt;

        #[cfg(feature = "tracing")]
        tracing::info!("starting terminal runtime loop");

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
                                    OverlayAction::KeepAndMessage(msg) => self.dispatch(msg),
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

        // Return the first error if any, otherwise return the final state
        result.and(cleanup_result)?;
        Ok(self.core.state)
    }

    /// Runs the interactive terminal event loop, blocking the current thread.
    ///
    /// This is a convenience wrapper around [`run_terminal`](Runtime::run_terminal) for
    /// applications that don't want to set up their own tokio runtime. It creates
    /// a multi-threaded tokio runtime internally and blocks on the async event loop.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the tokio runtime fails, or if
    /// [`run_terminal`](Runtime::run_terminal) returns an error.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // requires real terminal
    /// fn main() -> std::io::Result<()> {
    ///     let final_state = Runtime::<MyApp>::new_terminal()?.run_terminal_blocking()?;
    ///     println!("Final count: {}", final_state.count);
    ///     Ok(())
    /// }
    /// ```
    pub fn run_terminal_blocking(self) -> io::Result<A::State> {
        let rt = tokio::runtime::Runtime::new().map_err(io::Error::other)?;
        rt.block_on(self.run_terminal())
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
        // Run the on_teardown hook if configured
        if let Some(ref hook) = self.config.on_teardown {
            hook()?;
        }

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
