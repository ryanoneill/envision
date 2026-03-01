use super::*;

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
