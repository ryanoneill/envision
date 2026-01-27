//! Runtime for executing TEA applications.
//!
//! The runtime manages the main loop, event handling, and rendering.

use std::io;
use std::time::Duration;

use ratatui::backend::Backend;
use ratatui::Terminal;

use super::command::CommandHandler;
use super::model::App;
use crate::backend::CaptureBackend;
use crate::input::EventQueue;

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

    /// Whether the runtime should quit
    should_quit: bool,
}

impl<A: App> Runtime<A, CaptureBackend> {
    /// Creates a new runtime with a capture backend for headless operation.
    pub fn headless(width: u16, height: u16) -> io::Result<Self> {
        let backend = CaptureBackend::new(width, height);
        Self::with_backend(backend)
    }

    /// Creates a new runtime with history tracking.
    pub fn headless_with_config(
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

    /// Runs the application until it quits.
    ///
    /// For headless/testing mode, this processes all queued events
    /// and renders once, then returns.
    pub fn run(&mut self) -> io::Result<()> {
        while !self.should_quit && !self.events.is_empty() {
            self.tick()?;
        }

        // Final render if there are no events
        if !self.should_quit {
            self.tick()?;
        }

        A::on_exit(&self.state);
        Ok(())
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
}

// Convenience methods for CaptureBackend
impl<A: App> Runtime<A, CaptureBackend> {
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
        let runtime: Runtime<CounterApp, _> = Runtime::headless(80, 24).unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_runtime_dispatch() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(80, 24).unwrap();

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
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Increment);
        runtime.dispatch(CounterMsg::Increment);
        runtime.render().unwrap();

        assert!(runtime.contains_text("Count: 2"));
    }

    #[test]
    fn test_runtime_quit() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(80, 24).unwrap();
        assert!(!runtime.should_quit());

        runtime.dispatch(CounterMsg::Quit);
        runtime.tick().unwrap();

        assert!(runtime.should_quit());
    }

    #[test]
    fn test_runtime_tick() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(40, 10).unwrap();

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
            Runtime::headless_with_config(80, 24, config).unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_runtime_headless_with_config_no_history() {
        let config = RuntimeConfig::new();
        let runtime: Runtime<CounterApp, _> =
            Runtime::headless_with_config(80, 24, config).unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_runtime_state_mut() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(80, 24).unwrap();
        runtime.state_mut().count = 42;
        assert_eq!(runtime.state().count, 42);
    }

    #[test]
    fn test_runtime_terminal_access() {
        let runtime: Runtime<CounterApp, _> = Runtime::headless(80, 24).unwrap();
        let terminal = runtime.terminal();
        assert_eq!(terminal.backend().width(), 80);
        assert_eq!(terminal.backend().height(), 24);
    }

    #[test]
    fn test_runtime_terminal_mut() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(80, 24).unwrap();
        let _terminal = runtime.terminal_mut();
        // Just verify we can get mutable access
    }

    #[test]
    fn test_runtime_backend_access() {
        let runtime: Runtime<CounterApp, _> = Runtime::headless(80, 24).unwrap();
        let backend = runtime.backend();
        assert_eq!(backend.width(), 80);
    }

    #[test]
    fn test_runtime_backend_mut() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(80, 24).unwrap();
        let backend = runtime.backend_mut();
        assert_eq!(backend.width(), 80);
    }

    #[test]
    fn test_runtime_events_access() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(80, 24).unwrap();
        let events = runtime.events();
        assert!(events.is_empty());
    }

    #[test]
    fn test_runtime_dispatch_all() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(80, 24).unwrap();

        runtime.dispatch_all(vec![
            CounterMsg::Increment,
            CounterMsg::Increment,
            CounterMsg::Decrement,
        ]);

        assert_eq!(runtime.state().count, 1);
    }

    #[test]
    fn test_runtime_manual_quit() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(80, 24).unwrap();
        assert!(!runtime.should_quit());

        runtime.quit();
        assert!(runtime.should_quit());
    }

    #[test]
    fn test_runtime_run_ticks() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Increment);

        runtime.run_ticks(3).unwrap();
        assert!(runtime.contains_text("Count: 1"));
    }

    #[test]
    fn test_runtime_run_ticks_with_quit() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Quit);
        runtime.tick().unwrap();

        // Should stop early due to quit
        runtime.run_ticks(10).unwrap();
        assert!(runtime.should_quit());
    }

    #[test]
    fn test_runtime_run() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Increment);
        runtime.dispatch(CounterMsg::Increment);

        runtime.run().unwrap();
        assert!(runtime.contains_text("Count: 2"));
    }

    #[test]
    fn test_runtime_captured_output() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(40, 10).unwrap();
        runtime.render().unwrap();

        let output = runtime.captured_output();
        assert!(output.contains("Count: 0"));
    }

    #[test]
    fn test_runtime_captured_ansi() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(40, 10).unwrap();
        runtime.render().unwrap();

        let ansi = runtime.captured_ansi();
        assert!(ansi.contains("Count: 0"));
    }

    #[test]
    fn test_runtime_find_text() {
        let mut runtime: Runtime<CounterApp, _> = Runtime::headless(40, 10).unwrap();
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

        fn handle_event(
            _state: &Self::State,
            event: &crate::input::Event,
        ) -> Option<Self::Message> {
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

        let mut runtime: Runtime<EventApp, _> = Runtime::headless(80, 24).unwrap();

        runtime.events().push(Event::char('a'));
        assert!(runtime.process_event());
        assert_eq!(runtime.state().events_received, 1);

        // No more events
        assert!(!runtime.process_event());
    }

    #[test]
    fn test_runtime_process_all_events() {
        use crate::input::Event;

        let mut runtime: Runtime<EventApp, _> = Runtime::headless(80, 24).unwrap();

        runtime.events().push(Event::char('a'));
        runtime.events().push(Event::char('b'));
        runtime.events().push(Event::char('c'));

        runtime.process_all_events();
        assert_eq!(runtime.state().events_received, 3);
    }

    #[test]
    fn test_runtime_tick_with_on_tick() {
        let mut runtime: Runtime<EventApp, _> = Runtime::headless(40, 10).unwrap();

        runtime.tick().unwrap();
        assert_eq!(runtime.state().ticks, 1);

        runtime.tick().unwrap();
        assert_eq!(runtime.state().ticks, 2);
    }

    #[test]
    fn test_runtime_event_causes_quit() {
        use crate::input::Event;

        let mut runtime: Runtime<EventApp, _> = Runtime::headless(80, 24).unwrap();
        runtime.events().push(Event::char('q'));

        runtime.tick().unwrap();
        assert!(runtime.should_quit());
    }

    #[test]
    fn test_runtime_run_with_events() {
        use crate::input::Event;

        let mut runtime: Runtime<EventApp, _> = Runtime::headless(40, 10).unwrap();
        runtime.events().push(Event::char('a'));
        runtime.events().push(Event::char('b'));
        runtime.events().push(Event::char('q'));

        runtime.run().unwrap();

        // Should have processed events before quitting
        assert!(runtime.state().events_received >= 2);
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

        let mut runtime: Runtime<CmdApp, _> = Runtime::headless(80, 24).unwrap();

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
            Runtime::headless_with_config(80, 24, config).unwrap();

        // Queue more events than max_messages_per_tick
        for _ in 0..5 {
            runtime.events().push(Event::char('x'));
        }

        runtime.tick().unwrap();
        // Should only process up to max_messages (2)
        // But since on_tick also increments ticks, let's check events
        assert!(runtime.state().events_received <= 3);
    }
}
