#![cfg(feature = "full")]
//! Integration tests for Runtime and AppHarness `with_state` constructors.
//!
//! These tests verify that applications can be initialized from pre-built
//! state, bypassing `App::init()`.

use envision::harness::AppHarness;
use envision::{App, Command, Runtime};
use ratatui::prelude::*;

// ===========================================================================
// Shared App: CounterApp
// ===========================================================================

struct CounterApp;

#[derive(Clone, Default)]
struct CounterState {
    count: i32,
    label: String,
}

#[derive(Clone, Debug)]
enum CounterMsg {
    Increment,
    Decrement,
}

impl App for CounterApp {
    type State = CounterState;
    type Message = CounterMsg;

    fn init() -> (Self::State, Command<Self::Message>) {
        (
            CounterState {
                count: 0,
                label: "default".into(),
            },
            Command::none(),
        )
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            CounterMsg::Increment => state.count += 1,
            CounterMsg::Decrement => state.count -= 1,
        }
        Command::none()
    }

    fn view(state: &Self::State, frame: &mut Frame) {
        let text = format!("{}: {}", state.label, state.count);
        frame.render_widget(ratatui::widgets::Paragraph::new(text), frame.area());
    }
}

// ===========================================================================
// Shared App: InitCmdApp (tests that init_cmd is executed)
// ===========================================================================

struct InitCmdApp;

#[derive(Clone, Default)]
struct InitCmdState {
    initialized: bool,
    value: String,
}

#[derive(Clone, Debug)]
enum InitCmdMsg {
    SetInitialized(String),
}

impl App for InitCmdApp {
    type State = InitCmdState;
    type Message = InitCmdMsg;

    // init() omitted — this app only uses with_state constructors

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            InitCmdMsg::SetInitialized(value) => {
                state.initialized = true;
                state.value = value;
            }
        }
        Command::none()
    }

    fn view(state: &Self::State, frame: &mut Frame) {
        let text = if state.initialized {
            format!("Ready: {}", state.value)
        } else {
            "Not initialized".to_string()
        };
        frame.render_widget(ratatui::widgets::Paragraph::new(text), frame.area());
    }
}

// ===========================================================================
// Tests: Runtime::virtual_builder().state().build()
// ===========================================================================

#[test]
fn test_virtual_terminal_with_state_bypasses_init() {
    let state = CounterState {
        count: 42,
        label: "custom".into(),
    };
    let vt = Runtime::<CounterApp, _>::virtual_builder(80, 24)
        .state(state, Command::none())
        .build()
        .unwrap();

    assert_eq!(vt.state().count, 42);
    assert_eq!(vt.state().label, "custom");
}

#[test]
fn test_virtual_terminal_with_state_renders_correctly() {
    let state = CounterState {
        count: 99,
        label: "score".into(),
    };
    let mut vt = Runtime::<CounterApp, _>::virtual_builder(80, 24)
        .state(state, Command::none())
        .build()
        .unwrap();

    vt.render().unwrap();
    assert!(vt.contains_text("score: 99"));
}

#[test]
fn test_virtual_terminal_with_state_dispatch_works() {
    let state = CounterState {
        count: 10,
        label: "test".into(),
    };
    let mut vt = Runtime::<CounterApp, _>::virtual_builder(80, 24)
        .state(state, Command::none())
        .build()
        .unwrap();

    vt.dispatch(CounterMsg::Increment);
    assert_eq!(vt.state().count, 11);

    vt.dispatch(CounterMsg::Decrement);
    vt.dispatch(CounterMsg::Decrement);
    assert_eq!(vt.state().count, 9);
}

#[test]
fn test_virtual_terminal_with_state_init_cmd_executes() {
    let state = InitCmdState::default();
    let init_cmd = Command::message(InitCmdMsg::SetInitialized("from cmd".into()));

    let mut vt = Runtime::<InitCmdApp, _>::virtual_builder(80, 24)
        .state(state, init_cmd)
        .build()
        .unwrap();

    // Synchronous init commands are queued and dispatched on process_commands()
    vt.process_commands();

    assert!(vt.state().initialized);
    assert_eq!(vt.state().value, "from cmd");
}

#[test]
fn test_virtual_terminal_with_state_negative_count() {
    let state = CounterState {
        count: -100,
        label: "negative".into(),
    };
    let mut vt = Runtime::<CounterApp, _>::virtual_builder(80, 24)
        .state(state, Command::none())
        .build()
        .unwrap();

    vt.render().unwrap();
    assert!(vt.contains_text("negative: -100"));
}

// ===========================================================================
// Tests: Runtime::virtual_builder().state().config().build()
// ===========================================================================

#[test]
fn test_virtual_terminal_with_state_and_config() {
    use envision::RuntimeConfig;

    let state = CounterState {
        count: 7,
        label: "config".into(),
    };
    let config = RuntimeConfig {
        capture_history: true,
        history_capacity: 5,
        ..RuntimeConfig::default()
    };
    let mut vt = Runtime::<CounterApp, _>::virtual_builder(80, 24)
        .state(state, Command::none())
        .config(config)
        .build()
        .unwrap();

    vt.render().unwrap();
    assert!(vt.contains_text("config: 7"));
}

// ===========================================================================
// Tests: Runtime::builder() with CaptureBackend
// ===========================================================================

#[test]
fn test_builder_with_backend_and_state() {
    use envision::backend::CaptureBackend;

    let backend = CaptureBackend::new(60, 20);
    let state = CounterState {
        count: 55,
        label: "backend".into(),
    };
    let vt = Runtime::<CounterApp, _>::builder(backend)
        .state(state, Command::none())
        .build()
        .unwrap();

    assert_eq!(vt.state().count, 55);
    assert_eq!(vt.state().label, "backend");
}

// ===========================================================================
// Tests: AppHarness::with_state
// ===========================================================================

#[test]
fn test_harness_with_state_bypasses_init() {
    let state = CounterState {
        count: 100,
        label: "harness".into(),
    };
    let harness = AppHarness::<CounterApp>::with_state(80, 24, state, Command::none()).unwrap();

    assert_eq!(harness.state().count, 100);
    assert_eq!(harness.state().label, "harness");
}

#[test]
fn test_harness_with_state_dispatch_and_render() {
    let state = CounterState {
        count: 0,
        label: "interactive".into(),
    };
    let mut harness = AppHarness::<CounterApp>::with_state(80, 24, state, Command::none()).unwrap();

    harness.dispatch(CounterMsg::Increment);
    harness.dispatch(CounterMsg::Increment);
    harness.dispatch(CounterMsg::Increment);

    assert_eq!(harness.state().count, 3);

    harness.render().unwrap();
    assert!(harness.contains_text("interactive: 3"));
}

#[test]
fn test_harness_with_state_init_cmd_executes() {
    let state = InitCmdState::default();
    let init_cmd = Command::message(InitCmdMsg::SetInitialized("harness cmd".into()));

    let mut harness = AppHarness::<InitCmdApp>::with_state(80, 24, state, init_cmd).unwrap();

    // Synchronous init commands are queued and dispatched on tick()
    harness.tick().unwrap();

    assert!(harness.state().initialized);
    assert_eq!(harness.state().value, "harness cmd");
}

#[test]
fn test_harness_with_state_and_config() {
    use envision::RuntimeConfig;

    let state = CounterState {
        count: 77,
        label: "configured".into(),
    };
    let config = RuntimeConfig {
        capture_history: true,
        ..RuntimeConfig::default()
    };
    let harness =
        AppHarness::<CounterApp>::with_state_and_config(80, 24, state, Command::none(), config)
            .unwrap();

    assert_eq!(harness.state().count, 77);
}

// ===========================================================================
// Tests: Default init() still works (regression)
// ===========================================================================

#[test]
fn test_default_init_unchanged() {
    let vt = Runtime::<CounterApp, _>::virtual_builder(80, 24)
        .build()
        .unwrap();

    assert_eq!(vt.state().count, 0);
    assert_eq!(vt.state().label, "default");
}

#[test]
fn test_harness_default_init_unchanged() {
    let harness = AppHarness::<CounterApp>::new(80, 24).unwrap();

    assert_eq!(harness.state().count, 0);
    assert_eq!(harness.state().label, "default");
}
