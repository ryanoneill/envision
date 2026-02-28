//! Event types for terminal input.

use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};

/// A terminal input event.
///
/// This wraps crossterm's event types to provide a unified interface
/// for handling input events. The same type is used whether events come
/// from a real terminal or are injected programmatically.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
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

impl Event {
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
        matches!(self, Event::Key(_))
    }

    /// Returns true if this is a mouse event.
    pub fn is_mouse(&self) -> bool {
        matches!(self, Event::Mouse(_))
    }

    /// Returns the key event if this is one.
    pub fn as_key(&self) -> Option<&KeyEvent> {
        match self {
            Event::Key(e) => Some(e),
            _ => None,
        }
    }

    /// Returns the mouse event if this is one.
    pub fn as_mouse(&self) -> Option<&MouseEvent> {
        match self {
            Event::Mouse(e) => Some(e),
            _ => None,
        }
    }
}

impl From<KeyEvent> for Event {
    fn from(event: KeyEvent) -> Self {
        Event::Key(event)
    }
}

impl From<MouseEvent> for Event {
    fn from(event: MouseEvent) -> Self {
        Event::Mouse(event)
    }
}

impl From<crossterm::event::Event> for Event {
    fn from(event: crossterm::event::Event) -> Self {
        match event {
            crossterm::event::Event::Key(e) => Event::Key(e),
            crossterm::event::Event::Mouse(e) => Event::Mouse(e),
            crossterm::event::Event::Resize(w, h) => Event::Resize(w, h),
            crossterm::event::Event::FocusGained => Event::FocusGained,
            crossterm::event::Event::FocusLost => Event::FocusLost,
            crossterm::event::Event::Paste(s) => Event::Paste(s),
        }
    }
}

impl From<Event> for crossterm::event::Event {
    fn from(event: Event) -> Self {
        match event {
            Event::Key(e) => crossterm::event::Event::Key(e),
            Event::Mouse(e) => crossterm::event::Event::Mouse(e),
            Event::Resize(w, h) => crossterm::event::Event::Resize(w, h),
            Event::FocusGained => crossterm::event::Event::FocusGained,
            Event::FocusLost => crossterm::event::Event::FocusLost,
            Event::Paste(s) => crossterm::event::Event::Paste(s),
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

    /// Builds and wraps in a Event.
    pub fn into_event(self) -> Event {
        Event::Key(self.build())
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

    /// Builds and wraps in a Event.
    pub fn into_event(self) -> Event {
        Event::Mouse(self.build())
    }
}

impl Default for MouseEventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
