# Envision-owned input types — design

**Status:** approved
**Date:** 2026-04-11
**Target version:** 0.14.0 (breaking)
**Source:** backend abstraction investigation — decouple envision's
public API from crossterm's event types

## Problem

Envision's `Event` enum wraps crossterm's `KeyEvent` and `MouseEvent`
structs directly as variant payloads. `src/input/mod.rs` re-exports 8
crossterm types (`KeyCode`, `KeyEvent`, `KeyModifiers`, etc.) as
envision's public API. Every component that pattern-matches on key
codes is matching on crossterm's enum, and every downstream user who
writes `KeyCode::Char('q')` depends on a crossterm type even though
they imported it from `envision::input`.

Additionally, crossterm's key handling has known ambiguities:
- `Shift+A` sometimes arrives as `Char('A')` with SHIFT modifier,
  sometimes as `Char('A')` with no modifier
- `BackTab` exists as a separate variant instead of `Tab + SHIFT`
- Lock key state (`CapsLock`, `NumLock`) is mixed into event types
  that most TUI apps don't need
- Media keys, modifier-only keys, and other niche variants bloat the
  enum for no practical TUI benefit

## Goal

Define envision-owned input types (`Key`, `KeyEvent`, `Modifiers`,
`MouseEvent`, etc.) that:

1. Remove all crossterm types from the public API
2. Normalize key events so keybindings are unambiguous
3. Preserve the raw character for text input via `raw_char`
4. Drop niche key variants (Media, Modifier-only, lock keys) silently
5. Provide convenience constructors for the common case
6. Enable a future backend abstraction (0.15.0+) where the input
   types are backend-independent

Non-goals:
- Supporting alternative backends (termion, termwiz) in this PR.
  The types ARE backend-agnostic but the only converter implemented
  is `from_crossterm_*`. Additional converters land later.
- Full Unicode / IME / dead-key text input handling. `raw_char`
  preserves what the terminal sends; locale-aware composition is a
  separate project.
- Changing the `Event` enum's top-level variants (Key, Mouse, Resize,
  FocusGained, FocusLost, Paste). Only the inner payload types change.

## Design

### New module layout

```
src/input/
  mod.rs          — re-exports envision types (no crossterm)
  key.rs          — Key, KeyEvent, KeyEventKind, Modifiers (NEW)
  mouse.rs        — MouseEvent, MouseEventKind, MouseButton (NEW)
  convert.rs      — private crossterm→envision converters (NEW)
  events/mod.rs   — Event enum (MODIFIED: uses envision types)
  queue/mod.rs    — EventQueue (MODIFIED: uses envision types)
```

### `Key` enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A character key, always normalized to lowercase for ASCII letters.
    /// Check `modifiers.shift()` to detect uppercase intent.
    /// Read `raw_char` on `KeyEvent` for the actual terminal character.
    Char(char),
    /// Function key (F1 through F24).
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
```

16 variants. Crossterm's `BackTab`, `Null`, `CapsLock`, `ScrollLock`,
`NumLock`, `PrintScreen`, `Pause`, `Menu`, `KeypadBegin`,
`Media(MediaKeyCode)`, and `Modifier(ModifierKeyCode)` are all
dropped. Unknown keys produce `None` from the converter and are
silently ignored.

### `KeyEvent` struct

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    /// The key that was pressed, normalized.
    /// For ASCII letters, always lowercase regardless of shift or
    /// caps lock. Use `raw_char` for the actual terminal character.
    pub key: Key,
    /// Modifier keys held during the event.
    pub modifiers: Modifiers,
    /// Whether this is a press, release, or repeat.
    pub kind: KeyEventKind,
    /// The character the terminal actually sent, if this was a
    /// character key. Preserves case (uppercase for Shift+A or
    /// caps-lock A). `None` for non-character keys (Enter, arrows,
    /// function keys, etc.).
    ///
    /// Use this for text input. Use `key` for keybindings.
    pub raw_char: Option<char>,
}
```

Convenience constructors:

```rust
impl KeyEvent {
    /// Creates a press event with no modifiers.
    pub fn new(key: Key) -> Self;

    /// Creates a normalized Char press event.
    /// `char('A')` produces key=Char('a'), modifiers=SHIFT,
    /// raw_char=Some('A').
    pub fn char(c: char) -> Self;

    /// Creates a Ctrl+char press event.
    /// `ctrl('c')` produces key=Char('c'), modifiers=CONTROL,
    /// raw_char=Some('c').
    pub fn ctrl(c: char) -> Self;

    /// Returns true if this is a press event.
    pub fn is_press(&self) -> bool;

    /// Returns true if this is a release event.
    pub fn is_release(&self) -> bool;
}
```

### `KeyEventKind` enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyEventKind {
    Press,
    Release,
    Repeat,
}
```

### `Modifiers` struct

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Modifiers(u8);

impl Modifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1 << 0);
    pub const CONTROL: Self = Self(1 << 1);
    pub const ALT: Self = Self(1 << 2);
    pub const SUPER: Self = Self(1 << 3);

    pub fn shift(self) -> bool;
    pub fn ctrl(self) -> bool;
    pub fn alt(self) -> bool;
    pub fn super_key(self) -> bool;
    pub fn is_none(self) -> bool;
}

impl BitOr for Modifiers { ... }
impl BitAnd for Modifiers { ... }
impl BitOrAssign for Modifiers { ... }
```

### Mouse types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub column: u16,
    pub row: u16,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseEventKind {
    Down(MouseButton),
    Up(MouseButton),
    Drag(MouseButton),
    Moved,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}
```

### `Event` enum (modified)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    FocusGained,
    FocusLost,
    Paste(String),
}
```

Convenience methods (existing API surface, updated types):

```rust
impl Event {
    pub fn key(key: Key) -> Self;
    pub fn char(c: char) -> Self;
    pub fn ctrl(c: char) -> Self;
    pub fn as_key(&self) -> Option<&KeyEvent>;
    pub fn as_mouse(&self) -> Option<&MouseEvent>;
}
```

### Conversion layer (all private)

New file `src/input/convert.rs`:

```rust
pub(crate) fn from_crossterm_event(
    e: crossterm::event::Event,
) -> Option<Event>;

pub(crate) fn from_crossterm_key(
    e: crossterm::event::KeyEvent,
) -> Option<KeyEvent>;

pub(crate) fn from_crossterm_mouse(
    e: crossterm::event::MouseEvent,
) -> MouseEvent;

pub(crate) fn from_crossterm_modifiers(
    m: crossterm::event::KeyModifiers,
) -> Modifiers;
```

Normalization rules in `from_crossterm_key`:

| Crossterm input | Envision output |
|-----------------|-----------------|
| `Char('A')` + SHIFT | `key=Char('a')`, `mods=SHIFT`, `raw_char=Some('A')` |
| `Char('A')` + no SHIFT (caps lock) | `key=Char('a')`, `mods=NONE`, `raw_char=Some('A')` |
| `Char('a')` + no mods | `key=Char('a')`, `mods=NONE`, `raw_char=Some('a')` |
| `Char('!')` + SHIFT | `key=Char('!')`, `mods=SHIFT`, `raw_char=Some('!')` |
| `BackTab` | `key=Tab`, `mods=SHIFT`, `raw_char=None` |
| `Char('\x03')` (raw Ctrl+C) | `key=Char('c')`, `mods=CONTROL`, `raw_char=Some('\x03')` |
| `Enter` | `key=Enter`, `mods=NONE`, `raw_char=None` |
| `F(5)` | `key=F(5)`, `mods=NONE`, `raw_char=None` |
| `Media(Play)` | `None` (dropped) |
| `Modifier(LeftShift)` | `None` (dropped) |
| `Null` | `None` (dropped) |

Key normalization rule for letters: if `c.is_ascii_uppercase()`,
lowercase it regardless of whether SHIFT is set. The SHIFT modifier
from crossterm is preserved as-is. `raw_char` always gets the
original character.

For non-letter chars (symbols, digits): no normalization. `Char('!')`
stays as `Char('!')`. We don't try to infer the base key because
that's keyboard-layout-dependent.

For control characters (`'\x01'` through `'\x1a'`): convert to the
corresponding letter. `'\x03'` → `Char('c') + CONTROL`.

### What's removed from public API

1. All `pub use crossterm::event::*` re-exports from `src/input/mod.rs`
2. `From<crossterm::event::Event> for Event` (replaced by private function)
3. `From<Event> for crossterm::event::Event` (dropped — lossy after normalization)
4. `From<crossterm::event::KeyEvent> for Event` (dropped)
5. `From<crossterm::event::MouseEvent> for Event` (dropped)
6. `KeyEventBuilder` / `MouseEventBuilder` (rewritten to produce envision types)

### What changes in `src/input/mod.rs`

```rust
// BEFORE
pub use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
    MouseButton, MouseEvent, MouseEventKind,
};

// AFTER
pub use key::{Key, KeyEvent, KeyEventKind, Modifiers};
pub use mouse::{MouseButton, MouseEvent, MouseEventKind};
```

### What changes in `TerminalEventSubscription`

```rust
// BEFORE (src/app/subscription/terminal.rs)
pub struct TerminalEventSubscription<M, F>
where
    F: Fn(crossterm::event::Event) -> Option<M> + Send + 'static,

// AFTER
pub struct TerminalEventSubscription<M, F>
where
    F: Fn(Event) -> Option<M> + Send + 'static,
```

The subscription's event loop calls `from_crossterm_event(raw)`
before passing to the handler. Unknown events (Media, etc.) are
filtered out before the handler sees them.

### What changes in `src/app/runtime/terminal.rs`

```rust
// BEFORE
if let Some(envision_event) = Self::convert_crossterm_event(&event) {

// AFTER
if let Some(envision_event) = crate::input::convert::from_crossterm_event(event) {
```

The private `convert_crossterm_event` method is deleted. The converter
lives in `src/input/convert.rs` and is shared by the runtime and the
terminal subscription.

## Component migration

### Pattern changes

```rust
// BEFORE
use crate::input::{Event, KeyCode};

fn handle_event(state: &Self::State, event: &Event, ctx: &EventContext) -> Option<Self::Message> {
    if let Some(key) = event.as_key() {
        match key.code {
            KeyCode::Enter => Some(Msg::Submit),
            KeyCode::Char('j') | KeyCode::Down => Some(Msg::Down),
            KeyCode::BackTab => Some(Msg::FocusPrev),
            _ => None,
        }
    } else { None }
}

// AFTER
use crate::input::{Event, Key};

fn handle_event(state: &Self::State, event: &Event, ctx: &EventContext) -> Option<Self::Message> {
    if let Some(key) = event.as_key() {
        match key.key {
            Key::Enter => Some(Msg::Submit),
            Key::Char('j') | Key::Down => Some(Msg::Down),
            Key::Tab if key.modifiers.shift() => Some(Msg::FocusPrev),
            _ => None,
        }
    } else { None }
}
```

Substitution rules:
1. `use crate::input::{Event, KeyCode}` → `use crate::input::{Event, Key}`
2. `key.code` → `key.key`
3. `KeyCode::X` → `Key::X` (for all kept variants)
4. `KeyCode::BackTab` → `Key::Tab if key.modifiers.shift()`
5. `KeyCode::Char(c)` → `Key::Char(c)` (identical for lowercase; uppercase patterns need review — `Char('A')` in existing code may mean "user typed A" which now normalizes to `Char('a') + SHIFT`)
6. `KeyModifiers` references → `Modifiers`
7. `key.modifiers.contains(KeyModifiers::SHIFT)` → `key.modifiers.shift()`
8. `key.modifiers.contains(KeyModifiers::CONTROL)` → `key.modifiers.ctrl()`

### Text input components

Components that insert characters into a buffer (LineInput, TextArea,
InputField, NumberInput, SearchableList's search field, CommandPalette's
search field):

```rust
// BEFORE
KeyCode::Char(c) => { buffer.insert(c); }

// AFTER
Key::Char(_) => {
    if let Some(c) = key.raw_char {
        buffer.insert(c);
    }
}
```

These components read `raw_char` instead of the normalized `key` char
to preserve the user's actual input (uppercase from shift or caps lock,
symbols as typed).

## Files affected

**New files (~3):**
- `src/input/key.rs`
- `src/input/mouse.rs`
- `src/input/convert.rs`

**Core modifications (~6):**
- `src/input/mod.rs` — re-exports
- `src/input/events/mod.rs` — Event enum, remove From impls
- `src/input/queue/mod.rs` — use envision types
- `src/app/runtime/terminal.rs` — use converter
- `src/app/subscription/terminal.rs` — handler bound
- `src/lib.rs` — re-exports (KeyCode → Key, KeyModifiers → Modifiers)

**Component cascade (~73):**
- Every component's `handle_event` impl
- Text input components additionally change char insertion path

**Tests (~100+):**
- Component test files (KeyCode → Key, key.code → key.key)
- Integration tests
- Input module tests
- Tests that bypass the facade with direct crossterm imports

**Examples (~89):**
- Any example that handles key events

**Benchmarks (~3):**
- component_events.rs, memory.rs (construct key events)

**Documentation:**
- CHANGELOG.md — breaking changes
- MIGRATION.md — input type migration section
- CONTRIBUTING.md — update key handling examples
- README.md — if it references KeyCode

## PR strategy

**One atomic PR.** Same rationale as RenderContext: partial updates
won't compile because the framework passes `KeyEvent` to components
and the types must match.

Estimated: ~3000 lines changed across ~180 files. The per-file
transformation is smaller than RenderContext (mostly `KeyCode` → `Key`
and `key.code` → `key.key`).

## Testing

### Regression coverage

All existing tests must pass. Component behavior tests exercise the
full event→message→update cycle and are the primary regression guards.
Snapshot tests are unaffected (they test rendering, not input).

### New tests for input types

**Key/KeyEvent/Modifiers unit tests:**
- Construction, equality, Debug output
- `KeyEvent::char('A')` normalizes to `Key::Char('a')` + SHIFT + `raw_char=Some('A')`
- `KeyEvent::ctrl('c')` produces CONTROL modifier
- `KeyEvent::new(Key::Enter)` has no modifiers, no raw_char
- `Modifiers` flag operations: OR, AND, individual checks, is_none

**Normalization tests (the critical ones):**
- Shift+A → lowercase + SHIFT + raw uppercase
- Caps-lock A (no shift) → lowercase + no SHIFT + raw uppercase
- Plain a → lowercase + no mods + raw lowercase
- Ctrl+C (as '\x03') → Char('c') + CONTROL + raw '\x03'
- BackTab → Tab + SHIFT
- Symbols (Shift+1='!') → Char('!') + SHIFT (no base-key inference)
- Unknown keys (Media, Modifier) → None
- All 16 Key variants mapped from crossterm equivalents

**Mouse conversion tests:**
- Button mapping, modifier mapping, coordinate preservation

**Event convenience tests:**
- `Event::char('q')` → `Event::Key(KeyEvent { key: Char('q'), ... })`
- `Event::ctrl('c')` → correct
- `as_key()`, `as_mouse()` extractors

## Risk and rollback

**Risk: medium.** Comparable to RenderContext — many files, mechanical
transformation. The key normalization logic is the riskiest new code
(easy to get wrong for edge cases). The normalization test suite is
the primary defense.

**Rollback:** revert the single PR.
