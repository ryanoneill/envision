//! Async test harness for TEA applications with time control.
//!
//! This harness wraps `AsyncRuntime` and provides deterministic testing
//! capabilities using tokio's time control features.
//!
//! # Time Control
//!
//! When using `#[tokio::test(start_paused = true)]`, time is paused and
//! you can use `advance_time()` to manually advance the clock. This enables
//! deterministic testing of async operations and timers.
//!
//! # Example
//!
//! ```ignore
//! #[tokio::test(start_paused = true)]
//! async fn test_delayed_operation() {
//!     let mut harness = AsyncTestHarness::<MyApp>::new(80, 24);
//!
//!     // Dispatch an async command with a delay
//!     harness.dispatch(Msg::StartDelayedOp).await;
//!
//!     // State unchanged immediately
//!     assert!(!harness.state().operation_complete);
//!
//!     // Advance time past the delay
//!     harness.advance_time(Duration::from_secs(2)).await;
//!
//!     // Now the operation completed
//!     assert!(harness.state().operation_complete);
//! }
//! ```

use std::io;

use ratatui::layout::Position;
use tokio_util::sync::CancellationToken;

use crate::app::{App, AsyncRuntime, AsyncRuntimeConfig, BoxedSubscription, Subscription};
use crate::backend::CaptureBackend;
use crate::input::{Event, EventQueue};

/// Async test harness for TEA applications.
///
/// This harness provides:
/// - Time control for deterministic async testing
/// - Convenient dispatch and assertion methods
/// - Access to the underlying runtime and state
pub struct AsyncTestHarness<A: App>
where
    A::Message: Send + Clone + 'static,
{
    runtime: AsyncRuntime<A, CaptureBackend>,
}

impl<A: App> AsyncTestHarness<A>
where
    A::Message: Send + Clone + 'static,
{
    /// Creates a new async test harness with the given dimensions.
    ///
    /// Note: For time control, use `#[tokio::test(start_paused = true)]`.
    pub fn new(width: u16, height: u16) -> io::Result<Self> {
        let runtime = AsyncRuntime::virtual_terminal(width, height)?;
        Ok(Self { runtime })
    }

    /// Creates a new async test harness with custom configuration.
    pub fn with_config(width: u16, height: u16, config: AsyncRuntimeConfig) -> io::Result<Self> {
        let runtime = AsyncRuntime::virtual_terminal_with_config(width, height, config)?;
        Ok(Self { runtime })
    }

    // -------------------------------------------------------------------------
    // State Access
    // -------------------------------------------------------------------------

    /// Returns a reference to the current state.
    pub fn state(&self) -> &A::State {
        self.runtime.state()
    }

    /// Returns a mutable reference to the state.
    pub fn state_mut(&mut self) -> &mut A::State {
        self.runtime.state_mut()
    }

    /// Returns the captured output as a string.
    pub fn screen(&self) -> String {
        self.runtime.captured_output()
    }

    /// Returns the captured output with ANSI colors.
    pub fn screen_ansi(&self) -> String {
        self.runtime.captured_ansi()
    }

    /// Returns a reference to the backend.
    pub fn backend(&self) -> &CaptureBackend {
        self.runtime.backend()
    }

    /// Returns a mutable reference to the backend.
    pub fn backend_mut(&mut self) -> &mut CaptureBackend {
        self.runtime.backend_mut()
    }

    // -------------------------------------------------------------------------
    // Message Dispatch
    // -------------------------------------------------------------------------

    /// Dispatches a message and processes all resulting work.
    ///
    /// This dispatches the message, spawns any async commands, and processes
    /// any immediately available async results.
    pub fn dispatch(&mut self, msg: A::Message) {
        self.runtime.dispatch(msg);
        self.runtime.process_pending();
    }

    /// Dispatches multiple messages.
    pub fn dispatch_all(&mut self, messages: impl IntoIterator<Item = A::Message>) {
        for msg in messages {
            self.dispatch(msg);
        }
    }

    /// Returns a sender that can be used to send messages to the runtime.
    pub fn message_sender(&self) -> tokio::sync::mpsc::Sender<A::Message> {
        self.runtime.message_sender()
    }

    // -------------------------------------------------------------------------
    // Subscriptions
    // -------------------------------------------------------------------------

    /// Adds a subscription to the runtime.
    pub fn subscribe(&mut self, subscription: impl Subscription<A::Message>) {
        self.runtime.subscribe(subscription);
    }

    /// Adds multiple subscriptions to the runtime.
    pub fn subscribe_all(&mut self, subscriptions: Vec<BoxedSubscription<A::Message>>) {
        self.runtime.subscribe_all(subscriptions);
    }

    // -------------------------------------------------------------------------
    // Event Queue
    // -------------------------------------------------------------------------

    /// Returns a mutable reference to the event queue.
    pub fn events(&mut self) -> &mut EventQueue {
        self.runtime.events()
    }

    /// Queues a single event.
    pub fn push_event(&mut self, event: Event) {
        self.runtime.events().push(event);
    }

    /// Types a string as keyboard input.
    pub fn type_str(&mut self, s: &str) {
        self.runtime.events().type_str(s);
    }

    /// Simulates pressing Enter.
    pub fn enter(&mut self) {
        self.runtime.events().enter();
    }

    /// Simulates pressing Escape.
    pub fn escape(&mut self) {
        self.runtime.events().escape();
    }

    /// Simulates pressing Tab.
    pub fn tab(&mut self) {
        self.runtime.events().tab();
    }

    /// Simulates `Ctrl+<key>`.
    pub fn ctrl(&mut self, c: char) {
        self.runtime.events().ctrl(c);
    }

    /// Simulates a mouse click at the given position.
    pub fn click(&mut self, x: u16, y: u16) {
        self.runtime.events().click(x, y);
    }

    // -------------------------------------------------------------------------
    // Runtime Control
    // -------------------------------------------------------------------------

    /// Processes all pending events.
    pub fn process_events(&mut self) {
        self.runtime.process_all_events();
    }

    /// Runs a single tick of the application.
    pub fn tick(&mut self) -> io::Result<()> {
        self.runtime.tick()
    }

    /// Runs multiple ticks.
    pub fn run_ticks(&mut self, ticks: usize) -> io::Result<()> {
        self.runtime.run_ticks(ticks)
    }

    /// Renders the current state.
    pub fn render(&mut self) -> io::Result<()> {
        self.runtime.render()
    }

    /// Returns true if the runtime should quit.
    pub fn should_quit(&self) -> bool {
        self.runtime.should_quit()
    }

    /// Triggers a quit.
    pub fn quit(&mut self) {
        self.runtime.quit();
    }

    /// Returns the cancellation token.
    pub fn cancellation_token(&self) -> CancellationToken {
        self.runtime.cancellation_token()
    }

    // -------------------------------------------------------------------------
    // Content Queries
    // -------------------------------------------------------------------------

    /// Returns true if the screen contains the given text.
    pub fn contains_text(&self, needle: &str) -> bool {
        self.runtime.contains_text(needle)
    }

    /// Finds all positions of the given text.
    pub fn find_text(&self, needle: &str) -> Vec<Position> {
        self.runtime.find_text(needle)
    }

    /// Returns the content of a specific row.
    pub fn row(&self, y: u16) -> String {
        self.runtime.backend().row_content(y)
    }

    // -------------------------------------------------------------------------
    // Assertions
    // -------------------------------------------------------------------------

    /// Asserts that the screen contains the given text.
    ///
    /// # Panics
    ///
    /// Panics if the text is not found.
    pub fn assert_contains(&self, needle: &str) {
        if !self.contains_text(needle) {
            panic!(
                "Expected screen to contain '{}', but it was not found.\n\nScreen:\n{}",
                needle,
                self.screen()
            );
        }
    }

    /// Asserts that the screen does not contain the given text.
    ///
    /// # Panics
    ///
    /// Panics if the text is found.
    pub fn assert_not_contains(&self, needle: &str) {
        if self.contains_text(needle) {
            panic!(
                "Expected screen to NOT contain '{}', but it was found.\n\nScreen:\n{}",
                needle,
                self.screen()
            );
        }
    }
}

// Time control methods - only available during tests with tokio test-util
#[cfg(test)]
use std::time::Duration;

#[cfg(test)]
impl<A: App> AsyncTestHarness<A>
where
    A::Message: Send + Clone + 'static,
{
    /// Advances time by the specified duration.
    ///
    /// This only works when time is paused (e.g., with `#[tokio::test(start_paused = true)]`).
    /// After advancing time, all pending async work that was waiting for timers
    /// or delays will be processed.
    pub async fn advance_time(&mut self, duration: Duration) {
        // Advance in small increments to give spawned tasks a chance to wake
        // and process their timers
        let step = Duration::from_millis(10);
        let mut remaining = duration;

        while remaining > Duration::ZERO {
            let advance_by = remaining.min(step);
            tokio::time::advance(advance_by).await;

            // Yield to let spawned tasks run
            tokio::time::sleep(Duration::ZERO).await;
            tokio::task::yield_now().await;

            remaining = remaining.saturating_sub(advance_by);
        }

        // Final processing of any messages that arrived
        self.runtime.process_pending();
    }

    /// Sleeps for the specified duration.
    ///
    /// When time is paused, this immediately advances time without waiting.
    pub async fn sleep(&mut self, duration: Duration) {
        self.advance_time(duration).await;
    }

    /// Waits for a condition to become true, with a timeout.
    ///
    /// Returns true if the condition was met, false if it timed out.
    pub async fn wait_for<F>(&mut self, condition: F, timeout: Duration) -> bool
    where
        F: Fn(&A::State) -> bool,
    {
        let step = Duration::from_millis(10);
        let mut elapsed = Duration::ZERO;

        while elapsed < timeout {
            if condition(self.runtime.state()) {
                return true;
            }

            self.advance_time(step).await;
            elapsed += step;
        }

        condition(self.runtime.state())
    }

    /// Waits for the screen to contain the specified text.
    pub async fn wait_for_text(&mut self, needle: &str, timeout: Duration) -> bool {
        let step = Duration::from_millis(10);
        let mut elapsed = Duration::ZERO;

        while elapsed < timeout {
            self.runtime.render().ok();
            if self.contains_text(needle) {
                return true;
            }

            self.advance_time(step).await;
            elapsed += step;
        }

        self.runtime.render().ok();
        self.contains_text(needle)
    }
}

#[cfg(test)]
mod tests {
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
        let harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();
        assert_eq!(harness.state().count, 0);
    }

    #[test]
    fn test_async_harness_dispatch() {
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();

        harness.dispatch(TestMsg::Increment);
        assert_eq!(harness.state().count, 1);

        harness.dispatch(TestMsg::Increment);
        assert_eq!(harness.state().count, 2);
    }

    #[test]
    fn test_async_harness_dispatch_all() {
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();

        harness.dispatch_all(vec![
            TestMsg::Increment,
            TestMsg::Increment,
            TestMsg::Increment,
        ]);

        assert_eq!(harness.state().count, 3);
    }

    #[test]
    fn test_async_harness_render() {
        let mut harness = AsyncTestHarness::<TestApp>::new(40, 10).unwrap();
        harness.dispatch(TestMsg::Increment);
        harness.render().unwrap();

        assert!(harness.contains_text("Count: 1"));
    }

    #[test]
    fn test_async_harness_events() {
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();

        harness.type_str("hello");
        harness.enter();

        assert_eq!(harness.events().len(), 6);
    }

    #[test]
    fn test_async_harness_tick() {
        let mut harness = AsyncTestHarness::<TestApp>::new(40, 10).unwrap();
        harness.dispatch(TestMsg::Increment);
        harness.tick().unwrap();

        assert!(harness.contains_text("Count: 1"));
    }

    #[test]
    fn test_async_harness_quit() {
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();
        assert!(!harness.should_quit());

        harness.dispatch(TestMsg::Quit);
        harness.tick().unwrap();

        assert!(harness.should_quit());
    }

    #[test]
    fn test_async_harness_assert_contains() {
        let mut harness = AsyncTestHarness::<TestApp>::new(40, 10).unwrap();
        harness.render().unwrap();

        harness.assert_contains("Count: 0");
        harness.assert_not_contains("Unknown");
    }

    #[tokio::test(start_paused = true)]
    async fn test_async_harness_async_command() {
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();

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
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();

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
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();

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
        let mut harness = AsyncTestHarness::<TestApp>::new(40, 10).unwrap();

        // Increment count
        harness.dispatch(TestMsg::Increment);

        let success = harness
            .wait_for_text("Count: 1", Duration::from_secs(1))
            .await;

        assert!(success);
    }

    #[tokio::test(start_paused = true)]
    async fn test_async_harness_sleep() {
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();

        harness.dispatch(TestMsg::StartAsyncOp);
        harness.sleep(Duration::from_secs(5)).await;

        assert_eq!(harness.state().async_result, Some(42));
    }

    #[test]
    fn test_async_harness_screen() {
        let mut harness = AsyncTestHarness::<TestApp>::new(40, 10).unwrap();
        harness.render().unwrap();

        let screen = harness.screen();
        assert!(screen.contains("Count: 0"));
    }

    #[test]
    fn test_async_harness_row() {
        let mut harness = AsyncTestHarness::<TestApp>::new(40, 10).unwrap();
        harness.render().unwrap();

        let row = harness.row(0);
        assert!(row.contains("Count: 0"));
    }

    #[test]
    fn test_async_harness_find_text() {
        let mut harness = AsyncTestHarness::<TestApp>::new(40, 10).unwrap();
        harness.render().unwrap();

        let positions = harness.find_text("Count");
        assert!(!positions.is_empty());
    }

    #[test]
    fn test_async_harness_input_methods() {
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();

        harness.escape();
        harness.tab();
        harness.ctrl('c');
        harness.click(10, 20);
        harness.push_event(Event::char('x'));

        assert_eq!(harness.events().len(), 5);
    }

    #[test]
    fn test_async_harness_cancellation_token() {
        let harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();
        let token = harness.cancellation_token();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_async_harness_manual_quit() {
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();
        assert!(!harness.should_quit());

        harness.quit();
        assert!(harness.should_quit());
    }

    #[test]
    fn test_async_harness_with_config() {
        let config = AsyncRuntimeConfig::new()
            .tick_rate(Duration::from_millis(100))
            .with_history(5);

        let harness = AsyncTestHarness::<TestApp>::with_config(80, 24, config).unwrap();
        assert_eq!(harness.state().count, 0);
    }

    #[test]
    fn test_async_harness_process_events() {
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();
        harness.type_str("abc");
        harness.process_events();
        // Events processed (but TestApp doesn't handle key events)
    }

    #[test]
    fn test_async_harness_run_ticks() {
        let mut harness = AsyncTestHarness::<TestApp>::new(40, 10).unwrap();
        harness.dispatch(TestMsg::Increment);
        harness.run_ticks(3).unwrap();

        assert!(harness.contains_text("Count: 1"));
    }

    #[test]
    fn test_async_harness_state_mut() {
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();
        harness.state_mut().count = 42;
        assert_eq!(harness.state().count, 42);
    }

    #[test]
    fn test_async_harness_screen_ansi() {
        let mut harness = AsyncTestHarness::<TestApp>::new(40, 10).unwrap();
        harness.render().unwrap();

        let screen_ansi = harness.screen_ansi();
        assert!(screen_ansi.contains("Count: 0"));
    }

    #[test]
    fn test_async_harness_backend() {
        use ratatui::backend::Backend;
        let harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();
        let backend = harness.backend();
        assert_eq!(backend.size().unwrap().width, 80);
    }

    #[test]
    fn test_async_harness_backend_mut() {
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();
        let _backend = harness.backend_mut();
        // Just verify we can get a mutable reference
    }

    #[tokio::test]
    async fn test_async_harness_message_sender() {
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();
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

        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();
        let sub = TickSubscription::new(Duration::from_millis(10), || TestMsg::Increment);
        harness.subscribe(sub);
        // Just verify we can add a subscription
    }

    #[test]
    fn test_async_harness_subscribe_all() {
        use crate::app::{BoxedSubscription, TickSubscription};

        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();

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
        let mut harness = AsyncTestHarness::<TestApp>::new(40, 10).unwrap();

        // Don't dispatch anything that changes text to "Count: 5"
        let success = harness
            .wait_for_text("Count: 5", Duration::from_millis(100))
            .await;

        assert!(!success);
    }

    #[test]
    #[should_panic(expected = "Expected screen to contain 'MISSING'")]
    fn test_async_harness_assert_contains_panic() {
        let mut harness = AsyncTestHarness::<TestApp>::new(40, 10).unwrap();
        harness.render().unwrap();

        // This should panic because 'MISSING' is not on screen
        harness.assert_contains("MISSING");
    }

    #[test]
    #[should_panic(expected = "Expected screen to NOT contain 'Count'")]
    fn test_async_harness_assert_not_contains_panic() {
        let mut harness = AsyncTestHarness::<TestApp>::new(40, 10).unwrap();
        harness.render().unwrap();

        // This should panic because 'Count' is on screen
        harness.assert_not_contains("Count");
    }

    #[test]
    fn test_async_harness_events_direct() {
        let mut harness = AsyncTestHarness::<TestApp>::new(80, 24).unwrap();

        let events = harness.events();
        events.push(Event::char('a'));

        assert!(!harness.events().is_empty());
    }
}
