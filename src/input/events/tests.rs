use super::*;
use crate::input::key::{Key, KeyEvent, KeyEventKind, Modifiers};
use crate::input::mouse::{MouseButton, MouseEvent, MouseEventKind};

// -------------------------------------------------------------------------
// Event constructors
// -------------------------------------------------------------------------

#[test]
fn test_simulated_event_char() {
    let event = Event::char('a');
    assert!(event.is_key());
    let key = event.as_key().unwrap();
    assert_eq!(key.key, Key::Char('a'));
    assert!(key.modifiers.is_none());
    assert_eq!(key.raw_char, Some('a'));
}

#[test]
fn test_simulated_event_char_with() {
    let event = Event::char_with('A', Modifiers::SHIFT);
    let key = event.as_key().unwrap();
    // KeyEvent::char('A') normalizes to lowercase
    assert_eq!(key.key, Key::Char('a'));
    assert!(key.modifiers.shift());
    assert_eq!(key.raw_char, Some('A'));
}

#[test]
fn test_simulated_event_key() {
    let event = Event::key(Key::Enter);
    let key = event.as_key().unwrap();
    assert_eq!(key.key, Key::Enter);
    assert!(key.modifiers.is_none());
}

#[test]
fn test_simulated_event_key_with() {
    let event = Event::key_with(Key::Tab, Modifiers::SHIFT);
    let key = event.as_key().unwrap();
    assert_eq!(key.key, Key::Tab);
    assert!(key.modifiers.shift());
}

#[test]
fn test_simulated_event_ctrl() {
    let event = Event::ctrl('c');
    let key = event.as_key().unwrap();
    assert_eq!(key.key, Key::Char('c'));
    assert!(key.modifiers.ctrl());
}

#[test]
fn test_simulated_event_alt() {
    let event = Event::alt('x');
    let key = event.as_key().unwrap();
    assert_eq!(key.key, Key::Char('x'));
    assert!(key.modifiers.alt());
}

#[test]
fn test_simulated_event_click() {
    let event = Event::click(10, 20);
    assert!(event.is_mouse());
    let mouse = event.as_mouse().unwrap();
    assert_eq!(mouse.column, 10);
    assert_eq!(mouse.row, 20);
    assert!(matches!(
        mouse.kind,
        MouseEventKind::Down(MouseButton::Left)
    ));
}

#[test]
fn test_simulated_event_click_button() {
    let event = Event::click_button(5, 15, MouseButton::Right);
    let mouse = event.as_mouse().unwrap();
    assert_eq!(mouse.column, 5);
    assert_eq!(mouse.row, 15);
    assert!(matches!(
        mouse.kind,
        MouseEventKind::Down(MouseButton::Right)
    ));
}

#[test]
fn test_simulated_event_mouse_up() {
    let event = Event::mouse_up(10, 20);
    let mouse = event.as_mouse().unwrap();
    assert!(matches!(mouse.kind, MouseEventKind::Up(MouseButton::Left)));
}

#[test]
fn test_simulated_event_mouse_move() {
    let event = Event::mouse_move(30, 40);
    let mouse = event.as_mouse().unwrap();
    assert_eq!(mouse.column, 30);
    assert_eq!(mouse.row, 40);
    assert!(matches!(mouse.kind, MouseEventKind::Moved));
}

#[test]
fn test_simulated_event_mouse_drag() {
    let event = Event::mouse_drag(10, 20, MouseButton::Left);
    let mouse = event.as_mouse().unwrap();
    assert!(matches!(
        mouse.kind,
        MouseEventKind::Drag(MouseButton::Left)
    ));
}

#[test]
fn test_simulated_event_scroll_up() {
    let event = Event::scroll_up(5, 10);
    let mouse = event.as_mouse().unwrap();
    assert!(matches!(mouse.kind, MouseEventKind::ScrollUp));
}

#[test]
fn test_simulated_event_scroll_down() {
    let event = Event::scroll_down(5, 10);
    let mouse = event.as_mouse().unwrap();
    assert!(matches!(mouse.kind, MouseEventKind::ScrollDown));
}

#[test]
fn test_simulated_event_is_key_false() {
    let event = Event::click(0, 0);
    assert!(!event.is_key());
}

#[test]
fn test_simulated_event_is_mouse_false() {
    let event = Event::char('a');
    assert!(!event.is_mouse());
}

#[test]
fn test_simulated_event_as_key_none() {
    let event = Event::click(0, 0);
    assert!(event.as_key().is_none());
}

#[test]
fn test_simulated_event_as_mouse_none() {
    let event = Event::char('a');
    assert!(event.as_mouse().is_none());
}

// -------------------------------------------------------------------------
// From implementations
// -------------------------------------------------------------------------

#[test]
fn test_from_key_event() {
    let key = KeyEvent::new(Key::Char('z'));
    let event: Event = key.into();
    assert!(event.is_key());
}

#[test]
fn test_from_mouse_event() {
    let mouse = MouseEvent {
        kind: MouseEventKind::Moved,
        column: 0,
        row: 0,
        modifiers: Modifiers::NONE,
    };
    let event: Event = mouse.into();
    assert!(event.is_mouse());
}

// -------------------------------------------------------------------------
// KeyEventBuilder
// -------------------------------------------------------------------------

#[test]
fn test_key_event_builder() {
    let event = KeyEventBuilder::new().char('x').ctrl().shift().build();

    assert_eq!(event.key, Key::Char('x'));
    assert!(event.modifiers.ctrl());
    assert!(event.modifiers.shift());
}

#[test]
fn test_key_event_builder_code() {
    let event = KeyEventBuilder::new().code(Key::F(1)).build();

    assert_eq!(event.key, Key::F(1));
}

#[test]
fn test_key_event_builder_alt() {
    let event = KeyEventBuilder::new().char('a').alt().build();

    assert!(event.modifiers.alt());
}

#[test]
fn test_key_event_builder_modifiers() {
    let mods = Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT;
    let event = KeyEventBuilder::new().char('a').modifiers(mods).build();

    assert!(event.modifiers.ctrl());
    assert!(event.modifiers.alt());
    assert!(event.modifiers.shift());
}

#[test]
fn test_key_event_builder_kind() {
    let event = KeyEventBuilder::new()
        .char('a')
        .kind(KeyEventKind::Release)
        .build();

    assert_eq!(event.kind, KeyEventKind::Release);
}

#[test]
fn test_key_event_builder_into_event() {
    let event = KeyEventBuilder::new().char('b').into_event();

    assert!(event.is_key());
    assert_eq!(event.as_key().unwrap().key, Key::Char('b'));
}

#[test]
fn test_key_event_builder_default_code() {
    // When no key is set, should use Key::Esc
    let event = KeyEventBuilder::new().build();
    assert_eq!(event.key, Key::Esc);
}

// -------------------------------------------------------------------------
// MouseEventBuilder
// -------------------------------------------------------------------------

#[test]
fn test_mouse_event_builder() {
    let event = MouseEventBuilder::new()
        .at(5, 10)
        .right_click()
        .ctrl()
        .build();

    assert_eq!(event.column, 5);
    assert_eq!(event.row, 10);
    assert!(matches!(
        event.kind,
        MouseEventKind::Down(MouseButton::Right)
    ));
    assert!(event.modifiers.ctrl());
}

#[test]
fn test_mouse_event_builder_click() {
    let event = MouseEventBuilder::new().at(10, 20).click().build();

    assert!(matches!(
        event.kind,
        MouseEventKind::Down(MouseButton::Left)
    ));
}

#[test]
fn test_mouse_event_builder_middle_click() {
    let event = MouseEventBuilder::new().middle_click().build();

    assert!(matches!(
        event.kind,
        MouseEventKind::Down(MouseButton::Middle)
    ));
}

#[test]
fn test_mouse_event_builder_up() {
    let event = MouseEventBuilder::new().up().build();

    assert!(matches!(event.kind, MouseEventKind::Up(MouseButton::Left)));
}

#[test]
fn test_mouse_event_builder_drag() {
    let event = MouseEventBuilder::new().drag().build();

    assert!(matches!(
        event.kind,
        MouseEventKind::Drag(MouseButton::Left)
    ));
}

#[test]
fn test_mouse_event_builder_scroll_up() {
    let event = MouseEventBuilder::new().scroll_up().build();

    assert!(matches!(event.kind, MouseEventKind::ScrollUp));
}

#[test]
fn test_mouse_event_builder_scroll_down() {
    let event = MouseEventBuilder::new().scroll_down().build();

    assert!(matches!(event.kind, MouseEventKind::ScrollDown));
}

#[test]
fn test_mouse_event_builder_alt() {
    let event = MouseEventBuilder::new().alt().build();

    assert!(event.modifiers.alt());
}

#[test]
fn test_mouse_event_builder_shift() {
    let event = MouseEventBuilder::new().shift().build();

    assert!(event.modifiers.shift());
}

#[test]
fn test_mouse_event_builder_into_event() {
    let event = MouseEventBuilder::new().at(15, 25).click().into_event();

    assert!(event.is_mouse());
    let mouse = event.as_mouse().unwrap();
    assert_eq!(mouse.column, 15);
    assert_eq!(mouse.row, 25);
}

#[test]
fn test_mouse_event_builder_default() {
    let builder = MouseEventBuilder::default();
    let event = builder.build();
    assert_eq!(event.column, 0);
    assert_eq!(event.row, 0);
    assert!(matches!(event.kind, MouseEventKind::Moved));
}

// -------------------------------------------------------------------------
// Event::kind_name
// -------------------------------------------------------------------------

#[test]
fn test_kind_name_key() {
    assert_eq!(Event::char('a').kind_name(), "Key");
    assert_eq!(Event::key(Key::Enter).kind_name(), "Key");
    assert_eq!(Event::ctrl('c').kind_name(), "Key");
}

#[test]
fn test_kind_name_mouse() {
    assert_eq!(Event::click(0, 0).kind_name(), "Mouse");
    assert_eq!(Event::mouse_move(5, 5).kind_name(), "Mouse");
    assert_eq!(Event::scroll_up(0, 0).kind_name(), "Mouse");
}

#[test]
fn test_kind_name_resize() {
    assert_eq!(Event::Resize(80, 24).kind_name(), "Resize");
}

#[test]
fn test_kind_name_focus() {
    assert_eq!(Event::FocusGained.kind_name(), "FocusGained");
    assert_eq!(Event::FocusLost.kind_name(), "FocusLost");
}

#[test]
fn test_kind_name_paste() {
    assert_eq!(Event::Paste("hello".to_string()).kind_name(), "Paste");
}
