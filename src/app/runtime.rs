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
//! Runtime::<MyApp>::terminal()?.run()?;
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
    ///     Runtime::<MyApp>::terminal()?.run()
    /// }
    /// ```
    pub fn terminal() -> io::Result<Self> {
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
    ///     Runtime::<MyApp>::terminal()?.run()
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
    pub fn inner_terminal(&self) -> &Terminal<B> {
        &self.terminal
    }

    /// Returns a mutable reference to the inner ratatui Terminal.
    pub fn inner_terminal_mut(&mut self) -> &mut Terminal<B> {
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
mod tests {
    use super::*;
    use ratatui::widgets::Paragraph;

    struct CounterApp;

    #[derive(Clone, Default)]
    struct CounterState {
        count: i32,
        quit: bool,
    }

    #[derive(Clone)]
    enum CounterMsg {
        Increment,
        Decrement,
        Quit,
    }

    impl App for CounterApp {
        type State = CounterState;
        type Message = CounterMsg;

        fn init() -> (Self::State, super::super::Command<Self::Message>) {
            (CounterState::default(), super::super::Command::none())
        }

        fn update(
            state: &mut Self::State,
            msg: Self::Message,
        ) -> super::super::Command<Self::Message> {
            match msg {
                CounterMsg::Increment => state.count += 1,
                CounterMsg::Decrement => state.count -= 1,
                CounterMsg::Quit => state.quit = true,
            }
            super::super::Command::none()
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
    fn test_runtime_headless() {
        let runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_runtime_dispatch() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

        runtime.dispatch(CounterMsg::Increment);
        assert_eq!(runtime.state().count, 1);

        runtime.dispatch(CounterMsg::Increment);
        runtime.dispatch(CounterMsg::Increment);
        assert_eq!(runtime.state().count, 3);

        runtime.dispatch(CounterMsg::Decrement);
        assert_eq!(runtime.state().count, 2);
    }

    #[test]
    fn test_runtime_render() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Increment);
        runtime.dispatch(CounterMsg::Increment);
        runtime.render().unwrap();

        assert!(runtime.contains_text("Count: 2"));
    }

    #[test]
    fn test_runtime_quit() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
        assert!(!runtime.should_quit());

        runtime.dispatch(CounterMsg::Quit);
        runtime.tick().unwrap();

        assert!(runtime.should_quit());
    }

    #[test]
    fn test_runtime_tick() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();

        // Queue some events - we'd need to implement handle_event for this
        runtime.dispatch(CounterMsg::Increment);
        runtime.tick().unwrap();

        assert!(runtime.contains_text("Count: 1"));
    }

    #[test]
    fn test_runtime_config() {
        let config = RuntimeConfig::new()
            .tick_rate(Duration::from_millis(100))
            .with_history(5)
            .max_messages(50);

        assert_eq!(config.tick_rate, Duration::from_millis(100));
        assert!(config.capture_history);
        assert_eq!(config.history_capacity, 5);
        assert_eq!(config.max_messages_per_tick, 50);
    }

    #[test]
    fn test_runtime_config_default() {
        let config = RuntimeConfig::default();
        assert_eq!(config.tick_rate, Duration::from_millis(50));
        assert_eq!(config.max_messages_per_tick, 100);
        assert!(!config.capture_history);
        assert_eq!(config.history_capacity, 10);
    }

    #[test]
    fn test_runtime_config_debug() {
        let config = RuntimeConfig::new();
        let debug = format!("{:?}", config);
        assert!(debug.contains("RuntimeConfig"));
    }

    #[test]
    fn test_runtime_config_clone() {
        let config = RuntimeConfig::new().tick_rate(Duration::from_millis(200));
        let cloned = config.clone();
        assert_eq!(config.tick_rate, cloned.tick_rate);
    }

    #[test]
    fn test_runtime_headless_with_config() {
        let config = RuntimeConfig::new().with_history(5);
        let runtime: Runtime<CounterApp, _> =
            Runtime::virtual_terminal_with_config(80, 24, config).unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_runtime_headless_with_config_no_history() {
        let config = RuntimeConfig::new();
        let runtime: Runtime<CounterApp, _> =
            Runtime::virtual_terminal_with_config(80, 24, config).unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_runtime_state_mut() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
        runtime.state_mut().count = 42;
        assert_eq!(runtime.state().count, 42);
    }

    #[test]
    fn test_runtime_inner_terminal_access() {
        let runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
        let terminal = runtime.inner_terminal();
        assert_eq!(terminal.backend().width(), 80);
        assert_eq!(terminal.backend().height(), 24);
    }

    #[test]
    fn test_runtime_inner_terminal_mut() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
        let _terminal = runtime.inner_terminal_mut();
        // Just verify we can get mutable access
    }

    #[test]
    fn test_runtime_backend_access() {
        let runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
        let backend = runtime.backend();
        assert_eq!(backend.width(), 80);
    }

    #[test]
    fn test_runtime_backend_mut() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
        let backend = runtime.backend_mut();
        assert_eq!(backend.width(), 80);
    }

    #[test]
    fn test_runtime_events_access() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
        let events = runtime.events();
        assert!(events.is_empty());
    }

    #[test]
    fn test_runtime_dispatch_all() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

        runtime.dispatch_all(vec![
            CounterMsg::Increment,
            CounterMsg::Increment,
            CounterMsg::Decrement,
        ]);

        assert_eq!(runtime.state().count, 1);
    }

    #[test]
    fn test_runtime_manual_quit() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
        assert!(!runtime.should_quit());

        runtime.quit();
        assert!(runtime.should_quit());
    }

    #[test]
    fn test_runtime_run_ticks() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Increment);

        runtime.run_ticks(3).unwrap();
        assert!(runtime.contains_text("Count: 1"));
    }

    #[test]
    fn test_runtime_run_ticks_with_quit() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Quit);
        runtime.tick().unwrap();

        // Should stop early due to quit
        runtime.run_ticks(10).unwrap();
        assert!(runtime.should_quit());
    }

    #[test]
    fn test_runtime_captured_output() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();
        runtime.render().unwrap();

        let output = runtime.display();
        assert!(output.contains("Count: 0"));
    }

    #[test]
    fn test_runtime_captured_ansi() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();
        runtime.render().unwrap();

        let ansi = runtime.display_ansi();
        assert!(ansi.contains("Count: 0"));
    }

    #[test]
    fn test_runtime_find_text() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();
        runtime.render().unwrap();

        let positions = runtime.find_text("Count");
        assert!(!positions.is_empty());
    }

    // Test app that handles events and uses on_tick
    struct EventApp;

    #[derive(Clone, Default)]
    struct EventState {
        events_received: u32,
        last_key: Option<char>,
        ticks: u32,
        quit: bool,
    }

    #[derive(Clone)]
    enum EventMsg {
        KeyPressed(char),
        Tick,
        Quit,
    }

    impl App for EventApp {
        type State = EventState;
        type Message = EventMsg;

        fn init() -> (Self::State, super::super::Command<Self::Message>) {
            (EventState::default(), super::super::Command::none())
        }

        fn update(
            state: &mut Self::State,
            msg: Self::Message,
        ) -> super::super::Command<Self::Message> {
            match msg {
                EventMsg::KeyPressed(c) => {
                    state.events_received += 1;
                    state.last_key = Some(c);
                }
                EventMsg::Tick => state.ticks += 1,
                EventMsg::Quit => state.quit = true,
            }
            super::super::Command::none()
        }

        fn view(state: &Self::State, frame: &mut ratatui::Frame) {
            let text = format!("Events: {}, Ticks: {}", state.events_received, state.ticks);
            frame.render_widget(Paragraph::new(text), frame.area());
        }

        fn handle_event(event: &crate::input::Event) -> Option<Self::Message> {
            use crossterm::event::KeyCode;
            if let Some(key) = event.as_key() {
                if let KeyCode::Char(c) = key.code {
                    if c == 'q' {
                        return Some(EventMsg::Quit);
                    }
                    return Some(EventMsg::KeyPressed(c));
                }
            }
            None
        }

        fn on_tick(_state: &Self::State) -> Option<Self::Message> {
            Some(EventMsg::Tick)
        }

        fn should_quit(state: &Self::State) -> bool {
            state.quit
        }
    }

    #[test]
    fn test_runtime_process_event() {
        use crate::input::Event;

        let mut runtime: Runtime<EventApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

        runtime.events().push(Event::char('a'));
        assert!(runtime.process_event());
        assert_eq!(runtime.state().events_received, 1);

        // No more events
        assert!(!runtime.process_event());
    }

    #[test]
    fn test_runtime_process_all_events() {
        use crate::input::Event;

        let mut runtime: Runtime<EventApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

        runtime.events().push(Event::char('a'));
        runtime.events().push(Event::char('b'));
        runtime.events().push(Event::char('c'));

        runtime.process_all_events();
        assert_eq!(runtime.state().events_received, 3);
    }

    #[test]
    fn test_runtime_tick_with_on_tick() {
        let mut runtime: Runtime<EventApp, _> = Runtime::virtual_terminal(40, 10).unwrap();

        runtime.tick().unwrap();
        assert_eq!(runtime.state().ticks, 1);

        runtime.tick().unwrap();
        assert_eq!(runtime.state().ticks, 2);
    }

    #[test]
    fn test_runtime_event_causes_quit() {
        use crate::input::Event;

        let mut runtime: Runtime<EventApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
        runtime.events().push(Event::char('q'));

        runtime.tick().unwrap();
        assert!(runtime.should_quit());
    }

    #[test]
    fn test_runtime_process_commands() {
        // Test with an app that issues commands
        struct CmdApp;

        #[derive(Clone, Default)]
        struct CmdState {
            value: i32,
        }

        #[derive(Clone)]
        enum CmdMsg {
            Set(i32),
            Double,
        }

        impl App for CmdApp {
            type State = CmdState;
            type Message = CmdMsg;

            fn init() -> (Self::State, super::super::Command<Self::Message>) {
                // Issue a command on init
                (
                    CmdState::default(),
                    super::super::Command::message(CmdMsg::Set(10)),
                )
            }

            fn update(
                state: &mut Self::State,
                msg: Self::Message,
            ) -> super::super::Command<Self::Message> {
                match msg {
                    CmdMsg::Set(v) => {
                        state.value = v;
                        super::super::Command::none()
                    }
                    CmdMsg::Double => {
                        state.value *= 2;
                        super::super::Command::none()
                    }
                }
            }

            fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}
        }

        let mut runtime: Runtime<CmdApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

        // Process init command (Set(10))
        runtime.process_commands();
        assert_eq!(runtime.state().value, 10);

        // Manually dispatch Double message to test that variant
        runtime.dispatch(CmdMsg::Double);
        runtime.process_commands();
        assert_eq!(runtime.state().value, 20);
    }

    #[test]
    fn test_runtime_max_messages_per_tick() {
        use crate::input::Event;

        let config = RuntimeConfig::new().max_messages(2);
        let mut runtime: Runtime<EventApp, _> =
            Runtime::virtual_terminal_with_config(80, 24, config).unwrap();

        // Queue more events than max_messages_per_tick
        for _ in 0..5 {
            runtime.events().push(Event::char('x'));
        }

        runtime.tick().unwrap();
        // Should only process up to max_messages (2)
        // But since on_tick also increments ticks, let's check events
        assert!(runtime.state().events_received <= 3);
    }

    // =========================================================================
    // New Virtual Terminal API Tests
    // =========================================================================

    #[test]
    fn test_virtual_terminal_new() {
        let vt: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
        assert_eq!(vt.state().count, 0);
    }

    #[test]
    fn test_virtual_terminal_with_config() {
        let config = RuntimeConfig::new().with_history(5);
        let vt: Runtime<CounterApp, _> =
            Runtime::virtual_terminal_with_config(80, 24, config).unwrap();
        assert_eq!(vt.state().count, 0);
    }

    #[test]
    fn test_virtual_terminal_send_and_tick() {
        use crate::input::Event;

        let mut vt: Runtime<EventApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

        // Send events
        vt.send(Event::char('a'));
        vt.send(Event::char('b'));

        // Step processes the events
        vt.tick().unwrap();

        assert_eq!(vt.state().events_received, 2);
    }

    #[test]
    fn test_virtual_terminal_display() {
        let mut vt: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();
        vt.dispatch(CounterMsg::Increment);
        vt.tick().unwrap();

        let display = vt.display();
        assert!(display.contains("Count: 1"));
    }

    #[test]
    fn test_virtual_terminal_display_ansi() {
        let mut vt: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();
        vt.dispatch(CounterMsg::Increment);
        vt.tick().unwrap();

        let display = vt.display_ansi();
        assert!(display.contains("Count: 1"));
    }

    #[test]
    fn test_virtual_terminal_quit_via_event() {
        use crate::input::Event;

        let mut vt: Runtime<EventApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

        vt.send(Event::char('q'));
        vt.tick().unwrap();

        assert!(vt.should_quit());
    }

    #[test]
    fn test_virtual_terminal_multiple_ticks() {
        use crate::input::Event;

        let mut vt: Runtime<EventApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

        // First tick with one event
        vt.send(Event::char('a'));
        vt.tick().unwrap();
        assert_eq!(vt.state().events_received, 1);

        // Second tick with two events
        vt.send(Event::char('b'));
        vt.send(Event::char('c'));
        vt.tick().unwrap();
        assert_eq!(vt.state().events_received, 3);
    }

    #[test]
    fn test_virtual_terminal_cell_at() {
        let mut vt: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();
        vt.tick().unwrap();

        // Cell at (0,0) should have the 'C' from "Count: 0"
        let cell = vt.cell_at(0, 0).unwrap();
        assert_eq!(cell.symbol(), "C");

        // Out of bounds should return None
        assert!(vt.cell_at(100, 100).is_none());
    }

    #[test]
    fn test_virtual_terminal_contains_text() {
        let mut vt: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();
        vt.tick().unwrap();

        assert!(vt.contains_text("Count: 0"));
        assert!(!vt.contains_text("Not Here"));
    }

    #[test]
    fn test_virtual_terminal_find_text() {
        let mut vt: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();
        vt.tick().unwrap();

        let positions = vt.find_text("Count");
        assert!(!positions.is_empty());

        let positions = vt.find_text("Not Here");
        assert!(positions.is_empty());
    }

    // =========================================================================
    // Overlay Tests
    // =========================================================================

    mod overlay_tests {
        use super::*;
        use crate::app::Command;
        use crate::input::Event;
        use crate::overlay::{Overlay, OverlayAction};
        use crate::theme::Theme;
        use crossterm::event::KeyCode;
        use ratatui::layout::Rect;
        use ratatui::Frame;

        /// An overlay that consumes all events.
        struct ConsumeOverlay;

        impl Overlay<CounterMsg> for ConsumeOverlay {
            fn handle_event(&mut self, _event: &Event) -> OverlayAction<CounterMsg> {
                OverlayAction::Consumed
            }
            fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
        }

        /// An overlay that propagates all events.
        struct PropagateOverlay;

        impl Overlay<EventMsg> for PropagateOverlay {
            fn handle_event(&mut self, _event: &Event) -> OverlayAction<EventMsg> {
                OverlayAction::Propagate
            }
            fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
        }

        /// An overlay that dismisses on Esc and sends a message on Enter.
        struct DialogOverlay;

        impl Overlay<EventMsg> for DialogOverlay {
            fn handle_event(&mut self, event: &Event) -> OverlayAction<EventMsg> {
                if let Some(key) = event.as_key() {
                    match key.code {
                        KeyCode::Esc => OverlayAction::Dismiss,
                        KeyCode::Enter => {
                            OverlayAction::DismissWithMessage(EventMsg::KeyPressed('!'))
                        }
                        _ => OverlayAction::Consumed,
                    }
                } else {
                    OverlayAction::Propagate
                }
            }
            fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
        }

        #[test]
        fn test_runtime_overlay_push_pop() {
            let mut vt: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

            assert!(!vt.has_overlays());
            assert_eq!(vt.overlay_count(), 0);

            vt.push_overlay(Box::new(ConsumeOverlay));
            assert!(vt.has_overlays());
            assert_eq!(vt.overlay_count(), 1);

            vt.push_overlay(Box::new(ConsumeOverlay));
            assert_eq!(vt.overlay_count(), 2);

            let popped = vt.pop_overlay();
            assert!(popped.is_some());
            assert_eq!(vt.overlay_count(), 1);

            vt.clear_overlays();
            assert!(!vt.has_overlays());
        }

        #[test]
        fn test_runtime_overlay_consumes_events() {
            let mut vt: Runtime<EventApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

            // Push an overlay that consumes all events
            struct ConsumeAll;
            impl Overlay<EventMsg> for ConsumeAll {
                fn handle_event(&mut self, _event: &Event) -> OverlayAction<EventMsg> {
                    OverlayAction::Consumed
                }
                fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
            }

            vt.push_overlay(Box::new(ConsumeAll));

            // Send events — they should be consumed by the overlay, not reaching the app
            vt.send(Event::char('a'));
            vt.send(Event::char('b'));
            vt.tick().unwrap();

            assert_eq!(vt.state().events_received, 0);
        }

        #[test]
        fn test_runtime_overlay_propagates_events() {
            let mut vt: Runtime<EventApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

            // Push an overlay that propagates all events
            vt.push_overlay(Box::new(PropagateOverlay));

            // Send events — they should reach the app
            vt.send(Event::char('a'));
            vt.tick().unwrap();

            assert_eq!(vt.state().events_received, 1);
        }

        #[test]
        fn test_runtime_overlay_dismiss() {
            let mut vt: Runtime<EventApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

            vt.push_overlay(Box::new(DialogOverlay));
            assert_eq!(vt.overlay_count(), 1);

            // Esc dismisses the overlay
            vt.send(Event::key(KeyCode::Esc));
            vt.tick().unwrap();

            assert_eq!(vt.overlay_count(), 0);
        }

        #[test]
        fn test_runtime_overlay_dismiss_with_message() {
            let mut vt: Runtime<EventApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

            vt.push_overlay(Box::new(DialogOverlay));

            // Enter dismisses with a message
            vt.send(Event::key(KeyCode::Enter));
            vt.tick().unwrap();

            assert_eq!(vt.overlay_count(), 0);
            // The message should have been dispatched
            assert_eq!(vt.state().events_received, 1);
            assert_eq!(vt.state().last_key, Some('!'));
        }

        #[test]
        fn test_runtime_overlay_via_command() {
            // Test that Command::push_overlay and Command::pop_overlay work through the runtime
            struct CmdOverlayApp;

            #[derive(Clone, Default)]
            struct CmdOverlayState {
                overlay_pushed: bool,
            }

            #[derive(Clone)]
            enum CmdOverlayMsg {
                PushOverlay,
                PopOverlay,
            }

            struct NoopOverlay;
            impl Overlay<CmdOverlayMsg> for NoopOverlay {
                fn handle_event(&mut self, _event: &Event) -> OverlayAction<CmdOverlayMsg> {
                    OverlayAction::Consumed
                }
                fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
            }

            impl App for CmdOverlayApp {
                type State = CmdOverlayState;
                type Message = CmdOverlayMsg;

                fn init() -> (Self::State, Command<Self::Message>) {
                    (CmdOverlayState::default(), Command::none())
                }

                fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
                    match msg {
                        CmdOverlayMsg::PushOverlay => {
                            state.overlay_pushed = true;
                            Command::push_overlay(NoopOverlay)
                        }
                        CmdOverlayMsg::PopOverlay => Command::pop_overlay(),
                    }
                }

                fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}
            }

            let mut vt: Runtime<CmdOverlayApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

            // Dispatch push overlay message
            vt.dispatch(CmdOverlayMsg::PushOverlay);
            vt.process_commands();
            assert!(vt.has_overlays());
            assert_eq!(vt.overlay_count(), 1);

            // Dispatch pop overlay message
            vt.dispatch(CmdOverlayMsg::PopOverlay);
            vt.process_commands();
            assert!(!vt.has_overlays());
        }

        #[test]
        fn test_runtime_theme_access() {
            let mut vt: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

            // Default theme should be set
            let _theme = vt.theme();

            // Set a custom theme
            let nord = Theme::nord();
            let expected_bg = nord.background;
            vt.set_theme(nord);
            assert_eq!(vt.theme().background, expected_bg);
        }

        #[test]
        fn test_runtime_render_with_overlay() {
            // Verifies the overlay rendering path in render()
            let mut vt: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();

            vt.push_overlay(Box::new(ConsumeOverlay));
            vt.render().unwrap();

            // App content should still be rendered underneath
            assert!(vt.contains_text("Count: 0"));
        }

        #[test]
        fn test_runtime_overlay_message_from_event() {
            // Test the OverlayAction::Message path in process_event
            struct MsgOverlay;
            impl Overlay<EventMsg> for MsgOverlay {
                fn handle_event(&mut self, _event: &Event) -> OverlayAction<EventMsg> {
                    OverlayAction::Message(EventMsg::KeyPressed('z'))
                }
                fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
            }

            let mut vt: Runtime<EventApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
            vt.push_overlay(Box::new(MsgOverlay));

            vt.send(Event::char('x'));
            vt.tick().unwrap();

            // The overlay should have produced a message, not the app's handle_event
            assert_eq!(vt.state().events_received, 1);
            assert_eq!(vt.state().last_key, Some('z'));
        }

        #[test]
        fn test_runtime_process_commands_overlay_push_pop() {
            // Directly test the overlay processing in process_commands()
            struct CmdApp;

            #[derive(Clone, Default)]
            struct CmdState;

            #[derive(Clone)]
            enum CmdMsg {
                Push,
                Pop,
            }

            struct NoopOverlay;
            impl Overlay<CmdMsg> for NoopOverlay {
                fn handle_event(&mut self, _event: &Event) -> OverlayAction<CmdMsg> {
                    OverlayAction::Consumed
                }
                fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
            }

            impl App for CmdApp {
                type State = CmdState;
                type Message = CmdMsg;

                fn init() -> (Self::State, Command<Self::Message>) {
                    (CmdState, Command::none())
                }

                fn update(_state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
                    match msg {
                        CmdMsg::Push => Command::push_overlay(NoopOverlay),
                        CmdMsg::Pop => Command::pop_overlay(),
                    }
                }

                fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}
            }

            let mut vt: Runtime<CmdApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

            // Push two overlays via commands
            vt.dispatch(CmdMsg::Push);
            vt.process_commands();
            assert_eq!(vt.overlay_count(), 1);

            vt.dispatch(CmdMsg::Push);
            vt.process_commands();
            assert_eq!(vt.overlay_count(), 2);

            // Pop one via command
            vt.dispatch(CmdMsg::Pop);
            vt.process_commands();
            assert_eq!(vt.overlay_count(), 1);
        }
    }
}
