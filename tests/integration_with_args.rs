#![cfg(feature = "full")]
//! Integration tests for `RuntimeBuilder::with_args` and the
//! `ConfiguredRuntimeBuilder` build path.
//!
//! These tests verify that applications can be initialized from
//! caller-supplied args, with the args flowing through `App::init`.

use envision::harness::AppHarness;
use envision::{App, Command, Runtime};
use ratatui::prelude::*;

// ===========================================================================
// Shared App: CounterApp — Args carry the initial state shape
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

#[derive(Clone, Default)]
struct CounterArgs {
    initial_count: i32,
    initial_label: String,
}

impl App for CounterApp {
    type State = CounterState;
    type Message = CounterMsg;
    type Args = CounterArgs;

    fn init(args: CounterArgs) -> (Self::State, Command<Self::Message>) {
        (
            CounterState {
                count: args.initial_count,
                label: args.initial_label,
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
// Shared App: InitCmdApp — args carry an init message to be dispatched
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

struct InitCmdArgs {
    initial_message: String,
}

impl App for InitCmdApp {
    type State = InitCmdState;
    type Message = InitCmdMsg;
    type Args = InitCmdArgs;

    fn init(args: InitCmdArgs) -> (Self::State, Command<Self::Message>) {
        (
            InitCmdState::default(),
            Command::message(InitCmdMsg::SetInitialized(args.initial_message)),
        )
    }

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
// Tests: Runtime::virtual_builder().with_args().build()
// ===========================================================================

#[test]
fn test_virtual_terminal_with_args_initializes_state() {
    let vt = Runtime::<CounterApp, _>::virtual_builder(80, 24)
        .with_args(CounterArgs {
            initial_count: 42,
            initial_label: "custom".into(),
        })
        .build()
        .unwrap();

    assert_eq!(vt.state().count, 42);
    assert_eq!(vt.state().label, "custom");
}

#[test]
fn test_virtual_terminal_with_args_renders_correctly() {
    let mut vt = Runtime::<CounterApp, _>::virtual_builder(80, 24)
        .with_args(CounterArgs {
            initial_count: 99,
            initial_label: "score".into(),
        })
        .build()
        .unwrap();

    vt.render().unwrap();
    assert!(vt.contains_text("score: 99"));
}

#[test]
fn test_virtual_terminal_with_args_dispatch_works() {
    let mut vt = Runtime::<CounterApp, _>::virtual_builder(80, 24)
        .with_args(CounterArgs {
            initial_count: 10,
            initial_label: "test".into(),
        })
        .build()
        .unwrap();

    vt.dispatch(CounterMsg::Increment);
    assert_eq!(vt.state().count, 11);

    vt.dispatch(CounterMsg::Decrement);
    vt.dispatch(CounterMsg::Decrement);
    assert_eq!(vt.state().count, 9);
}

#[test]
fn test_virtual_terminal_with_args_init_cmd_executes() {
    let mut vt = Runtime::<InitCmdApp, _>::virtual_builder(80, 24)
        .with_args(InitCmdArgs {
            initial_message: "from cmd".into(),
        })
        .build()
        .unwrap();

    // Synchronous init commands are queued and dispatched on process_commands()
    vt.process_commands();

    assert!(vt.state().initialized);
    assert_eq!(vt.state().value, "from cmd");
}

#[test]
fn test_virtual_terminal_with_args_negative_count() {
    let mut vt = Runtime::<CounterApp, _>::virtual_builder(80, 24)
        .with_args(CounterArgs {
            initial_count: -100,
            initial_label: "negative".into(),
        })
        .build()
        .unwrap();

    vt.render().unwrap();
    assert!(vt.contains_text("negative: -100"));
}

// ===========================================================================
// Tests: Runtime::virtual_builder().with_args().config().build()
// ===========================================================================

#[test]
fn test_virtual_terminal_with_args_and_config() {
    use envision::RuntimeConfig;

    let config = RuntimeConfig {
        capture_history: true,
        history_capacity: 5,
        ..RuntimeConfig::default()
    };
    let mut vt = Runtime::<CounterApp, _>::virtual_builder(80, 24)
        .with_args(CounterArgs {
            initial_count: 7,
            initial_label: "config".into(),
        })
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
fn test_builder_with_backend_and_args() {
    use envision::backend::CaptureBackend;

    let backend = CaptureBackend::new(60, 20);
    let vt = Runtime::<CounterApp, _>::builder(backend)
        .with_args(CounterArgs {
            initial_count: 55,
            initial_label: "backend".into(),
        })
        .build()
        .unwrap();

    assert_eq!(vt.state().count, 55);
    assert_eq!(vt.state().label, "backend");
}

// ===========================================================================
// Tests: AppHarness with args via the runtime helper
// ===========================================================================
//
// AppHarness provides a no-args constructor for App impls whose `Args = ()`.
// For apps with non-`()` args, callers can construct the underlying runtime
// directly and wrap it in their own harness adapter — but the most common
// path is to use the new() constructor with a `()`-args `App`. The CounterApp
// here uses non-`()` args, so we exercise that path directly via the runtime.

#[test]
fn test_harness_default_init_with_unit_args_app() {
    // A separate, no-args app verifies AppHarness::new still works.
    struct DefaultApp;
    #[derive(Clone, Default)]
    struct DefaultState {
        label: String,
    }
    #[derive(Clone, Debug)]
    enum DefaultMsg {}

    impl App for DefaultApp {
        type State = DefaultState;
        type Message = DefaultMsg;
        type Args = ();

        fn init(_args: ()) -> (Self::State, Command<Self::Message>) {
            (
                DefaultState {
                    label: "default".into(),
                },
                Command::none(),
            )
        }

        fn update(_: &mut Self::State, _: Self::Message) -> Command<Self::Message> {
            Command::none()
        }

        fn view(_: &Self::State, _: &mut Frame) {}
    }

    let harness = AppHarness::<DefaultApp>::new(80, 24).unwrap();
    assert_eq!(harness.state().label, "default");
}

// ===========================================================================
// Tests: Default args path still works (regression)
// ===========================================================================

#[test]
fn test_default_args_path_unchanged() {
    let vt = Runtime::<CounterApp, _>::virtual_builder(80, 24)
        .with_args(CounterArgs::default())
        .build()
        .unwrap();

    assert_eq!(vt.state().count, 0);
    assert_eq!(vt.state().label, "");
}
