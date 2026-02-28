use super::*;
use std::time::Duration;
use tokio_stream::StreamExt;

#[derive(Clone, Debug, PartialEq)]
enum TestMsg {
    Tick,
    Timer,
    Value(i32),
    Quit,
}

#[tokio::test]
async fn test_tick_subscription() {
    let cancel = CancellationToken::new();
    let sub = Box::new(TickSubscription::new(Duration::from_millis(10), || {
        TestMsg::Tick
    }));

    let mut stream = sub.into_stream(cancel.clone());

    // Get first tick
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Tick));

    // Cancel and verify stream ends
    cancel.cancel();
}

#[tokio::test]
async fn test_tick_builder() {
    let cancel = CancellationToken::new();
    let sub = Box::new(tick(Duration::from_millis(10)).with_message(|| TestMsg::Tick));

    let mut stream = sub.into_stream(cancel.clone());
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Tick));

    cancel.cancel();
}

#[tokio::test]
async fn test_timer_subscription() {
    let cancel = CancellationToken::new();
    let sub = Box::new(TimerSubscription::after(
        Duration::from_millis(10),
        TestMsg::Timer,
    ));

    let mut stream = sub.into_stream(cancel);

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Timer));

    // Timer should only fire once
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_timer_cancellation() {
    let cancel = CancellationToken::new();
    let sub = Box::new(TimerSubscription::after(
        Duration::from_secs(10),
        TestMsg::Timer,
    ));

    let mut stream = sub.into_stream(cancel.clone());

    // Cancel before timer fires
    cancel.cancel();

    // Stream should end
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_channel_subscription() {
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel(10);
    let sub = Box::new(ChannelSubscription::new(rx));

    let mut stream = sub.into_stream(cancel.clone());

    // Send messages
    tx.send(TestMsg::Value(1)).await.unwrap();
    tx.send(TestMsg::Value(2)).await.unwrap();

    // Receive messages
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(2)));

    // Drop sender to close channel
    drop(tx);

    // Stream should end
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_stream_subscription() {
    let cancel = CancellationToken::new();
    let values = vec![TestMsg::Value(1), TestMsg::Value(2), TestMsg::Value(3)];
    let inner_stream = tokio_stream::iter(values);
    let sub = Box::new(StreamSubscription::new(inner_stream));

    let mut stream = sub.into_stream(cancel);

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(2)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(3)));

    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_mapped_subscription() {
    let cancel = CancellationToken::new();
    let inner = TickSubscription::new(Duration::from_millis(10), || 42i32);
    let sub = Box::new(inner.map(TestMsg::Value));

    let mut stream = sub.into_stream(cancel.clone());

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(42)));

    cancel.cancel();
}

#[tokio::test]
async fn test_batch_subscription() {
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel(10);

    let timer = Box::new(TimerSubscription::after(
        Duration::from_millis(5),
        TestMsg::Timer,
    )) as BoxedSubscription<TestMsg>;
    let channel = Box::new(ChannelSubscription::new(rx)) as BoxedSubscription<TestMsg>;

    let sub = Box::new(batch(vec![timer, channel]));
    let mut stream = sub.into_stream(cancel.clone());

    // Send a channel message
    tx.send(TestMsg::Value(1)).await.unwrap();

    // Collect messages (order may vary)
    let mut received = Vec::new();
    for _ in 0..2 {
        if let Some(msg) = stream.next().await {
            received.push(msg);
        }
    }

    assert!(received.contains(&TestMsg::Timer));
    assert!(received.contains(&TestMsg::Value(1)));

    cancel.cancel();
}

#[test]
fn test_tick_builder_every() {
    let builder = TickSubscriptionBuilder::every(Duration::from_secs(1));
    let sub = builder.with_message(|| TestMsg::Tick);
    assert_eq!(sub.interval, Duration::from_secs(1));
}

#[test]
fn test_timer_after() {
    let timer = TimerSubscription::after(Duration::from_secs(5), TestMsg::Timer);
    assert_eq!(timer.delay, Duration::from_secs(5));
    assert_eq!(timer.message, TestMsg::Timer);
}

#[tokio::test]
async fn test_interval_immediate_subscription() {
    let cancel = CancellationToken::new();
    let sub = Box::new(IntervalImmediateSubscription::new(
        Duration::from_millis(100),
        || TestMsg::Tick,
    ));

    let mut stream = sub.into_stream(cancel.clone());

    // Should fire immediately without waiting for interval
    let start = std::time::Instant::now();
    let msg = stream.next().await;
    let elapsed = start.elapsed();

    assert_eq!(msg, Some(TestMsg::Tick));
    // First message should be immediate (less than the interval)
    assert!(
        elapsed < Duration::from_millis(50),
        "First message should be immediate, took {:?}",
        elapsed
    );

    cancel.cancel();
}

#[tokio::test]
async fn test_interval_immediate_builder() {
    let cancel = CancellationToken::new();
    let sub =
        Box::new(interval_immediate(Duration::from_millis(100)).with_message(|| TestMsg::Tick));

    let mut stream = sub.into_stream(cancel.clone());

    // Should fire immediately
    let start = std::time::Instant::now();
    let msg = stream.next().await;
    let elapsed = start.elapsed();

    assert_eq!(msg, Some(TestMsg::Tick));
    assert!(elapsed < Duration::from_millis(50));

    cancel.cancel();
}

#[tokio::test(start_paused = true)]
async fn test_interval_immediate_vs_tick() {
    // Both subscriptions produce their first message, but IntervalImmediate
    // yields before any async machinery while Tick goes through interval.tick().
    let cancel1 = CancellationToken::new();
    let cancel2 = CancellationToken::new();

    let immediate = Box::new(IntervalImmediateSubscription::new(
        Duration::from_millis(50),
        || TestMsg::Tick,
    ));
    let regular = Box::new(TickSubscription::new(Duration::from_millis(50), || {
        TestMsg::Tick
    }));

    let mut immediate_stream = immediate.into_stream(cancel1.clone());
    let mut regular_stream = regular.into_stream(cancel2.clone());

    // Both produce their first tick
    let immediate_first = immediate_stream.next().await;
    assert!(immediate_first.is_some());

    let regular_first = regular_stream.next().await;
    assert!(regular_first.is_some());

    // After the first tick, both require waiting for the interval
    // Advance time by the interval duration to get the second tick
    tokio::time::advance(Duration::from_millis(50)).await;

    let immediate_second = immediate_stream.next().await;
    assert!(immediate_second.is_some());

    let regular_second = regular_stream.next().await;
    assert!(regular_second.is_some());

    cancel1.cancel();
    cancel2.cancel();
}

#[tokio::test]
async fn test_filter_subscription() {
    let cancel = CancellationToken::new();
    let values = vec![
        TestMsg::Value(1),
        TestMsg::Value(2),
        TestMsg::Value(3),
        TestMsg::Value(4),
        TestMsg::Value(5),
    ];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(FilterSubscription::new(
        inner,
        |msg| matches!(msg, TestMsg::Value(n) if *n % 2 == 0),
    ));

    let mut stream = sub.into_stream(cancel);

    // Should only get even values
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(2)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(4)));

    // Stream should end
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_filter_subscription_all_filtered() {
    let cancel = CancellationToken::new();
    let values = vec![TestMsg::Value(1), TestMsg::Value(3), TestMsg::Value(5)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(FilterSubscription::new(
        inner,
        |msg| matches!(msg, TestMsg::Value(n) if *n % 2 == 0),
    ));

    let mut stream = sub.into_stream(cancel);

    // All values are odd, so nothing should pass through
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_filter_subscription_none_filtered() {
    let cancel = CancellationToken::new();
    let values = vec![TestMsg::Value(2), TestMsg::Value(4)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(FilterSubscription::new(inner, |_| true));

    let mut stream = sub.into_stream(cancel);

    // All values should pass through
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(2)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(4)));

    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_take_subscription() {
    let cancel = CancellationToken::new();
    let values = vec![
        TestMsg::Value(1),
        TestMsg::Value(2),
        TestMsg::Value(3),
        TestMsg::Value(4),
        TestMsg::Value(5),
    ];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(TakeSubscription::new(inner, 3));

    let mut stream = sub.into_stream(cancel);

    // Should only get first 3 values
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(2)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(3)));

    // Stream should end after 3
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_take_subscription_zero() {
    let cancel = CancellationToken::new();
    let values = vec![TestMsg::Value(1), TestMsg::Value(2)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(TakeSubscription::new(inner, 0));

    let mut stream = sub.into_stream(cancel);

    // Should get nothing
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_take_subscription_more_than_available() {
    let cancel = CancellationToken::new();
    let values = vec![TestMsg::Value(1), TestMsg::Value(2)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(TakeSubscription::new(inner, 100));

    let mut stream = sub.into_stream(cancel);

    // Should get all available values then end
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(2)));

    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_take_one() {
    let cancel = CancellationToken::new();
    let values = vec![TestMsg::Value(1), TestMsg::Value(2), TestMsg::Value(3)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(TakeSubscription::new(inner, 1));

    let mut stream = sub.into_stream(cancel);

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_debounce_subscription() {
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel(10);
    let inner = ChannelSubscription::new(rx);
    let sub = Box::new(DebounceSubscription::new(inner, Duration::from_millis(50)));

    let mut stream = sub.into_stream(cancel.clone());

    // Send multiple messages quickly (should be debounced to just the last one)
    tx.send(TestMsg::Value(1)).await.unwrap();
    tx.send(TestMsg::Value(2)).await.unwrap();
    tx.send(TestMsg::Value(3)).await.unwrap();

    // Give debounce time to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Should only get the last value
    // Use a longer timeout to handle coverage instrumentation overhead
    let msg = tokio::time::timeout(Duration::from_millis(500), stream.next()).await;
    assert_eq!(msg.unwrap(), Some(TestMsg::Value(3)));

    // Close channel
    drop(tx);

    // Stream should end
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_debounce_emits_pending_on_stream_end() {
    let cancel = CancellationToken::new();
    let values = vec![TestMsg::Value(1), TestMsg::Value(2)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(DebounceSubscription::new(inner, Duration::from_secs(10)));

    let mut stream = sub.into_stream(cancel);

    // Even with long debounce, pending message should emit when stream ends
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(2)));

    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_debounce_with_slow_messages() {
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel(10);
    let inner = ChannelSubscription::new(rx);
    // Short debounce window
    let sub = Box::new(DebounceSubscription::new(inner, Duration::from_millis(20)));

    let mut stream = sub.into_stream(cancel.clone());

    // Send first message
    tx.send(TestMsg::Value(1)).await.unwrap();
    // Wait longer than debounce
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Should get first message
    let msg = tokio::time::timeout(Duration::from_millis(50), stream.next()).await;
    assert_eq!(msg.unwrap(), Some(TestMsg::Value(1)));

    // Send second message
    tx.send(TestMsg::Value(2)).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Should get second message
    let msg = tokio::time::timeout(Duration::from_millis(50), stream.next()).await;
    assert_eq!(msg.unwrap(), Some(TestMsg::Value(2)));

    cancel.cancel();
}

#[tokio::test]
async fn test_debounce_cancellation() {
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel(10);
    let inner = ChannelSubscription::new(rx);
    let sub = Box::new(DebounceSubscription::new(inner, Duration::from_secs(10)));

    let mut stream = sub.into_stream(cancel.clone());

    // Send a message (won't emit due to long debounce)
    tx.send(TestMsg::Value(1)).await.unwrap();

    // Cancel immediately
    cancel.cancel();

    // Stream should end without emitting
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_throttle_subscription() {
    let cancel = CancellationToken::new();
    let values = vec![
        TestMsg::Value(1),
        TestMsg::Value(2),
        TestMsg::Value(3),
        TestMsg::Value(4),
        TestMsg::Value(5),
    ];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    // Very long throttle - should only get the first message
    let sub = Box::new(ThrottleSubscription::new(inner, Duration::from_secs(10)));

    let mut stream = sub.into_stream(cancel);

    // Should get first message immediately (throttle allows first through)
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    // Stream ends (all others were throttled)
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_throttle_allows_spaced_messages() {
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel(10);
    let inner = ChannelSubscription::new(rx);
    let sub = Box::new(ThrottleSubscription::new(inner, Duration::from_millis(20)));

    let mut stream = sub.into_stream(cancel.clone());

    // First message - should pass
    tx.send(TestMsg::Value(1)).await.unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    let msg = tokio::time::timeout(Duration::from_millis(50), stream.next()).await;
    assert_eq!(msg.unwrap(), Some(TestMsg::Value(1)));

    // Wait longer than throttle duration
    tokio::time::sleep(Duration::from_millis(30)).await;

    // Second message after throttle period - should pass
    tx.send(TestMsg::Value(2)).await.unwrap();
    let msg = tokio::time::timeout(Duration::from_millis(50), stream.next()).await;
    assert_eq!(msg.unwrap(), Some(TestMsg::Value(2)));

    cancel.cancel();
}

#[tokio::test]
async fn test_throttle_drops_rapid_messages() {
    let cancel = CancellationToken::new();
    // Use a finite stream of values that arrive "instantly"
    let values = vec![
        TestMsg::Value(1),
        TestMsg::Value(2),
        TestMsg::Value(3),
        TestMsg::Value(4),
        TestMsg::Value(5),
    ];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    // With a long throttle, only the first message should pass
    let sub = Box::new(ThrottleSubscription::new(inner, Duration::from_millis(100)));

    let mut stream = sub.into_stream(cancel);

    // Should get first message (allowed through)
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    // Stream ends (all others 2,3,4,5 were throttled/dropped)
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_throttle_zero_duration() {
    let cancel = CancellationToken::new();
    let values = vec![TestMsg::Value(1), TestMsg::Value(2), TestMsg::Value(3)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    // Zero throttle - all messages should pass
    let sub = Box::new(ThrottleSubscription::new(inner, Duration::ZERO));

    let mut stream = sub.into_stream(cancel);

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(2)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(3)));

    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[test]
fn test_terminal_event_subscription_creation() {
    use crossterm::event::{Event, KeyCode, KeyEvent};

    // Test that we can create a TerminalEventSubscription
    let _sub = TerminalEventSubscription::new(|event| {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            ..
        }) = event
        {
            Some(TestMsg::Quit)
        } else {
            None
        }
    });

    // Test the convenience function
    let _sub2 = terminal_events(|event| {
        if let Event::Key(KeyEvent {
            code: KeyCode::Enter,
            ..
        }) = event
        {
            Some(TestMsg::Tick)
        } else {
            None
        }
    });
}

#[test]
fn test_terminal_event_handler_filters_events() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    // Create handler that only responds to 'q'
    let handler = |event: Event| -> Option<TestMsg> {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            ..
        }) = event
        {
            Some(TestMsg::Quit)
        } else {
            None
        }
    };

    // Test q key
    let q_event = Event::Key(KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    });
    assert_eq!(handler(q_event), Some(TestMsg::Quit));

    // Test other key (should be None)
    let a_event = Event::Key(KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    });
    assert_eq!(handler(a_event), None);

    // Test resize event (should be None)
    let resize_event = Event::Resize(80, 24);
    assert_eq!(handler(resize_event), None);
}

#[test]
fn test_terminal_event_handler_with_modifiers() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    // Create handler that responds to Ctrl+C
    let handler = |event: Event| -> Option<TestMsg> {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers,
            ..
        }) = event
        {
            if modifiers.contains(KeyModifiers::CONTROL) {
                Some(TestMsg::Quit)
            } else {
                None
            }
        } else {
            None
        }
    };

    // Test Ctrl+C
    let ctrl_c = Event::Key(KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    });
    assert_eq!(handler(ctrl_c), Some(TestMsg::Quit));

    // Test plain 'c' (should be None)
    let plain_c = Event::Key(KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    });
    assert_eq!(handler(plain_c), None);
}

#[test]
fn test_terminal_event_handler_resize() {
    use crossterm::event::Event;

    #[derive(Debug, Clone, PartialEq)]
    enum ResizeMsg {
        Resize(u16, u16),
    }

    let handler = |event: Event| -> Option<ResizeMsg> {
        if let Event::Resize(width, height) = event {
            Some(ResizeMsg::Resize(width, height))
        } else {
            None
        }
    };

    let resize_event = Event::Resize(120, 40);
    assert_eq!(handler(resize_event), Some(ResizeMsg::Resize(120, 40)));

    // Key event should be None
    let key_event = Event::Key(crossterm::event::KeyEvent {
        code: crossterm::event::KeyCode::Enter,
        modifiers: crossterm::event::KeyModifiers::empty(),
        kind: crossterm::event::KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    });
    assert_eq!(handler(key_event), None);
}

// Note: We can't test TerminalEventSubscription::into_stream in unit tests
// because crossterm's EventStream requires a real terminal to be attached.
// The handler logic is tested through the test_terminal_event_* tests above
// which verify the event handling works correctly.

#[derive(Clone, Debug, PartialEq)]
enum TestMsgWithQuit {
    Quit,
    Key(char),
}

#[test]
fn test_terminal_events_convenience_function() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    let sub = terminal_events(|event: Event| -> Option<TestMsgWithQuit> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => Some(TestMsgWithQuit::Quit),
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => Some(TestMsgWithQuit::Key(c)),
            _ => None,
        }
    });

    // Verify the handler works correctly by testing it directly
    let q_event = Event::Key(KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    });
    assert_eq!((sub.event_handler)(q_event), Some(TestMsgWithQuit::Quit));
}

// Tests for SubscriptionExt fluent methods

#[tokio::test]
async fn test_subscription_ext_filter() {
    let cancel = CancellationToken::new();
    let values = vec![
        TestMsg::Value(1),
        TestMsg::Value(2),
        TestMsg::Value(3),
        TestMsg::Value(4),
    ];
    let inner = StreamSubscription::new(tokio_stream::iter(values));

    // Use fluent filter method
    let sub = Box::new(inner.filter(|msg| matches!(msg, TestMsg::Value(n) if *n % 2 == 0)));

    let mut stream = sub.into_stream(cancel);

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(2)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(4)));

    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_subscription_ext_take() {
    let cancel = CancellationToken::new();
    let values = vec![
        TestMsg::Value(1),
        TestMsg::Value(2),
        TestMsg::Value(3),
        TestMsg::Value(4),
    ];
    let inner = StreamSubscription::new(tokio_stream::iter(values));

    // Use fluent take method
    let sub = Box::new(inner.take(2));

    let mut stream = sub.into_stream(cancel);

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(2)));

    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_subscription_ext_debounce() {
    let cancel = CancellationToken::new();
    let values = vec![TestMsg::Value(1), TestMsg::Value(2)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));

    // Use fluent debounce method
    let sub = Box::new(inner.debounce(Duration::from_secs(10)));

    let mut stream = sub.into_stream(cancel);

    // Should emit pending on stream end
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(2)));

    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_subscription_ext_throttle() {
    let cancel = CancellationToken::new();
    let values = vec![TestMsg::Value(1), TestMsg::Value(2), TestMsg::Value(3)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));

    // Use fluent throttle method with long duration
    let sub = Box::new(inner.throttle(Duration::from_secs(10)));

    let mut stream = sub.into_stream(cancel);

    // Only first should pass
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_subscription_ext_chaining() {
    let cancel = CancellationToken::new();
    let values = vec![
        TestMsg::Value(1),
        TestMsg::Value(2),
        TestMsg::Value(3),
        TestMsg::Value(4),
        TestMsg::Value(5),
        TestMsg::Value(6),
    ];
    let inner = StreamSubscription::new(tokio_stream::iter(values));

    // Chain multiple extension methods
    let sub = Box::new(
        inner
            .filter(|msg| matches!(msg, TestMsg::Value(n) if *n % 2 == 0))
            .take(2),
    );

    let mut stream = sub.into_stream(cancel);

    // Should filter to even (2, 4, 6) then take 2 (2, 4)
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(2)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(4)));

    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_subscription_ext_map_and_filter() {
    let cancel = CancellationToken::new();
    let inner = TickSubscription::new(Duration::from_millis(10), || 42i32);

    // Map then filter
    let sub = Box::new(
        inner
            .map(TestMsg::Value)
            .filter(|msg| matches!(msg, TestMsg::Value(n) if *n > 0))
            .take(1),
    );

    let mut stream = sub.into_stream(cancel.clone());

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(42)));

    cancel.cancel();
}

#[tokio::test]
async fn test_subscription_ext_filter_map_take() {
    let cancel = CancellationToken::new();
    let values = vec![1i32, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let inner = StreamSubscription::new(tokio_stream::iter(values));

    // Filter, map, then take
    let sub = Box::new(
        inner
            .filter(|n| n % 2 == 0) // Keep even: 2, 4, 6, 8, 10
            .map(|n| TestMsg::Value(n * 10)) // Multiply by 10
            .take(3), // Take first 3
    );

    let mut stream = sub.into_stream(cancel);

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(20)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(40)));

    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(60)));

    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_empty_batch_subscription() {
    let cancel = CancellationToken::new();
    let sub = Box::new(batch::<TestMsg>(vec![]));

    let mut stream = sub.into_stream(cancel);

    // Empty batch should end immediately
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_channel_subscription_cancellation() {
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel(10);
    let sub = Box::new(ChannelSubscription::new(rx));

    let mut stream = sub.into_stream(cancel.clone());

    // Send a message
    tx.send(TestMsg::Value(1)).await.unwrap();
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    // Cancel the subscription
    cancel.cancel();

    // Stream should end
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_stream_subscription_cancellation() {
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel(10);
    let receiver_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let sub = Box::new(StreamSubscription::new(receiver_stream));

    let mut stream = sub.into_stream(cancel.clone());

    // Send a message
    tx.send(TestMsg::Value(1)).await.unwrap();
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    // Cancel the subscription
    cancel.cancel();

    // Stream should end
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_interval_immediate_cancellation() {
    let cancel = CancellationToken::new();
    let sub = Box::new(IntervalImmediateSubscription::new(
        Duration::from_millis(10),
        || TestMsg::Tick,
    ));

    let mut stream = sub.into_stream(cancel.clone());

    // Get the immediate first message
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Tick));

    // Get the second message after interval
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Tick));

    // Cancel the subscription
    cancel.cancel();

    // Yield to let cancellation propagate
    tokio::task::yield_now().await;

    // Stream should eventually end (might get one more buffered message on some platforms)
    let mut ended = false;
    for _ in 0..3 {
        let msg = stream.next().await;
        if msg.is_none() {
            ended = true;
            break;
        }
    }
    assert!(ended, "Stream should have ended after cancellation");
}

#[tokio::test]
async fn test_debounce_empty_stream() {
    let cancel = CancellationToken::new();
    let values: Vec<TestMsg> = vec![];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(DebounceSubscription::new(inner, Duration::from_millis(50)));

    let mut stream = sub.into_stream(cancel);

    // Empty stream should end immediately with no pending message
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_mapped_subscription_empty_stream() {
    let cancel = CancellationToken::new();
    let values: Vec<i32> = vec![];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(MappedSubscription::new(inner, TestMsg::Value));

    let mut stream = sub.into_stream(cancel);

    // Mapped empty stream should end immediately
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[test]
fn test_filter_subscription_new() {
    let values = vec![TestMsg::Value(1)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let _sub = FilterSubscription::new(inner, |_| true);
    // Construction test - subscription created successfully
}

#[test]
fn test_take_subscription_new() {
    let values = vec![TestMsg::Value(1)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = TakeSubscription::new(inner, 5);
    assert_eq!(sub.count, 5);
}

#[test]
fn test_debounce_subscription_new() {
    let values = vec![TestMsg::Value(1)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = DebounceSubscription::new(inner, Duration::from_millis(100));
    assert_eq!(sub.duration, Duration::from_millis(100));
}

#[test]
fn test_throttle_subscription_new() {
    let values = vec![TestMsg::Value(1)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = ThrottleSubscription::new(inner, Duration::from_millis(200));
    assert_eq!(sub.duration, Duration::from_millis(200));
}

#[test]
fn test_mapped_subscription_new() {
    let values = vec![42i32];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let _sub = MappedSubscription::new(inner, TestMsg::Value);
    // Construction test - subscription created successfully
}

#[test]
fn test_batch_subscription_new() {
    let subs: Vec<BoxedSubscription<TestMsg>> = vec![];
    let sub = BatchSubscription::new(subs);
    assert!(sub.subscriptions.is_empty());
}

#[test]
fn test_interval_immediate_builder_every() {
    let builder = IntervalImmediateBuilder::every(Duration::from_secs(2));
    let sub = builder.with_message(|| TestMsg::Tick);
    assert_eq!(sub.interval, Duration::from_secs(2));
}

#[tokio::test]
async fn test_tick_cancellation() {
    let cancel = CancellationToken::new();
    let sub = Box::new(TickSubscription::new(Duration::from_millis(10), || {
        TestMsg::Tick
    }));

    let mut stream = sub.into_stream(cancel.clone());

    // Get first tick
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Tick));

    // Cancel before next tick
    cancel.cancel();

    // Yield to let cancellation propagate
    tokio::task::yield_now().await;

    // Stream should eventually end (might get one more buffered message on some platforms)
    let mut ended = false;
    for _ in 0..3 {
        let msg = stream.next().await;
        if msg.is_none() {
            ended = true;
            break;
        }
    }
    assert!(ended, "Stream should have ended after cancellation");
}

#[tokio::test]
async fn test_filter_subscription_empty_input() {
    let cancel = CancellationToken::new();
    let values: Vec<TestMsg> = vec![];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(FilterSubscription::new(inner, |_| true));

    let mut stream = sub.into_stream(cancel);

    // Empty input ends immediately
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_throttle_empty_stream() {
    let cancel = CancellationToken::new();
    let values: Vec<TestMsg> = vec![];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(ThrottleSubscription::new(inner, Duration::from_millis(50)));

    let mut stream = sub.into_stream(cancel);

    // Empty stream ends immediately
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_take_empty_stream() {
    let cancel = CancellationToken::new();
    let values: Vec<TestMsg> = vec![];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(TakeSubscription::new(inner, 10));

    let mut stream = sub.into_stream(cancel);

    // Empty stream ends immediately
    let msg = stream.next().await;
    assert_eq!(msg, None);
}
