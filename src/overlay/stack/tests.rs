use super::*;
use crate::component::RenderContext;
use crate::input::Key;

struct ConsumeOverlay;

impl Overlay<i32> for ConsumeOverlay {
    fn handle_event(&mut self, _event: &Event) -> OverlayAction<i32> {
        OverlayAction::Consumed
    }

    fn view(&self, _ctx: &mut RenderContext<'_, '_>) {}
}

struct PropagateOverlay;

impl Overlay<i32> for PropagateOverlay {
    fn handle_event(&mut self, _event: &Event) -> OverlayAction<i32> {
        OverlayAction::Propagate
    }

    fn view(&self, _ctx: &mut RenderContext<'_, '_>) {}
}

struct MessageOverlay {
    value: i32,
}

impl Overlay<i32> for MessageOverlay {
    fn handle_event(&mut self, _event: &Event) -> OverlayAction<i32> {
        OverlayAction::KeepAndMessage(self.value)
    }

    fn view(&self, _ctx: &mut RenderContext<'_, '_>) {}
}

#[test]
fn test_stack_new() {
    let stack: OverlayStack<i32> = OverlayStack::new();
    assert!(stack.is_empty());
    assert!(!stack.is_active());
    assert_eq!(stack.len(), 0);
}

#[test]
fn test_stack_push_pop() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();
    stack.push(Box::new(ConsumeOverlay));
    assert!(!stack.is_empty());
    assert!(stack.is_active());
    assert_eq!(stack.len(), 1);

    let _ = stack.pop();
    assert!(stack.is_empty());
}

#[test]
fn test_stack_clear() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();
    stack.push(Box::new(ConsumeOverlay));
    stack.push(Box::new(ConsumeOverlay));
    assert_eq!(stack.len(), 2);

    stack.clear();
    assert!(stack.is_empty());
}

#[test]
fn test_stack_event_consumed() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();
    stack.push(Box::new(ConsumeOverlay));

    let action = stack.handle_event(&Event::char('a'));
    assert!(matches!(action, OverlayAction::Consumed));
}

#[test]
fn test_stack_event_propagates_through_empty() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();

    let action = stack.handle_event(&Event::char('a'));
    assert!(matches!(action, OverlayAction::Propagate));
}

#[test]
fn test_stack_event_propagates_through_propagating_overlay() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();
    stack.push(Box::new(PropagateOverlay));

    let action = stack.handle_event(&Event::char('a'));
    assert!(matches!(action, OverlayAction::Propagate));
}

#[test]
fn test_stack_topmost_gets_event_first() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();
    // Bottom: propagates. Top: consumes.
    stack.push(Box::new(PropagateOverlay));
    stack.push(Box::new(ConsumeOverlay));

    let action = stack.handle_event(&Event::char('a'));
    assert!(matches!(action, OverlayAction::Consumed));
}

#[test]
fn test_stack_dismiss_action() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();

    struct DismissOverlay;
    impl Overlay<i32> for DismissOverlay {
        fn handle_event(&mut self, event: &Event) -> OverlayAction<i32> {
            if let Some(key) = event.as_key() {
                if key.code == Key::Esc {
                    return OverlayAction::Dismiss;
                }
            }
            OverlayAction::Consumed
        }
        fn view(&self, _ctx: &mut RenderContext<'_, '_>) {}
    }

    stack.push(Box::new(DismissOverlay));
    let action = stack.handle_event(&Event::key(Key::Esc));
    assert!(matches!(action, OverlayAction::Dismiss));
}

#[test]
fn test_stack_message_action() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();
    stack.push(Box::new(MessageOverlay { value: 42 }));

    let action = stack.handle_event(&Event::char('x'));
    assert!(matches!(action, OverlayAction::KeepAndMessage(42)));
}

#[test]
fn test_stack_dismiss_with_message() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();

    struct DismissWithMsg;
    impl Overlay<i32> for DismissWithMsg {
        fn handle_event(&mut self, _: &Event) -> OverlayAction<i32> {
            OverlayAction::DismissWithMessage(99)
        }
        fn view(&self, _ctx: &mut RenderContext<'_, '_>) {}
    }

    stack.push(Box::new(DismissWithMsg));
    let action = stack.handle_event(&Event::key(Key::Enter));
    assert!(matches!(action, OverlayAction::DismissWithMessage(99)));
}

#[test]
fn test_stack_default_is_empty() {
    let stack: OverlayStack<i32> = OverlayStack::default();
    assert!(stack.is_empty());
}

#[test]
fn test_stack_render_empty() {
    use crate::theme::Theme;

    let stack: OverlayStack<i32> = OverlayStack::new();
    let backend = ratatui::backend::TestBackend::new(40, 10);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    let theme = Theme::default();

    // Should not panic
    terminal
        .draw(|frame| {
            let area = frame.area();
            let mut ctx = RenderContext::new(frame, area, &theme);
            stack.render(&mut ctx);
        })
        .unwrap();
}

#[test]
fn test_stack_render_with_overlays() {
    use crate::theme::Theme;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    // An overlay that tracks how many times view() is called
    struct TrackingOverlay {
        call_count: Arc<AtomicU32>,
    }

    impl Overlay<i32> for TrackingOverlay {
        fn handle_event(&mut self, _event: &Event) -> OverlayAction<i32> {
            OverlayAction::Consumed
        }
        fn view(&self, _ctx: &mut RenderContext<'_, '_>) {
            self.call_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    let count1 = Arc::new(AtomicU32::new(0));
    let count2 = Arc::new(AtomicU32::new(0));

    let mut stack: OverlayStack<i32> = OverlayStack::new();
    stack.push(Box::new(TrackingOverlay {
        call_count: count1.clone(),
    }));
    stack.push(Box::new(TrackingOverlay {
        call_count: count2.clone(),
    }));

    let backend = ratatui::backend::TestBackend::new(40, 10);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    let theme = Theme::default();
    terminal
        .draw(|frame| {
            let area = frame.area();
            let mut ctx = RenderContext::new(frame, area, &theme);
            stack.render(&mut ctx);
        })
        .unwrap();

    // Both overlays should have been rendered (bottom-up)
    assert_eq!(count1.load(Ordering::Relaxed), 1);
    assert_eq!(count2.load(Ordering::Relaxed), 1);
}
