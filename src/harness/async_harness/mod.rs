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
        self.runtime.display()
    }

    /// Returns the captured output with ANSI colors.
    pub fn screen_ansi(&self) -> String {
        self.runtime.display_ansi()
    }

    /// Returns the cell at the given position, or `None` if out of bounds.
    ///
    /// Use this to assert on cell styling:
    /// ```ignore
    /// let cell = harness.cell_at(5, 3).unwrap();
    /// assert_eq!(cell.fg, SerializableColor::Green);
    /// ```
    pub fn cell_at(&self, x: u16, y: u16) -> Option<&crate::backend::EnhancedCell> {
        self.runtime.backend().cell(x, y)
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
mod tests;
