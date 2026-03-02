//! Subscriptions for long-running async operations in TEA applications.
//!
//! Subscriptions are async streams that produce messages over time. They're useful
//! for timers, websockets, file watchers, and other ongoing async operations.
//!
//! # Example
//!
//! ```rust
//! use envision::app::{Subscription, TickSubscription};
//! use std::time::Duration;
//!
//! // Create a subscription that fires every second
//! let tick = TickSubscription::new(Duration::from_secs(1), || "tick");
//! ```

mod batch;
mod combinators;
mod core;
mod ext;
mod interval;
mod terminal;

pub use batch::{batch, BatchSubscription};
pub use combinators::{
    DebounceSubscription, FilterSubscription, MappedSubscription, TakeSubscription,
    ThrottleSubscription,
};
pub use core::{
    tick, BoxedSubscription, ChannelSubscription, StreamSubscription, Subscription,
    TickSubscription, TickSubscriptionBuilder, TimerSubscription,
};
pub use ext::SubscriptionExt;
pub use interval::{interval_immediate, IntervalImmediateBuilder, IntervalImmediateSubscription};
pub use terminal::{terminal_events, TerminalEventSubscription};

#[cfg(test)]
pub(crate) use tokio::sync::mpsc;
#[cfg(test)]
pub(crate) use tokio_util::sync::CancellationToken;

#[cfg(test)]
mod tests;
