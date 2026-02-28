//! Runtime for executing TEA applications.
//!
//! The runtime manages the main loop, event handling, and rendering.
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
//! // Simple usage
//! Runtime::<MyApp>::new_terminal()?.run()?;
//! ```
//!
//! This sets up raw mode, alternate screen, and mouse capture, then runs
//! a blocking event loop that polls for real terminal events.
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

use super::command::CommandHandler;
use super::model::App;
use crate::backend::CaptureBackend;
use crate::input::{Event, EventQueue};
use crate::overlay::{Overlay, OverlayAction, OverlayStack};
use crate::theme::Theme;

/// Configuration for the runtime.
#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    /// How often to poll for events (default: 50ms)
    pub tick_rate: Duration,

    /// Maximum number of messages to process per tick (prevents infinite loops)
    pub max_messages_per_tick: usize,

    /// Whether to capture frame history
    pub capture_history: bool,

    /// Number of frames to keep in history
    pub history_capacity: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(50),
            max_messages_per_tick: 100,
            capture_history: false,
            history_capacity: 10,
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
}

/// The runtime that executes a TEA application.
///
/// This manages the main loop, event handling, state updates, and rendering.
pub struct Runtime<A: App, B: Backend> {
    /// The application state
    state: A::State,

    /// The terminal
    terminal: Terminal<B>,

    /// Event queue for input
    events: EventQueue,

    /// Command handler
    commands: CommandHandler<A::Message>,

    /// Configuration
    config: RuntimeConfig,

    /// The overlay stack for modal UI layers
    overlay_stack: OverlayStack<A::Message>,

    /// The current theme
    theme: Theme,

    /// Whether the runtime should quit
    should_quit: bool,
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
    /// Call `run()` to start the interactive event loop.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn main() -> std::io::Result<()> {
    ///     Runtime::<MyApp>::new_terminal()?.run()
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
    /// This is the main entry point for terminal applications. It:
    /// - Polls for terminal events (keyboard, mouse, resize)
    /// - Dispatches events through the App's `handle_event`
    /// - Calls `on_tick` at the configured interval
    /// - Renders the UI
    /// - Cleans up the terminal on exit
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn main() -> std::io::Result<()> {
    ///     Runtime::<MyApp>::new_terminal()?.run()
    /// }
    /// ```
    pub fn run(mut self) -> io::Result<()> {
        // Initial render
        self.render()?;

        let result = self.run_event_loop();

        // Cleanup terminal - always attempt cleanup even on error
        let cleanup_result = self.cleanup_terminal();

        // Call on_exit
        A::on_exit(&self.state);

        // Return the first error if any
        result.and(cleanup_result)
    }

    /// The internal event loop.
    fn run_event_loop(&mut self) -> io::Result<()> {
        loop {
            // Check if we should quit
            if self.should_quit || A::should_quit(&self.state) {
                break;
            }

            // Poll for events with timeout
            if crossterm::event::poll(self.config.tick_rate)? {
                let event = crossterm::event::read()?;

                // Convert crossterm event to our Event type and dispatch
                if let Some(envision_event) = Self::convert_crossterm_event(&event) {
                    match self.overlay_stack.handle_event(&envision_event) {
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
                            if let Some(msg) =
                                A::handle_event_with_state(&self.state, &envision_event)
                            {
                                self.dispatch(msg);
                            }
                        }
                    }
                }
            }

            // Process any pending commands
            self.process_commands();

            // Handle tick
            if let Some(msg) = A::on_tick(&self.state) {
                self.dispatch(msg);
            }

            // Check quit flag again after processing
            if self.should_quit || A::should_quit(&self.state) {
                break;
            }

            // Render
            self.render()?;
        }

        Ok(())
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
        self.terminal.backend_mut().execute(LeaveAlternateScreen)?;
        self.terminal.backend_mut().execute(DisableMouseCapture)?;
        self.terminal.show_cursor()?;
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
        self.events.push(event);
    }

    /// Returns the current display content as plain text.
    ///
    /// This is what would be shown on a terminal screen.
    pub fn display(&self) -> String {
        self.terminal.backend().to_string()
    }

    /// Returns the display content with ANSI color codes.
    pub fn display_ansi(&self) -> String {
        self.terminal.backend().to_ansi()
    }
}

impl<A: App, B: Backend> Runtime<A, B> {
    /// Creates a new runtime with the specified backend.
    pub fn with_backend(backend: B) -> io::Result<Self> {
        Self::with_backend_and_config(backend, RuntimeConfig::default())
    }

    /// Creates a new runtime with backend and config.
    pub fn with_backend_and_config(backend: B, config: RuntimeConfig) -> io::Result<Self> {
        let terminal = Terminal::new(backend)?;
        let (state, init_cmd) = A::init();

        let mut commands = CommandHandler::new();
        commands.execute(init_cmd);

        Ok(Self {
            state,
            terminal,
            events: EventQueue::new(),
            commands,
            config,
            overlay_stack: OverlayStack::new(),
            theme: Theme::default(),
            should_quit: false,
        })
    }

    /// Returns a reference to the current state.
    pub fn state(&self) -> &A::State {
        &self.state
    }

    /// Returns a mutable reference to the state.
    pub fn state_mut(&mut self) -> &mut A::State {
        &mut self.state
    }

    /// Returns a reference to the inner ratatui Terminal.
    pub fn terminal(&self) -> &Terminal<B> {
        &self.terminal
    }

    /// Returns a mutable reference to the inner ratatui Terminal.
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

    /// Dispatches a message to update the state.
    pub fn dispatch(&mut self, msg: A::Message) {
        let cmd = A::update(&mut self.state, msg);
        self.commands.execute(cmd);

        if self.commands.should_quit() {
            self.should_quit = true;
        }
    }

    /// Dispatches multiple messages.
    pub fn dispatch_all(&mut self, messages: impl IntoIterator<Item = A::Message>) {
        for msg in messages {
            self.dispatch(msg);
        }
    }

    /// Processes any pending commands and returns resulting messages.
    pub fn process_commands(&mut self) {
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
    /// This is the primary method for advancing the application. It performs
    /// a full cycle: process commands, drain events, call on_tick, check quit,
    /// and render.
    ///
    /// For more granular control:
    /// - [`process_all_events`](Runtime::process_all_events) — Drain the event queue only
    /// - [`process_event`](Runtime::process_event) — Process exactly one event
    /// - [`run_ticks`](Runtime::run_ticks) — Convenience: run N full tick cycles
    pub fn tick(&mut self) -> io::Result<()> {
        // Process pending commands first
        self.process_commands();

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

    /// Sets the quit flag.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Runs for a specified number of ticks.
    pub fn run_ticks(&mut self, ticks: usize) -> io::Result<()> {
        for _ in 0..ticks {
            if self.should_quit {
                break;
            }
            self.tick()?;
        }
        Ok(())
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
        self.terminal.backend().cell(x, y)
    }

    /// Returns true if the display contains the given text.
    pub fn contains_text(&self, needle: &str) -> bool {
        self.terminal.backend().contains_text(needle)
    }

    /// Finds all positions of the given text in the display.
    pub fn find_text(&self, needle: &str) -> Vec<ratatui::layout::Position> {
        self.terminal.backend().find_text(needle)
    }
}

#[cfg(test)]
mod tests;
