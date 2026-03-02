use std::time::Duration;

use super::combinators::{
    DebounceSubscription, FilterSubscription, MappedSubscription, TakeSubscription,
    ThrottleSubscription,
};
use super::Subscription;

/// Extension trait for subscriptions.
///
/// Provides fluent methods for composing and transforming subscriptions.
///
/// # Example
///
/// ```rust
/// use envision::app::{SubscriptionExt, tick};
/// use std::time::Duration;
///
/// // Create a tick subscription with filtering and limiting
/// let sub = tick(Duration::from_millis(100))
///     .with_message(|| 42i32)
///     .filter(|n| *n > 0)
///     .take(10)
///     .throttle(Duration::from_millis(200));
/// ```
pub trait SubscriptionExt<M>: Subscription<M> + Sized {
    /// Maps the messages of this subscription.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::{SubscriptionExt, tick};
    /// use std::time::Duration;
    ///
    /// let sub = tick(Duration::from_secs(1))
    ///     .with_message(|| 42)
    ///     .map(|n| format!("value: {}", n));
    /// ```
    fn map<N, F>(self, f: F) -> MappedSubscription<M, N, F, Self>
    where
        F: Fn(M) -> N + Send + 'static,
    {
        MappedSubscription::new(self, f)
    }

    /// Filters messages from this subscription.
    ///
    /// Only messages for which the predicate returns `true` are emitted.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::{SubscriptionExt, tick};
    /// use std::time::Duration;
    ///
    /// let sub = tick(Duration::from_secs(1))
    ///     .with_message(|| 42i32)
    ///     .filter(|n| *n > 0);
    /// ```
    fn filter<P>(self, predicate: P) -> FilterSubscription<M, Self, P>
    where
        P: Fn(&M) -> bool + Send + 'static,
    {
        FilterSubscription::new(self, predicate)
    }

    /// Takes only the first N messages from this subscription.
    ///
    /// After N messages, the subscription ends.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::{SubscriptionExt, tick};
    /// use std::time::Duration;
    ///
    /// let sub = tick(Duration::from_secs(1))
    ///     .with_message(|| "tick")
    ///     .take(5);
    /// ```
    fn take(self, count: usize) -> TakeSubscription<M, Self> {
        TakeSubscription::new(self, count)
    }

    /// Debounces messages from this subscription.
    ///
    /// Only emits a message after a quiet period has passed. If a new message
    /// arrives before the quiet period expires, the timer resets. Only the most
    /// recent message is emitted.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::{SubscriptionExt, tick};
    /// use std::time::Duration;
    ///
    /// // Only emit after 300ms of no new messages
    /// let sub = tick(Duration::from_millis(100))
    ///     .with_message(|| "tick")
    ///     .debounce(Duration::from_millis(300));
    /// ```
    fn debounce(self, duration: Duration) -> DebounceSubscription<M, Self> {
        DebounceSubscription::new(self, duration)
    }

    /// Throttles messages from this subscription.
    ///
    /// Limits the rate of message emission. At most one message is emitted
    /// per duration. The first message passes immediately, subsequent messages
    /// are dropped until the duration has passed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::{SubscriptionExt, tick};
    /// use std::time::Duration;
    ///
    /// // Emit at most once every 100ms
    /// let sub = tick(Duration::from_millis(50))
    ///     .with_message(|| "tick")
    ///     .throttle(Duration::from_millis(100));
    /// ```
    fn throttle(self, duration: Duration) -> ThrottleSubscription<M, Self> {
        ThrottleSubscription::new(self, duration)
    }
}

impl<M, S: Subscription<M>> SubscriptionExt<M> for S {}
