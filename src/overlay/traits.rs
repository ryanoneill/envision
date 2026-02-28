//! Overlay trait definition.

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::input::Event;
use crate::theme::Theme;

use super::OverlayAction;

/// A modal overlay that can intercept events and render on top of the main view.
///
/// Overlays own their transient UI state (search query, cursor position, scroll
/// offset) via `&mut self` on `handle_event`. The `Send` bound enables future
/// async compatibility.
///
/// # Example
///
/// ```ignore
/// struct ConfirmDialog {
///     message: String,
/// }
///
/// impl Overlay<MyMsg> for ConfirmDialog {
///     fn handle_event(&mut self, event: &Event) -> OverlayAction<MyMsg> {
///         if let Some(key) = event.as_key() {
///             match key.code {
///                 KeyCode::Char('y') => OverlayAction::DismissWithMessage(MyMsg::Confirmed),
///                 KeyCode::Char('n') | KeyCode::Esc => OverlayAction::Dismiss,
///                 _ => OverlayAction::Consumed,
///             }
///         } else {
///             OverlayAction::Propagate
///         }
///     }
///
///     fn view(&self, frame: &mut Frame, area: Rect, _theme: &Theme) {
///         // Render the confirmation dialog
///     }
/// }
/// ```
pub trait Overlay<M>: Send {
    /// Handle an input event.
    ///
    /// The overlay can mutate itself (e.g., update a search buffer).
    fn handle_event(&mut self, event: &Event) -> OverlayAction<M>;

    /// Render the overlay on top of the main view.
    fn view(&self, frame: &mut Frame, area: Rect, theme: &Theme);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    struct TestOverlay {
        consumed_count: u32,
    }

    impl Overlay<String> for TestOverlay {
        fn handle_event(&mut self, event: &Event) -> OverlayAction<String> {
            if let Some(key) = event.as_key() {
                match key.code {
                    KeyCode::Esc => OverlayAction::Dismiss,
                    KeyCode::Enter => OverlayAction::DismissWithMessage("confirmed".to_string()),
                    _ => {
                        self.consumed_count += 1;
                        OverlayAction::Consumed
                    }
                }
            } else {
                OverlayAction::Propagate
            }
        }

        fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {
            // no-op for testing
        }
    }

    #[test]
    fn test_overlay_handle_event_consumed() {
        let mut overlay = TestOverlay { consumed_count: 0 };
        let event = Event::char('a');

        let action = overlay.handle_event(&event);
        assert!(matches!(action, OverlayAction::Consumed));
        assert_eq!(overlay.consumed_count, 1);
    }

    #[test]
    fn test_overlay_handle_event_dismiss() {
        let mut overlay = TestOverlay { consumed_count: 0 };
        let event = Event::key(KeyCode::Esc);

        let action = overlay.handle_event(&event);
        assert!(matches!(action, OverlayAction::Dismiss));
    }

    #[test]
    fn test_overlay_handle_event_dismiss_with_message() {
        let mut overlay = TestOverlay { consumed_count: 0 };
        let event = Event::key(KeyCode::Enter);

        let action = overlay.handle_event(&event);
        assert!(matches!(action, OverlayAction::DismissWithMessage(ref s) if s == "confirmed"));
    }

    #[test]
    fn test_overlay_handle_event_propagate() {
        let mut overlay = TestOverlay { consumed_count: 0 };
        let event = Event::Resize(80, 24);

        let action = overlay.handle_event(&event);
        assert!(matches!(action, OverlayAction::Propagate));
    }
}
