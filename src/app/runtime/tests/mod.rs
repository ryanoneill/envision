mod async_tests;

use super::*;
use ratatui::widgets::Paragraph;

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

    fn init() -> (Self::State, super::super::Command<Self::Message>) {
        (CounterState::default(), super::super::Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> super::super::Command<Self::Message> {
        match msg {
            CounterMsg::Increment => state.count += 1,
            CounterMsg::Decrement => state.count -= 1,
            CounterMsg::IncrementBy(n) => state.count += n,
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
fn test_runtime_config_default() {
    let config = RuntimeConfig::default();
    assert_eq!(config.tick_rate, Duration::from_millis(50));
    assert_eq!(config.frame_rate, Duration::from_millis(16));
    assert_eq!(config.max_messages_per_tick, 100);
    assert!(!config.capture_history);
    assert_eq!(config.history_capacity, 10);
    assert_eq!(config.message_channel_capacity, 256);
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
fn test_runtime_terminal_access() {
    let runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
    let terminal = runtime.terminal();
    assert_eq!(terminal.backend().width(), 80);
    assert_eq!(terminal.backend().height(), 24);
}

#[test]
fn test_runtime_terminal_mut() {
    let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
    let _terminal = runtime.terminal_mut();
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
fn test_runtime_cancellation_token() {
    let runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
    let token = runtime.cancellation_token();
    assert!(!token.is_cancelled());
}

#[test]
fn test_runtime_message_sender() {
    let runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
    let _sender = runtime.message_sender();
}

#[test]
fn test_runtime_error_sender() {
    let runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
    let _error_tx = runtime.error_sender();
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
    assert!(!runtime.cancellation_token().is_cancelled());

    runtime.quit();
    assert!(runtime.should_quit());
    assert!(runtime.cancellation_token().is_cancelled());
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

    fn update(state: &mut Self::State, msg: Self::Message) -> super::super::Command<Self::Message> {
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
// Virtual Terminal API Tests
// =========================================================================

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
                    KeyCode::Enter => OverlayAction::DismissWithMessage(EventMsg::KeyPressed('!')),
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
