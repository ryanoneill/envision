use super::*;
use crate::app::Command;
use ratatui::widgets::Paragraph;

struct TestApp;

#[derive(Clone, Default)]
struct TestState {
    count: i32,
    async_result: Option<i32>,
    quit: bool,
}

#[derive(Clone, Debug)]
enum TestMsg {
    Increment,
    SetAsyncResult(i32),
    StartAsyncOp,
    Quit,
}

impl App for TestApp {
    type State = TestState;
    type Message = TestMsg;

    fn init() -> (Self::State, Command<Self::Message>) {
        (TestState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            TestMsg::Increment => {
                state.count += 1;
                Command::none()
            }
            TestMsg::SetAsyncResult(v) => {
                state.async_result = Some(v);
                Command::none()
            }
            TestMsg::StartAsyncOp => Command::perform_async(async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Some(TestMsg::SetAsyncResult(42))
            }),
            TestMsg::Quit => {
                state.quit = true;
                Command::none()
            }
        }
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
fn test_async_harness_new() {
    let harness = AppHarness::<TestApp>::new(80, 24).unwrap();
    assert_eq!(harness.state().count, 0);
}

#[test]
fn test_async_harness_dispatch() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();

    harness.dispatch(TestMsg::Increment);
    assert_eq!(harness.state().count, 1);

    harness.dispatch(TestMsg::Increment);
    assert_eq!(harness.state().count, 2);
}

#[test]
fn test_async_harness_dispatch_all() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();

    harness.dispatch_all(vec![
        TestMsg::Increment,
        TestMsg::Increment,
        TestMsg::Increment,
    ]);

    assert_eq!(harness.state().count, 3);
}

#[test]
fn test_async_harness_render() {
    let mut harness = AppHarness::<TestApp>::new(40, 10).unwrap();
    harness.dispatch(TestMsg::Increment);
    harness.render().unwrap();

    assert!(harness.contains_text("Count: 1"));
}

#[test]
fn test_async_harness_events() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();

    harness.type_str("hello");
    harness.enter();

    assert_eq!(harness.events().len(), 6);
}

#[test]
fn test_async_harness_tick() {
    let mut harness = AppHarness::<TestApp>::new(40, 10).unwrap();
    harness.dispatch(TestMsg::Increment);
    harness.tick().unwrap();

    assert!(harness.contains_text("Count: 1"));
}

#[test]
fn test_async_harness_quit() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();
    assert!(!harness.should_quit());

    harness.dispatch(TestMsg::Quit);
    harness.tick().unwrap();

    assert!(harness.should_quit());
}

#[test]
fn test_async_harness_assert_contains() {
    let mut harness = AppHarness::<TestApp>::new(40, 10).unwrap();
    harness.render().unwrap();

    harness.assert_contains("Count: 0");
    harness.assert_not_contains("Unknown");
}

#[tokio::test(start_paused = true)]
async fn test_async_harness_async_command() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();

    // Start async operation
    harness.dispatch(TestMsg::StartAsyncOp);

    // Result not available yet
    assert!(harness.state().async_result.is_none());

    // Advance time past the delay
    harness.advance_time(Duration::from_secs(2)).await;

    // Now the result should be available
    assert_eq!(harness.state().async_result, Some(42));
}

#[tokio::test(start_paused = true)]
async fn test_async_harness_wait_for() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();

    harness.dispatch(TestMsg::StartAsyncOp);

    let success = harness
        .wait_for(
            |state| state.async_result.is_some(),
            Duration::from_secs(10),
        )
        .await;

    assert!(success);
    assert_eq!(harness.state().async_result, Some(42));
}

#[tokio::test(start_paused = true)]
async fn test_async_harness_wait_for_timeout() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();

    // Don't start any async op, so condition will never be true
    let success = harness
        .wait_for(
            |state| state.async_result.is_some(),
            Duration::from_millis(100),
        )
        .await;

    assert!(!success);
}

#[tokio::test(start_paused = true)]
async fn test_async_harness_wait_for_text() {
    let mut harness = AppHarness::<TestApp>::new(40, 10).unwrap();

    // Increment count
    harness.dispatch(TestMsg::Increment);

    let success = harness
        .wait_for_text("Count: 1", Duration::from_secs(1))
        .await;

    assert!(success);
}

#[tokio::test(start_paused = true)]
async fn test_async_harness_sleep() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();

    harness.dispatch(TestMsg::StartAsyncOp);
    harness.sleep(Duration::from_secs(5)).await;

    assert_eq!(harness.state().async_result, Some(42));
}

#[test]
fn test_async_harness_screen() {
    let mut harness = AppHarness::<TestApp>::new(40, 10).unwrap();
    harness.render().unwrap();

    let screen = harness.screen();
    assert!(screen.contains("Count: 0"));
}

#[test]
fn test_async_harness_row() {
    let mut harness = AppHarness::<TestApp>::new(40, 10).unwrap();
    harness.render().unwrap();

    let row = harness.row(0);
    assert!(row.contains("Count: 0"));
}

#[test]
fn test_async_harness_find_text() {
    let mut harness = AppHarness::<TestApp>::new(40, 10).unwrap();
    harness.render().unwrap();

    let positions = harness.find_text("Count");
    assert!(!positions.is_empty());
}

#[test]
fn test_async_harness_input_methods() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();

    harness.escape();
    harness.tab();
    harness.ctrl('c');
    harness.click(10, 20);
    harness.push_event(Event::char('x'));

    assert_eq!(harness.events().len(), 5);
}

#[test]
fn test_async_harness_cancellation_token() {
    let harness = AppHarness::<TestApp>::new(80, 24).unwrap();
    let token = harness.cancellation_token();
    assert!(!token.is_cancelled());
}

#[test]
fn test_async_harness_manual_quit() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();
    assert!(!harness.should_quit());

    harness.quit();
    assert!(harness.should_quit());
}

#[test]
fn test_async_harness_with_config() {
    let config = RuntimeConfig::new()
        .tick_rate(Duration::from_millis(100))
        .with_history(5);

    let harness = AppHarness::<TestApp>::with_config(80, 24, config).unwrap();
    assert_eq!(harness.state().count, 0);
}

#[test]
fn test_async_harness_process_events() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();
    harness.type_str("abc");
    harness.process_events();
    // Events processed (but TestApp doesn't handle key events)
}

#[test]
fn test_async_harness_run_ticks() {
    let mut harness = AppHarness::<TestApp>::new(40, 10).unwrap();
    harness.dispatch(TestMsg::Increment);
    harness.run_ticks(3).unwrap();

    assert!(harness.contains_text("Count: 1"));
}

#[test]
fn test_async_harness_state_mut() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();
    harness.state_mut().count = 42;
    assert_eq!(harness.state().count, 42);
}

#[test]
fn test_async_harness_screen_ansi() {
    let mut harness = AppHarness::<TestApp>::new(40, 10).unwrap();
    harness.render().unwrap();

    let screen_ansi = harness.screen_ansi();
    assert!(screen_ansi.contains("Count: 0"));
}

#[test]
fn test_async_harness_cell_at() {
    let mut harness = AppHarness::<TestApp>::new(40, 10).unwrap();
    harness.render().unwrap();

    // Cell at (0,0) should have the 'C' from "Count: 0"
    let cell = harness.cell_at(0, 0).unwrap();
    assert_eq!(cell.symbol(), "C");

    // Out of bounds should return None
    assert!(harness.cell_at(100, 100).is_none());
}

#[test]
fn test_async_harness_backend() {
    use ratatui::backend::Backend;
    let harness = AppHarness::<TestApp>::new(80, 24).unwrap();
    let backend = harness.backend();
    assert_eq!(backend.size().unwrap().width, 80);
}

#[test]
fn test_async_harness_backend_mut() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();
    let _backend = harness.backend_mut();
    // Just verify we can get a mutable reference
}

#[tokio::test]
async fn test_async_harness_message_sender() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();
    let sender = harness.message_sender();

    // Send a message via the channel
    sender.send(TestMsg::Increment).await.unwrap();

    // Let the runtime process it
    tokio::time::sleep(Duration::from_millis(1)).await;
    harness.runtime.process_pending();

    assert_eq!(harness.state().count, 1);
}

#[test]
fn test_async_harness_subscribe() {
    use crate::app::TickSubscription;

    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();
    let sub = TickSubscription::new(Duration::from_millis(10), || TestMsg::Increment);
    harness.subscribe(sub);
    // Just verify we can add a subscription
}

#[test]
fn test_async_harness_subscribe_all() {
    use crate::app::{BoxedSubscription, TickSubscription};

    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();

    let sub1: BoxedSubscription<TestMsg> =
        Box::new(TickSubscription::new(Duration::from_millis(10), || {
            TestMsg::Increment
        }));
    let sub2: BoxedSubscription<TestMsg> =
        Box::new(TickSubscription::new(Duration::from_millis(10), || {
            TestMsg::Increment
        }));

    harness.subscribe_all(vec![sub1, sub2]);
    // Just verify we can add multiple subscriptions
}

#[tokio::test(start_paused = true)]
async fn test_async_harness_wait_for_text_timeout() {
    let mut harness = AppHarness::<TestApp>::new(40, 10).unwrap();

    // Don't dispatch anything that changes text to "Count: 5"
    let success = harness
        .wait_for_text("Count: 5", Duration::from_millis(100))
        .await;

    assert!(!success);
}

#[test]
#[should_panic(expected = "Expected screen to contain 'MISSING'")]
fn test_async_harness_assert_contains_panic() {
    let mut harness = AppHarness::<TestApp>::new(40, 10).unwrap();
    harness.render().unwrap();

    // This should panic because 'MISSING' is not on screen
    harness.assert_contains("MISSING");
}

#[test]
#[should_panic(expected = "Expected screen to NOT contain 'Count'")]
fn test_async_harness_assert_not_contains_panic() {
    let mut harness = AppHarness::<TestApp>::new(40, 10).unwrap();
    harness.render().unwrap();

    // This should panic because 'Count' is on screen
    harness.assert_not_contains("Count");
}

#[test]
fn test_async_harness_events_direct() {
    let mut harness = AppHarness::<TestApp>::new(80, 24).unwrap();

    let events = harness.events();
    events.push(Event::char('a'));

    assert!(!harness.events().is_empty());
}
