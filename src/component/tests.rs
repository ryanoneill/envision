use super::*;
use crate::theme::Theme;
use ratatui::widgets::Paragraph;

// Test component implementation
struct TestCounter;

#[derive(Clone, Default)]
struct TestCounterState {
    value: i32,
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

    fn view(
        state: &Self::State,
        frame: &mut Frame,
        area: Rect,
        _theme: &Theme,
        _ctx: &ViewContext,
    ) {
        let text = format!("Count: {}", state.value);
        frame.render_widget(Paragraph::new(text), area);
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
            TestCounter::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let text = terminal.backend().to_string();
    assert!(text.contains("Count: 0"));
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
fn test_toggleable_show_hide() {
    let mut state = TestCounter::init();

    TestCounter::hide(&mut state);
    assert!(!TestCounter::is_visible(&state));

    TestCounter::show(&mut state);
    assert!(TestCounter::is_visible(&state));
}
