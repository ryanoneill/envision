//! Private converters from crossterm event types to envision event types.
//!
//! These functions are `pub(crate)` so the runtime and subscription modules
//! can call them, but they are not part of envision's public API.

use super::events::Event;
use super::key::{Key, KeyEvent, KeyEventKind, Modifiers};
use super::mouse::{MouseButton, MouseEvent, MouseEventKind};

/// Converts a crossterm event to an envision event.
///
/// Returns `None` for key events that envision doesn't model (e.g. CapsLock)
/// and for key release/repeat events (only presses are forwarded).
pub(crate) fn from_crossterm_event(event: crossterm::event::Event) -> Option<Event> {
    match event {
        crossterm::event::Event::Key(key) => {
            // Only handle key press events, not release or repeat
            if key.kind == crossterm::event::KeyEventKind::Press {
                from_crossterm_key(key).map(Event::Key)
            } else {
                None
            }
        }
        crossterm::event::Event::Mouse(mouse) => Some(Event::Mouse(from_crossterm_mouse(mouse))),
        crossterm::event::Event::Resize(w, h) => Some(Event::Resize(w, h)),
        crossterm::event::Event::FocusGained => Some(Event::FocusGained),
        crossterm::event::Event::FocusLost => Some(Event::FocusLost),
        crossterm::event::Event::Paste(s) => Some(Event::Paste(s)),
    }
}

/// Converts a crossterm key event to an envision key event.
///
/// Returns `None` for key codes that envision doesn't model.
pub(crate) fn from_crossterm_key(key: crossterm::event::KeyEvent) -> Option<KeyEvent> {
    let mut modifiers = from_crossterm_modifiers(key.modifiers);
    let kind = from_crossterm_key_kind(key.kind);

    let (envision_key, raw_char) = match key.code {
        crossterm::event::KeyCode::Char(c) => {
            // Normalize control characters (Ctrl+letter sends 0x01-0x1A)
            if c.is_ascii_control() && c != '\t' && c != '\r' && c != '\n' {
                let letter = (c as u8 + b'a' - 1) as char;
                modifiers |= Modifiers::CONTROL;
                (Key::Char(letter), Some(c))
            }
            // Normalize uppercase ASCII letters to lowercase
            else if c.is_ascii_uppercase() {
                (Key::Char(c.to_ascii_lowercase()), Some(c))
            }
            // Everything else passes through
            else {
                (Key::Char(c), Some(c))
            }
        }
        crossterm::event::KeyCode::F(n) => (Key::F(n), None),
        crossterm::event::KeyCode::Backspace => (Key::Backspace, None),
        crossterm::event::KeyCode::Enter => (Key::Enter, None),
        crossterm::event::KeyCode::Left => (Key::Left, None),
        crossterm::event::KeyCode::Right => (Key::Right, None),
        crossterm::event::KeyCode::Up => (Key::Up, None),
        crossterm::event::KeyCode::Down => (Key::Down, None),
        crossterm::event::KeyCode::Home => (Key::Home, None),
        crossterm::event::KeyCode::End => (Key::End, None),
        crossterm::event::KeyCode::PageUp => (Key::PageUp, None),
        crossterm::event::KeyCode::PageDown => (Key::PageDown, None),
        crossterm::event::KeyCode::Tab => (Key::Tab, None),
        crossterm::event::KeyCode::BackTab => {
            modifiers |= Modifiers::SHIFT;
            (Key::Tab, None)
        }
        crossterm::event::KeyCode::Delete => (Key::Delete, None),
        crossterm::event::KeyCode::Insert => (Key::Insert, None),
        crossterm::event::KeyCode::Esc => (Key::Esc, None),
        // Dropped variants: Null, CapsLock, ScrollLock, NumLock,
        // PrintScreen, Pause, Menu, KeypadBegin, Media, Modifier
        _ => return None,
    };

    Some(KeyEvent {
        code: envision_key,
        modifiers,
        kind,
        raw_char,
    })
}

/// Converts a crossterm mouse event to an envision mouse event.
pub(crate) fn from_crossterm_mouse(mouse: crossterm::event::MouseEvent) -> MouseEvent {
    MouseEvent {
        kind: from_crossterm_mouse_kind(mouse.kind),
        column: mouse.column,
        row: mouse.row,
        modifiers: from_crossterm_modifiers(mouse.modifiers),
    }
}

/// Converts crossterm key modifiers to envision modifiers.
pub(crate) fn from_crossterm_modifiers(mods: crossterm::event::KeyModifiers) -> Modifiers {
    let mut result = Modifiers::NONE;
    if mods.contains(crossterm::event::KeyModifiers::SHIFT) {
        result |= Modifiers::SHIFT;
    }
    if mods.contains(crossterm::event::KeyModifiers::CONTROL) {
        result |= Modifiers::CONTROL;
    }
    if mods.contains(crossterm::event::KeyModifiers::ALT) {
        result |= Modifiers::ALT;
    }
    if mods.contains(crossterm::event::KeyModifiers::SUPER) {
        result |= Modifiers::SUPER;
    }
    result
}

pub(crate) fn from_crossterm_key_kind(kind: crossterm::event::KeyEventKind) -> KeyEventKind {
    match kind {
        crossterm::event::KeyEventKind::Press => KeyEventKind::Press,
        crossterm::event::KeyEventKind::Release => KeyEventKind::Release,
        crossterm::event::KeyEventKind::Repeat => KeyEventKind::Repeat,
    }
}

pub(crate) fn from_crossterm_mouse_kind(kind: crossterm::event::MouseEventKind) -> MouseEventKind {
    match kind {
        crossterm::event::MouseEventKind::Down(b) => MouseEventKind::Down(from_crossterm_button(b)),
        crossterm::event::MouseEventKind::Up(b) => MouseEventKind::Up(from_crossterm_button(b)),
        crossterm::event::MouseEventKind::Drag(b) => MouseEventKind::Drag(from_crossterm_button(b)),
        crossterm::event::MouseEventKind::Moved => MouseEventKind::Moved,
        crossterm::event::MouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
        crossterm::event::MouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
        crossterm::event::MouseEventKind::ScrollLeft => MouseEventKind::ScrollLeft,
        crossterm::event::MouseEventKind::ScrollRight => MouseEventKind::ScrollRight,
    }
}

pub(crate) fn from_crossterm_button(button: crossterm::event::MouseButton) -> MouseButton {
    match button {
        crossterm::event::MouseButton::Left => MouseButton::Left,
        crossterm::event::MouseButton::Right => MouseButton::Right,
        crossterm::event::MouseButton::Middle => MouseButton::Middle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event as ct;

    fn ct_key(code: ct::KeyCode) -> ct::KeyEvent {
        ct::KeyEvent::new(code, ct::KeyModifiers::empty())
    }

    fn ct_key_with_mods(code: ct::KeyCode, mods: ct::KeyModifiers) -> ct::KeyEvent {
        ct::KeyEvent::new(code, mods)
    }

    // ========== Key normalization tests ==========

    #[test]
    fn test_lowercase_char() {
        let result = from_crossterm_key(ct_key(ct::KeyCode::Char('a'))).unwrap();
        assert_eq!(result.code, Key::Char('a'));
        assert!(result.modifiers.is_none());
        assert_eq!(result.raw_char, Some('a'));
    }

    #[test]
    fn test_uppercase_with_shift_normalizes() {
        let result = from_crossterm_key(ct_key_with_mods(
            ct::KeyCode::Char('A'),
            ct::KeyModifiers::SHIFT,
        ))
        .unwrap();
        assert_eq!(result.code, Key::Char('a'));
        assert!(result.modifiers.shift());
        assert_eq!(result.raw_char, Some('A'));
    }

    #[test]
    fn test_uppercase_without_shift_caps_lock() {
        // Caps lock sends uppercase without SHIFT modifier
        let result = from_crossterm_key(ct_key(ct::KeyCode::Char('A'))).unwrap();
        assert_eq!(result.code, Key::Char('a'));
        assert!(!result.modifiers.shift());
        assert_eq!(result.raw_char, Some('A'));
    }

    #[test]
    fn test_symbol_with_shift_preserved() {
        let result = from_crossterm_key(ct_key_with_mods(
            ct::KeyCode::Char('!'),
            ct::KeyModifiers::SHIFT,
        ))
        .unwrap();
        assert_eq!(result.code, Key::Char('!'));
        assert!(result.modifiers.shift());
        assert_eq!(result.raw_char, Some('!'));
    }

    #[test]
    fn test_ctrl_c_from_modifier() {
        let result = from_crossterm_key(ct_key_with_mods(
            ct::KeyCode::Char('c'),
            ct::KeyModifiers::CONTROL,
        ))
        .unwrap();
        assert_eq!(result.code, Key::Char('c'));
        assert!(result.modifiers.ctrl());
        assert_eq!(result.raw_char, Some('c'));
    }

    #[test]
    fn test_ctrl_c_from_raw_control_char() {
        // Some terminals send '\x03' instead of 'c' + CONTROL
        let result = from_crossterm_key(ct_key(ct::KeyCode::Char('\x03'))).unwrap();
        assert_eq!(result.code, Key::Char('c'));
        assert!(result.modifiers.ctrl());
        assert_eq!(result.raw_char, Some('\x03'));
    }

    #[test]
    fn test_backtab_becomes_tab_with_shift() {
        let result = from_crossterm_key(ct_key(ct::KeyCode::BackTab)).unwrap();
        assert_eq!(result.code, Key::Tab);
        assert!(result.modifiers.shift());
        assert!(result.raw_char.is_none());
    }

    #[test]
    fn test_enter() {
        let result = from_crossterm_key(ct_key(ct::KeyCode::Enter)).unwrap();
        assert_eq!(result.code, Key::Enter);
        assert!(result.modifiers.is_none());
        assert!(result.raw_char.is_none());
    }

    #[test]
    fn test_function_key() {
        let result = from_crossterm_key(ct_key(ct::KeyCode::F(5))).unwrap();
        assert_eq!(result.code, Key::F(5));
        assert!(result.raw_char.is_none());
    }

    #[test]
    fn test_arrows() {
        assert_eq!(
            from_crossterm_key(ct_key(ct::KeyCode::Left)).unwrap().code,
            Key::Left
        );
        assert_eq!(
            from_crossterm_key(ct_key(ct::KeyCode::Right)).unwrap().code,
            Key::Right
        );
        assert_eq!(
            from_crossterm_key(ct_key(ct::KeyCode::Up)).unwrap().code,
            Key::Up
        );
        assert_eq!(
            from_crossterm_key(ct_key(ct::KeyCode::Down)).unwrap().code,
            Key::Down
        );
    }

    #[test]
    fn test_navigation_keys() {
        assert_eq!(
            from_crossterm_key(ct_key(ct::KeyCode::Home)).unwrap().code,
            Key::Home
        );
        assert_eq!(
            from_crossterm_key(ct_key(ct::KeyCode::End)).unwrap().code,
            Key::End
        );
        assert_eq!(
            from_crossterm_key(ct_key(ct::KeyCode::PageUp))
                .unwrap()
                .code,
            Key::PageUp
        );
        assert_eq!(
            from_crossterm_key(ct_key(ct::KeyCode::PageDown))
                .unwrap()
                .code,
            Key::PageDown
        );
    }

    #[test]
    fn test_editing_keys() {
        assert_eq!(
            from_crossterm_key(ct_key(ct::KeyCode::Backspace))
                .unwrap()
                .code,
            Key::Backspace
        );
        assert_eq!(
            from_crossterm_key(ct_key(ct::KeyCode::Delete))
                .unwrap()
                .code,
            Key::Delete
        );
        assert_eq!(
            from_crossterm_key(ct_key(ct::KeyCode::Insert))
                .unwrap()
                .code,
            Key::Insert
        );
        assert_eq!(
            from_crossterm_key(ct_key(ct::KeyCode::Tab)).unwrap().code,
            Key::Tab
        );
        assert_eq!(
            from_crossterm_key(ct_key(ct::KeyCode::Esc)).unwrap().code,
            Key::Esc
        );
    }

    #[test]
    fn test_dropped_keys_return_none() {
        assert!(from_crossterm_key(ct_key(ct::KeyCode::Null)).is_none());
        assert!(from_crossterm_key(ct_key(ct::KeyCode::CapsLock)).is_none());
        assert!(from_crossterm_key(ct_key(ct::KeyCode::NumLock)).is_none());
        assert!(from_crossterm_key(ct_key(ct::KeyCode::ScrollLock)).is_none());
        assert!(from_crossterm_key(ct_key(ct::KeyCode::PrintScreen)).is_none());
        assert!(from_crossterm_key(ct_key(ct::KeyCode::Pause)).is_none());
        assert!(from_crossterm_key(ct_key(ct::KeyCode::Menu)).is_none());
        assert!(from_crossterm_key(ct_key(ct::KeyCode::KeypadBegin)).is_none());
    }

    #[test]
    fn test_key_event_kind_mapping() {
        let press = ct::KeyEvent {
            kind: ct::KeyEventKind::Press,
            ..ct_key(ct::KeyCode::Enter)
        };
        assert_eq!(from_crossterm_key(press).unwrap().kind, KeyEventKind::Press);

        let release = ct::KeyEvent {
            kind: ct::KeyEventKind::Release,
            ..ct_key(ct::KeyCode::Enter)
        };
        assert_eq!(
            from_crossterm_key(release).unwrap().kind,
            KeyEventKind::Release
        );

        let repeat = ct::KeyEvent {
            kind: ct::KeyEventKind::Repeat,
            ..ct_key(ct::KeyCode::Enter)
        };
        assert_eq!(
            from_crossterm_key(repeat).unwrap().kind,
            KeyEventKind::Repeat
        );
    }

    // ========== Modifier conversion tests ==========

    #[test]
    fn test_modifier_mapping() {
        let shift = from_crossterm_modifiers(ct::KeyModifiers::SHIFT);
        assert!(shift.shift());
        assert!(!shift.ctrl());

        let ctrl = from_crossterm_modifiers(ct::KeyModifiers::CONTROL);
        assert!(ctrl.ctrl());
        assert!(!ctrl.shift());

        let alt = from_crossterm_modifiers(ct::KeyModifiers::ALT);
        assert!(alt.alt());

        let sup = from_crossterm_modifiers(ct::KeyModifiers::SUPER);
        assert!(sup.super_key());
    }

    #[test]
    fn test_combined_modifiers() {
        let mods = from_crossterm_modifiers(ct::KeyModifiers::SHIFT | ct::KeyModifiers::CONTROL);
        assert!(mods.shift());
        assert!(mods.ctrl());
        assert!(!mods.alt());
    }

    #[test]
    fn test_empty_modifiers() {
        let mods = from_crossterm_modifiers(ct::KeyModifiers::empty());
        assert!(mods.is_none());
    }

    // ========== Mouse conversion tests ==========

    #[test]
    fn test_mouse_button_mapping() {
        let ct_mouse = ct::MouseEvent {
            kind: ct::MouseEventKind::Down(ct::MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: ct::KeyModifiers::empty(),
        };
        let result = from_crossterm_mouse(ct_mouse);
        assert_eq!(result.kind, MouseEventKind::Down(MouseButton::Left));
        assert_eq!(result.column, 10);
        assert_eq!(result.row, 5);
        assert!(result.modifiers.is_none());
    }

    #[test]
    fn test_mouse_scroll() {
        let ct_mouse = ct::MouseEvent {
            kind: ct::MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: ct::KeyModifiers::empty(),
        };
        assert_eq!(
            from_crossterm_mouse(ct_mouse).kind,
            MouseEventKind::ScrollUp
        );
    }

    #[test]
    fn test_mouse_with_modifiers() {
        let ct_mouse = ct::MouseEvent {
            kind: ct::MouseEventKind::Down(ct::MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: ct::KeyModifiers::CONTROL,
        };
        let result = from_crossterm_mouse(ct_mouse);
        assert!(result.modifiers.ctrl());
    }

    // ========== Event-level conversion tests ==========

    #[test]
    fn test_event_key_press() {
        let ct_event = ct::Event::Key(ct_key(ct::KeyCode::Enter));
        let result = from_crossterm_event(ct_event).unwrap();
        assert!(matches!(result, Event::Key(ke) if ke.code == Key::Enter));
    }

    #[test]
    fn test_event_key_release_filtered() {
        let mut key = ct_key(ct::KeyCode::Enter);
        key.kind = ct::KeyEventKind::Release;
        let ct_event = ct::Event::Key(key);
        assert!(from_crossterm_event(ct_event).is_none());
    }

    #[test]
    fn test_event_key_repeat_filtered() {
        let mut key = ct_key(ct::KeyCode::Enter);
        key.kind = ct::KeyEventKind::Repeat;
        let ct_event = ct::Event::Key(key);
        assert!(from_crossterm_event(ct_event).is_none());
    }

    #[test]
    fn test_event_dropped_key_filtered() {
        let ct_event = ct::Event::Key(ct_key(ct::KeyCode::Null));
        assert!(from_crossterm_event(ct_event).is_none());
    }

    #[test]
    fn test_event_mouse() {
        let ct_event = ct::Event::Mouse(ct::MouseEvent {
            kind: ct::MouseEventKind::Down(ct::MouseButton::Left),
            column: 5,
            row: 10,
            modifiers: ct::KeyModifiers::empty(),
        });
        let result = from_crossterm_event(ct_event).unwrap();
        assert!(matches!(result, Event::Mouse(_)));
    }

    #[test]
    fn test_event_resize() {
        let ct_event = ct::Event::Resize(80, 24);
        let result = from_crossterm_event(ct_event).unwrap();
        assert!(matches!(result, Event::Resize(80, 24)));
    }

    #[test]
    fn test_event_focus_gained() {
        let ct_event = ct::Event::FocusGained;
        let result = from_crossterm_event(ct_event).unwrap();
        assert!(matches!(result, Event::FocusGained));
    }

    #[test]
    fn test_event_focus_lost() {
        let ct_event = ct::Event::FocusLost;
        let result = from_crossterm_event(ct_event).unwrap();
        assert!(matches!(result, Event::FocusLost));
    }

    #[test]
    fn test_event_paste() {
        let ct_event = ct::Event::Paste("hello".to_string());
        let result = from_crossterm_event(ct_event).unwrap();
        assert!(matches!(result, Event::Paste(s) if s == "hello"));
    }
}
