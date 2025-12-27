//! Event types for input simulation.

use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
    MouseButton, MouseEvent, MouseEventKind,
};

/// A simulated input event.
///
/// This wraps crossterm's event types to provide a unified interface
/// for simulating input in tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimulatedEvent {
    /// A keyboard event
    Key(KeyEvent),

    /// A mouse event
    Mouse(MouseEvent),

    /// A resize event (width, height)
    Resize(u16, u16),

    /// Focus gained
    FocusGained,

    /// Focus lost
    FocusLost,

    /// A paste event (bracketed paste content)
    Paste(String),
}

impl SimulatedEvent {
    /// Creates a key press event for a character.
    pub fn char(c: char) -> Self {
        Self::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
    }

    /// Creates a key press event for a character with modifiers.
    pub fn char_with(c: char, modifiers: KeyModifiers) -> Self {
        Self::Key(KeyEvent::new(KeyCode::Char(c), modifiers))
    }

    /// Creates a key press event for a special key.
    pub fn key(code: KeyCode) -> Self {
        Self::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    /// Creates a key press event with modifiers.
    pub fn key_with(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self::Key(KeyEvent::new(code, modifiers))
    }

    /// Creates a Ctrl+key event.
    pub fn ctrl(c: char) -> Self {
        Self::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL))
    }

    /// Creates an Alt+key event.
    pub fn alt(c: char) -> Self {
        Self::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT))
    }

    /// Creates a mouse click event at the specified position.
    pub fn click(x: u16, y: u16) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// Creates a mouse click event with a specific button.
    pub fn click_button(x: u16, y: u16, button: MouseButton) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::Down(button),
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// Creates a mouse release event.
    pub fn mouse_up(x: u16, y: u16) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// Creates a mouse move event.
    pub fn mouse_move(x: u16, y: u16) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// Creates a mouse drag event.
    pub fn mouse_drag(x: u16, y: u16, button: MouseButton) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::Drag(button),
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// Creates a scroll up event.
    pub fn scroll_up(x: u16, y: u16) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// Creates a scroll down event.
    pub fn scroll_down(x: u16, y: u16) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// Returns true if this is a key event.
    pub fn is_key(&self) -> bool {
        matches!(self, SimulatedEvent::Key(_))
    }

    /// Returns true if this is a mouse event.
    pub fn is_mouse(&self) -> bool {
        matches!(self, SimulatedEvent::Mouse(_))
    }

    /// Returns the key event if this is one.
    pub fn as_key(&self) -> Option<&KeyEvent> {
        match self {
            SimulatedEvent::Key(e) => Some(e),
            _ => None,
        }
    }

    /// Returns the mouse event if this is one.
    pub fn as_mouse(&self) -> Option<&MouseEvent> {
        match self {
            SimulatedEvent::Mouse(e) => Some(e),
            _ => None,
        }
    }
}

impl From<KeyEvent> for SimulatedEvent {
    fn from(event: KeyEvent) -> Self {
        SimulatedEvent::Key(event)
    }
}

impl From<MouseEvent> for SimulatedEvent {
    fn from(event: MouseEvent) -> Self {
        SimulatedEvent::Mouse(event)
    }
}

impl From<crossterm::event::Event> for SimulatedEvent {
    fn from(event: crossterm::event::Event) -> Self {
        match event {
            crossterm::event::Event::Key(e) => SimulatedEvent::Key(e),
            crossterm::event::Event::Mouse(e) => SimulatedEvent::Mouse(e),
            crossterm::event::Event::Resize(w, h) => SimulatedEvent::Resize(w, h),
            crossterm::event::Event::FocusGained => SimulatedEvent::FocusGained,
            crossterm::event::Event::FocusLost => SimulatedEvent::FocusLost,
            crossterm::event::Event::Paste(s) => SimulatedEvent::Paste(s),
        }
    }
}

impl From<SimulatedEvent> for crossterm::event::Event {
    fn from(event: SimulatedEvent) -> Self {
        match event {
            SimulatedEvent::Key(e) => crossterm::event::Event::Key(e),
            SimulatedEvent::Mouse(e) => crossterm::event::Event::Mouse(e),
            SimulatedEvent::Resize(w, h) => crossterm::event::Event::Resize(w, h),
            SimulatedEvent::FocusGained => crossterm::event::Event::FocusGained,
            SimulatedEvent::FocusLost => crossterm::event::Event::FocusLost,
            SimulatedEvent::Paste(s) => crossterm::event::Event::Paste(s),
        }
    }
}

/// Builder for creating key events with specific properties.
#[derive(Clone, Debug)]
pub struct KeyEventBuilder {
    code: Option<KeyCode>,
    modifiers: KeyModifiers,
    kind: KeyEventKind,
    state: KeyEventState,
}

impl Default for KeyEventBuilder {
    fn default() -> Self {
        Self {
            code: None,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }
}

impl KeyEventBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the key code.
    pub fn code(mut self, code: KeyCode) -> Self {
        self.code = Some(code);
        self
    }

    /// Sets the key to a character.
    pub fn char(mut self, c: char) -> Self {
        self.code = Some(KeyCode::Char(c));
        self
    }

    /// Adds the Control modifier.
    pub fn ctrl(mut self) -> Self {
        self.modifiers |= KeyModifiers::CONTROL;
        self
    }

    /// Adds the Alt modifier.
    pub fn alt(mut self) -> Self {
        self.modifiers |= KeyModifiers::ALT;
        self
    }

    /// Adds the Shift modifier.
    pub fn shift(mut self) -> Self {
        self.modifiers |= KeyModifiers::SHIFT;
        self
    }

    /// Sets the modifiers directly.
    pub fn modifiers(mut self, modifiers: KeyModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Sets the event kind (Press, Release, Repeat).
    pub fn kind(mut self, kind: KeyEventKind) -> Self {
        self.kind = kind;
        self
    }

    /// Builds the key event.
    pub fn build(self) -> KeyEvent {
        KeyEvent {
            code: self.code.unwrap_or(KeyCode::Null),
            modifiers: self.modifiers,
            kind: self.kind,
            state: self.state,
        }
    }

    /// Builds and wraps in a SimulatedEvent.
    pub fn into_event(self) -> SimulatedEvent {
        SimulatedEvent::Key(self.build())
    }
}

/// Builder for creating mouse events with specific properties.
#[derive(Clone, Debug)]
pub struct MouseEventBuilder {
    kind: MouseEventKind,
    column: u16,
    row: u16,
    modifiers: KeyModifiers,
}

impl MouseEventBuilder {
    /// Creates a new builder at position (0, 0).
    pub fn new() -> Self {
        Self {
            kind: MouseEventKind::Moved,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        }
    }

    /// Sets the position.
    pub fn at(mut self, x: u16, y: u16) -> Self {
        self.column = x;
        self.row = y;
        self
    }

    /// Sets the event to a click.
    pub fn click(mut self) -> Self {
        self.kind = MouseEventKind::Down(MouseButton::Left);
        self
    }

    /// Sets the event to a right-click.
    pub fn right_click(mut self) -> Self {
        self.kind = MouseEventKind::Down(MouseButton::Right);
        self
    }

    /// Sets the event to a middle-click.
    pub fn middle_click(mut self) -> Self {
        self.kind = MouseEventKind::Down(MouseButton::Middle);
        self
    }

    /// Sets the event to a mouse up.
    pub fn up(mut self) -> Self {
        self.kind = MouseEventKind::Up(MouseButton::Left);
        self
    }

    /// Sets the event to a drag.
    pub fn drag(mut self) -> Self {
        self.kind = MouseEventKind::Drag(MouseButton::Left);
        self
    }

    /// Sets the event to a scroll up.
    pub fn scroll_up(mut self) -> Self {
        self.kind = MouseEventKind::ScrollUp;
        self
    }

    /// Sets the event to a scroll down.
    pub fn scroll_down(mut self) -> Self {
        self.kind = MouseEventKind::ScrollDown;
        self
    }

    /// Adds the Control modifier.
    pub fn ctrl(mut self) -> Self {
        self.modifiers |= KeyModifiers::CONTROL;
        self
    }

    /// Adds the Alt modifier.
    pub fn alt(mut self) -> Self {
        self.modifiers |= KeyModifiers::ALT;
        self
    }

    /// Adds the Shift modifier.
    pub fn shift(mut self) -> Self {
        self.modifiers |= KeyModifiers::SHIFT;
        self
    }

    /// Builds the mouse event.
    pub fn build(self) -> MouseEvent {
        MouseEvent {
            kind: self.kind,
            column: self.column,
            row: self.row,
            modifiers: self.modifiers,
        }
    }

    /// Builds and wraps in a SimulatedEvent.
    pub fn into_event(self) -> SimulatedEvent {
        SimulatedEvent::Mouse(self.build())
    }
}

impl Default for MouseEventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulated_event_char() {
        let event = SimulatedEvent::char('a');
        assert!(event.is_key());
        let key = event.as_key().unwrap();
        assert_eq!(key.code, KeyCode::Char('a'));
        assert_eq!(key.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_simulated_event_ctrl() {
        let event = SimulatedEvent::ctrl('c');
        let key = event.as_key().unwrap();
        assert_eq!(key.code, KeyCode::Char('c'));
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_simulated_event_click() {
        let event = SimulatedEvent::click(10, 20);
        assert!(event.is_mouse());
        let mouse = event.as_mouse().unwrap();
        assert_eq!(mouse.column, 10);
        assert_eq!(mouse.row, 20);
        assert!(matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)));
    }

    #[test]
    fn test_key_event_builder() {
        let event = KeyEventBuilder::new()
            .char('x')
            .ctrl()
            .shift()
            .build();

        assert_eq!(event.code, KeyCode::Char('x'));
        assert!(event.modifiers.contains(KeyModifiers::CONTROL));
        assert!(event.modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_mouse_event_builder() {
        let event = MouseEventBuilder::new()
            .at(5, 10)
            .right_click()
            .ctrl()
            .build();

        assert_eq!(event.column, 5);
        assert_eq!(event.row, 10);
        assert!(matches!(event.kind, MouseEventKind::Down(MouseButton::Right)));
        assert!(event.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_crossterm_conversion() {
        let simulated = SimulatedEvent::key(KeyCode::Enter);
        let crossterm: crossterm::event::Event = simulated.clone().into();
        let back: SimulatedEvent = crossterm.into();
        assert_eq!(simulated, back);
    }
}
