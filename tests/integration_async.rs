#![cfg(feature = "full")]
//! Async integration tests exercising commands, message channels, subscriptions,
//! and error handling.
//!
//! Every test whose outcome depends on a timer (a `tokio::time::sleep` inside
//! an async command, a tick/timer subscription, ...) runs under
//! `#[tokio::test(start_paused = true)]`. With the clock paused, tokio only
//! auto-advances time when every task is idle, so when the test's own
//! `sleep(d)` returns, every spawned task whose timers expire before `d` has
//! already run to completion. This makes these tests independent of
//! wall-clock speed (slow or overloaded CI runners).

use std::time::Duration;

use envision::{App, Command, Runtime};
use ratatui::prelude::*;

// ===========================================================================
// Shared App: AsyncLoader (tests Command::perform_async)
// ===========================================================================

struct AsyncLoaderApp;

/// Simulated latency of the `AsyncLoaderApp` load operation.
const LOAD_DELAY: Duration = Duration::from_millis(10);

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
    type Args = ();

    fn init(_args: ()) -> (Self::State, Command<Self::Message>) {
        (AsyncLoaderState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            AsyncLoaderMsg::StartLoad => {
                state.loading = true;
                Command::perform_async(async {
                    tokio::time::sleep(LOAD_DELAY).await;
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
    type Args = ();

    fn init(_args: ()) -> (Self::State, Command<Self::Message>) {
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
    type Args = ();

    fn init(_args: ()) -> (Self::State, Command<Self::Message>) {
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

/// Simulated latency of each step in the `ChainedApp` chain.
const CHAIN_STEP_DELAY: Duration = Duration::from_millis(5);

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
    type Args = ();

    fn init(_args: ()) -> (Self::State, Command<Self::Message>) {
        (ChainedState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            ChainedMsg::StartChain => {
                // Step 1: async operation
                Command::perform_async(async {
                    tokio::time::sleep(CHAIN_STEP_DELAY).await;
                    Some(ChainedMsg::Step1Complete)
                })
            }
            ChainedMsg::Step1Complete => {
                state.step1_done = true;
                // Step 2: another async triggered by step 1
                Command::perform_async(async {
                    tokio::time::sleep(CHAIN_STEP_DELAY).await;
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

#[tokio::test(start_paused = true)]
async fn test_command_perform_async_updates_state() {
    let mut vt = Runtime::<AsyncLoaderApp, _>::virtual_builder(40, 10)
        .build()
        .unwrap();

    assert!(vt.state().data.is_none());
    assert!(!vt.state().loading);

    // Dispatch the load command
    vt.dispatch(AsyncLoaderMsg::StartLoad);
    assert!(vt.state().loading);

    // Advance the paused clock past the load delay; the spawned task's
    // timer fires (and the task sends its message) before we resume.
    tokio::time::sleep(LOAD_DELAY * 2).await;
    vt.process_pending();

    assert!(!vt.state().loading);
    assert_eq!(vt.state().data, Some("hello world".to_string()));
}

#[tokio::test(start_paused = true)]
async fn test_command_perform_async_chained() {
    let mut vt = Runtime::<ChainedApp, _>::virtual_builder(40, 10)
        .build()
        .unwrap();

    // Start the chain: spawns the step-1 task.
    vt.dispatch(ChainedMsg::StartChain);

    // Round 1: step 1's timer fires and its message is queued; processing
    // it dispatches Step1Complete, which spawns the step-2 task.
    tokio::time::sleep(CHAIN_STEP_DELAY * 2).await;
    vt.process_pending();
    assert!(vt.state().step1_done);
    assert!(!vt.state().step2_done);

    // Round 2: step 2's timer fires; processing dispatches Step2Complete.
    tokio::time::sleep(CHAIN_STEP_DELAY * 2).await;
    vt.process_pending();
    assert!(vt.state().step2_done);
    assert_eq!(vt.state().final_result, Some("chain complete".to_string()));
}

// ===========================================================================
// Tests: try_perform_async
// ===========================================================================

#[tokio::test]
async fn test_try_perform_async_error_reporting() {
    let mut vt = Runtime::<FallibleApp, _>::virtual_builder(40, 10)
        .build()
        .unwrap();

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
    let mut vt = Runtime::<FallibleApp, _>::virtual_builder(40, 10)
        .build()
        .unwrap();

    vt.dispatch(FallibleMsg::StartSucceeding);
    tokio::time::sleep(Duration::from_millis(50)).await;
    vt.process_pending();

    assert_eq!(vt.state().data, Some("success data".to_string()));
}

#[tokio::test]
async fn test_try_perform_async_error_then_success() {
    let mut vt = Runtime::<FallibleApp, _>::virtual_builder(40, 10)
        .build()
        .unwrap();

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
    let mut vt = Runtime::<TickCounterApp, _>::virtual_builder(40, 10)
        .build()
        .unwrap();
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
    let mut vt = Runtime::<TickCounterApp, _>::virtual_builder(40, 10)
        .build()
        .unwrap();
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
    let mut vt = Runtime::<TickCounterApp, _>::virtual_builder(40, 10)
        .build()
        .unwrap();
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

#[tokio::test(start_paused = true)]
async fn test_render_reflects_async_state() {
    let mut vt = Runtime::<AsyncLoaderApp, _>::virtual_builder(40, 10)
        .build()
        .unwrap();

    vt.render().unwrap();
    assert!(vt.contains_text("Idle"));

    vt.dispatch(AsyncLoaderMsg::StartLoad);
    vt.render().unwrap();
    assert!(vt.contains_text("Loading..."));

    tokio::time::sleep(LOAD_DELAY * 2).await;
    vt.process_pending();
    vt.render().unwrap();
    assert!(vt.contains_text("Data: hello world"));
    assert!(!vt.contains_text("Loading..."));
}

#[tokio::test(start_paused = true)]
async fn test_render_after_chained_async() {
    let mut vt = Runtime::<ChainedApp, _>::virtual_builder(60, 10)
        .build()
        .unwrap();

    vt.render().unwrap();
    assert!(vt.contains_text("pending"));

    vt.dispatch(ChainedMsg::StartChain);

    // The chain needs exactly two rounds: each round lets one step's timer
    // fire, then `process_pending` dispatches its message (step 1's message
    // spawns step 2's task).
    tokio::time::sleep(CHAIN_STEP_DELAY * 2).await;
    vt.process_pending();
    vt.render().unwrap();
    assert!(vt.state().step1_done);
    assert!(vt.contains_text("pending"));

    tokio::time::sleep(CHAIN_STEP_DELAY * 2).await;
    vt.process_pending();

    vt.render().unwrap();
    assert!(vt.contains_text("chain complete"));
}

// Tests: AppHarness async methods (test-utils feature)
// ===========================================================================

#[cfg(feature = "test-utils")]
mod app_harness_tests {
    use std::time::Duration;

    use envision::harness::AppHarness;
    use envision::{App, Command};
    use ratatui::prelude::*;

    struct TimedApp;

    #[derive(Clone, Default)]
    struct TimedState {
        data: Option<String>,
        loading: bool,
    }

    #[derive(Clone, Debug)]
    enum TimedMsg {
        StartLoad,
        DataLoaded(String),
    }

    impl App for TimedApp {
        type State = TimedState;
        type Message = TimedMsg;
        type Args = ();

        fn init(_args: ()) -> (Self::State, Command<Self::Message>) {
            (TimedState::default(), Command::none())
        }

        fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
            match msg {
                TimedMsg::StartLoad => {
                    state.loading = true;
                    Command::perform_async(async {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        Some(TimedMsg::DataLoaded("loaded".into()))
                    })
                }
                TimedMsg::DataLoaded(data) => {
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

    #[tokio::test(start_paused = true)]
    async fn test_app_harness_advance_time() {
        let mut harness = AppHarness::<TimedApp>::new(40, 10).unwrap();

        harness.dispatch(TimedMsg::StartLoad);
        assert!(harness.state().loading);
        assert!(harness.state().data.is_none());

        // Advance time past the 100ms sleep
        harness.advance_time(Duration::from_millis(150)).await;

        assert!(!harness.state().loading);
        assert_eq!(harness.state().data, Some("loaded".to_string()));
    }

    #[tokio::test(start_paused = true)]
    async fn test_app_harness_wait_for() {
        let mut harness = AppHarness::<TimedApp>::new(40, 10).unwrap();

        harness.dispatch(TimedMsg::StartLoad);

        let found = harness
            .wait_for(|state| state.data.is_some(), Duration::from_secs(1))
            .await;

        assert!(found);
        assert_eq!(harness.state().data, Some("loaded".to_string()));
    }

    #[tokio::test(start_paused = true)]
    async fn test_app_harness_wait_for_text() {
        let mut harness = AppHarness::<TimedApp>::new(40, 10).unwrap();

        harness.dispatch(TimedMsg::StartLoad);

        let found = harness
            .wait_for_text("Data: loaded", Duration::from_secs(1))
            .await;

        assert!(found);
        assert!(harness.contains_text("Data: loaded"));
    }
}

// ===========================================================================
// Tests: Subscriptions
// ===========================================================================

#[tokio::test(start_paused = true)]
async fn test_tick_subscription_delivers_messages() {
    use envision::app::TickSubscription;

    let mut vt = Runtime::<TickCounterApp, _>::virtual_builder(40, 10)
        .build()
        .unwrap();

    // Subscribe with a 10ms tick interval
    let sub = TickSubscription::new(Duration::from_millis(10), || TickCounterMsg::Tick);
    vt.subscribe(sub);

    // With the clock paused, the interval ticks at exactly t = 0, 10, 20,
    // 30, 40 and 50ms before the test resumes at t = 55ms.
    tokio::time::sleep(Duration::from_millis(55)).await;
    vt.process_pending();

    assert_eq!(vt.state().count, 6);
}

#[tokio::test(start_paused = true)]
async fn test_timer_subscription_fires_once() {
    use envision::app::TimerSubscription;

    let mut vt = Runtime::<TickCounterApp, _>::virtual_builder(40, 10)
        .build()
        .unwrap();

    let sub = TimerSubscription::after(Duration::from_millis(20), TickCounterMsg::Tick);
    vt.subscribe(sub);

    // Wait for the timer to fire
    tokio::time::sleep(Duration::from_millis(50)).await;
    vt.process_pending();

    // Timer fires exactly once
    assert_eq!(vt.state().count, 1);

    // Wait more - should not fire again
    tokio::time::sleep(Duration::from_millis(50)).await;
    vt.process_pending();
    assert_eq!(vt.state().count, 1);
}

#[tokio::test]
async fn test_channel_subscription_forwards_messages() {
    use tokio::sync::mpsc;

    let mut vt = Runtime::<TickCounterApp, _>::virtual_builder(40, 10)
        .build()
        .unwrap();

    let (tx, rx) = mpsc::channel::<TickCounterMsg>(10);
    let sub = envision::app::ChannelSubscription::new(rx);
    vt.subscribe(sub);

    // Send messages through the external channel
    tx.send(TickCounterMsg::Tick).await.unwrap();
    tx.send(TickCounterMsg::Tick).await.unwrap();
    tx.send(TickCounterMsg::Tick).await.unwrap();

    // Wait for forwarding
    tokio::time::sleep(Duration::from_millis(20)).await;
    vt.process_pending();

    assert_eq!(vt.state().count, 3);
}

#[tokio::test(start_paused = true)]
async fn test_subscription_cancellation() {
    use envision::app::TickSubscription;

    let mut vt = Runtime::<TickCounterApp, _>::virtual_builder(40, 10)
        .build()
        .unwrap();

    let sub = TickSubscription::new(Duration::from_millis(10), || TickCounterMsg::Tick);
    vt.subscribe(sub);

    // With the clock paused, the interval ticks at exactly t = 0, 10, 20
    // and 30ms before the test resumes at t = 35ms.
    tokio::time::sleep(Duration::from_millis(35)).await;
    vt.process_pending();
    assert_eq!(vt.state().count, 4);

    // Quit cancels subscriptions. The subscription task observes the
    // cancellation before the clock can advance to the next tick (t = 40ms).
    vt.quit();

    // No new messages arrive once the subscription has been cancelled,
    // however far time advances.
    tokio::time::sleep(Duration::from_millis(100)).await;
    vt.process_pending();
    assert_eq!(
        vt.state().count,
        4,
        "Expected no new messages after cancellation"
    );
}
