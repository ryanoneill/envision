use super::*;
use crate::input::{Event, KeyCode, KeyModifiers};

#[path = "selection_tests.rs"]
mod selection_tests;

#[test]
fn test_init() {
    let state = InputField::init();
    assert!(state.is_empty());
    assert_eq!(state.value(), "");
    assert_eq!(state.cursor_position(), 0);
}

#[test]
fn test_new() {
    let state = InputFieldState::new();
    assert!(state.is_empty());
    assert_eq!(state.value(), "");
    assert_eq!(state.cursor_position(), 0);
    assert_eq!(state.placeholder(), "");
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

    let output = InputField::update(&mut state, InputFieldMessage::Insert('a'));
    assert_eq!(state.value(), "a");
    assert_eq!(state.cursor_position(), 1);
    assert_eq!(output, Some(InputFieldOutput::Changed("a".to_string())));

    InputField::update(&mut state, InputFieldMessage::Insert('b'));
    assert_eq!(state.value(), "ab");
    assert_eq!(state.cursor_position(), 2);
}

#[test]
fn test_insert_unicode() {
    let mut state = InputField::init();

    InputField::update(&mut state, InputFieldMessage::Insert('日'));
    InputField::update(&mut state, InputFieldMessage::Insert('本'));
    assert_eq!(state.value(), "日本");
    assert_eq!(state.cursor_position(), 2);
    assert_eq!(state.len(), 2);
}

#[test]
fn test_backspace() {
    let mut state = InputFieldState::with_value("abc");

    let output = InputField::update(&mut state, InputFieldMessage::Backspace);
    assert_eq!(state.value(), "ab");
    assert_eq!(output, Some(InputFieldOutput::Changed("ab".to_string())));

    InputField::update(&mut state, InputFieldMessage::Backspace);
    InputField::update(&mut state, InputFieldMessage::Backspace);
    assert_eq!(state.value(), "");

    // Backspace on empty should return None
    let output = InputField::update(&mut state, InputFieldMessage::Backspace);
    assert_eq!(output, None);
}

#[test]
fn test_delete() {
    let mut state = InputFieldState::with_value("abc");
    state.set_cursor_position(0);

    let output = InputField::update(&mut state, InputFieldMessage::Delete);
    assert_eq!(state.value(), "bc");
    assert_eq!(output, Some(InputFieldOutput::Changed("bc".to_string())));

    // Move to end and delete should return None
    state.cursor = state.value.len();
    let output = InputField::update(&mut state, InputFieldMessage::Delete);
    assert_eq!(output, None);
}

#[test]
fn test_cursor_movement() {
    let mut state = InputFieldState::with_value("hello");

    InputField::update(&mut state, InputFieldMessage::Left);
    assert_eq!(state.cursor_position(), 4);

    InputField::update(&mut state, InputFieldMessage::Left);
    assert_eq!(state.cursor_position(), 3);

    InputField::update(&mut state, InputFieldMessage::Right);
    assert_eq!(state.cursor_position(), 4);

    InputField::update(&mut state, InputFieldMessage::Home);
    assert_eq!(state.cursor_position(), 0);

    InputField::update(&mut state, InputFieldMessage::End);
    assert_eq!(state.cursor_position(), 5);
}

#[test]
fn test_cursor_bounds() {
    let mut state = InputFieldState::with_value("hi");

    // Can't go left past beginning
    state.set_cursor_position(0);
    InputField::update(&mut state, InputFieldMessage::Left);
    assert_eq!(state.cursor_position(), 0);

    // Can't go right past end
    state.set_cursor_position(10); // Over the length
    assert_eq!(state.cursor_position(), 2); // Clamped
    InputField::update(&mut state, InputFieldMessage::Right);
    assert_eq!(state.cursor_position(), 2);
}

#[test]
fn test_word_navigation() {
    let mut state = InputFieldState::with_value("hello world test");

    // Start at end
    InputField::update(&mut state, InputFieldMessage::WordLeft);
    assert_eq!(state.cursor_position(), 12); // Start of "test"

    InputField::update(&mut state, InputFieldMessage::WordLeft);
    assert_eq!(state.cursor_position(), 6); // Start of "world"

    InputField::update(&mut state, InputFieldMessage::WordLeft);
    assert_eq!(state.cursor_position(), 0); // Start of "hello"

    InputField::update(&mut state, InputFieldMessage::WordRight);
    assert_eq!(state.cursor_position(), 6); // After "hello "

    InputField::update(&mut state, InputFieldMessage::WordRight);
    assert_eq!(state.cursor_position(), 12); // After "world "
}

#[test]
fn test_delete_word_back() {
    let mut state = InputFieldState::with_value("hello world");

    let output = InputField::update(&mut state, InputFieldMessage::DeleteWordBack);
    assert_eq!(state.value(), "hello ");
    assert_eq!(
        output,
        Some(InputFieldOutput::Changed("hello ".to_string()))
    );

    InputField::update(&mut state, InputFieldMessage::DeleteWordBack);
    assert_eq!(state.value(), "");

    // Delete word back on empty
    let output = InputField::update(&mut state, InputFieldMessage::DeleteWordBack);
    assert_eq!(output, None);
}

#[test]
fn test_delete_word_forward() {
    let mut state = InputFieldState::with_value("hello world");
    state.set_cursor_position(0);

    let output = InputField::update(&mut state, InputFieldMessage::DeleteWordForward);
    assert_eq!(state.value(), "world");
    assert_eq!(output, Some(InputFieldOutput::Changed("world".to_string())));

    // Cursor at end
    state.cursor = state.value.len();
    let output = InputField::update(&mut state, InputFieldMessage::DeleteWordForward);
    assert_eq!(output, None);
}

#[test]
fn test_clear() {
    let mut state = InputFieldState::with_value("hello");

    let output = InputField::update(&mut state, InputFieldMessage::Clear);
    assert_eq!(state.value(), "");
    assert_eq!(state.cursor_position(), 0);
    assert_eq!(output, Some(InputFieldOutput::Changed("".to_string())));

    // Clear empty should return None
    let output = InputField::update(&mut state, InputFieldMessage::Clear);
    assert_eq!(output, None);
}

#[test]
fn test_set_value() {
    let mut state = InputField::init();

    let output = InputField::update(
        &mut state,
        InputFieldMessage::SetValue("new value".to_string()),
    );
    assert_eq!(state.value(), "new value");
    assert_eq!(state.cursor_position(), 9);
    assert_eq!(
        output,
        Some(InputFieldOutput::Changed("new value".to_string()))
    );

    // Setting same value returns None
    let output = InputField::update(
        &mut state,
        InputFieldMessage::SetValue("new value".to_string()),
    );
    assert_eq!(output, None);
}

#[test]
fn test_submit() {
    let mut state = InputFieldState::with_value("submitted text");

    let output = InputField::update(&mut state, InputFieldMessage::Submit);
    assert_eq!(
        output,
        Some(InputFieldOutput::Submitted("submitted text".to_string()))
    );
    // Value should remain unchanged
    assert_eq!(state.value(), "submitted text");
}

#[test]
fn test_insert_at_cursor() {
    let mut state = InputFieldState::with_value("helo");
    state.set_cursor_position(3);

    InputField::update(&mut state, InputFieldMessage::Insert('l'));
    assert_eq!(state.value(), "hello");
    assert_eq!(state.cursor_position(), 4);
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
            InputField::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_unfocused() {
    let state = InputFieldState::with_value("Hello");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            InputField::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let mut state = InputFieldState::with_value("Hello");
    state.set_disabled(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            InputField::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_placeholder() {
    let mut state = InputField::init();
    state.set_placeholder("Enter text...");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            InputField::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_insert_emoji() {
    let mut state = InputFieldState::new();
    InputField::update(&mut state, InputFieldMessage::Insert('\u{1F600}')); // grinning face
    assert_eq!(state.value(), "\u{1F600}");
    assert_eq!(state.cursor_position(), 1);
}

#[test]
fn test_cursor_with_multi_byte() {
    let mut state = InputFieldState::new();
    // Insert CJK character followed by emoji
    InputField::update(&mut state, InputFieldMessage::Insert('日'));
    InputField::update(&mut state, InputFieldMessage::Insert('\u{1F600}'));
    InputField::update(&mut state, InputFieldMessage::Insert('本'));
    assert_eq!(state.value(), "日\u{1F600}本");
    assert_eq!(state.cursor_position(), 3);
}

#[test]
fn test_backspace_emoji() {
    let mut state = InputFieldState::new();
    InputField::update(&mut state, InputFieldMessage::Insert('A'));
    InputField::update(&mut state, InputFieldMessage::Insert('\u{1F600}'));
    InputField::update(&mut state, InputFieldMessage::Insert('B'));
    assert_eq!(state.value(), "A\u{1F600}B");

    // Backspace should delete 'B'
    InputField::update(&mut state, InputFieldMessage::Backspace);
    assert_eq!(state.value(), "A\u{1F600}");

    // Backspace should delete the emoji
    InputField::update(&mut state, InputFieldMessage::Backspace);
    assert_eq!(state.value(), "A");
}

#[test]
fn test_combining_diacritics() {
    let mut state = InputFieldState::new();
    // Insert 'e' followed by combining acute accent (U+0301)
    InputField::update(&mut state, InputFieldMessage::Insert('e'));
    InputField::update(&mut state, InputFieldMessage::Insert('\u{0301}'));
    // The value should contain both characters
    assert!(state.value().contains('e'));
    assert!(state.value().contains('\u{0301}'));
}

#[test]
fn test_word_nav_with_emoji() {
    let mut state = InputFieldState::new();
    // Type "hello 😀 world"
    for c in "hello ".chars() {
        InputField::update(&mut state, InputFieldMessage::Insert(c));
    }
    InputField::update(&mut state, InputFieldMessage::Insert('\u{1F600}'));
    for c in " world".chars() {
        InputField::update(&mut state, InputFieldMessage::Insert(c));
    }
    assert_eq!(state.value(), "hello \u{1F600} world");

    // Move to beginning
    InputField::update(&mut state, InputFieldMessage::Home);
    assert_eq!(state.cursor_position(), 0);

    // WordRight should move through words
    InputField::update(&mut state, InputFieldMessage::WordRight);
    assert!(state.cursor_position() > 0);
}

// handle_event tests

#[test]
fn test_handle_event_char_insert() {
    let mut state = InputField::init();
    InputField::set_focused(&mut state, true);
    let msg = InputField::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, Some(InputFieldMessage::Insert('a')));
}

#[test]
fn test_handle_event_backspace() {
    let mut state = InputField::init();
    InputField::set_focused(&mut state, true);
    let msg = InputField::handle_event(&state, &Event::key(KeyCode::Backspace));
    assert_eq!(msg, Some(InputFieldMessage::Backspace));
}

#[test]
fn test_handle_event_delete() {
    let mut state = InputField::init();
    InputField::set_focused(&mut state, true);
    let msg = InputField::handle_event(&state, &Event::key(KeyCode::Delete));
    assert_eq!(msg, Some(InputFieldMessage::Delete));
}

#[test]
fn test_handle_event_left() {
    let mut state = InputField::init();
    InputField::set_focused(&mut state, true);
    let msg = InputField::handle_event(&state, &Event::key(KeyCode::Left));
    assert_eq!(msg, Some(InputFieldMessage::Left));
}

#[test]
fn test_handle_event_right() {
    let mut state = InputField::init();
    InputField::set_focused(&mut state, true);
    let msg = InputField::handle_event(&state, &Event::key(KeyCode::Right));
    assert_eq!(msg, Some(InputFieldMessage::Right));
}

#[test]
fn test_handle_event_home() {
    let mut state = InputField::init();
    InputField::set_focused(&mut state, true);
    let msg = InputField::handle_event(&state, &Event::key(KeyCode::Home));
    assert_eq!(msg, Some(InputFieldMessage::Home));
}

#[test]
fn test_handle_event_end() {
    let mut state = InputField::init();
    InputField::set_focused(&mut state, true);
    let msg = InputField::handle_event(&state, &Event::key(KeyCode::End));
    assert_eq!(msg, Some(InputFieldMessage::End));
}

#[test]
fn test_handle_event_enter() {
    let mut state = InputField::init();
    InputField::set_focused(&mut state, true);
    let msg = InputField::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(InputFieldMessage::Submit));
}

#[test]
fn test_handle_event_ctrl_left() {
    let mut state = InputField::init();
    InputField::set_focused(&mut state, true);
    let msg = InputField::handle_event(
        &state,
        &Event::key_with(KeyCode::Left, KeyModifiers::CONTROL),
    );
    assert_eq!(msg, Some(InputFieldMessage::WordLeft));
}

#[test]
fn test_handle_event_ctrl_right() {
    let mut state = InputField::init();
    InputField::set_focused(&mut state, true);
    let msg = InputField::handle_event(
        &state,
        &Event::key_with(KeyCode::Right, KeyModifiers::CONTROL),
    );
    assert_eq!(msg, Some(InputFieldMessage::WordRight));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = InputField::init();
    let msg = InputField::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, None);
}

#[test]
fn test_dispatch_event_char() {
    let mut state = InputField::init();
    InputField::set_focused(&mut state, true);
    let output = InputField::dispatch_event(&mut state, &Event::char('a'));
    assert_eq!(output, Some(InputFieldOutput::Changed("a".to_string())));
}

#[test]
fn test_instance_is_focused() {
    let mut state = InputField::init();
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
}

#[test]
fn test_instance_handle_event() {
    let mut state = InputField::init();
    state.set_focused(true);
    let msg = state.handle_event(&Event::char('a'));
    assert_eq!(msg, Some(InputFieldMessage::Insert('a')));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = InputField::init();
    state.set_focused(true);
    let output = state.dispatch_event(&Event::char('a'));
    assert_eq!(output, Some(InputFieldOutput::Changed("a".to_string())));
}

// ========== Disabled State Tests ==========

#[test]
fn test_is_disabled_default() {
    let state = InputField::init();
    assert!(!state.is_disabled());
}

#[test]
fn test_set_disabled() {
    let mut state = InputField::init();
    state.set_disabled(true);
    assert!(state.is_disabled());
    state.set_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_with_disabled() {
    let state = InputFieldState::new().with_disabled(true);
    assert!(state.is_disabled());

    let state = InputFieldState::new().with_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_with_value_not_disabled() {
    let state = InputFieldState::with_value("hello");
    assert!(!state.is_disabled());
}

#[test]
fn test_with_placeholder_not_disabled() {
    let state = InputFieldState::with_placeholder("Enter text...");
    assert!(!state.is_disabled());
}

#[test]
fn test_handle_event_ignored_when_disabled() {
    let mut state = InputField::init();
    state.set_focused(true);
    state.set_disabled(true);

    let msg = InputField::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, None);

    let msg = InputField::handle_event(&state, &Event::key(KeyCode::Backspace));
    assert_eq!(msg, None);

    let msg = InputField::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, None);

    let msg = InputField::handle_event(&state, &Event::key(KeyCode::Left));
    assert_eq!(msg, None);
}

#[test]
fn test_update_ignored_when_disabled() {
    let mut state = InputFieldState::with_value("hello");
    state.set_disabled(true);

    let output = InputField::update(&mut state, InputFieldMessage::Insert('x'));
    assert_eq!(output, None);
    assert_eq!(state.value(), "hello");

    let output = InputField::update(&mut state, InputFieldMessage::Backspace);
    assert_eq!(output, None);
    assert_eq!(state.value(), "hello");

    let output = InputField::update(&mut state, InputFieldMessage::Clear);
    assert_eq!(output, None);
    assert_eq!(state.value(), "hello");

    let output = InputField::update(&mut state, InputFieldMessage::Submit);
    assert_eq!(output, None);
}

#[test]
fn test_dispatch_event_ignored_when_disabled() {
    let mut state = InputField::init();
    state.set_focused(true);
    state.set_disabled(true);

    let output = InputField::dispatch_event(&mut state, &Event::char('a'));
    assert_eq!(output, None);
    assert!(state.is_empty());
}

#[test]
fn test_instance_is_disabled() {
    let mut state = InputField::init();
    assert!(!state.is_disabled());
    state.set_disabled(true);
    assert!(state.is_disabled());
}

#[test]
fn test_instance_handle_event_disabled() {
    let mut state = InputField::init();
    state.set_focused(true);
    state.set_disabled(true);
    let msg = state.handle_event(&Event::char('a'));
    assert_eq!(msg, None);
}

#[test]
fn test_instance_dispatch_event_disabled() {
    let mut state = InputField::init();
    state.set_focused(true);
    state.set_disabled(true);
    let output = state.dispatch_event(&Event::char('a'));
    assert_eq!(output, None);
}

#[test]
fn test_instance_update_disabled() {
    let mut state = InputFieldState::with_value("hello");
    state.set_disabled(true);
    let output = state.update(InputFieldMessage::Insert('x'));
    assert_eq!(output, None);
    assert_eq!(state.value(), "hello");
}

// ========== cursor_display_position Tests ==========

#[test]
fn test_cursor_display_position_ascii() {
    let state = InputFieldState::with_value("hello");
    // For ASCII, display position equals character position.
    assert_eq!(state.cursor_display_position(), 5);
    assert_eq!(state.cursor_position(), 5);
}

#[test]
fn test_cursor_display_position_emoji() {
    let mut state = InputFieldState::new();
    InputField::update(&mut state, InputFieldMessage::Insert('A'));
    InputField::update(&mut state, InputFieldMessage::Insert('\u{1F600}'));
    InputField::update(&mut state, InputFieldMessage::Insert('B'));
    // "A😀B" — char pos is 3, but display is 1 + 2 + 1 = 4
    assert_eq!(state.cursor_position(), 3);
    assert_eq!(state.cursor_display_position(), 4);

    // Move cursor left (before 'B'), display pos should be 1 + 2 = 3
    InputField::update(&mut state, InputFieldMessage::Left);
    assert_eq!(state.cursor_position(), 2);
    assert_eq!(state.cursor_display_position(), 3);
}

#[test]
fn test_cursor_display_position_cjk() {
    let mut state = InputFieldState::new();
    InputField::update(&mut state, InputFieldMessage::Insert('日'));
    InputField::update(&mut state, InputFieldMessage::Insert('本'));
    // "日本" — char pos is 2, display is 2 + 2 = 4
    assert_eq!(state.cursor_position(), 2);
    assert_eq!(state.cursor_display_position(), 4);
}

#[test]
fn test_cursor_display_position_mixed() {
    let mut state = InputFieldState::new();
    // "A日😀B"
    InputField::update(&mut state, InputFieldMessage::Insert('A'));
    InputField::update(&mut state, InputFieldMessage::Insert('日'));
    InputField::update(&mut state, InputFieldMessage::Insert('\u{1F600}'));
    InputField::update(&mut state, InputFieldMessage::Insert('B'));
    // char pos = 4, display = 1 + 2 + 2 + 1 = 6
    assert_eq!(state.cursor_position(), 4);
    assert_eq!(state.cursor_display_position(), 6);
}

#[test]
fn test_cursor_display_position_empty() {
    let state = InputFieldState::new();
    assert_eq!(state.cursor_display_position(), 0);
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{with_annotations, WidgetType};
    let mut state = InputFieldState::new();
    InputField::update(&mut state, InputFieldMessage::Insert('H'));
    InputField::update(&mut state, InputFieldMessage::Insert('i'));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                InputField::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Input);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.value, Some("Hi".to_string()));
}
