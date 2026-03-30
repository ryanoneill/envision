use super::*;
use crate::input::{Event, KeyCode, KeyModifiers};

fn focused_state(value: &str) -> TextAreaState {
    let mut state = TextAreaState::with_value(value);
    state.set_focused(true);
    state
}

// =============================================================================
// Line Numbers Tests
// =============================================================================

#[test]
fn test_line_numbers_default_off() {
    let state = TextAreaState::new();
    assert!(!state.show_line_numbers());
}

#[test]
fn test_with_line_numbers_builder() {
    let state = TextAreaState::new().with_line_numbers(true);
    assert!(state.show_line_numbers());
}

#[test]
fn test_set_show_line_numbers() {
    let mut state = TextAreaState::new();
    state.set_show_line_numbers(true);
    assert!(state.show_line_numbers());
    state.set_show_line_numbers(false);
    assert!(!state.show_line_numbers());
}

#[test]
fn test_toggle_line_numbers_message() {
    let mut state = TextAreaState::new();
    assert!(!state.show_line_numbers());
    TextArea::update(&mut state, TextAreaMessage::ToggleLineNumbers);
    assert!(state.show_line_numbers());
    TextArea::update(&mut state, TextAreaMessage::ToggleLineNumbers);
    assert!(!state.show_line_numbers());
}

#[test]
fn test_ctrl_l_toggles_line_numbers() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(&state, &Event::ctrl('l'));
    assert_eq!(msg, Some(TextAreaMessage::ToggleLineNumbers));
}

#[test]
fn test_gutter_width_off() {
    let state = TextAreaState::new();
    assert_eq!(state.gutter_width(), 0);
}

#[test]
fn test_gutter_width_small() {
    // Less than 100 lines: 1 digit + 3 padding/separator = 4
    let state = TextAreaState::with_value("line1\nline2").with_line_numbers(true);
    assert_eq!(state.gutter_width(), 4);
}

#[test]
fn test_gutter_width_medium() {
    // 100+ lines: 3 digits + 3 padding/separator = 6
    let lines: Vec<&str> = (0..100).map(|_| "x").collect();
    let state = TextAreaState::with_value(lines.join("\n")).with_line_numbers(true);
    assert_eq!(state.gutter_width(), 6);
}

#[test]
fn test_view_with_line_numbers() {
    let mut state = TextAreaState::with_value("Hello\nWorld\nFoo");
    state.set_show_line_numbers(true);
    state.focused = true;
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 8);

    terminal
        .draw(|frame| {
            TextArea::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_without_line_numbers() {
    let state = TextAreaState::with_value("Hello\nWorld");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 8);

    terminal
        .draw(|frame| {
            TextArea::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    // Should render normally without gutter - no line number pattern like " 1 │"
    let output = terminal.backend().to_string();
    assert!(!output.contains(" 1 \u{2502}"));
}

// =============================================================================
// Search State Tests
// =============================================================================

#[test]
fn test_search_not_active_by_default() {
    let state = TextAreaState::new();
    assert!(!state.is_searching());
    assert_eq!(state.search_query(), None);
    assert!(state.search_matches().is_empty());
}

#[test]
fn test_start_search() {
    let mut state = TextAreaState::new();
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    assert!(state.is_searching());
    assert_eq!(state.search_query(), Some(""));
}

#[test]
fn test_set_search_query() {
    let mut state = TextAreaState::with_value("hello world hello");
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("hello".to_string()),
    );
    assert_eq!(state.search_query(), Some("hello"));
    assert_eq!(state.search_matches().len(), 2);
}

#[test]
fn test_search_matches_positions() {
    let mut state = TextAreaState::with_value("abc abc abc");
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("abc".to_string()),
    );
    // Should find 3 matches at byte positions 0, 4, 8
    assert_eq!(state.search_matches().len(), 3);
    assert_eq!(state.search_matches()[0], (0, 0));
    assert_eq!(state.search_matches()[1], (0, 4));
    assert_eq!(state.search_matches()[2], (0, 8));
}

#[test]
fn test_search_multiline() {
    let mut state = TextAreaState::with_value("hello\nworld\nhello");
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("hello".to_string()),
    );
    assert_eq!(state.search_matches().len(), 2);
    assert_eq!(state.search_matches()[0], (0, 0));
    assert_eq!(state.search_matches()[1], (2, 0));
}

#[test]
fn test_next_match() {
    let mut state = TextAreaState::with_value("abc abc abc");
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("abc".to_string()),
    );
    // Should be at first match
    assert_eq!(state.current_match_index(), 0);
    assert_eq!(state.cursor_position(), (0, 0));

    TextArea::update(&mut state, TextAreaMessage::NextMatch);
    assert_eq!(state.current_match_index(), 1);
    assert_eq!(state.cursor_col(), 4);

    TextArea::update(&mut state, TextAreaMessage::NextMatch);
    assert_eq!(state.current_match_index(), 2);
    assert_eq!(state.cursor_col(), 8);

    // Wraps around
    TextArea::update(&mut state, TextAreaMessage::NextMatch);
    assert_eq!(state.current_match_index(), 0);
    assert_eq!(state.cursor_col(), 0);
}

#[test]
fn test_prev_match() {
    let mut state = TextAreaState::with_value("abc abc abc");
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("abc".to_string()),
    );
    // At first match, go prev wraps to last
    TextArea::update(&mut state, TextAreaMessage::PrevMatch);
    assert_eq!(state.current_match_index(), 2);
    assert_eq!(state.cursor_col(), 8);

    TextArea::update(&mut state, TextAreaMessage::PrevMatch);
    assert_eq!(state.current_match_index(), 1);
    assert_eq!(state.cursor_col(), 4);
}

#[test]
fn test_clear_search() {
    let mut state = TextAreaState::with_value("hello world");
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("hello".to_string()),
    );
    assert!(state.is_searching());

    TextArea::update(&mut state, TextAreaMessage::ClearSearch);
    assert!(!state.is_searching());
    assert_eq!(state.search_query(), None);
    assert!(state.search_matches().is_empty());
}

#[test]
fn test_search_no_matches() {
    let mut state = TextAreaState::with_value("hello world");
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("xyz".to_string()),
    );
    assert!(state.search_matches().is_empty());
    assert_eq!(state.current_match_position(), None);
}

#[test]
fn test_next_match_no_matches() {
    let mut state = TextAreaState::with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("xyz".to_string()),
    );
    // Should not panic
    TextArea::update(&mut state, TextAreaMessage::NextMatch);
    TextArea::update(&mut state, TextAreaMessage::PrevMatch);
    assert!(state.search_matches().is_empty());
}

#[test]
fn test_search_empty_query() {
    let mut state = TextAreaState::with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(&mut state, TextAreaMessage::SetSearchQuery(String::new()));
    assert!(state.search_matches().is_empty());
}

// =============================================================================
// Search Key Binding Tests
// =============================================================================

#[test]
fn test_ctrl_f_starts_search() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(&state, &Event::ctrl('f'));
    assert_eq!(msg, Some(TextAreaMessage::StartSearch));
}

#[test]
fn test_escape_clears_search_when_active() {
    let mut state = focused_state("hello world");
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("hello".to_string()),
    );
    let msg = TextArea::handle_event(&state, &Event::key(KeyCode::Esc));
    assert_eq!(msg, Some(TextAreaMessage::ClearSearch));
}

#[test]
fn test_n_next_match_when_searching() {
    let mut state = focused_state("hello hello");
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("hello".to_string()),
    );
    let msg = TextArea::handle_event(&state, &Event::char('n'));
    assert_eq!(msg, Some(TextAreaMessage::NextMatch));
}

#[test]
fn test_shift_n_prev_match_when_searching() {
    let mut state = focused_state("hello hello");
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("hello".to_string()),
    );
    let msg = TextArea::handle_event(
        &state,
        &Event::key_with(KeyCode::Char('N'), KeyModifiers::SHIFT),
    );
    assert_eq!(msg, Some(TextAreaMessage::PrevMatch));
}

#[test]
fn test_n_inserts_when_not_searching() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(&state, &Event::char('n'));
    assert_eq!(msg, Some(TextAreaMessage::Insert('n')));
}

#[test]
fn test_escape_ignored_when_not_searching() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(&state, &Event::key(KeyCode::Esc));
    assert_eq!(msg, None);
}

// =============================================================================
// Search Recomputation on Edit Tests
// =============================================================================

#[test]
fn test_search_recomputes_on_insert() {
    let mut state = TextAreaState::with_value("ab ab");
    state.set_cursor_position(0, 2); // After "ab"
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("ab".to_string()),
    );
    assert_eq!(state.search_matches().len(), 2);

    // Insert a character that breaks a match
    TextArea::update(&mut state, TextAreaMessage::Insert('X'));
    // "abX ab" - still has "ab" at position 0 and position 4
    assert_eq!(state.search_matches().len(), 2);
}

#[test]
fn test_search_recomputes_on_delete() {
    let mut state = TextAreaState::with_value("abc abc");
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("abc".to_string()),
    );
    assert_eq!(state.search_matches().len(), 2);

    // Delete first character
    TextArea::update(&mut state, TextAreaMessage::Delete);
    // "bc abc" - only one match now
    assert_eq!(state.search_matches().len(), 1);
}

// =============================================================================
// View with Search Highlights Tests
// =============================================================================

#[test]
fn test_view_with_search_highlights() {
    let mut state = TextAreaState::with_value("hello world hello");
    state.focused = true;
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("hello".to_string()),
    );
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 8);

    terminal
        .draw(|frame| {
            TextArea::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    // The view should render without panicking
    let output = terminal.backend().to_string();
    assert!(output.contains("hello"));
    assert!(output.contains("world"));
}

#[test]
fn test_view_with_line_numbers_and_search() {
    let mut state = TextAreaState::with_value("hello\nworld\nhello");
    state.set_show_line_numbers(true);
    state.focused = true;
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("hello".to_string()),
    );
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 8);

    terminal
        .draw(|frame| {
            TextArea::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Disabled State Tests
// =============================================================================

#[test]
fn test_toggle_line_numbers_disabled() {
    let mut state = TextAreaState::new();
    state.set_disabled(true);
    let output = TextArea::update(&mut state, TextAreaMessage::ToggleLineNumbers);
    assert_eq!(output, None);
    assert!(!state.show_line_numbers());
}

#[test]
fn test_search_disabled() {
    let mut state = TextAreaState::with_value("hello");
    state.set_disabled(true);
    let output = TextArea::update(&mut state, TextAreaMessage::StartSearch);
    assert_eq!(output, None);
    assert!(!state.is_searching());
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_start_search_idempotent() {
    let mut state = TextAreaState::with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("hello".to_string()),
    );
    assert_eq!(state.search_matches().len(), 1);

    // Starting search again should not clear the query
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    assert!(state.is_searching());
    assert_eq!(state.search_query(), Some("hello"));
}

#[test]
fn test_current_match_position() {
    let mut state = TextAreaState::with_value("abc def abc");
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::StartSearch);
    TextArea::update(
        &mut state,
        TextAreaMessage::SetSearchQuery("abc".to_string()),
    );
    assert_eq!(state.current_match_position(), Some((0, 0)));

    TextArea::update(&mut state, TextAreaMessage::NextMatch);
    assert_eq!(state.current_match_position(), Some((0, 8)));
}
