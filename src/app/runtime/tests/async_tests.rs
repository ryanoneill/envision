use super::*;
use crate::app::command::BoxedError;
use crate::app::Command;
use std::time::Duration;

// =========================================================================
// Async Command and Message Channel Tests
// =========================================================================

#[tokio::test]
async fn test_runtime_async_command() {
    let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

    // Create an async command
    let cmd = Command::perform_async(async { Some(CounterMsg::IncrementBy(5)) });

    // Execute the command
    runtime.commands.execute(cmd);
    runtime.spawn_pending_commands();

    // Wait for the message
    tokio::time::sleep(Duration::from_millis(10)).await;
    runtime.process_pending();

    assert_eq!(runtime.state().count, 5);
}

#[tokio::test]
async fn test_runtime_message_channel() {
    let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
    let sender = runtime.message_sender();

    // Send a message via the channel
    sender.send(CounterMsg::Increment).await.unwrap();
    sender.send(CounterMsg::Increment).await.unwrap();

    // Process the messages
    runtime.process_pending();
    assert_eq!(runtime.state().count, 2);
}

// =========================================================================
// Error Handling Tests
// =========================================================================

#[tokio::test]
async fn test_runtime_take_errors() {
    let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
    let error_tx = runtime.error_sender();

    // No errors initially
    let errors = runtime.take_errors();
    assert!(errors.is_empty());

    // Send an error
    let err: BoxedError = Box::new(std::io::Error::other("test error"));
    error_tx.send(err).await.unwrap();

    // Should have one error
    let errors = runtime.take_errors();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].to_string().contains("test error"));

    // Errors are consumed
    let errors = runtime.take_errors();
    assert!(errors.is_empty());
}

#[tokio::test]
async fn test_runtime_has_errors() {
    let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
    let error_tx = runtime.error_sender();

    // No errors initially
    assert!(!runtime.has_errors());

    // Send an error
    let err: BoxedError = Box::new(std::io::Error::other("test error"));
    error_tx.send(err).await.unwrap();

    // Give the channel a moment to process
    tokio::time::sleep(Duration::from_millis(1)).await;

    // Should have errors now
    assert!(runtime.has_errors());

    // Consume the errors
    let _ = runtime.take_errors();

    // No more errors
    assert!(!runtime.has_errors());
}

#[tokio::test]
async fn test_runtime_error_from_spawned_task() {
    let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
    let error_tx = runtime.error_sender();

    // Spawn a task that reports an error
    tokio::spawn(async move {
        let err: BoxedError = Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "resource not found",
        ));
        let _ = error_tx.send(err).await;
    });

    // Wait for the task to complete
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Should have the error
    let errors = runtime.take_errors();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].to_string().contains("resource not found"));
}

#[tokio::test]
async fn test_runtime_multiple_errors() {
    let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();
    let error_tx = runtime.error_sender();

    // Send multiple errors
    for i in 0..3 {
        let err: BoxedError = Box::new(std::io::Error::other(format!("error {}", i)));
        error_tx.send(err).await.unwrap();
    }

    // Should have all three errors
    let errors = runtime.take_errors();
    assert_eq!(errors.len(), 3);
}

// =========================================================================
// Fallible Async Command Tests
// =========================================================================

struct FallibleApp;

#[derive(Clone, Default)]
struct FallibleState {
    value: Option<i32>,
}

#[derive(Clone, Debug)]
enum FallibleMsg {
    FetchSuccess,
    FetchFailure,
    Loaded(i32),
}

impl App for FallibleApp {
    type State = FallibleState;
    type Message = FallibleMsg;

    fn init() -> (Self::State, Command<Self::Message>) {
        (FallibleState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            FallibleMsg::FetchSuccess => {
                Command::try_perform_async(async { Ok::<_, std::io::Error>(42) }, |n| {
                    Some(FallibleMsg::Loaded(n))
                })
            }
            FallibleMsg::FetchFailure => Command::try_perform_async(
                async {
                    Err::<i32, _>(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "data not found",
                    ))
                },
                |n| Some(FallibleMsg::Loaded(n)),
            ),
            FallibleMsg::Loaded(n) => {
                state.value = Some(n);
                Command::none()
            }
        }
    }

    fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}
}

#[tokio::test]
async fn test_runtime_try_perform_async_success() {
    let mut runtime: Runtime<FallibleApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

    // Dispatch a message that triggers a successful async operation
    runtime.dispatch(FallibleMsg::FetchSuccess);

    // Wait for the async task to complete
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Process pending messages from the spawned task
    runtime.process_pending();

    // State should be updated with the loaded value
    assert_eq!(runtime.state().value, Some(42));

    // No errors should be in the channel
    assert!(!runtime.has_errors());
}

#[tokio::test]
async fn test_runtime_try_perform_async_failure() {
    let mut runtime: Runtime<FallibleApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

    // Dispatch a message that triggers a failing async operation
    runtime.dispatch(FallibleMsg::FetchFailure);

    // Wait for the async task to complete
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Process pending (there shouldn't be any messages, just the error)
    runtime.process_pending();

    // State should NOT be updated (error occurred)
    assert_eq!(runtime.state().value, None);

    // Error should be in the channel
    let errors = runtime.take_errors();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].to_string().contains("data not found"));
}

// =========================================================================
// Subscription Tests
// =========================================================================

#[tokio::test]
async fn test_runtime_subscribe() {
    use crate::app::subscription::TickSubscription;

    let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

    // Subscribe to a tick that fires every 10ms
    let sub = TickSubscription::new(Duration::from_millis(10), || CounterMsg::Increment);
    runtime.subscribe(sub);

    // Spawn a task to send quit after some ticks
    let tx = runtime.message_sender();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let _ = tx.send(CounterMsg::Quit).await;
    });

    // Run the event loop - subscriptions are polled here
    runtime.run().await.unwrap();

    // Should have quit cleanly
    assert!(runtime.should_quit());
}

#[tokio::test]
async fn test_runtime_subscribe_all() {
    use crate::app::subscription::{BoxedSubscription, TickSubscription};

    let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

    // Create multiple subscriptions
    let sub1: BoxedSubscription<CounterMsg> =
        Box::new(TickSubscription::new(Duration::from_millis(10), || {
            CounterMsg::Increment
        }));
    let sub2: BoxedSubscription<CounterMsg> =
        Box::new(TickSubscription::new(Duration::from_millis(10), || {
            CounterMsg::Increment
        }));

    runtime.subscribe_all(vec![sub1, sub2]);

    // Wait a bit for ticks
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Clean up
    runtime.quit();
}

// =========================================================================
// Run Loop Tests
// =========================================================================

#[tokio::test]
async fn test_runtime_run() {
    let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(40, 10).unwrap();

    // Increment counter
    runtime.dispatch(CounterMsg::Increment);

    // Spawn task to quit after a short delay
    let tx = runtime.message_sender();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let _ = tx.send(CounterMsg::Quit).await;
    });

    // Run the event loop
    runtime.run().await.unwrap();

    // Should have quit
    assert!(runtime.should_quit());
    assert!(runtime.contains_text("Count: 1"));
}

#[tokio::test]
async fn test_runtime_run_cancelled() {
    let mut runtime: Runtime<CounterApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

    let token = runtime.cancellation_token();

    // Spawn task to cancel after a short delay
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        token.cancel();
    });

    // Run the event loop
    runtime.run().await.unwrap();

    // Should have quit due to cancellation
    assert!(runtime.should_quit());
}

// =========================================================================
// Init Command Tests
// =========================================================================

struct InitCommandApp;

#[derive(Clone, Default)]
struct InitCommandState {
    initialized: bool,
}

#[derive(Clone)]
enum InitCommandMsg {
    Initialized,
}

impl App for InitCommandApp {
    type State = InitCommandState;
    type Message = InitCommandMsg;

    fn init() -> (Self::State, Command<Self::Message>) {
        // Return a command that sends Initialized message
        (
            InitCommandState::default(),
            Command::message(InitCommandMsg::Initialized),
        )
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            InitCommandMsg::Initialized => state.initialized = true,
        }
        Command::none()
    }

    fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}
}

#[test]
fn test_runtime_init_command() {
    let mut runtime: Runtime<InitCommandApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

    // Process sync commands from init
    runtime.process_pending();

    assert!(runtime.state().initialized);
}

// =========================================================================
// Ticking App Tests
// =========================================================================

struct TickingApp;

#[derive(Clone, Default)]
struct TickingState {
    ticks: i32,
    quit: bool,
}

#[derive(Clone)]
enum TickingMsg {
    Tick,
}

impl App for TickingApp {
    type State = TickingState;
    type Message = TickingMsg;

    fn init() -> (Self::State, Command<Self::Message>) {
        (TickingState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            TickingMsg::Tick => {
                state.ticks += 1;
                if state.ticks >= 3 {
                    state.quit = true;
                }
            }
        }
        Command::none()
    }

    fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}

    fn should_quit(state: &Self::State) -> bool {
        state.quit
    }

    fn on_tick(_state: &Self::State) -> Option<Self::Message> {
        Some(TickingMsg::Tick)
    }
}

#[test]
fn test_runtime_ticking_app() {
    let mut runtime: Runtime<TickingApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

    // Each tick should increment
    runtime.tick().unwrap();
    assert_eq!(runtime.state().ticks, 1);

    runtime.tick().unwrap();
    assert_eq!(runtime.state().ticks, 2);

    // Third tick should trigger quit
    runtime.tick().unwrap();
    assert_eq!(runtime.state().ticks, 3);
    assert!(runtime.should_quit());
}

#[tokio::test]
async fn test_runtime_run_with_on_tick() {
    let mut runtime: Runtime<TickingApp, _> = Runtime::virtual_terminal(80, 24).unwrap();

    // Run the event loop - should quit after 3 ticks
    runtime.run().await.unwrap();

    assert!(runtime.should_quit());
    assert!(runtime.state().ticks >= 3);
}
