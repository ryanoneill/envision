use super::*;
use ratatui::widgets::Paragraph;

struct TestApp;

#[derive(Clone, Default)]
struct TestState {
    counter: i32,
}

#[derive(Clone)]
enum TestMsg {
    Increment,
    Decrement,
}

impl App for TestApp {
    type State = TestState;
    type Message = TestMsg;

    fn init() -> (Self::State, Command<Self::Message>) {
        (TestState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            TestMsg::Increment => state.counter += 1,
            TestMsg::Decrement => state.counter -= 1,
        }
        Command::none()
    }

    fn view(state: &Self::State, frame: &mut Frame) {
        let text = format!("Counter: {}", state.counter);
        frame.render_widget(Paragraph::new(text), frame.area());
    }
}

#[test]
fn test_app_init() {
    let (state, cmd) = TestApp::init();
    assert_eq!(state.counter, 0);
    assert!(cmd.is_none());
}

#[test]
fn test_app_update() {
    let (mut state, _) = TestApp::init();

    TestApp::update(&mut state, TestMsg::Increment);
    assert_eq!(state.counter, 1);

    TestApp::update(&mut state, TestMsg::Increment);
    assert_eq!(state.counter, 2);

    TestApp::update(&mut state, TestMsg::Decrement);
    assert_eq!(state.counter, 1);
}

#[test]
fn test_default_handle_event() {
    let event = Event::char('a');

    // Default implementation returns None
    let result = TestApp::handle_event(&event);
    assert!(result.is_none());
}

#[test]
fn test_default_should_quit() {
    let (state, _) = TestApp::init();

    // Default implementation returns false
    assert!(!TestApp::should_quit(&state));
}

#[test]
fn test_default_on_tick() {
    let (state, _) = TestApp::init();

    // Default implementation returns None
    let result = TestApp::on_tick(&state);
    assert!(result.is_none());
}

#[test]
fn test_default_on_exit() {
    let (state, _) = TestApp::init();

    // Default implementation does nothing (no panic)
    TestApp::on_exit(&state);
}

#[test]
fn test_app_view() {
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    let (state, _) = TestApp::init();
    let backend = CaptureBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            TestApp::view(&state, frame);
        })
        .unwrap();

    let text = terminal.backend().to_string();
    assert!(text.contains("Counter: 0"));
}

// Test an app with custom implementations of optional methods
struct CustomApp;

#[derive(Clone, Default)]
struct CustomState {
    should_exit: bool,
    tick_count: u32,
}

#[derive(Clone)]
enum CustomMsg {
    Tick,
    Quit,
    KeyPressed(char),
}

impl App for CustomApp {
    type State = CustomState;
    type Message = CustomMsg;

    fn init() -> (Self::State, Command<Self::Message>) {
        (CustomState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            CustomMsg::Tick => state.tick_count += 1,
            CustomMsg::Quit => state.should_exit = true,
            CustomMsg::KeyPressed(_) => {}
        }
        Command::none()
    }

    fn view(_state: &Self::State, _frame: &mut Frame) {}

    fn handle_event(event: &Event) -> Option<Self::Message> {
        use crossterm::event::KeyCode;
        if let Some(key) = event.as_key() {
            if let KeyCode::Char(c) = key.code {
                if c == 'q' {
                    return Some(CustomMsg::Quit);
                }
                return Some(CustomMsg::KeyPressed(c));
            }
        }
        None
    }

    fn should_quit(state: &Self::State) -> bool {
        state.should_exit
    }

    fn on_tick(_state: &Self::State) -> Option<Self::Message> {
        Some(CustomMsg::Tick)
    }

    fn on_exit(_state: &Self::State) {
        // Could save state or cleanup here
    }
}

#[test]
fn test_custom_handle_event() {
    // Test quit key
    let quit_event = Event::char('q');
    let result = CustomApp::handle_event(&quit_event);
    assert!(matches!(result, Some(CustomMsg::Quit)));

    // Test other key
    let other_event = Event::char('a');
    let result = CustomApp::handle_event(&other_event);
    assert!(matches!(result, Some(CustomMsg::KeyPressed('a'))));
}

#[test]
fn test_custom_should_quit() {
    let (mut state, _) = CustomApp::init();

    assert!(!CustomApp::should_quit(&state));

    CustomApp::update(&mut state, CustomMsg::Quit);
    assert!(CustomApp::should_quit(&state));
}

#[test]
fn test_custom_on_tick() {
    let (state, _) = CustomApp::init();

    let result = CustomApp::on_tick(&state);
    assert!(matches!(result, Some(CustomMsg::Tick)));
}

#[test]
fn test_custom_on_exit() {
    let (state, _) = CustomApp::init();

    // Should not panic
    CustomApp::on_exit(&state);
}

#[test]
fn test_app_non_clone_state() {
    // Verify that App::State does not require Clone
    struct NonCloneApp;

    // Intentionally does NOT derive Clone
    struct NonCloneState {
        value: i32,
    }

    #[derive(Clone)]
    enum NonCloneMsg {
        Set(i32),
    }

    impl App for NonCloneApp {
        type State = NonCloneState;
        type Message = NonCloneMsg;

        fn init() -> (Self::State, Command<Self::Message>) {
            (NonCloneState { value: 0 }, Command::none())
        }

        fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
            match msg {
                NonCloneMsg::Set(v) => state.value = v,
            }
            Command::none()
        }

        fn view(state: &Self::State, frame: &mut Frame) {
            let text = format!("Value: {}", state.value);
            frame.render_widget(Paragraph::new(text), frame.area());
        }
    }

    let (mut state, cmd) = NonCloneApp::init();
    assert!(cmd.is_none());
    assert_eq!(state.value, 0);

    NonCloneApp::update(&mut state, NonCloneMsg::Set(42));
    assert_eq!(state.value, 42);
}

#[test]
fn test_message_clone() {
    let msg = TestMsg::Increment;
    let cloned = msg.clone();

    let (mut state, _) = TestApp::init();
    TestApp::update(&mut state, msg);
    TestApp::update(&mut state, cloned);

    assert_eq!(state.counter, 2);
}

#[test]
fn test_handle_event_with_state_default_delegation() {
    // CustomApp overrides handle_event but NOT handle_event_with_state.
    // Calling handle_event_with_state should delegate to handle_event.
    use crossterm::event::KeyCode;

    let (state, _) = CustomApp::init();

    let event = Event::key(KeyCode::Char('q'));
    let msg = CustomApp::handle_event_with_state(&state, &event);
    assert!(matches!(msg, Some(CustomMsg::Quit)));

    let event = Event::key(KeyCode::Char('a'));
    let msg = CustomApp::handle_event_with_state(&state, &event);
    assert!(matches!(msg, Some(CustomMsg::KeyPressed('a'))));
}
