use super::*;

#[test]
fn test_init() {
    let state = InputField::init();
    assert!(state.is_empty());
    assert_eq!(state.value(), "");
    assert_eq!(state.cursor_position(), 0);
}

#[test]
fn test_with_value() {
    let state = InputFieldState::with_value("hello");
    assert_eq!(state.value(), "hello");
    assert_eq!(state.cursor_position(), 5);
}

#[test]
fn test_with_placeholder() {
    let state = InputFieldState::with_placeholder("Enter text...");
    assert_eq!(state.placeholder(), "Enter text...");
    assert!(state.is_empty());
}

#[test]
fn test_insert_char() {
    let mut state = InputField::init();

    let output = InputField::update(&mut state, InputMessage::Insert('a'));
    assert_eq!(state.value(), "a");
    assert_eq!(state.cursor_position(), 1);
    assert_eq!(output, Some(InputOutput::Changed("a".to_string())));

    InputField::update(&mut state, InputMessage::Insert('b'));
    assert_eq!(state.value(), "ab");
    assert_eq!(state.cursor_position(), 2);
}

#[test]
fn test_insert_unicode() {
    let mut state = InputField::init();

    InputField::update(&mut state, InputMessage::Insert('日'));
    InputField::update(&mut state, InputMessage::Insert('本'));
    assert_eq!(state.value(), "日本");
    assert_eq!(state.cursor_position(), 2);
    assert_eq!(state.len(), 2);
}

#[test]
fn test_backspace() {
    let mut state = InputFieldState::with_value("abc");

    let output = InputField::update(&mut state, InputMessage::Backspace);
    assert_eq!(state.value(), "ab");
    assert_eq!(output, Some(InputOutput::Changed("ab".to_string())));

    InputField::update(&mut state, InputMessage::Backspace);
    InputField::update(&mut state, InputMessage::Backspace);
    assert_eq!(state.value(), "");

    // Backspace on empty should return None
    let output = InputField::update(&mut state, InputMessage::Backspace);
    assert_eq!(output, None);
}

#[test]
fn test_delete() {
    let mut state = InputFieldState::with_value("abc");
    state.set_cursor(0);

    let output = InputField::update(&mut state, InputMessage::Delete);
    assert_eq!(state.value(), "bc");
    assert_eq!(output, Some(InputOutput::Changed("bc".to_string())));

    // Move to end and delete should return None
    state.cursor = state.value.len();
    let output = InputField::update(&mut state, InputMessage::Delete);
    assert_eq!(output, None);
}

#[test]
fn test_cursor_movement() {
    let mut state = InputFieldState::with_value("hello");

    InputField::update(&mut state, InputMessage::Left);
    assert_eq!(state.cursor_position(), 4);

    InputField::update(&mut state, InputMessage::Left);
    assert_eq!(state.cursor_position(), 3);

    InputField::update(&mut state, InputMessage::Right);
    assert_eq!(state.cursor_position(), 4);

    InputField::update(&mut state, InputMessage::Home);
    assert_eq!(state.cursor_position(), 0);

    InputField::update(&mut state, InputMessage::End);
    assert_eq!(state.cursor_position(), 5);
}

#[test]
fn test_cursor_bounds() {
    let mut state = InputFieldState::with_value("hi");

    // Can't go left past beginning
    state.set_cursor(0);
    InputField::update(&mut state, InputMessage::Left);
    assert_eq!(state.cursor_position(), 0);

    // Can't go right past end
    state.set_cursor(10); // Over the length
    assert_eq!(state.cursor_position(), 2); // Clamped
    InputField::update(&mut state, InputMessage::Right);
    assert_eq!(state.cursor_position(), 2);
}

#[test]
fn test_word_navigation() {
    let mut state = InputFieldState::with_value("hello world test");

    // Start at end
    InputField::update(&mut state, InputMessage::WordLeft);
    assert_eq!(state.cursor_position(), 12); // Start of "test"

    InputField::update(&mut state, InputMessage::WordLeft);
    assert_eq!(state.cursor_position(), 6); // Start of "world"

    InputField::update(&mut state, InputMessage::WordLeft);
    assert_eq!(state.cursor_position(), 0); // Start of "hello"

    InputField::update(&mut state, InputMessage::WordRight);
    assert_eq!(state.cursor_position(), 6); // After "hello "

    InputField::update(&mut state, InputMessage::WordRight);
    assert_eq!(state.cursor_position(), 12); // After "world "
}

#[test]
fn test_delete_word_back() {
    let mut state = InputFieldState::with_value("hello world");

    let output = InputField::update(&mut state, InputMessage::DeleteWordBack);
    assert_eq!(state.value(), "hello ");
    assert_eq!(output, Some(InputOutput::Changed("hello ".to_string())));

    InputField::update(&mut state, InputMessage::DeleteWordBack);
    assert_eq!(state.value(), "");

    // Delete word back on empty
    let output = InputField::update(&mut state, InputMessage::DeleteWordBack);
    assert_eq!(output, None);
}

#[test]
fn test_delete_word_forward() {
    let mut state = InputFieldState::with_value("hello world");
    state.set_cursor(0);

    let output = InputField::update(&mut state, InputMessage::DeleteWordForward);
    assert_eq!(state.value(), "world");
    assert_eq!(output, Some(InputOutput::Changed("world".to_string())));

    // Cursor at end
    state.cursor = state.value.len();
    let output = InputField::update(&mut state, InputMessage::DeleteWordForward);
    assert_eq!(output, None);
}

#[test]
fn test_clear() {
    let mut state = InputFieldState::with_value("hello");

    let output = InputField::update(&mut state, InputMessage::Clear);
    assert_eq!(state.value(), "");
    assert_eq!(state.cursor_position(), 0);
    assert_eq!(output, Some(InputOutput::Changed("".to_string())));

    // Clear empty should return None
    let output = InputField::update(&mut state, InputMessage::Clear);
    assert_eq!(output, None);
}

#[test]
fn test_set_value() {
    let mut state = InputField::init();

    let output = InputField::update(&mut state, InputMessage::SetValue("new value".to_string()));
    assert_eq!(state.value(), "new value");
    assert_eq!(state.cursor_position(), 9);
    assert_eq!(output, Some(InputOutput::Changed("new value".to_string())));

    // Setting same value returns None
    let output = InputField::update(&mut state, InputMessage::SetValue("new value".to_string()));
    assert_eq!(output, None);
}

#[test]
fn test_submit() {
    let mut state = InputFieldState::with_value("submitted text");

    let output = InputField::update(&mut state, InputMessage::Submit);
    assert_eq!(
        output,
        Some(InputOutput::Submitted("submitted text".to_string()))
    );
    // Value should remain unchanged
    assert_eq!(state.value(), "submitted text");
}

#[test]
fn test_insert_at_cursor() {
    let mut state = InputFieldState::with_value("helo");
    state.set_cursor(3);

    InputField::update(&mut state, InputMessage::Insert('l'));
    assert_eq!(state.value(), "hello");
    assert_eq!(state.cursor_position(), 4);
}

#[test]
fn test_focusable() {
    let mut state = InputField::init();

    assert!(!InputField::is_focused(&state));

    InputField::set_focused(&mut state, true);
    assert!(InputField::is_focused(&state));

    InputField::blur(&mut state);
    assert!(!InputField::is_focused(&state));
}

#[test]
fn test_len() {
    let state = InputFieldState::with_value("hello");
    assert_eq!(state.len(), 5);

    let state = InputFieldState::with_value("日本語");
    assert_eq!(state.len(), 3);
}

#[test]
fn test_view() {
    let mut state = InputFieldState::with_value("Hello");
    state.focused = true;
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            InputField::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Hello"));
}

#[test]
fn test_view_placeholder() {
    let mut state = InputField::init();
    state.set_placeholder("Enter text...");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            InputField::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Enter text..."));
}
