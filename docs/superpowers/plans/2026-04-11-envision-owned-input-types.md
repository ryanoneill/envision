# Envision-Owned Input Types Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace crossterm re-exports with envision-owned input types (`Key`, `KeyEvent`, `Modifiers`, `MouseEvent`), normalizing letter keys to lowercase with `raw_char` preserving the original terminal character.

**Architecture:** Three-phase approach: (1) define new types with unit tests (compilable alongside crossterm), (2) add private crossterm→envision converters with normalization tests, (3) atomic switch — rewire Event enum, migrate all 73 components, 89 examples, tests, benchmarks, runtime, and subscriptions in one commit.

**Tech Stack:** Rust (edition 2024), ratatui, crossterm (still a dependency, just hidden), cargo-nextest.

**Spec:** `docs/superpowers/specs/2026-04-11-envision-owned-input-types-design.md`

**Target version:** 0.14.0 (breaking)

---

## Project Context

- **Working branch:** `envision-owned-input-types` (created from main, spec committed at `228b8fc`).
- **Previous refactor context:** The RenderContext refactor (PR #407) just landed on main. Components now use `ctx: &mut RenderContext<'_, '_>` for view and `ctx: &EventContext` for handle_event. This plan builds on that.
- **Atomic strategy:** Tasks 1-2 add types/converters non-destructively. Task 3 is the atomic switch — same pattern as the RenderContext refactor.
- **Signed commits required.** `commit.gpgsign=true`.
- **No warnings allowed.** `cargo clippy --all-targets -- -D warnings`.

---

## File Structure

**New files:**
- `src/input/key.rs` — `Key`, `KeyEvent`, `KeyEventKind`, `Modifiers` + unit tests
- `src/input/mouse.rs` — `MouseEvent`, `MouseEventKind`, `MouseButton` + unit tests
- `src/input/convert.rs` — private crossterm→envision converters + normalization tests

**Modified files (core):**
- `src/input/mod.rs` — re-exports change from crossterm to envision types
- `src/input/events/mod.rs` — `Event` enum uses envision types, `From` impls removed
- `src/input/queue/mod.rs` — uses envision types
- `src/app/runtime/terminal.rs` — uses `from_crossterm_event()` converter
- `src/app/subscription/terminal.rs` — handler bound changes to `Fn(Event)`
- `src/lib.rs` — re-exports `Key` and `Modifiers` instead of `KeyCode` and `KeyModifiers`

**Modified files (cascade):**
- 73 component `mod.rs` files + their test files
- 89 example files
- Integration test files in `tests/`
- Benchmark files in `benches/`
- `CHANGELOG.md`, `MIGRATION.md`, `CONTRIBUTING.md`

---

## Task 1: Define envision-owned types (TDD)

**Goal:** Create `src/input/key.rs` and `src/input/mouse.rs` with all type definitions and unit tests. These coexist with crossterm types — nothing references them yet.

**Files:**
- Create: `src/input/key.rs`
- Create: `src/input/mouse.rs`
- Modify: `src/input/mod.rs` (add module declarations, NOT re-exports yet)

---

- [ ] **Step 1.1: Create `src/input/key.rs` with all key types**

Create the file with the full type definitions. Include doc comments, derives, and all impl blocks:

```rust
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
/// use envision::input::Key;
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
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Delete,
    Insert,
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
/// use envision::input::{Key, KeyEvent, Modifiers};
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
    /// use envision::input::{Key, KeyEvent};
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
    /// use envision::input::{Key, KeyEvent, Modifiers};
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
    /// use envision::input::{Key, KeyEvent, Modifiers};
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
/// use envision::input::Modifiers;
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
```

- [ ] **Step 1.2: Create `src/input/mouse.rs` with all mouse types**

```rust
//! Envision-owned mouse input types.

use super::key::Modifiers;

/// A mouse event.
///
/// # Example
///
/// ```rust
/// use envision::input::{MouseEvent, MouseEventKind, MouseButton, Modifiers};
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
```

- [ ] **Step 1.3: Wire modules into `src/input/mod.rs` (declarations only, NOT re-exports yet)**

Add module declarations to `src/input/mod.rs` WITHOUT changing any existing re-exports. The new modules coexist with crossterm:

Add these lines after the existing `mod queue;` declaration:

```rust
pub mod key;
pub mod mouse;
```

Leave the existing `pub use crossterm::event::*` lines untouched.

- [ ] **Step 1.4: Verify compilation and run tests**

```bash
cargo check -p envision 2>&1 | tail -5
cargo nextest run -p envision input::key::tests input::mouse::tests 2>&1 | tail -10
cargo test --doc -p envision Key KeyEvent Modifiers MouseEvent MouseButton 2>&1 | tail -10
```

Expected: all compile, all tests pass (new tests + existing tests unaffected).

- [ ] **Step 1.5: Format, lint, commit**

```bash
cargo fmt
cargo clippy -p envision -- -D warnings 2>&1 | tail -3
git add src/input/key.rs src/input/mouse.rs src/input/mod.rs
git commit -S -m "$(cat <<'EOF'
Add envision-owned Key, KeyEvent, Modifiers, and mouse types

Defines envision's own input types alongside the existing crossterm
re-exports. Key normalizes ASCII letters to lowercase; KeyEvent
carries raw_char preserving the original terminal character. Modifiers
is a plain u8 struct with shift/ctrl/alt/super flags.

Types coexist with crossterm — the Event enum still wraps crossterm
types. The atomic switch happens in a follow-up commit.

Part of docs/superpowers/specs/2026-04-11-envision-owned-input-types-design.md

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Add crossterm conversion layer (TDD)

**Goal:** Create `src/input/convert.rs` with private converters from crossterm events to envision events. Heavily tested — the normalization logic is the riskiest new code.

**Files:**
- Create: `src/input/convert.rs`
- Modify: `src/input/mod.rs` (add module declaration)

---

- [ ] **Step 2.1: Create `src/input/convert.rs` with converters and tests**

This is the core of the refactor. The converters must be correct for every case in the normalization table from the spec.

```rust
//! Private converters from crossterm event types to envision event types.
//!
//! These functions are `pub(crate)` so the runtime and subscription modules
//! can call them, but they are not part of envision's public API.

use super::events::Event;
use super::key::{Key, KeyEvent, KeyEventKind, Modifiers};
use super::mouse::{MouseButton, MouseEvent, MouseEventKind};

/// Converts a crossterm event to an envision event.
///
/// Returns `None` for events that envision doesn't model (e.g., media keys,
/// modifier-only events, null keys).
pub(crate) fn from_crossterm_event(event: crossterm::event::Event) -> Option<Event> {
    match event {
        crossterm::event::Event::Key(key) => {
            from_crossterm_key(key).map(Event::Key)
        }
        crossterm::event::Event::Mouse(mouse) => {
            Some(Event::Mouse(from_crossterm_mouse(mouse)))
        }
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
        key: envision_key,
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

fn from_crossterm_key_kind(kind: crossterm::event::KeyEventKind) -> KeyEventKind {
    match kind {
        crossterm::event::KeyEventKind::Press => KeyEventKind::Press,
        crossterm::event::KeyEventKind::Release => KeyEventKind::Release,
        crossterm::event::KeyEventKind::Repeat => KeyEventKind::Repeat,
    }
}

fn from_crossterm_mouse_kind(kind: crossterm::event::MouseEventKind) -> MouseEventKind {
    match kind {
        crossterm::event::MouseEventKind::Down(b) => {
            MouseEventKind::Down(from_crossterm_button(b))
        }
        crossterm::event::MouseEventKind::Up(b) => {
            MouseEventKind::Up(from_crossterm_button(b))
        }
        crossterm::event::MouseEventKind::Drag(b) => {
            MouseEventKind::Drag(from_crossterm_button(b))
        }
        crossterm::event::MouseEventKind::Moved => MouseEventKind::Moved,
        crossterm::event::MouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
        crossterm::event::MouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
        crossterm::event::MouseEventKind::ScrollLeft => MouseEventKind::ScrollLeft,
        crossterm::event::MouseEventKind::ScrollRight => MouseEventKind::ScrollRight,
    }
}

fn from_crossterm_button(button: crossterm::event::MouseButton) -> MouseButton {
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
        assert_eq!(result.key, Key::Char('a'));
        assert!(result.modifiers.is_none());
        assert_eq!(result.raw_char, Some('a'));
    }

    #[test]
    fn test_uppercase_with_shift_normalizes() {
        let result = from_crossterm_key(ct_key_with_mods(
            ct::KeyCode::Char('A'),
            ct::KeyModifiers::SHIFT,
        )).unwrap();
        assert_eq!(result.key, Key::Char('a'));
        assert!(result.modifiers.shift());
        assert_eq!(result.raw_char, Some('A'));
    }

    #[test]
    fn test_uppercase_without_shift_caps_lock() {
        // Caps lock sends uppercase without SHIFT modifier
        let result = from_crossterm_key(ct_key(ct::KeyCode::Char('A'))).unwrap();
        assert_eq!(result.key, Key::Char('a'));
        assert!(!result.modifiers.shift());
        assert_eq!(result.raw_char, Some('A'));
    }

    #[test]
    fn test_symbol_with_shift_preserved() {
        let result = from_crossterm_key(ct_key_with_mods(
            ct::KeyCode::Char('!'),
            ct::KeyModifiers::SHIFT,
        )).unwrap();
        assert_eq!(result.key, Key::Char('!'));
        assert!(result.modifiers.shift());
        assert_eq!(result.raw_char, Some('!'));
    }

    #[test]
    fn test_ctrl_c_from_modifier() {
        let result = from_crossterm_key(ct_key_with_mods(
            ct::KeyCode::Char('c'),
            ct::KeyModifiers::CONTROL,
        )).unwrap();
        assert_eq!(result.key, Key::Char('c'));
        assert!(result.modifiers.ctrl());
        assert_eq!(result.raw_char, Some('c'));
    }

    #[test]
    fn test_ctrl_c_from_raw_control_char() {
        // Some terminals send '\x03' instead of 'c' + CONTROL
        let result = from_crossterm_key(ct_key(ct::KeyCode::Char('\x03'))).unwrap();
        assert_eq!(result.key, Key::Char('c'));
        assert!(result.modifiers.ctrl());
        assert_eq!(result.raw_char, Some('\x03'));
    }

    #[test]
    fn test_backtab_becomes_tab_with_shift() {
        let result = from_crossterm_key(ct_key(ct::KeyCode::BackTab)).unwrap();
        assert_eq!(result.key, Key::Tab);
        assert!(result.modifiers.shift());
        assert!(result.raw_char.is_none());
    }

    #[test]
    fn test_enter() {
        let result = from_crossterm_key(ct_key(ct::KeyCode::Enter)).unwrap();
        assert_eq!(result.key, Key::Enter);
        assert!(result.modifiers.is_none());
        assert!(result.raw_char.is_none());
    }

    #[test]
    fn test_function_key() {
        let result = from_crossterm_key(ct_key(ct::KeyCode::F(5))).unwrap();
        assert_eq!(result.key, Key::F(5));
        assert!(result.raw_char.is_none());
    }

    #[test]
    fn test_arrows() {
        assert_eq!(from_crossterm_key(ct_key(ct::KeyCode::Left)).unwrap().key, Key::Left);
        assert_eq!(from_crossterm_key(ct_key(ct::KeyCode::Right)).unwrap().key, Key::Right);
        assert_eq!(from_crossterm_key(ct_key(ct::KeyCode::Up)).unwrap().key, Key::Up);
        assert_eq!(from_crossterm_key(ct_key(ct::KeyCode::Down)).unwrap().key, Key::Down);
    }

    #[test]
    fn test_navigation_keys() {
        assert_eq!(from_crossterm_key(ct_key(ct::KeyCode::Home)).unwrap().key, Key::Home);
        assert_eq!(from_crossterm_key(ct_key(ct::KeyCode::End)).unwrap().key, Key::End);
        assert_eq!(from_crossterm_key(ct_key(ct::KeyCode::PageUp)).unwrap().key, Key::PageUp);
        assert_eq!(from_crossterm_key(ct_key(ct::KeyCode::PageDown)).unwrap().key, Key::PageDown);
    }

    #[test]
    fn test_editing_keys() {
        assert_eq!(from_crossterm_key(ct_key(ct::KeyCode::Backspace)).unwrap().key, Key::Backspace);
        assert_eq!(from_crossterm_key(ct_key(ct::KeyCode::Delete)).unwrap().key, Key::Delete);
        assert_eq!(from_crossterm_key(ct_key(ct::KeyCode::Insert)).unwrap().key, Key::Insert);
        assert_eq!(from_crossterm_key(ct_key(ct::KeyCode::Tab)).unwrap().key, Key::Tab);
        assert_eq!(from_crossterm_key(ct_key(ct::KeyCode::Esc)).unwrap().key, Key::Esc);
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
        assert_eq!(from_crossterm_key(release).unwrap().kind, KeyEventKind::Release);

        let repeat = ct::KeyEvent {
            kind: ct::KeyEventKind::Repeat,
            ..ct_key(ct::KeyCode::Enter)
        };
        assert_eq!(from_crossterm_key(repeat).unwrap().kind, KeyEventKind::Repeat);
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
        let mods = from_crossterm_modifiers(
            ct::KeyModifiers::SHIFT | ct::KeyModifiers::CONTROL,
        );
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
        assert_eq!(from_crossterm_mouse(ct_mouse).kind, MouseEventKind::ScrollUp);
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

    // ========== Full event conversion tests ==========

    #[test]
    fn test_event_key() {
        let ct_event = ct::Event::Key(ct_key(ct::KeyCode::Enter));
        let result = from_crossterm_event(ct_event).unwrap();
        assert!(matches!(result, Event::Key(k) if k.key == Key::Enter));
    }

    #[test]
    fn test_event_resize() {
        let ct_event = ct::Event::Resize(80, 24);
        assert_eq!(from_crossterm_event(ct_event), Some(Event::Resize(80, 24)));
    }

    #[test]
    fn test_event_focus() {
        assert_eq!(
            from_crossterm_event(ct::Event::FocusGained),
            Some(Event::FocusGained),
        );
        assert_eq!(
            from_crossterm_event(ct::Event::FocusLost),
            Some(Event::FocusLost),
        );
    }

    #[test]
    fn test_event_paste() {
        let ct_event = ct::Event::Paste("hello".to_string());
        assert_eq!(
            from_crossterm_event(ct_event),
            Some(Event::Paste("hello".to_string())),
        );
    }
}
```

- [ ] **Step 2.2: Wire the module in `src/input/mod.rs`**

Add after existing module declarations:

```rust
pub(crate) mod convert;
```

- [ ] **Step 2.3: Verify compilation and run tests**

```bash
cargo check -p envision 2>&1 | tail -5
cargo nextest run -p envision input::convert::tests 2>&1 | tail -10
```

Expected: all compile, all converter tests pass. Note: `from_crossterm_event` references `Event::Key(...)` etc. which currently uses crossterm's `KeyEvent`. This will cause a type mismatch because the `Event` enum still uses crossterm types. 

**IMPORTANT:** This means convert.rs can't compile yet because it produces envision `KeyEvent` but `Event::Key` expects crossterm's `KeyEvent`. There are two solutions:

**Solution A:** Have convert.rs return envision types NOT wrapped in Event — just `fn from_crossterm_key(ct::KeyEvent) -> Option<KeyEvent>` and `fn from_crossterm_mouse(ct::MouseEvent) -> MouseEvent`. The `from_crossterm_event` function that wraps them in `Event` is added in Task 3 when the Event enum switches.

**Solution B:** Have convert.rs define its own temporary `ConvertedEvent` type that wraps envision types. Delete it in Task 3.

**Go with Solution A** — simpler. Remove `from_crossterm_event` from this task. Only implement the individual type converters (`from_crossterm_key`, `from_crossterm_mouse`, `from_crossterm_modifiers`). The `from_crossterm_event` function is added in Task 3 alongside the Event enum switch.

Update the file accordingly: remove `from_crossterm_event`, remove the `use super::events::Event;` import, remove the event-level tests (`test_event_key`, `test_event_resize`, etc.). Keep only the key, mouse, and modifier converter functions and their tests.

- [ ] **Step 2.4: Re-run verification**

```bash
cargo check -p envision 2>&1 | tail -5
cargo nextest run -p envision input::convert::tests input::key::tests input::mouse::tests 2>&1 | tail -10
```

Expected: clean compile, all tests pass.

- [ ] **Step 2.5: Format, lint, commit**

```bash
cargo fmt
cargo clippy -p envision -- -D warnings 2>&1 | tail -3
git add src/input/convert.rs src/input/mod.rs
git commit -S -m "$(cat <<'EOF'
Add crossterm-to-envision input type converters

Private converter functions that normalize crossterm events into
envision's owned types: lowercase letters, BackTab → Tab+SHIFT,
control chars → letter+CONTROL, dropped niche keys. Includes 25+
normalization tests covering all cases from the spec table.

The from_crossterm_event wrapper is added in the atomic switch
commit when the Event enum changes to use envision types.

Part of docs/superpowers/specs/2026-04-11-envision-owned-input-types-design.md

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Atomic switch — Event enum + all callers (one commit)

**Goal:** Switch `Event` to use envision types, remove crossterm re-exports, add `from_crossterm_event`, and cascade through all components, examples, tests, benchmarks, runtime, and subscriptions.

**Files:** ~180 files. Same atomic strategy as the RenderContext refactor.

---

- [ ] **Step 3.1: Switch the `Event` enum in `src/input/events/mod.rs`**

Replace crossterm imports with envision imports. Update the `Event` enum variants. Remove all `From<crossterm::*>` impls. Add `from_crossterm_event` to `src/input/convert.rs`. Update convenience methods (`Event::char`, `Event::ctrl`, `Event::key`, `as_key`, `as_mouse`) to use envision types.

The `Event` enum becomes:

```rust
use super::key::{Key, KeyEvent, KeyEventKind, Modifiers};
use super::mouse::{MouseButton, MouseEvent, MouseEventKind};

pub enum Event {
    Key(KeyEvent),       // envision's KeyEvent
    Mouse(MouseEvent),   // envision's MouseEvent
    Resize(u16, u16),
    FocusGained,
    FocusLost,
    Paste(String),
}
```

Update `Event::char(c)` to delegate to `KeyEvent::char(c)`:

```rust
pub fn char(c: char) -> Self {
    Self::Key(KeyEvent::char(c))
}
```

Update `Event::ctrl(c)` to delegate to `KeyEvent::ctrl(c)`.

Update `Event::key(key: Key)` (was `Event::key(code: KeyCode)`):

```rust
pub fn key(key: Key) -> Self {
    Self::Key(KeyEvent::new(key))
}
```

Update `as_key()` to return `Option<&KeyEvent>` (envision's).

Remove `KeyEventBuilder` and `MouseEventBuilder` classes OR rewrite them to produce envision types.

- [ ] **Step 3.2: Update `src/input/mod.rs` re-exports**

Replace:
```rust
pub use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
```

With:
```rust
pub use key::{Key, KeyEvent, KeyEventKind, Modifiers};
pub use mouse::{MouseButton, MouseEvent, MouseEventKind};
```

Update the module doc example to use envision types.

- [ ] **Step 3.3: Update `src/input/queue/mod.rs`**

Replace `crossterm::event::KeyCode` and `crossterm::event::MouseButton` with envision types. Update the `key()`, `char()`, `ctrl()`, `type_str()`, `drag()`, etc. helper methods.

- [ ] **Step 3.4: Add `from_crossterm_event` to `src/input/convert.rs`**

Now that the `Event` enum uses envision types, add the top-level converter:

```rust
pub(crate) fn from_crossterm_event(event: crossterm::event::Event) -> Option<Event> {
    match event {
        crossterm::event::Event::Key(key) => from_crossterm_key(key).map(Event::Key),
        crossterm::event::Event::Mouse(mouse) => Some(Event::Mouse(from_crossterm_mouse(mouse))),
        crossterm::event::Event::Resize(w, h) => Some(Event::Resize(w, h)),
        crossterm::event::Event::FocusGained => Some(Event::FocusGained),
        crossterm::event::Event::FocusLost => Some(Event::FocusLost),
        crossterm::event::Event::Paste(s) => Some(Event::Paste(s)),
    }
}
```

Add tests for event-level conversion.

- [ ] **Step 3.5: Update `src/app/runtime/terminal.rs`**

Replace `Self::convert_crossterm_event(&event)` with `crate::input::convert::from_crossterm_event(event)`. Delete the private `convert_crossterm_event` method.

- [ ] **Step 3.6: Update `src/app/subscription/terminal.rs`**

Change handler bound from `Fn(crossterm::event::Event) -> Option<M>` to `Fn(Event) -> Option<M>`. The subscription's event loop calls `from_crossterm_event` before invoking the handler.

- [ ] **Step 3.7: Update `src/lib.rs` re-exports**

Replace `KeyCode` with `Key` and `KeyModifiers` with `Modifiers` in the lib.rs re-export blocks and prelude. Remove any `crossterm::event::*` re-exports.

- [ ] **Step 3.8: Cascade through all 73 components**

For each component, apply these substitutions in `handle_event`:

1. `use crate::input::{Event, KeyCode}` → `use crate::input::{Event, Key}`
2. `use crate::input::{Event, KeyCode, KeyModifiers}` → `use crate::input::{Event, Key, Modifiers}`
3. `key.code` → `key.key` in match expressions
4. `KeyCode::X` → `Key::X` for all mapped variants
5. `KeyCode::BackTab` → `Key::Tab` with `if key.modifiers.shift()` guard
6. `KeyCode::Char(c) => { buffer.insert(c) }` in text input components → `Key::Char(_) => { if let Some(c) = key.raw_char { buffer.insert(c) } }`
7. `key.modifiers.contains(KeyModifiers::SHIFT)` → `key.modifiers.shift()`
8. `key.modifiers.contains(KeyModifiers::CONTROL)` → `key.modifiers.ctrl()`
9. `KeyCode::Char('G') if shift` patterns (used by scrollable components for Shift+G = go to end) → `Key::Char('g') if key.modifiers.shift()`

Work alphabetically through all 73 components. After each batch of ~10, run `cargo check` to track progress.

- [ ] **Step 3.9: Update all test files**

Component test files use `Event::key(KeyCode::Enter)` → `Event::key(Key::Enter)`, `Event::char('q')` stays the same, tests that construct crossterm events directly → use envision constructors.

Tests that do `use crossterm::event::KeyCode` → `use crate::input::Key`.

- [ ] **Step 3.10: Update all 89 example files**

Any example handling key events: `KeyCode` → `Key`, `key.code` → `key.key`, etc.

- [ ] **Step 3.11: Update integration tests in `tests/`**

Same substitutions as component tests.

- [ ] **Step 3.12: Update benchmarks in `benches/`**

`component_events.rs`, `memory.rs` — update event construction.

- [ ] **Step 3.13: Update doc tests in source files**

Search for `KeyCode` in `///` doc comments and update. Also update doc examples in `src/input/mod.rs`.

- [ ] **Step 3.14: Update `CHANGELOG.md`**

Add under `## [Unreleased]` → `### Breaking`:

```markdown
- **Crossterm event types replaced with envision-owned types.** 
  `KeyCode` is now `Key`, `KeyModifiers` is now `Modifiers`, and
  `KeyEvent`/`MouseEvent` are envision-defined structs. Letter keys
  are normalized to lowercase; use `raw_char` for text input.
  `BackTab` is replaced by `Tab` with `modifiers.shift()`.
  See MIGRATION.md for the upgrade path.

- **`TerminalEventSubscription` handler now receives `Event`**
  instead of `crossterm::event::Event`. Raw crossterm access is
  no longer part of the public API.
```

- [ ] **Step 3.15: Update `MIGRATION.md`**

Add a section for the input type migration with before/after examples:
1. Basic keybinding (`KeyCode::Enter` → `Key::Enter`)
2. Character matching (`KeyCode::Char('q')` → `Key::Char('q')`)
3. BackTab handling (`KeyCode::BackTab` → `Key::Tab if modifiers.shift()`)
4. Text input (`KeyCode::Char(c) => insert(c)` → `Key::Char(_) => insert(raw_char)`)
5. Modifier checking (`contains(KeyModifiers::SHIFT)` → `modifiers.shift()`)
6. Import changes (`use envision::input::{KeyCode, KeyModifiers}` → `{Key, Modifiers}`)

- [ ] **Step 3.16: Update `CONTRIBUTING.md`**

Replace any remaining `KeyCode` references with `Key`.

- [ ] **Step 3.17: Full verification**

```bash
cargo check -p envision --all-targets 2>&1 | tail -5
cargo nextest run -p envision 2>&1 | tail -5
cargo test --doc -p envision 2>&1 | tail -5
cargo build --examples --all-features 2>&1 | tail -5
cargo clippy -p envision --all-targets -- -D warnings 2>&1 | tail -3
cargo fmt --check

# No crossterm types in public API:
grep -rn "crossterm" src/input/mod.rs src/lib.rs | grep "pub use"
# Expected: empty

# No KeyCode references remain:
grep -rn "KeyCode" src/ tests/ examples/ benches/ | grep -v "convert.rs\|//\|doc" | head -5
# Expected: empty (only in convert.rs which uses crossterm internally)
```

- [ ] **Step 3.18: Commit (atomic, signed)**

```bash
git add -A
git commit -S -m "$(cat <<'EOF'
Replace crossterm event types with envision-owned input types

BREAKING CHANGE for 0.14.0.

Replaces crossterm re-exports with envision's own Key, KeyEvent,
Modifiers, MouseEvent types. ASCII letter keys are normalized to
lowercase; raw_char preserves the terminal's original character for
text input. BackTab becomes Tab+SHIFT. Niche keys (Media, Modifier,
lock keys) are dropped. Crossterm conversions are private.

Cascade through all 73 components, 89 examples, integration tests,
benchmarks, runtime, and subscriptions. CHANGELOG and MIGRATION
updated.

Part of docs/superpowers/specs/2026-04-11-envision-owned-input-types-design.md

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Push and open PR

- [ ] **Step 4.1: Push**

```bash
git push -u origin envision-owned-input-types
```

- [ ] **Step 4.2: Open PR**

```bash
gh pr create --title "Replace crossterm events with envision-owned input types (BREAKING, 0.14.0)" --body "$(cat <<'EOF'
## Summary

**Breaking change for 0.14.0.** Replaces crossterm re-exports with envision-owned input types.

**Key changes:**
- `KeyCode` → `Key` (16-variant enum, normalized)
- `KeyModifiers` → `Modifiers` (plain struct with shift/ctrl/alt/super flags)
- `KeyEvent` — envision-owned, with `raw_char` preserving the terminal character
- `MouseEvent` / `MouseEventKind` / `MouseButton` — envision-owned, mirroring crossterm's shape
- `BackTab` → `Tab` with `modifiers.shift()`
- Letter keys always normalized to lowercase (key field); original preserved in raw_char
- All crossterm types removed from public API
- `TerminalEventSubscription` handler now receives `Event` not `crossterm::event::Event`

Design spec: `docs/superpowers/specs/2026-04-11-envision-owned-input-types-design.md`

## Test plan

- [x] All existing tests pass
- [x] Normalization test suite (25+ tests for key conversion edge cases)
- [x] All 89 examples compile
- [x] `cargo clippy --all-targets -- -D warnings` clean
- [x] Zero crossterm types in public re-exports
- [x] CHANGELOG and MIGRATION updated

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 4.3: Check CI**

```bash
gh pr checks $(gh pr view --json number -q .number)
```

---

## Definition of done

- [ ] `Key`, `KeyEvent`, `KeyEventKind`, `Modifiers` exist in `src/input/key.rs`
- [ ] `MouseEvent`, `MouseEventKind`, `MouseButton` exist in `src/input/mouse.rs`
- [ ] Private converters exist in `src/input/convert.rs` with 25+ normalization tests
- [ ] `Event` enum uses envision types
- [ ] `src/input/mod.rs` re-exports envision types, NOT crossterm
- [ ] `src/lib.rs` re-exports `Key` and `Modifiers` (not `KeyCode` and `KeyModifiers`)
- [ ] `TerminalEventSubscription` handler takes `Fn(Event)`
- [ ] All 73 components migrated (`KeyCode` → `Key`, `key.code` → `key.key`)
- [ ] Text input components use `raw_char` for character insertion
- [ ] `BackTab` patterns converted to `Tab + shift()` guards
- [ ] All examples, integration tests, and benchmarks updated
- [ ] Zero `KeyCode` references outside `convert.rs`
- [ ] Zero `pub use crossterm` in `src/input/mod.rs` or `src/lib.rs`
- [ ] CHANGELOG, MIGRATION, CONTRIBUTING updated
- [ ] PR opened, CI green
