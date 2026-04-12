//! Envision-owned keyboard input types.
//!
//! These types replace the crossterm re-exports that envision previously
//! used for keyboard events. Letter keys are normalized to lowercase;
//! use [`KeyEvent::raw_char`] for the actual terminal character.

use std::ops::{BitAnd, BitOr, BitOrAssign};

/// A keyboard key, normalized.
///
/// For ASCII letters, the `Char` variant always contains the lowercase
/// form regardless of shift or caps lock state. Check
/// [`KeyEvent::modifiers`] for shift state, and [`KeyEvent::raw_char`]
/// for the character the terminal actually sent.
///
/// # Example
///
/// ```rust
/// use envision::input::key::Key;
///
/// // Pattern-match on normalized keys for keybindings
/// let key = Key::Char('q');
/// assert!(matches!(key, Key::Char('q')));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A character key. Always lowercase for ASCII letters.
    Char(char),
    /// A function key (F1 through F24).
    F(u8),
    /// The backspace key.
    Backspace,
    /// The enter/return key.
    Enter,
    /// The left arrow key.
    Left,
    /// The right arrow key.
    Right,
    /// The up arrow key.
    Up,
    /// The down arrow key.
    Down,
    /// The home key.
    Home,
    /// The end key.
    End,
    /// The page up key.
    PageUp,
    /// The page down key.
    PageDown,
    /// The tab key.
    Tab,
    /// The delete key.
    Delete,
    /// The insert key.
    Insert,
    /// The escape key.
    Esc,
}

/// A keyboard event with normalization and raw character preservation.
///
/// # Two views of the same keypress
///
/// - **`key`**: normalized for keybindings. ASCII letters are always
///   lowercase. Use this for `match` arms in `handle_event`.
/// - **`raw_char`**: the character the terminal actually sent. Preserves
///   case (uppercase for Shift or Caps Lock). Use this for text input.
///
/// # Example
///
/// ```rust
/// use envision::input::key::{Key, KeyEvent, Modifiers};
///
/// // Constructors normalize automatically
/// let event = KeyEvent::char('A');
/// assert_eq!(event.key, Key::Char('a'));        // normalized
/// assert!(event.modifiers.shift());             // shift inferred
/// assert_eq!(event.raw_char, Some('A'));         // original preserved
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    /// The key, normalized. ASCII letters are always lowercase.
    pub key: Key,
    /// Modifier keys held during the event.
    pub modifiers: Modifiers,
    /// Whether this is a press, release, or repeat.
    pub kind: KeyEventKind,
    /// The character the terminal actually sent, if this was a
    /// character key. `None` for non-character keys.
    pub raw_char: Option<char>,
}

impl KeyEvent {
    /// Creates a key press event with no modifiers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::key::{Key, KeyEvent};
    ///
    /// let event = KeyEvent::new(Key::Enter);
    /// assert_eq!(event.key, Key::Enter);
    /// assert!(event.modifiers.is_none());
    /// assert!(event.raw_char.is_none());
    /// ```
    pub fn new(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers::NONE,
            kind: KeyEventKind::Press,
            raw_char: None,
        }
    }

    /// Creates a normalized character key press.
    ///
    /// Uppercase letters are normalized: `char('A')` produces
    /// `key=Char('a')`, `modifiers=SHIFT`, `raw_char=Some('A')`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::key::{Key, KeyEvent, Modifiers};
    ///
    /// let lower = KeyEvent::char('a');
    /// assert_eq!(lower.key, Key::Char('a'));
    /// assert!(lower.modifiers.is_none());
    /// assert_eq!(lower.raw_char, Some('a'));
    ///
    /// let upper = KeyEvent::char('A');
    /// assert_eq!(upper.key, Key::Char('a'));
    /// assert!(upper.modifiers.shift());
    /// assert_eq!(upper.raw_char, Some('A'));
    /// ```
    pub fn char(c: char) -> Self {
        if c.is_ascii_uppercase() {
            Self {
                key: Key::Char(c.to_ascii_lowercase()),
                modifiers: Modifiers::SHIFT,
                kind: KeyEventKind::Press,
                raw_char: Some(c),
            }
        } else {
            Self {
                key: Key::Char(c),
                modifiers: Modifiers::NONE,
                kind: KeyEventKind::Press,
                raw_char: Some(c),
            }
        }
    }

    /// Creates a Ctrl+character key press.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::input::key::{Key, KeyEvent, Modifiers};
    ///
    /// let event = KeyEvent::ctrl('c');
    /// assert_eq!(event.key, Key::Char('c'));
    /// assert!(event.modifiers.ctrl());
    /// ```
    pub fn ctrl(c: char) -> Self {
        Self {
            key: Key::Char(c.to_ascii_lowercase()),
            modifiers: Modifiers::CONTROL,
            kind: KeyEventKind::Press,
            raw_char: Some(c),
        }
    }

    /// Returns true if this is a press event.
    pub fn is_press(&self) -> bool {
        self.kind == KeyEventKind::Press
    }

    /// Returns true if this is a release event.
    pub fn is_release(&self) -> bool {
        self.kind == KeyEventKind::Release
    }
}

/// The kind of key event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyEventKind {
    /// A key was pressed.
    Press,
    /// A key was released (not supported by all terminals).
    Release,
    /// A key is being held and is repeating.
    Repeat,
}

/// Modifier keys held during an input event.
///
/// # Example
///
/// ```rust
/// use envision::input::key::Modifiers;
///
/// let mods = Modifiers::CONTROL | Modifiers::SHIFT;
/// assert!(mods.ctrl());
/// assert!(mods.shift());
/// assert!(!mods.alt());
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Modifiers(u8);

impl Modifiers {
    /// No modifier keys held.
    pub const NONE: Self = Self(0);
    /// Shift key held.
    pub const SHIFT: Self = Self(1 << 0);
    /// Control key held.
    pub const CONTROL: Self = Self(1 << 1);
    /// Alt/Option key held.
    pub const ALT: Self = Self(1 << 2);
    /// Super/Cmd/Win key held.
    pub const SUPER: Self = Self(1 << 3);

    /// Returns true if the shift key is held.
    pub fn shift(self) -> bool {
        self.0 & Self::SHIFT.0 != 0
    }

    /// Returns true if the control key is held.
    pub fn ctrl(self) -> bool {
        self.0 & Self::CONTROL.0 != 0
    }

    /// Returns true if the alt/option key is held.
    pub fn alt(self) -> bool {
        self.0 & Self::ALT.0 != 0
    }

    /// Returns true if the super/cmd/win key is held.
    pub fn super_key(self) -> bool {
        self.0 & Self::SUPER.0 != 0
    }

    /// Returns true if no modifier keys are held.
    pub fn is_none(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for Modifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Modifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for Modifiers {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_equality() {
        assert_eq!(Key::Char('a'), Key::Char('a'));
        assert_ne!(Key::Char('a'), Key::Char('b'));
        assert_ne!(Key::Char('a'), Key::Enter);
        assert_eq!(Key::F(5), Key::F(5));
        assert_ne!(Key::F(5), Key::F(6));
    }

    #[test]
    fn test_key_event_new() {
        let event = KeyEvent::new(Key::Enter);
        assert_eq!(event.key, Key::Enter);
        assert!(event.modifiers.is_none());
        assert_eq!(event.kind, KeyEventKind::Press);
        assert!(event.raw_char.is_none());
    }

    #[test]
    fn test_key_event_char_lowercase() {
        let event = KeyEvent::char('a');
        assert_eq!(event.key, Key::Char('a'));
        assert!(event.modifiers.is_none());
        assert_eq!(event.raw_char, Some('a'));
    }

    #[test]
    fn test_key_event_char_uppercase_normalizes() {
        let event = KeyEvent::char('A');
        assert_eq!(event.key, Key::Char('a'));
        assert!(event.modifiers.shift());
        assert_eq!(event.raw_char, Some('A'));
    }

    #[test]
    fn test_key_event_ctrl() {
        let event = KeyEvent::ctrl('c');
        assert_eq!(event.key, Key::Char('c'));
        assert!(event.modifiers.ctrl());
        assert!(!event.modifiers.shift());
        assert_eq!(event.raw_char, Some('c'));
    }

    #[test]
    fn test_key_event_is_press_release() {
        let press = KeyEvent::new(Key::Enter);
        assert!(press.is_press());
        assert!(!press.is_release());

        let release = KeyEvent {
            kind: KeyEventKind::Release,
            ..KeyEvent::new(Key::Enter)
        };
        assert!(!release.is_press());
        assert!(release.is_release());
    }

    #[test]
    fn test_modifiers_default() {
        let m = Modifiers::default();
        assert!(m.is_none());
        assert!(!m.shift());
        assert!(!m.ctrl());
        assert!(!m.alt());
        assert!(!m.super_key());
    }

    #[test]
    fn test_modifiers_individual() {
        assert!(Modifiers::SHIFT.shift());
        assert!(!Modifiers::SHIFT.ctrl());
        assert!(Modifiers::CONTROL.ctrl());
        assert!(!Modifiers::CONTROL.shift());
        assert!(Modifiers::ALT.alt());
        assert!(Modifiers::SUPER.super_key());
    }

    #[test]
    fn test_modifiers_bitor() {
        let mods = Modifiers::CONTROL | Modifiers::SHIFT;
        assert!(mods.ctrl());
        assert!(mods.shift());
        assert!(!mods.alt());
    }

    #[test]
    fn test_modifiers_bitor_assign() {
        let mut mods = Modifiers::NONE;
        mods |= Modifiers::ALT;
        assert!(mods.alt());
        assert!(!mods.ctrl());
    }

    #[test]
    fn test_modifiers_bitand() {
        let mods = Modifiers::CONTROL | Modifiers::SHIFT;
        let masked = mods & Modifiers::CONTROL;
        assert!(masked.ctrl());
        assert!(!masked.shift());
    }

    #[test]
    fn test_key_event_kind_equality() {
        assert_eq!(KeyEventKind::Press, KeyEventKind::Press);
        assert_ne!(KeyEventKind::Press, KeyEventKind::Release);
    }
}
