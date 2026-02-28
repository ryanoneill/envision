use super::*;
use crossterm::event::KeyCode;

struct ConsumeOverlay;

impl Overlay<i32> for ConsumeOverlay {
    fn handle_event(&mut self, _event: &Event) -> OverlayAction<i32> {
        OverlayAction::Consumed
    }

    fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
}

struct PropagateOverlay;

impl Overlay<i32> for PropagateOverlay {
    fn handle_event(&mut self, _event: &Event) -> OverlayAction<i32> {
        OverlayAction::Propagate
    }

    fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
}

struct MessageOverlay {
    value: i32,
}

impl Overlay<i32> for MessageOverlay {
    fn handle_event(&mut self, _event: &Event) -> OverlayAction<i32> {
        OverlayAction::Message(self.value)
    }

    fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
}

#[test]
fn test_stack_new() {
    let stack: OverlayStack<i32> = OverlayStack::new();
    assert!(stack.is_empty());
    assert!(!stack.is_active());
    assert_eq!(stack.len(), 0);
}

#[test]
fn test_stack_default() {
    let stack: OverlayStack<i32> = OverlayStack::default();
    assert!(stack.is_empty());
}

#[test]
fn test_stack_push_pop() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();

    stack.push(Box::new(ConsumeOverlay));
    assert_eq!(stack.len(), 1);
    assert!(stack.is_active());
    assert!(!stack.is_empty());

    stack.push(Box::new(PropagateOverlay));
    assert_eq!(stack.len(), 2);

    let popped = stack.pop();
    assert!(popped.is_some());
    assert_eq!(stack.len(), 1);

    let popped = stack.pop();
    assert!(popped.is_some());
    assert!(stack.is_empty());

    assert!(stack.pop().is_none());
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
fn test_stack_handle_event_empty() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();
    let event = Event::char('a');

    let action = stack.handle_event(&event);
    assert!(matches!(action, OverlayAction::Propagate));
}

#[test]
fn test_stack_handle_event_consumed() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();
    stack.push(Box::new(ConsumeOverlay));

    let event = Event::char('a');
    let action = stack.handle_event(&event);
    assert!(matches!(action, OverlayAction::Consumed));
}

#[test]
fn test_stack_handle_event_propagate_to_bottom() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();
    // Bottom: message overlay
    stack.push(Box::new(MessageOverlay { value: 42 }));
    // Top: propagate overlay
    stack.push(Box::new(PropagateOverlay));

    let event = Event::char('a');
    let action = stack.handle_event(&event);
    // Top propagates, bottom produces message
    assert!(matches!(action, OverlayAction::Message(42)));
}

#[test]
fn test_stack_handle_event_top_consumes() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();
    // Bottom: message overlay (won't be reached)
    stack.push(Box::new(MessageOverlay { value: 42 }));
    // Top: consume overlay
    stack.push(Box::new(ConsumeOverlay));

    let event = Event::char('a');
    let action = stack.handle_event(&event);
    // Top consumes, bottom is never reached
    assert!(matches!(action, OverlayAction::Consumed));
}

#[test]
fn test_stack_handle_event_all_propagate() {
    let mut stack: OverlayStack<i32> = OverlayStack::new();
    stack.push(Box::new(PropagateOverlay));
    stack.push(Box::new(PropagateOverlay));

    let event = Event::char('a');
    let action = stack.handle_event(&event);
    assert!(matches!(action, OverlayAction::Propagate));
}

#[test]
fn test_stack_handle_event_dismiss() {
    struct DismissOverlay;
    impl Overlay<i32> for DismissOverlay {
        fn handle_event(&mut self, event: &Event) -> OverlayAction<i32> {
            if let Some(key) = event.as_key() {
                if key.code == KeyCode::Esc {
                    return OverlayAction::Dismiss;
                }
            }
            OverlayAction::Consumed
        }
        fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
    }

    let mut stack: OverlayStack<i32> = OverlayStack::new();
    stack.push(Box::new(DismissOverlay));

    let event = Event::key(KeyCode::Esc);
    let action = stack.handle_event(&event);
    assert!(matches!(action, OverlayAction::Dismiss));
}

#[test]
fn test_stack_render_empty() {
    // Rendering an empty stack should be a no-op (no panic)
    let stack: OverlayStack<i32> = OverlayStack::new();
    let backend = ratatui::backend::TestBackend::new(40, 10);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    let theme = Theme::default();
    terminal
        .draw(|frame| {
            stack.render(frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_stack_render_with_overlays() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    // An overlay that tracks how many times view() is called
    struct TrackingOverlay {
        call_count: Arc<AtomicU32>,
    }

    impl Overlay<i32> for TrackingOverlay {
        fn handle_event(&mut self, _event: &Event) -> OverlayAction<i32> {
            OverlayAction::Consumed
        }
        fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {
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
            stack.render(frame, frame.area(), &theme);
        })
        .unwrap();

    // Both overlays should have been rendered (bottom-up)
    assert_eq!(count1.load(Ordering::Relaxed), 1);
    assert_eq!(count2.load(Ordering::Relaxed), 1);
}
