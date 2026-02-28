use super::*;

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::input::Event;
use crate::overlay::{Overlay, OverlayAction};
use crate::theme::Theme;

#[derive(Clone, Debug, PartialEq)]
enum TestMsg {
    Hello,
    World,
    FromCallback,
}

struct TestOverlay;

impl Overlay<TestMsg> for TestOverlay {
    fn handle_event(&mut self, _event: &Event) -> OverlayAction<TestMsg> {
        OverlayAction::Consumed
    }

    fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
}

#[test]
fn test_new_initializes_empty() {
    let core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    assert!(core.pending_messages.is_empty());
    assert!(core.pending_overlay_pushes.is_empty());
    assert_eq!(core.pending_overlay_pops, 0);
    assert!(!core.should_quit());
}

#[test]
fn test_execute_sync_action_message() {
    let mut core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    let result = core.execute_sync_action(CommandAction::Message(TestMsg::Hello));
    assert!(result.is_none());
    assert_eq!(core.pending_messages, vec![TestMsg::Hello]);
}

#[test]
fn test_execute_sync_action_batch() {
    let mut core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    let result =
        core.execute_sync_action(CommandAction::Batch(vec![TestMsg::Hello, TestMsg::World]));
    assert!(result.is_none());
    assert_eq!(core.pending_messages, vec![TestMsg::Hello, TestMsg::World]);
}

#[test]
fn test_execute_sync_action_quit() {
    let mut core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    let result = core.execute_sync_action(CommandAction::Quit);
    assert!(result.is_none());
    assert!(core.should_quit());
}

#[test]
fn test_execute_sync_action_callback_with_message() {
    let mut core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    let result = core.execute_sync_action(CommandAction::Callback(Box::new(|| {
        Some(TestMsg::FromCallback)
    })));
    assert!(result.is_none());
    assert_eq!(core.pending_messages, vec![TestMsg::FromCallback]);
}

#[test]
fn test_execute_sync_action_callback_without_message() {
    let mut core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    let result = core.execute_sync_action(CommandAction::Callback(Box::new(|| None)));
    assert!(result.is_none());
    assert!(core.pending_messages.is_empty());
}

#[test]
fn test_execute_sync_action_push_overlay() {
    let mut core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    let overlay = Box::new(TestOverlay);
    let result = core.execute_sync_action(CommandAction::PushOverlay(overlay));
    assert!(result.is_none());
    assert_eq!(core.pending_overlay_pushes.len(), 1);
}

#[test]
fn test_execute_sync_action_pop_overlay() {
    let mut core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    let result = core.execute_sync_action(CommandAction::PopOverlay);
    assert!(result.is_none());
    assert_eq!(core.pending_overlay_pops, 1);

    core.execute_sync_action(CommandAction::PopOverlay);
    assert_eq!(core.pending_overlay_pops, 2);
}

#[test]
fn test_execute_sync_action_async_passthrough() {
    let mut core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    let future = Box::pin(async { Some(TestMsg::Hello) });
    let result = core.execute_sync_action(CommandAction::Async(future));
    assert!(result.is_some());
    assert!(matches!(result.unwrap(), CommandAction::Async(_)));
}

#[test]
fn test_execute_sync_action_async_fallible_passthrough() {
    let mut core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    let future = Box::pin(async { Ok(Some(TestMsg::Hello)) });
    let result = core.execute_sync_action(CommandAction::AsyncFallible(future));
    assert!(result.is_some());
    assert!(matches!(result.unwrap(), CommandAction::AsyncFallible(_)));
}

#[test]
fn test_take_messages_consumes_and_returns() {
    let mut core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    core.execute_sync_action(CommandAction::Message(TestMsg::Hello));
    core.execute_sync_action(CommandAction::Message(TestMsg::World));

    let messages = core.take_messages();
    assert_eq!(messages, vec![TestMsg::Hello, TestMsg::World]);

    let second = core.take_messages();
    assert!(second.is_empty());
}

#[test]
fn test_take_overlay_pushes_consumes_and_returns() {
    let mut core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    core.execute_sync_action(CommandAction::PushOverlay(Box::new(TestOverlay)));
    core.execute_sync_action(CommandAction::PushOverlay(Box::new(TestOverlay)));

    let pushes = core.take_overlay_pushes();
    assert_eq!(pushes.len(), 2);

    let second = core.take_overlay_pushes();
    assert!(second.is_empty());
}

#[test]
fn test_take_overlay_pops_returns_count_and_resets() {
    let mut core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    core.execute_sync_action(CommandAction::PopOverlay);
    core.execute_sync_action(CommandAction::PopOverlay);
    core.execute_sync_action(CommandAction::PopOverlay);

    let count = core.take_overlay_pops();
    assert_eq!(count, 3);

    let second = core.take_overlay_pops();
    assert_eq!(second, 0);
}

#[test]
fn test_should_quit_and_reset_quit() {
    let mut core: CommandHandlerCore<TestMsg> = CommandHandlerCore::new();
    assert!(!core.should_quit());

    core.execute_sync_action(CommandAction::Quit);
    assert!(core.should_quit());

    core.reset_quit();
    assert!(!core.should_quit());
}
