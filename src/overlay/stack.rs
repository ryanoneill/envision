//! Overlay stack implementation.

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::input::Event;
use crate::theme::Theme;

use super::action::OverlayAction;
use super::traits::Overlay;

/// A stack of overlays managed by the runtime.
///
/// The stack renders overlays bottom-up (so the topmost draws last) and
/// processes events top-down (so the topmost gets first chance to handle).
pub struct OverlayStack<M> {
    layers: Vec<Box<dyn Overlay<M>>>,
}

impl<M> OverlayStack<M> {
    /// Creates a new empty overlay stack.
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Pushes an overlay onto the top of the stack.
    pub fn push(&mut self, overlay: Box<dyn Overlay<M>>) {
        self.layers.push(overlay);
    }

    /// Pops the topmost overlay from the stack.
    pub fn pop(&mut self) -> Option<Box<dyn Overlay<M>>> {
        self.layers.pop()
    }

    /// Clears all overlays from the stack.
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Returns true if there are active overlays.
    pub fn is_active(&self) -> bool {
        !self.layers.is_empty()
    }

    /// Returns true if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Returns the number of overlays on the stack.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Processes an event through the stack (top-down).
    ///
    /// Returns the action from the first overlay that doesn't Propagate,
    /// or Propagate if all overlays propagate (or stack is empty).
    pub(crate) fn handle_event(&mut self, event: &Event) -> OverlayAction<M> {
        // Process top-down (last element is topmost)
        for overlay in self.layers.iter_mut().rev() {
            match overlay.handle_event(event) {
                OverlayAction::Propagate => continue,
                action => return action,
            }
        }
        OverlayAction::Propagate
    }

    /// Renders all overlays bottom-up (so topmost draws last).
    pub(crate) fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        for overlay in &self.layers {
            overlay.view(frame, area, theme);
        }
    }
}

impl<M> Default for OverlayStack<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
}
