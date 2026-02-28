use super::*;
use crate::theme::Theme;
use ratatui::widgets::Paragraph;

// Test component implementation
struct TestCounter;

#[derive(Clone, Default)]
struct TestCounterState {
    value: i32,
    focused: bool,
    visible: bool,
}

#[derive(Clone)]
enum TestCounterMsg {
    Increment,
    Decrement,
}

#[derive(Clone, PartialEq, Debug)]
enum TestCounterOutput {
    Changed(i32),
}

impl Component for TestCounter {
    type State = TestCounterState;
    type Message = TestCounterMsg;
    type Output = TestCounterOutput;

    fn init() -> Self::State {
        TestCounterState {
            value: 0,
            focused: false,
            visible: true,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            TestCounterMsg::Increment => state.value += 1,
            TestCounterMsg::Decrement => state.value -= 1,
        }
        Some(TestCounterOutput::Changed(state.value))
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, _theme: &Theme) {
        let text = format!("Count: {}", state.value);
        frame.render_widget(Paragraph::new(text), area);
    }
}

impl Focusable for TestCounter {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Toggleable for TestCounter {
    fn is_visible(state: &Self::State) -> bool {
        state.visible
    }

    fn set_visible(state: &mut Self::State, visible: bool) {
        state.visible = visible;
    }
}

// Component trait tests

#[test]
fn test_component_init() {
    let state = TestCounter::init();
    assert_eq!(state.value, 0);
    assert!(!state.focused);
    assert!(state.visible);
}

#[test]
fn test_component_update() {
    let mut state = TestCounter::init();

    let output = TestCounter::update(&mut state, TestCounterMsg::Increment);
    assert_eq!(state.value, 1);
    assert_eq!(output, Some(TestCounterOutput::Changed(1)));

    let output = TestCounter::update(&mut state, TestCounterMsg::Increment);
    assert_eq!(state.value, 2);
    assert_eq!(output, Some(TestCounterOutput::Changed(2)));

    let output = TestCounter::update(&mut state, TestCounterMsg::Decrement);
    assert_eq!(state.value, 1);
    assert_eq!(output, Some(TestCounterOutput::Changed(1)));
}

#[test]
fn test_component_view() {
    let state = TestCounter::init();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            TestCounter::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let text = terminal.backend().to_string();
    assert!(text.contains("Count: 0"));
}

#[test]
fn test_state_clone() {
    let mut state = TestCounter::init();
    TestCounter::update(&mut state, TestCounterMsg::Increment);

    let snapshot = state.clone();
    TestCounter::update(&mut state, TestCounterMsg::Increment);

    assert_eq!(snapshot.value, 1);
    assert_eq!(state.value, 2);
}

// Focusable trait tests

#[test]
fn test_focusable_is_focused() {
    let state = TestCounter::init();
    assert!(!TestCounter::is_focused(&state));
}

#[test]
fn test_focusable_set_focused() {
    let mut state = TestCounter::init();

    TestCounter::set_focused(&mut state, true);
    assert!(TestCounter::is_focused(&state));

    TestCounter::set_focused(&mut state, false);
    assert!(!TestCounter::is_focused(&state));
}

#[test]
fn test_focusable_focus() {
    let mut state = TestCounter::init();

    TestCounter::focus(&mut state);
    assert!(TestCounter::is_focused(&state));
}

#[test]
fn test_focusable_blur() {
    let mut state = TestCounter::init();
    TestCounter::set_focused(&mut state, true);

    TestCounter::blur(&mut state);
    assert!(!TestCounter::is_focused(&state));
}

// Toggleable trait tests

#[test]
fn test_toggleable_is_visible() {
    let state = TestCounter::init();
    assert!(TestCounter::is_visible(&state));
}

#[test]
fn test_toggleable_set_visible() {
    let mut state = TestCounter::init();

    TestCounter::set_visible(&mut state, false);
    assert!(!TestCounter::is_visible(&state));

    TestCounter::set_visible(&mut state, true);
    assert!(TestCounter::is_visible(&state));
}

#[test]
fn test_toggleable_toggle() {
    let mut state = TestCounter::init();
    assert!(TestCounter::is_visible(&state));

    TestCounter::toggle(&mut state);
    assert!(!TestCounter::is_visible(&state));

    TestCounter::toggle(&mut state);
    assert!(TestCounter::is_visible(&state));
}

#[test]
fn test_toggleable_show() {
    let mut state = TestCounter::init();
    TestCounter::set_visible(&mut state, false);

    TestCounter::show(&mut state);
    assert!(TestCounter::is_visible(&state));
}

#[test]
fn test_toggleable_hide() {
    let mut state = TestCounter::init();

    TestCounter::hide(&mut state);
    assert!(!TestCounter::is_visible(&state));
}

// Test component with unit Output type
struct NoOutputComponent;

#[derive(Clone, Default)]
struct NoOutputState {
    data: String,
}

#[derive(Clone)]
enum NoOutputMsg {
    SetData(String),
}

impl Component for NoOutputComponent {
    type State = NoOutputState;
    type Message = NoOutputMsg;
    type Output = ();

    fn init() -> Self::State {
        NoOutputState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            NoOutputMsg::SetData(data) => state.data = data,
        }
        None // No output needed
    }

    fn view(_state: &Self::State, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
}

#[test]
fn test_component_non_clone_state() {
    // Verify that Component::State does not require Clone
    struct NonCloneComponent;

    // Intentionally does NOT derive Clone
    struct NonCloneState {
        value: i32,
    }

    #[derive(Clone)]
    enum NonCloneMsg {
        Set(i32),
    }

    #[derive(Clone)]
    enum NonCloneOutput {
        Changed(i32),
    }

    impl Component for NonCloneComponent {
        type State = NonCloneState;
        type Message = NonCloneMsg;
        type Output = NonCloneOutput;

        fn init() -> Self::State {
            NonCloneState { value: 0 }
        }

        fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
            match msg {
                NonCloneMsg::Set(v) => {
                    state.value = v;
                    Some(NonCloneOutput::Changed(v))
                }
            }
        }

        fn view(state: &Self::State, frame: &mut Frame, area: Rect, _theme: &Theme) {
            let text = format!("Value: {}", state.value);
            frame.render_widget(Paragraph::new(text), area);
        }
    }

    let mut state = NonCloneComponent::init();
    assert_eq!(state.value, 0);

    let output = NonCloneComponent::update(&mut state, NonCloneMsg::Set(42));
    assert_eq!(state.value, 42);
    assert!(matches!(output, Some(NonCloneOutput::Changed(42))));
}

#[test]
fn test_component_no_output() {
    let mut state = NoOutputComponent::init();
    let output = NoOutputComponent::update(&mut state, NoOutputMsg::SetData("test".into()));
    assert!(output.is_none());
    assert_eq!(state.data, "test");
}
