//! Event types for terminal input.

use super::key::{Key, KeyEvent, Modifiers};
use super::mouse::{MouseButton, MouseEvent, MouseEventKind};

/// A terminal input event.
///
/// This provides a unified interface for handling input events. The same
/// type is used whether events come from a real terminal or are injected
/// programmatically.
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::Event;
    ///
    /// let event = Event::char('a');
    /// assert!(event.is_key());
    /// ```
    pub fn char(c: char) -> Self {
        Self::Key(KeyEvent::char(c))
    }

    /// Creates a key press event for a character with modifiers.
    pub fn char_with(c: char, modifiers: Modifiers) -> Self {
        let mut ke = KeyEvent::char(c);
        ke.modifiers |= modifiers;
        Self::Key(ke)
    }

    /// Creates a key press event for a special key.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::{Event, Key};
    ///
    /// let event = Event::key(Key::Enter);
    /// assert!(event.is_key());
    /// ```
    pub fn key(key: Key) -> Self {
        Self::Key(KeyEvent::new(key))
    }

    /// Creates a key press event with modifiers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::{Event, Key, Modifiers};
    ///
    /// let event = Event::key_with(Key::Char('s'), Modifiers::CONTROL);
    /// assert!(event.is_key());
    /// ```
    pub fn key_with(key: Key, modifiers: Modifiers) -> Self {
        let mut ev = KeyEvent::new(key);
        ev.modifiers |= modifiers;
        Self::Key(ev)
    }

    /// Creates a Ctrl+key event.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::Event;
    ///
    /// let event = Event::ctrl('c');
    /// assert!(event.is_key());
    /// ```
    pub fn ctrl(c: char) -> Self {
        Self::Key(KeyEvent::ctrl(c))
    }

    /// Creates an Alt+key event.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::Event;
    ///
    /// let event = Event::alt('x');
    /// assert!(event.is_key());
    /// ```
    pub fn alt(c: char) -> Self {
        Self::Key(KeyEvent {
            code: Key::Char(c.to_ascii_lowercase()),
            modifiers: Modifiers::ALT,
            kind: super::key::KeyEventKind::Press,
            raw_char: Some(c),
        })
    }

    /// Creates a mouse click event at the specified position.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::Event;
    ///
    /// let event = Event::click(10, 5);
    /// assert!(event.is_mouse());
    /// ```
    pub fn click(x: u16, y: u16) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: x,
            row: y,
            modifiers: Modifiers::NONE,
        })
    }

    /// Creates a mouse click event with a specific button.
    pub fn click_button(x: u16, y: u16, button: MouseButton) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::Down(button),
            column: x,
            row: y,
            modifiers: Modifiers::NONE,
        })
    }

    /// Creates a mouse release event.
    pub fn mouse_up(x: u16, y: u16) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: x,
            row: y,
            modifiers: Modifiers::NONE,
        })
    }

    /// Creates a mouse move event.
    pub fn mouse_move(x: u16, y: u16) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column: x,
            row: y,
            modifiers: Modifiers::NONE,
        })
    }

    /// Creates a mouse drag event.
    pub fn mouse_drag(x: u16, y: u16, button: MouseButton) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::Drag(button),
            column: x,
            row: y,
            modifiers: Modifiers::NONE,
        })
    }

    /// Creates a scroll up event.
    pub fn scroll_up(x: u16, y: u16) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: x,
            row: y,
            modifiers: Modifiers::NONE,
        })
    }

    /// Creates a scroll down event.
    pub fn scroll_down(x: u16, y: u16) -> Self {
        Self::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: x,
            row: y,
            modifiers: Modifiers::NONE,
        })
    }

    /// Returns true if this is a key event.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::Event;
    ///
    /// assert!(Event::char('a').is_key());
    /// assert!(!Event::click(0, 0).is_key());
    /// ```
    pub fn is_key(&self) -> bool {
        matches!(self, Event::Key(_))
    }

    /// Returns true if this is a mouse event.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::Event;
    ///
    /// assert!(Event::click(0, 0).is_mouse());
    /// assert!(!Event::char('a').is_mouse());
    /// ```
    pub fn is_mouse(&self) -> bool {
        matches!(self, Event::Mouse(_))
    }

    /// Returns the key event if this is one.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::{Event, Key};
    ///
    /// let event = Event::key(Key::Enter);
    /// assert!(event.as_key().is_some());
    /// assert!(Event::click(0, 0).as_key().is_none());
    /// ```
    pub fn as_key(&self) -> Option<&KeyEvent> {
        match self {
            Event::Key(e) => Some(e),
            _ => None,
        }
    }

    /// Returns the mouse event if this is one.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::Event;
    ///
    /// let event = Event::click(5, 10);
    /// assert!(event.as_mouse().is_some());
    /// assert!(Event::char('a').as_mouse().is_none());
    /// ```
    pub fn as_mouse(&self) -> Option<&MouseEvent> {
        match self {
            Event::Mouse(e) => Some(e),
            _ => None,
        }
    }

    /// Returns a short string identifying the event variant.
    ///
    /// This is useful for logging and tracing. It returns the variant
    /// name (e.g., `"Key"`, `"Mouse"`, `"Resize"`) without the inner data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::{Event, Key};
    ///
    /// assert_eq!(Event::char('a').kind_name(), "Key");
    /// assert_eq!(Event::click(0, 0).kind_name(), "Mouse");
    /// assert_eq!(Event::Resize(80, 24).kind_name(), "Resize");
    /// assert_eq!(Event::FocusGained.kind_name(), "FocusGained");
    /// assert_eq!(Event::FocusLost.kind_name(), "FocusLost");
    /// ```
    pub fn kind_name(&self) -> &'static str {
        match self {
            Event::Key(_) => "Key",
            Event::Mouse(_) => "Mouse",
            Event::Resize(_, _) => "Resize",
            Event::FocusGained => "FocusGained",
            Event::FocusLost => "FocusLost",
            Event::Paste(_) => "Paste",
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

/// Builder for creating key events with specific properties.
#[derive(Clone, Debug)]
pub struct KeyEventBuilder {
    key: Option<Key>,
    modifiers: Modifiers,
    kind: super::key::KeyEventKind,
}

impl Default for KeyEventBuilder {
    fn default() -> Self {
        Self {
            key: None,
            modifiers: Modifiers::NONE,
            kind: super::key::KeyEventKind::Press,
        }
    }
}

impl KeyEventBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the key.
    pub fn code(mut self, key: Key) -> Self {
        self.key = Some(key);
        self
    }

    /// Sets the key to a character.
    pub fn char(mut self, c: char) -> Self {
        self.key = Some(Key::Char(c));
        self
    }

    /// Adds the Control modifier.
    pub fn ctrl(mut self) -> Self {
        self.modifiers |= Modifiers::CONTROL;
        self
    }

    /// Adds the Alt modifier.
    pub fn alt(mut self) -> Self {
        self.modifiers |= Modifiers::ALT;
        self
    }

    /// Adds the Shift modifier.
    pub fn shift(mut self) -> Self {
        self.modifiers |= Modifiers::SHIFT;
        self
    }

    /// Sets the modifiers directly.
    pub fn modifiers(mut self, modifiers: Modifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Sets the event kind (Press, Release, Repeat).
    pub fn kind(mut self, kind: super::key::KeyEventKind) -> Self {
        self.kind = kind;
        self
    }

    /// Builds the key event.
    pub fn build(self) -> KeyEvent {
        let key = self.key.unwrap_or(Key::Esc);
        KeyEvent {
            code: key,
            modifiers: self.modifiers,
            kind: self.kind,
            raw_char: match key {
                Key::Char(c) => Some(c),
                _ => None,
            },
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
    modifiers: Modifiers,
}

impl MouseEventBuilder {
    /// Creates a new builder at position (0, 0).
    pub fn new() -> Self {
        Self {
            kind: MouseEventKind::Moved,
            column: 0,
            row: 0,
            modifiers: Modifiers::NONE,
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
        self.modifiers |= Modifiers::CONTROL;
        self
    }

    /// Adds the Alt modifier.
    pub fn alt(mut self) -> Self {
        self.modifiers |= Modifiers::ALT;
        self
    }

    /// Adds the Shift modifier.
    pub fn shift(mut self) -> Self {
        self.modifiers |= Modifiers::SHIFT;
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
