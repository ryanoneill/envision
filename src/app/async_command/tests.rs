use super::*;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
enum TestMsg {
    A,
    B,
    AsyncResult(i32),
}

#[test]
fn test_async_handler_sync_message() {
    let mut handler = AsyncCommandHandler::new();
    handler.execute(Command::message(TestMsg::A));

    let messages = handler.take_messages();
    assert_eq!(messages, vec![TestMsg::A]);
    assert!(!handler.has_pending_futures());
}

#[test]
fn test_async_handler_sync_batch() {
    let mut handler = AsyncCommandHandler::new();
    handler.execute(Command::batch([TestMsg::A, TestMsg::B]));

    let messages = handler.take_messages();
    assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
}

#[test]
fn test_async_handler_quit() {
    let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
    assert!(!handler.should_quit());

    handler.execute(Command::quit());
    assert!(handler.should_quit());
}

#[test]
fn test_async_handler_callback() {
    let mut handler = AsyncCommandHandler::new();
    handler.execute(Command::perform(|| Some(TestMsg::A)));

    let messages = handler.take_messages();
    assert_eq!(messages, vec![TestMsg::A]);
}

#[test]
fn test_async_handler_async_command_pending() {
    let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();

    handler.execute(Command::perform_async(async {
        Some(TestMsg::AsyncResult(42))
    }));

    assert!(handler.has_pending_futures());
    assert_eq!(handler.pending_future_count(), 1);
    assert!(handler.take_messages().is_empty());
}

#[tokio::test]
async fn test_async_handler_spawn_and_receive() {
    let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
    let (msg_tx, mut msg_rx) = mpsc::channel(10);
    let (err_tx, _err_rx) = mpsc::channel(10);
    let cancel = CancellationToken::new();

    handler.execute(Command::perform_async(async {
        Some(TestMsg::AsyncResult(42))
    }));

    handler.spawn_pending(msg_tx, err_tx, cancel);
    assert!(!handler.has_pending_futures());

    // Receive the message from the spawned task
    let msg = msg_rx.recv().await.expect("Should receive message");
    assert_eq!(msg, TestMsg::AsyncResult(42));
}

#[tokio::test]
async fn test_async_handler_spawn_none_result() {
    let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
    let (msg_tx, mut msg_rx) = mpsc::channel(10);
    let (err_tx, _err_rx) = mpsc::channel(10);
    let cancel = CancellationToken::new();

    handler.execute(Command::perform_async(async { None }));

    handler.spawn_pending(msg_tx, err_tx, cancel);

    // Give the task time to complete
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Should not receive any message
    assert!(msg_rx.try_recv().is_err());
}

#[tokio::test]
async fn test_async_handler_cancellation() {
    let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
    let (msg_tx, mut msg_rx) = mpsc::channel(10);
    let (err_tx, _err_rx) = mpsc::channel(10);
    let cancel = CancellationToken::new();

    // Create a slow async command
    handler.execute(Command::perform_async(async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Some(TestMsg::AsyncResult(42))
    }));

    handler.spawn_pending(msg_tx, err_tx, cancel.clone());

    // Cancel immediately
    cancel.cancel();

    // Give the task time to notice cancellation
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Should not receive any message
    assert!(msg_rx.try_recv().is_err());
}

#[tokio::test]
async fn test_async_handler_multiple_async() {
    let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
    let (msg_tx, mut msg_rx) = mpsc::channel(10);
    let (err_tx, _err_rx) = mpsc::channel(10);
    let cancel = CancellationToken::new();

    handler.execute(Command::perform_async(async {
        Some(TestMsg::AsyncResult(1))
    }));
    handler.execute(Command::perform_async(async {
        Some(TestMsg::AsyncResult(2))
    }));
    handler.execute(Command::perform_async(async {
        Some(TestMsg::AsyncResult(3))
    }));

    assert_eq!(handler.pending_future_count(), 3);

    handler.spawn_pending(msg_tx, err_tx, cancel);

    // Collect all messages
    let mut messages = Vec::new();
    for _ in 0..3 {
        let msg = msg_rx.recv().await.expect("Should receive message");
        messages.push(msg);
    }

    // Order is not guaranteed, so just check we got all values
    assert!(messages.contains(&TestMsg::AsyncResult(1)));
    assert!(messages.contains(&TestMsg::AsyncResult(2)));
    assert!(messages.contains(&TestMsg::AsyncResult(3)));
}

#[test]
fn test_async_handler_reset_quit() {
    let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
    handler.execute(Command::quit());
    assert!(handler.should_quit());

    handler.reset_quit();
    assert!(!handler.should_quit());
}

#[test]
fn test_async_handler_default() {
    let handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::default();
    assert!(!handler.should_quit());
    assert!(!handler.has_pending_futures());
}

#[test]
fn test_async_handler_push_overlay() {
    use crate::input::Event;
    use crate::overlay::{Overlay, OverlayAction};
    use crate::theme::Theme;
    use ratatui::layout::Rect;

    struct TestOverlay;
    impl Overlay<TestMsg> for TestOverlay {
        fn handle_event(&mut self, _event: &Event) -> OverlayAction<TestMsg> {
            OverlayAction::Consumed
        }
        fn view(&self, _frame: &mut ratatui::Frame, _area: Rect, _theme: &Theme) {}
    }

    let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
    handler.execute(Command::push_overlay(TestOverlay));

    let pushes = handler.take_overlay_pushes();
    assert_eq!(pushes.len(), 1);

    // Second take should be empty
    let pushes = handler.take_overlay_pushes();
    assert!(pushes.is_empty());
}

#[test]
fn test_async_handler_pop_overlay() {
    let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
    handler.execute(Command::pop_overlay());
    handler.execute(Command::pop_overlay());

    let pops = handler.take_overlay_pops();
    assert_eq!(pops, 2);

    // Second take should be zero
    let pops = handler.take_overlay_pops();
    assert_eq!(pops, 0);
}
