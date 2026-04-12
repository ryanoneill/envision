//! Envision-owned mouse input types.

use super::key::Modifiers;

/// A mouse event.
///
/// # Example
///
/// ```rust
/// use envision::input::mouse::{MouseEvent, MouseEventKind, MouseButton};
/// use envision::input::key::Modifiers;
///
/// let event = MouseEvent {
///     kind: MouseEventKind::Down(MouseButton::Left),
///     column: 10,
///     row: 5,
///     modifiers: Modifiers::NONE,
/// };
/// assert_eq!(event.column, 10);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseEvent {
    /// The kind of mouse event.
    pub kind: MouseEventKind,
    /// The column (x coordinate).
    pub column: u16,
    /// The row (y coordinate).
    pub row: u16,
    /// Modifier keys held during the event.
    pub modifiers: Modifiers,
}

/// The kind of mouse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseEventKind {
    /// A button was pressed.
    Down(MouseButton),
    /// A button was released.
    Up(MouseButton),
    /// The mouse was dragged while a button was held.
    Drag(MouseButton),
    /// The mouse was moved (no button held).
    Moved,
    /// Scroll wheel up.
    ScrollUp,
    /// Scroll wheel down.
    ScrollDown,
    /// Scroll wheel left.
    ScrollLeft,
    /// Scroll wheel right.
    ScrollRight,
}

/// A mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// The left mouse button.
    Left,
    /// The right mouse button.
    Right,
    /// The middle mouse button.
    Middle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_event_construction() {
        let event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: Modifiers::NONE,
        };
        assert_eq!(event.column, 10);
        assert_eq!(event.row, 5);
        assert!(event.modifiers.is_none());
    }

    #[test]
    fn test_mouse_event_kind_equality() {
        assert_eq!(
            MouseEventKind::Down(MouseButton::Left),
            MouseEventKind::Down(MouseButton::Left),
        );
        assert_ne!(
            MouseEventKind::Down(MouseButton::Left),
            MouseEventKind::Down(MouseButton::Right),
        );
        assert_ne!(
            MouseEventKind::Down(MouseButton::Left),
            MouseEventKind::Up(MouseButton::Left),
        );
    }

    #[test]
    fn test_mouse_button_equality() {
        assert_eq!(MouseButton::Left, MouseButton::Left);
        assert_ne!(MouseButton::Left, MouseButton::Right);
    }

    #[test]
    fn test_mouse_with_modifiers() {
        let event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: Modifiers::CONTROL | Modifiers::SHIFT,
        };
        assert!(event.modifiers.ctrl());
        assert!(event.modifiers.shift());
    }
}
