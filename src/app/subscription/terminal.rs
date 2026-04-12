use std::pin::Pin;

use tokio_stream::Stream;
use tokio_util::sync::CancellationToken;

use super::Subscription;
use crate::input::Event;

/// A subscription that reads terminal input events from crossterm.
///
/// This subscription uses crossterm's async event stream to read keyboard,
/// mouse, paste, focus, and resize events. Each event is passed through
/// a handler function that can optionally produce a message.
///
/// # Example
///
/// ```rust
/// use envision::app::TerminalEventSubscription;
/// use envision::input::{Event, Key};
///
/// let sub = TerminalEventSubscription::new(|event| {
///     match &event {
///         Event::Key(key) if key.key == Key::Char('q') => {
///             Some("quit".to_string())
///         }
///         Event::Key(key) if key.key == Key::Up => {
///             Some("up".to_string())
///         }
///         _ => None,
///     }
/// });
/// ```
pub struct TerminalEventSubscription<M, F>
where
    F: Fn(Event) -> Option<M> + Send + 'static,
{
    pub(crate) event_handler: F,
    _phantom: std::marker::PhantomData<M>,
}

impl<M, F> TerminalEventSubscription<M, F>
where
    F: Fn(Event) -> Option<M> + Send + 'static,
{
    /// Creates a new terminal event subscription.
    pub fn new(event_handler: F) -> Self {
        Self {
            event_handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<M, F> Subscription<M> for TerminalEventSubscription<M, F>
where
    M: Send + 'static,
    F: Fn(Event) -> Option<M> + Send + 'static,
{
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>> {
        use crossterm::event::EventStream;
        use tokio_stream::StreamExt;

        let handler = self.event_handler;

        Box::pin(async_stream::stream! {
            let mut reader = EventStream::new();
            loop {
                tokio::select! {
                    maybe_event = reader.next() => {
                        match maybe_event {
                            Some(Ok(ct_event)) => {
                                if let Some(event) = crate::input::convert::from_crossterm_event(ct_event) {
                                    if let Some(msg) = (handler)(event) {
                                        yield msg;
                                    }
                                }
                            }
                            Some(Err(_)) => break,
                            None => break,
                        }
                    }
                    _ = cancel.cancelled() => break,
                }
            }
        })
    }
}

/// Creates a terminal event subscription.
///
/// This is a convenience function for creating a [`TerminalEventSubscription`].
///
/// # Example
///
/// ```rust
/// use envision::app::terminal_events;
/// use envision::input::{Event, Key};
///
/// let sub = terminal_events(|event| {
///     if let Event::Key(key) = &event {
///         if key.key == Key::Char('q') {
///             return Some("quit".to_string());
///         }
///     }
///     None
/// });
/// ```
pub fn terminal_events<M, F>(handler: F) -> TerminalEventSubscription<M, F>
where
    F: Fn(Event) -> Option<M> + Send + 'static,
{
    TerminalEventSubscription::new(handler)
}
