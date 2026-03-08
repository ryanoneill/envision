#![cfg(feature = "full")]
//! Async integration tests exercising commands, message channels, and error handling.
//!
//! NOTE: Subscription tests (TickSubscription, TimerSubscription, ChannelSubscription)
//! are intentionally omitted. Subscription streams are created via `subscribe()` but
//! are not polled by `tick()`, `process_pending()`, or `run()`. This is a known gap
//! that requires redesigning the subscription polling mechanism. See the planned
//! runtime API redesign for details.

use std::time::Duration;

use envision::{App, Command, Runtime};
use ratatui::prelude::*;

// ===========================================================================
// Shared App: AsyncLoader (tests Command::perform_async)
// ===========================================================================

struct AsyncLoaderApp;

#[derive(Clone, Default)]
struct AsyncLoaderState {
    data: Option<String>,
    loading: bool,
}

#[derive(Clone, Debug)]
enum AsyncLoaderMsg {
    StartLoad,
    DataLoaded(String),
}

impl App for AsyncLoaderApp {
    type State = AsyncLoaderState;
    type Message = AsyncLoaderMsg;

    fn init() -> (Self::State, Command<Self::Message>) {
        (AsyncLoaderState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            AsyncLoaderMsg::StartLoad => {
                state.loading = true;
                Command::perform_async(async {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Some(AsyncLoaderMsg::DataLoaded("hello world".into()))
                })
            }
            AsyncLoaderMsg::DataLoaded(data) => {
                state.loading = false;
                state.data = Some(data);
                Command::none()
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame) {
        let text = if state.loading {
            "Loading...".to_string()
        } else if let Some(ref data) = state.data {
            format!("Data: {}", data)
        } else {
            "Idle".to_string()
        };
        frame.render_widget(ratatui::widgets::Paragraph::new(text), frame.area());
    }
}

// ===========================================================================
// Shared App: FallibleApp (tests try_perform_async)
// ===========================================================================

struct FallibleApp;

#[derive(Clone, Default)]
struct FallibleState {
    data: Option<String>,
}

#[derive(Clone, Debug)]
enum FallibleMsg {
    StartFailing,
    StartSucceeding,
    DataLoaded(String),
}

impl App for FallibleApp {
    type State = FallibleState;
    type Message = FallibleMsg;

    fn init() -> (Self::State, Command<Self::Message>) {
        (FallibleState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            FallibleMsg::StartFailing => Command::try_perform_async(
                async {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "resource not found",
                    ))
                },
                |_: ()| Some(FallibleMsg::DataLoaded("unreachable".into())),
            ),
            FallibleMsg::StartSucceeding => Command::try_perform_async(
                async { Ok::<_, std::io::Error>("success data".to_string()) },
                |data| Some(FallibleMsg::DataLoaded(data)),
            ),
            FallibleMsg::DataLoaded(data) => {
                state.data = Some(data);
                Command::none()
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame) {
        let text = state.data.as_deref().unwrap_or("no data");
        frame.render_widget(ratatui::widgets::Paragraph::new(text), frame.area());
    }
}

// ===========================================================================
// Shared App: TickCounter (tests message channel delivery)
// ===========================================================================

struct TickCounterApp;

#[derive(Clone, Default)]
struct TickCounterState {
    count: u32,
}

#[derive(Clone, Debug)]
enum TickCounterMsg {
    Tick,
}

impl App for TickCounterApp {
    type State = TickCounterState;
    type Message = TickCounterMsg;

    fn init() -> (Self::State, Command<Self::Message>) {
        (TickCounterState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            TickCounterMsg::Tick => state.count += 1,
        }
        Command::none()
    }

    fn view(state: &Self::State, frame: &mut Frame) {
        let text = format!("Count: {}", state.count);
        frame.render_widget(ratatui::widgets::Paragraph::new(text), frame.area());
    }
}

// ===========================================================================
// Shared App: ChainedApp (tests command chaining: one async triggers another)
// ===========================================================================

struct ChainedApp;

#[derive(Clone, Default)]
struct ChainedState {
    step1_done: bool,
    step2_done: bool,
    final_result: Option<String>,
}

#[derive(Clone, Debug)]
enum ChainedMsg {
    StartChain,
    Step1Complete,
    Step2Complete(String),
}

impl App for ChainedApp {
    type State = ChainedState;
    type Message = ChainedMsg;

    fn init() -> (Self::State, Command<Self::Message>) {
        (ChainedState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            ChainedMsg::StartChain => {
                // Step 1: async operation
                Command::perform_async(async {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    Some(ChainedMsg::Step1Complete)
                })
            }
            ChainedMsg::Step1Complete => {
                state.step1_done = true;
                // Step 2: another async triggered by step 1
                Command::perform_async(async {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    Some(ChainedMsg::Step2Complete("chain complete".into()))
                })
            }
            ChainedMsg::Step2Complete(result) => {
                state.step2_done = true;
                state.final_result = Some(result);
                Command::none()
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame) {
        let text = state.final_result.as_deref().unwrap_or("pending");
        frame.render_widget(ratatui::widgets::Paragraph::new(text), frame.area());
    }
}

// ===========================================================================
// Tests: Command::perform_async
// ===========================================================================

#[tokio::test]
async fn test_command_perform_async_updates_state() {
    let mut vt = Runtime::<AsyncLoaderApp, _>::virtual_terminal(40, 10).unwrap();

    assert!(vt.state().data.is_none());
    assert!(!vt.state().loading);

    // Dispatch the load command
    vt.dispatch(AsyncLoaderMsg::StartLoad);
    assert!(vt.state().loading);

    // Wait for the async task to complete
    tokio::time::sleep(Duration::from_millis(50)).await;
    vt.process_pending();

    assert!(!vt.state().loading);
    assert_eq!(vt.state().data, Some("hello world".to_string()));
}

#[tokio::test]
async fn test_command_perform_async_chained() {
    let mut vt = Runtime::<ChainedApp, _>::virtual_terminal(40, 10).unwrap();

    // Start the chain
    vt.dispatch(ChainedMsg::StartChain);

    // Wait for step 1
    tokio::time::sleep(Duration::from_millis(20)).await;
    vt.process_pending();
    assert!(vt.state().step1_done);

    // Wait for step 2 (triggered by step 1)
    tokio::time::sleep(Duration::from_millis(20)).await;
    vt.process_pending();
    assert!(vt.state().step2_done);
    assert_eq!(vt.state().final_result, Some("chain complete".to_string()));
}

// ===========================================================================
// Tests: try_perform_async
// ===========================================================================

#[tokio::test]
async fn test_try_perform_async_error_reporting() {
    let mut vt = Runtime::<FallibleApp, _>::virtual_terminal(40, 10).unwrap();

    vt.dispatch(FallibleMsg::StartFailing);
    tokio::time::sleep(Duration::from_millis(50)).await;
    vt.process_pending();

    // State should NOT have been updated
    assert!(vt.state().data.is_none());

    // Error should be collected
    let errors = vt.take_errors();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].to_string().contains("resource not found"));
}

#[tokio::test]
async fn test_try_perform_async_success_updates_state() {
    let mut vt = Runtime::<FallibleApp, _>::virtual_terminal(40, 10).unwrap();

    vt.dispatch(FallibleMsg::StartSucceeding);
    tokio::time::sleep(Duration::from_millis(50)).await;
    vt.process_pending();

    assert_eq!(vt.state().data, Some("success data".to_string()));
}

#[tokio::test]
async fn test_try_perform_async_error_then_success() {
    let mut vt = Runtime::<FallibleApp, _>::virtual_terminal(40, 10).unwrap();

    // First: a failing command
    vt.dispatch(FallibleMsg::StartFailing);
    tokio::time::sleep(Duration::from_millis(20)).await;
    vt.process_pending();

    assert!(vt.state().data.is_none());
    let errors = vt.take_errors();
    assert_eq!(errors.len(), 1);

    // Second: a succeeding command — state should update despite previous error
    vt.dispatch(FallibleMsg::StartSucceeding);
    tokio::time::sleep(Duration::from_millis(20)).await;
    vt.process_pending();

    assert_eq!(vt.state().data, Some("success data".to_string()));
    // No new errors
    assert!(vt.take_errors().is_empty());
}

// ===========================================================================
// Tests: Message channel
// ===========================================================================

#[tokio::test]
async fn test_message_channel_delivers_messages() {
    let mut vt = Runtime::<TickCounterApp, _>::virtual_terminal(40, 10).unwrap();
    let tx = vt.message_sender();

    // Send messages via the channel
    tx.send(TickCounterMsg::Tick).await.unwrap();
    tx.send(TickCounterMsg::Tick).await.unwrap();
    tx.send(TickCounterMsg::Tick).await.unwrap();

    vt.process_pending();
    assert_eq!(vt.state().count, 3);
}

#[tokio::test]
async fn test_message_channel_interleaved_with_dispatch() {
    let mut vt = Runtime::<TickCounterApp, _>::virtual_terminal(40, 10).unwrap();
    let tx = vt.message_sender();

    // Direct dispatch
    vt.dispatch(TickCounterMsg::Tick);
    assert_eq!(vt.state().count, 1);

    // Channel message
    tx.send(TickCounterMsg::Tick).await.unwrap();
    vt.process_pending();
    assert_eq!(vt.state().count, 2);

    // Another direct dispatch
    vt.dispatch(TickCounterMsg::Tick);
    assert_eq!(vt.state().count, 3);
}

#[tokio::test]
async fn test_message_channel_from_spawned_task() {
    let mut vt = Runtime::<TickCounterApp, _>::virtual_terminal(40, 10).unwrap();
    let tx = vt.message_sender();

    // Spawn a task that sends messages
    tokio::spawn(async move {
        for _ in 0..10 {
            tx.send(TickCounterMsg::Tick).await.unwrap();
        }
    });

    // Wait for spawned task to complete
    tokio::time::sleep(Duration::from_millis(50)).await;
    vt.process_pending();

    assert_eq!(vt.state().count, 10);
}

// ===========================================================================
// Tests: Render after async operations
// ===========================================================================

#[tokio::test]
async fn test_render_reflects_async_state() {
    let mut vt = Runtime::<AsyncLoaderApp, _>::virtual_terminal(40, 10).unwrap();

    vt.render().unwrap();
    assert!(vt.contains_text("Idle"));

    vt.dispatch(AsyncLoaderMsg::StartLoad);
    vt.render().unwrap();
    assert!(vt.contains_text("Loading..."));

    tokio::time::sleep(Duration::from_millis(50)).await;
    vt.process_pending();
    vt.render().unwrap();
    assert!(vt.contains_text("Data: hello world"));
    assert!(!vt.contains_text("Loading..."));
}

#[tokio::test]
async fn test_render_after_chained_async() {
    let mut vt = Runtime::<ChainedApp, _>::virtual_terminal(60, 10).unwrap();

    vt.render().unwrap();
    assert!(vt.contains_text("pending"));

    vt.dispatch(ChainedMsg::StartChain);
    tokio::time::sleep(Duration::from_millis(50)).await;
    vt.process_pending();
    tokio::time::sleep(Duration::from_millis(50)).await;
    vt.process_pending();

    vt.render().unwrap();
    assert!(vt.contains_text("chain complete"));
}
