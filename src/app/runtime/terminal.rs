//! Terminal mode runtime implementation.
//!
//! Provides the `Runtime` methods for running applications in a real terminal
//! using crossterm for input and alternate screen management.

use std::io::{self, Stdout};

use crate::error;

use crossterm::ExecutableCommand;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;

use super::Runtime;
use super::config::RuntimeConfig;
use crate::app::model::App;

/// Restores the terminal to its normal state.
///
/// Disables raw mode, leaves the alternate screen, disables mouse capture,
/// and shows the cursor. Call this in panic handlers or cleanup code to
/// ensure the terminal is left in a usable state.
///
/// This is a standalone function that does not require a [`Runtime`]
/// instance — use it when you need terminal cleanup outside the normal
/// runtime lifecycle (e.g., in a `std::panic::set_hook` handler).
///
/// # Errors
///
/// Returns an error if any of the terminal cleanup operations fail.
/// Errors are generally non-fatal — the terminal may already be in the
/// correct state.
///
/// # Example
///
/// ```rust,no_run
/// // Install a panic hook that restores the terminal
/// let original_hook = std::panic::take_hook();
/// std::panic::set_hook(Box::new(move |info| {
///     let _ = envision::terminal::restore();
///     original_hook(info);
/// }));
/// ```
pub fn restore_terminal() -> crate::error::Result<()> {
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    io::stdout().execute(DisableMouseCapture)?;
    // Show cursor using crossterm directly (no Terminal instance needed)
    crossterm::execute!(io::stdout(), crossterm::cursor::Show)?;
    Ok(())
}
use crate::overlay::OverlayAction;

// =============================================================================
// Terminal Mode - for real terminal applications
// =============================================================================

impl<A: App> Runtime<A, CrosstermBackend<Stdout>> {
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
    /// ```rust,no_run
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState { count: u32 }
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState { count: 0 }, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// #[tokio::main]
    /// async fn main() -> envision::Result<()> {
    ///     let final_state = Runtime::<MyApp, _>::terminal_builder()?
    ///         .build()?
    ///         .run_terminal()
    ///         .await?;
    ///     println!("Final count: {}", final_state.count);
    ///     Ok(())
    /// }
    /// ```
    pub async fn run_terminal(mut self) -> error::Result<A::State> {
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
                            if let Some(envision_event) = crate::input::convert::from_crossterm_event(event) {
                                #[cfg(feature = "tracing")]
                                tracing::debug!(event = ?envision_event, "terminal received event");

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
                            break Err(e.into());
                        }
                        None => {
                            // Event stream ended
                            break Ok(());
                        }
                    }
                }

                // Handle async messages from spawned tasks
                Some(msg) = self.message_rx.recv() => {
                    #[cfg(feature = "tracing")]
                    tracing::debug!("terminal received async message");

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
                    #[cfg(feature = "tracing")]
                    tracing::info!("terminal received cancellation");

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
    /// ```rust,no_run
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState { count: u32 }
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState { count: 0 }, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// fn main() -> envision::Result<()> {
    ///     let final_state = Runtime::<MyApp, _>::terminal_builder()?
    ///         .build()?
    ///         .run_terminal_blocking()?;
    ///     println!("Final count: {}", final_state.count);
    ///     Ok(())
    /// }
    /// ```
    pub fn run_terminal_blocking(self) -> error::Result<A::State> {
        let rt = tokio::runtime::Runtime::new().map_err(io::Error::other)?;
        rt.block_on(self.run_terminal())
    }

    /// Sets up the terminal for TUI operation and returns the backend.
    ///
    /// This shared helper ensures both `terminal_with_config` and
    /// `terminal_with_state_and_config` perform identical setup:
    /// - Enables raw mode
    /// - Enters alternate screen
    /// - Enables mouse capture
    /// - Runs the `on_setup` hook if configured
    pub(super) fn setup_terminal(
        config: &RuntimeConfig,
    ) -> error::Result<CrosstermBackend<Stdout>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(EnableMouseCapture)?;

        // Run the on_setup hook if configured
        if let Some(ref hook) = config.on_setup {
            hook()?;
        }

        Ok(CrosstermBackend::new(stdout))
    }

    /// Cleans up terminal state.
    fn cleanup_terminal(&mut self) -> error::Result<()> {
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
