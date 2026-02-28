use super::*;

// -------------------------------------------------------------------------
// Event constructors
// -------------------------------------------------------------------------

#[test]
fn test_simulated_event_char() {
    let event = Event::char('a');
    assert!(event.is_key());
    let key = event.as_key().unwrap();
    assert_eq!(key.code, KeyCode::Char('a'));
    assert_eq!(key.modifiers, KeyModifiers::NONE);
}

#[test]
fn test_simulated_event_char_with() {
    let event = Event::char_with('A', KeyModifiers::SHIFT);
    let key = event.as_key().unwrap();
    assert_eq!(key.code, KeyCode::Char('A'));
    assert!(key.modifiers.contains(KeyModifiers::SHIFT));
}

#[test]
fn test_simulated_event_key() {
    let event = Event::key(KeyCode::Enter);
    let key = event.as_key().unwrap();
    assert_eq!(key.code, KeyCode::Enter);
    assert_eq!(key.modifiers, KeyModifiers::NONE);
}

#[test]
fn test_simulated_event_key_with() {
    let event = Event::key_with(KeyCode::Tab, KeyModifiers::SHIFT);
    let key = event.as_key().unwrap();
    assert_eq!(key.code, KeyCode::Tab);
    assert!(key.modifiers.contains(KeyModifiers::SHIFT));
}

#[test]
fn test_simulated_event_ctrl() {
    let event = Event::ctrl('c');
    let key = event.as_key().unwrap();
    assert_eq!(key.code, KeyCode::Char('c'));
    assert!(key.modifiers.contains(KeyModifiers::CONTROL));
}

#[test]
fn test_simulated_event_alt() {
    let event = Event::alt('x');
    let key = event.as_key().unwrap();
    assert_eq!(key.code, KeyCode::Char('x'));
    assert!(key.modifiers.contains(KeyModifiers::ALT));
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
    let key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE);
    let event: Event = key.into();
    assert!(event.is_key());
}

#[test]
fn test_from_mouse_event() {
    let mouse = MouseEvent {
        kind: MouseEventKind::Moved,
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    };
    let event: Event = mouse.into();
    assert!(event.is_mouse());
}

#[test]
fn test_crossterm_conversion() {
    let simulated = Event::key(KeyCode::Enter);
    let crossterm: crossterm::event::Event = simulated.clone().into();
    let back: Event = crossterm.into();
    assert_eq!(simulated, back);
}

#[test]
fn test_crossterm_conversion_resize() {
    let event = crossterm::event::Event::Resize(80, 24);
    let simulated: Event = event.into();
    assert!(matches!(simulated, Event::Resize(80, 24)));

    let back: crossterm::event::Event = simulated.into();
    assert!(matches!(back, crossterm::event::Event::Resize(80, 24)));
}

#[test]
fn test_crossterm_conversion_focus() {
    let gained = crossterm::event::Event::FocusGained;
    let simulated: Event = gained.into();
    assert!(matches!(simulated, Event::FocusGained));

    let back: crossterm::event::Event = simulated.into();
    assert!(matches!(back, crossterm::event::Event::FocusGained));

    let lost = crossterm::event::Event::FocusLost;
    let simulated: Event = lost.into();
    assert!(matches!(simulated, Event::FocusLost));

    let back: crossterm::event::Event = simulated.into();
    assert!(matches!(back, crossterm::event::Event::FocusLost));
}

#[test]
fn test_crossterm_conversion_paste() {
    let event = crossterm::event::Event::Paste("hello".to_string());
    let simulated: Event = event.into();
    assert!(matches!(simulated, Event::Paste(ref s) if s == "hello"));

    let back: crossterm::event::Event = simulated.into();
    assert!(matches!(back, crossterm::event::Event::Paste(ref s) if s == "hello"));
}

// -------------------------------------------------------------------------
// KeyEventBuilder
// -------------------------------------------------------------------------

#[test]
fn test_key_event_builder() {
    let event = KeyEventBuilder::new().char('x').ctrl().shift().build();

    assert_eq!(event.code, KeyCode::Char('x'));
    assert!(event.modifiers.contains(KeyModifiers::CONTROL));
    assert!(event.modifiers.contains(KeyModifiers::SHIFT));
}

#[test]
fn test_key_event_builder_code() {
    let event = KeyEventBuilder::new().code(KeyCode::F(1)).build();

    assert_eq!(event.code, KeyCode::F(1));
}

#[test]
fn test_key_event_builder_alt() {
    let event = KeyEventBuilder::new().char('a').alt().build();

    assert!(event.modifiers.contains(KeyModifiers::ALT));
}

#[test]
fn test_key_event_builder_modifiers() {
    let mods = KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT;
    let event = KeyEventBuilder::new().char('a').modifiers(mods).build();

    assert_eq!(event.modifiers, mods);
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
    assert_eq!(event.as_key().unwrap().code, KeyCode::Char('b'));
}

#[test]
fn test_key_event_builder_default_code() {
    // When no code is set, should use KeyCode::Null
    let event = KeyEventBuilder::new().build();
    assert_eq!(event.code, KeyCode::Null);
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
    assert!(event.modifiers.contains(KeyModifiers::CONTROL));
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

    assert!(event.modifiers.contains(KeyModifiers::ALT));
}

#[test]
fn test_mouse_event_builder_shift() {
    let event = MouseEventBuilder::new().shift().build();

    assert!(event.modifiers.contains(KeyModifiers::SHIFT));
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
