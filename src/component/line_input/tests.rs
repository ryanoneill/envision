use super::*;
use crate::component::Component;
use crate::component::test_utils::setup_render;

// ---- Construction / Accessors ----

#[test]
fn test_new_empty() {
    let state = LineInputState::new();
    assert!(state.is_empty());
    assert_eq!(state.value(), "");
    assert_eq!(state.len(), 0);
    assert_eq!(state.cursor_byte_offset(), 0);
}

#[test]
fn test_with_value() {
    let state = LineInputState::with_value("hello");
    assert_eq!(state.value(), "hello");
    assert_eq!(state.cursor_byte_offset(), 5);
    assert_eq!(state.len(), 5);
}

#[test]
fn test_with_placeholder() {
    let state = LineInputState::new().with_placeholder("Type here...");
    assert_eq!(state.placeholder(), "Type here...");
}

#[test]
fn test_with_max_history() {
    let state = LineInputState::new().with_max_history(5);
    assert_eq!(state.history_count(), 0);
}

#[test]
fn test_set_value() {
    let mut state = LineInputState::with_value("hello");
    state.set_value("world");
    assert_eq!(state.value(), "world");
    assert_eq!(state.cursor_byte_offset(), 5);
}

#[test]
fn test_set_placeholder() {
    let mut state = LineInputState::new();
    state.set_placeholder("Enter command...");
    assert_eq!(state.placeholder(), "Enter command...");
}

#[test]
fn test_display_width() {
    let mut state = LineInputState::new();
    assert_eq!(state.display_width(), 80);
    state.set_display_width(40);
    assert_eq!(state.display_width(), 40);
}

// ---- Editing: Insert ----

#[test]
fn test_insert_char() {
    let mut state = LineInputState::new();

    let output = state.update(LineInputMessage::Insert('h'));
    assert_eq!(state.value(), "h");
    assert_eq!(state.cursor_byte_offset(), 1);
    assert_eq!(output, Some(LineInputOutput::Changed("h".to_string())));
}

#[test]
fn test_insert_multiple_chars() {
    let mut state = LineInputState::new();

    state.update(LineInputMessage::Insert('h'));
    state.update(LineInputMessage::Insert('i'));
    assert_eq!(state.value(), "hi");
    assert_eq!(state.cursor_byte_offset(), 2);
}

#[test]
fn test_insert_unicode() {
    let mut state = LineInputState::new();

    state.update(LineInputMessage::Insert('世'));
    assert_eq!(state.value(), "世");
    assert_eq!(state.cursor_byte_offset(), 3);
}

#[test]
fn test_insert_emoji() {
    let mut state = LineInputState::new();

    state.update(LineInputMessage::Insert('🎉'));
    assert_eq!(state.value(), "🎉");
    assert_eq!(state.cursor_byte_offset(), 4);
}

// ---- Editing: Backspace ----

#[test]
fn test_backspace() {
    let mut state = LineInputState::with_value("hello");

    let output = state.update(LineInputMessage::Backspace);
    assert_eq!(state.value(), "hell");
    assert_eq!(state.cursor_byte_offset(), 4);
    assert_eq!(output, Some(LineInputOutput::Changed("hell".to_string())));
}

#[test]
fn test_backspace_at_start() {
    let mut state = LineInputState::new();

    let output = state.update(LineInputMessage::Backspace);
    assert_eq!(output, None);
}

#[test]
fn test_backspace_with_selection() {
    let mut state = LineInputState::with_value("hello");

    // Select "ell"
    state.cursor = 1;
    state.selection_anchor = Some(4);
    let output = state.update(LineInputMessage::Backspace);
    assert_eq!(state.value(), "ho");
    assert_eq!(output, Some(LineInputOutput::Changed("ho".to_string())));
}

// ---- Editing: Delete ----

#[test]
fn test_delete() {
    let mut state = LineInputState::with_value("hello");

    state.cursor = 0;
    let output = state.update(LineInputMessage::Delete);
    assert_eq!(state.value(), "ello");
    assert_eq!(output, Some(LineInputOutput::Changed("ello".to_string())));
}

#[test]
fn test_delete_at_end() {
    let mut state = LineInputState::with_value("hello");

    let output = state.update(LineInputMessage::Delete);
    assert_eq!(output, None);
}

// ---- Editing: Word delete ----

#[test]
fn test_delete_word_back() {
    let mut state = LineInputState::with_value("hello world");

    let output = state.update(LineInputMessage::DeleteWordBack);
    assert_eq!(state.value(), "hello ");
    assert_eq!(output, Some(LineInputOutput::Changed("hello ".to_string())));
}

#[test]
fn test_delete_word_forward() {
    let mut state = LineInputState::with_value("hello world");

    state.cursor = 5;
    let output = state.update(LineInputMessage::DeleteWordForward);
    assert_eq!(state.value(), "helloworld");
    assert_eq!(
        output,
        Some(LineInputOutput::Changed("helloworld".to_string()))
    );
}

// ---- Editing: Clear ----

#[test]
fn test_clear() {
    let mut state = LineInputState::with_value("hello");

    let output = state.update(LineInputMessage::Clear);
    assert_eq!(state.value(), "");
    assert_eq!(state.cursor_byte_offset(), 0);
    assert_eq!(output, Some(LineInputOutput::Changed(String::new())));
}

#[test]
fn test_clear_empty() {
    let mut state = LineInputState::new();

    let output = state.update(LineInputMessage::Clear);
    assert_eq!(output, None);
}

// ---- Editing: Paste ----

#[test]
fn test_paste() {
    let mut state = LineInputState::with_value("ab");

    state.cursor = 1;
    let output = state.update(LineInputMessage::Paste("xyz".to_string()));
    assert_eq!(state.value(), "axyzb");
    assert_eq!(state.cursor_byte_offset(), 4);
    assert_eq!(output, Some(LineInputOutput::Changed("axyzb".to_string())));
}

#[test]
fn test_paste_strips_newlines() {
    let mut state = LineInputState::new();

    state.update(LineInputMessage::Paste("line1\nline2\r\nline3".to_string()));
    assert_eq!(state.value(), "line1line2line3");
}

// ---- Editing: SetValue ----

#[test]
fn test_set_value_message() {
    let mut state = LineInputState::with_value("old");

    let output = state.update(LineInputMessage::SetValue("new".to_string()));
    assert_eq!(state.value(), "new");
    assert_eq!(state.cursor_byte_offset(), 3);
    assert_eq!(output, Some(LineInputOutput::Changed("new".to_string())));
}

// ---- Movement ----

#[test]
fn test_move_left() {
    let mut state = LineInputState::with_value("hello");

    state.update(LineInputMessage::Left);
    assert_eq!(state.cursor_byte_offset(), 4);
}

#[test]
fn test_move_right() {
    let mut state = LineInputState::with_value("hello");

    state.cursor = 0;
    state.update(LineInputMessage::Right);
    assert_eq!(state.cursor_byte_offset(), 1);
}

#[test]
fn test_move_home() {
    let mut state = LineInputState::with_value("hello");

    state.update(LineInputMessage::Home);
    assert_eq!(state.cursor_byte_offset(), 0);
}

#[test]
fn test_move_end() {
    let mut state = LineInputState::with_value("hello");

    state.cursor = 0;
    state.update(LineInputMessage::End);
    assert_eq!(state.cursor_byte_offset(), 5);
}

#[test]
fn test_move_word_left() {
    let mut state = LineInputState::with_value("hello world");

    state.update(LineInputMessage::WordLeft);
    assert_eq!(state.cursor_byte_offset(), 6);
}

#[test]
fn test_move_word_right() {
    let mut state = LineInputState::with_value("hello world");

    state.cursor = 0;
    state.update(LineInputMessage::WordRight);
    assert_eq!(state.cursor_byte_offset(), 6);
}

#[test]
fn test_visual_up() {
    let mut state = LineInputState::with_value("hello world!");

    state.set_display_width(5);
    // cursor at end: "hello" | " worl" | "d!"
    // cursor at byte 12 → row 2, col 2
    // VisualUp → row 1, col 2 → byte 7
    state.update(LineInputMessage::VisualUp);
    assert_eq!(state.cursor_byte_offset(), 7);
}

#[test]
fn test_visual_down() {
    let mut state = LineInputState::with_value("hello world!");

    state.set_display_width(5);
    state.cursor = 0;
    // cursor at byte 0 → row 0, col 0
    // VisualDown → row 1, col 0 → byte 5
    state.update(LineInputMessage::VisualDown);
    assert_eq!(state.cursor_byte_offset(), 5);
}

#[test]
fn test_movement_clears_selection() {
    let mut state = LineInputState::with_value("hello");

    state.selection_anchor = Some(0);
    state.update(LineInputMessage::Left);
    assert!(!state.has_selection());
}

// ---- Selection ----

#[test]
fn test_select_left() {
    let mut state = LineInputState::with_value("hello");

    state.update(LineInputMessage::SelectLeft);
    assert!(state.has_selection());
    assert_eq!(state.selected_text(), Some("o"));
}

#[test]
fn test_select_right() {
    let mut state = LineInputState::with_value("hello");

    state.cursor = 0;
    state.update(LineInputMessage::SelectRight);
    assert!(state.has_selection());
    assert_eq!(state.selected_text(), Some("h"));
}

#[test]
fn test_select_home() {
    let mut state = LineInputState::with_value("hello");

    state.update(LineInputMessage::SelectHome);
    assert_eq!(state.selected_text(), Some("hello"));
}

#[test]
fn test_select_end() {
    let mut state = LineInputState::with_value("hello");

    state.cursor = 0;
    state.update(LineInputMessage::SelectEnd);
    assert_eq!(state.selected_text(), Some("hello"));
}

#[test]
fn test_select_word_left() {
    let mut state = LineInputState::with_value("hello world");

    state.update(LineInputMessage::SelectWordLeft);
    assert_eq!(state.selected_text(), Some("world"));
}

#[test]
fn test_select_word_right() {
    let mut state = LineInputState::with_value("hello world");

    state.cursor = 0;
    state.update(LineInputMessage::SelectWordRight);
    assert_eq!(state.selected_text(), Some("hello "));
}

#[test]
fn test_select_all() {
    let mut state = LineInputState::with_value("hello");

    state.update(LineInputMessage::SelectAll);
    assert_eq!(state.selected_text(), Some("hello"));
}

#[test]
fn test_select_all_empty() {
    let mut state = LineInputState::new();

    let output = state.update(LineInputMessage::SelectAll);
    assert_eq!(output, None);
    assert!(!state.has_selection());
}

// ---- Clipboard ----

#[test]
fn test_copy() {
    let mut state = LineInputState::with_value("hello");

    state.selection_anchor = Some(0);
    // cursor at 5, anchor at 0 → selects "hello"
    let output = state.update(LineInputMessage::Copy);
    assert_eq!(output, Some(LineInputOutput::Copied("hello".to_string())));
    assert_eq!(state.clipboard(), "hello");
    // Buffer unchanged
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_copy_no_selection() {
    let mut state = LineInputState::with_value("hello");

    let output = state.update(LineInputMessage::Copy);
    assert_eq!(output, None);
}

#[test]
fn test_cut() {
    let mut state = LineInputState::with_value("hello");

    state.selection_anchor = Some(0);
    let output = state.update(LineInputMessage::Cut);
    assert_eq!(state.clipboard(), "hello");
    assert_eq!(state.value(), "");
    assert_eq!(output, Some(LineInputOutput::Changed(String::new())));
}

#[test]
fn test_cut_no_selection() {
    let mut state = LineInputState::with_value("hello");

    let output = state.update(LineInputMessage::Cut);
    assert_eq!(output, None);
}

// ---- History ----

#[test]
fn test_submit_pushes_to_history() {
    let mut state = LineInputState::with_value("hello");

    let output = state.update(LineInputMessage::Submit);
    assert_eq!(
        output,
        Some(LineInputOutput::Submitted("hello".to_string()))
    );
    assert_eq!(state.value(), "");
    assert_eq!(state.cursor_byte_offset(), 0);
    assert_eq!(state.history_count(), 1);
}

#[test]
fn test_history_prev_next() {
    let mut state = LineInputState::with_value("first");

    state.update(LineInputMessage::Submit);
    state.update(LineInputMessage::Insert('x'));

    // Navigate back
    let output = state.update(LineInputMessage::HistoryPrev);
    assert_eq!(state.value(), "first");
    assert_eq!(output, Some(LineInputOutput::Changed("first".to_string())));
    assert!(state.is_browsing_history());

    // Navigate forward (restores stashed "x")
    let output = state.update(LineInputMessage::HistoryNext);
    assert_eq!(state.value(), "x");
    assert_eq!(output, Some(LineInputOutput::Changed("x".to_string())));
    assert!(!state.is_browsing_history());
}

#[test]
fn test_history_prev_empty() {
    let mut state = LineInputState::new();

    let output = state.update(LineInputMessage::HistoryPrev);
    assert_eq!(output, None);
}

#[test]
fn test_history_next_not_browsing() {
    let mut state = LineInputState::new();

    let output = state.update(LineInputMessage::HistoryNext);
    assert_eq!(output, None);
}

#[test]
fn test_edit_exits_browse() {
    let mut state = LineInputState::with_value("first");

    state.update(LineInputMessage::Submit);

    // Browse to "first"
    state.update(LineInputMessage::HistoryPrev);
    assert!(state.is_browsing_history());

    // Edit while browsing exits browse mode
    state.update(LineInputMessage::Insert('!'));
    assert!(!state.is_browsing_history());
    assert_eq!(state.value(), "first!");
}

// ---- Undo / Redo ----

#[test]
fn test_undo_insert() {
    let mut state = LineInputState::new();

    state.update(LineInputMessage::Insert('a'));
    state.update(LineInputMessage::Insert(' ')); // whitespace breaks group
    state.update(LineInputMessage::Insert('b'));

    // Undo "b" → back to "a "
    let output = state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "a ");
    assert!(output.is_some());

    // Undo " " → back to "a"
    state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "a");

    // Undo "a" → back to ""
    state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "");
}

#[test]
fn test_redo() {
    let mut state = LineInputState::new();

    state.update(LineInputMessage::Insert('a'));
    state.update(LineInputMessage::Insert(' '));
    state.update(LineInputMessage::Insert('b'));

    state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "a ");

    state.update(LineInputMessage::Redo);
    assert_eq!(state.value(), "a b");
}

#[test]
fn test_undo_empty() {
    let mut state = LineInputState::new();

    let output = state.update(LineInputMessage::Undo);
    assert_eq!(output, None);
}

#[test]
fn test_redo_empty() {
    let mut state = LineInputState::new();

    let output = state.update(LineInputMessage::Redo);
    assert_eq!(output, None);
}

#[test]
fn test_undo_clear() {
    let mut state = LineInputState::with_value("hello");

    state.update(LineInputMessage::Clear);
    assert_eq!(state.value(), "");

    state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_undo_paste() {
    let mut state = LineInputState::new();

    state.update(LineInputMessage::Paste("hello world".to_string()));
    assert_eq!(state.value(), "hello world");

    state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "");
}

#[test]
fn test_undo_backspace() {
    let mut state = LineInputState::with_value("hi");

    state.update(LineInputMessage::Backspace);
    assert_eq!(state.value(), "h");

    state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "hi");
}

// ---- dispatch_event ----

// ---- Cursor visual position ----

#[test]
fn test_cursor_visual_position() {
    let mut state = LineInputState::with_value("hello world!");
    state.set_display_width(5);
    // cursor at end → "hello" | " worl" | "d!" → row 2, col 2
    assert_eq!(state.cursor_visual_position(), (2, 2));
}

// ---- Snapshot tests ----

#[test]
fn test_snapshot_focused() {
    let (mut terminal, theme) = setup_render(20, 3);
    let state = LineInputState::with_value("hello");

    terminal
        .draw(|frame| {
            LineInput::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_unfocused() {
    let (mut terminal, theme) = setup_render(20, 3);
    let state = LineInputState::with_value("hello");
    terminal
        .draw(|frame| {
            LineInput::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let (mut terminal, theme) = setup_render(20, 3);
    let state = LineInputState::with_value("hello");
    terminal
        .draw(|frame| {
            LineInput::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_placeholder() {
    let (mut terminal, theme) = setup_render(20, 3);
    let state = LineInputState::new().with_placeholder("Type here...");
    terminal
        .draw(|frame| {
            LineInput::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_wrapped() {
    let (mut terminal, theme) = setup_render(12, 5);
    let state = LineInputState::with_value("hello world!");

    terminal
        .draw(|frame| {
            LineInput::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_wide_chars() {
    let (mut terminal, theme) = setup_render(12, 4);
    let state = LineInputState::with_value("世界你好ab");

    terminal
        .draw(|frame| {
            LineInput::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_selection() {
    let (mut terminal, theme) = setup_render(20, 3);
    let mut state = LineInputState::with_value("hello world");

    state.cursor = 0;
    state.selection_anchor = Some(5);
    terminal
        .draw(|frame| {
            LineInput::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// max_length
// =============================================================================

#[test]
fn test_max_length_builder() {
    let state = LineInputState::new().with_max_length(10);
    assert_eq!(state.max_length(), Some(10));
}

#[test]
fn test_max_length_default_none() {
    let state = LineInputState::new();
    assert_eq!(state.max_length(), None);
}

#[test]
fn test_max_length_setter() {
    let mut state = LineInputState::new();
    state.set_max_length(Some(5));
    assert_eq!(state.max_length(), Some(5));
    state.set_max_length(None);
    assert_eq!(state.max_length(), None);
}

#[test]
fn test_max_length_insert_allowed() {
    let mut state = LineInputState::new().with_max_length(5);

    let output = LineInput::update(&mut state, LineInputMessage::Insert('a'));
    assert!(output.is_some());
    assert_eq!(state.value(), "a");
}

#[test]
fn test_max_length_insert_rejected() {
    let mut state = LineInputState::with_value("abcde").with_max_length(5);

    let output = LineInput::update(&mut state, LineInputMessage::Insert('f'));
    assert_eq!(output, None);
    assert_eq!(state.value(), "abcde");
}

#[test]
fn test_max_length_none_unlimited() {
    let mut state = LineInputState::new();

    for c in "this is a long string that should work fine".chars() {
        LineInput::update(&mut state, LineInputMessage::Insert(c));
    }
    assert_eq!(state.value(), "this is a long string that should work fine");
}

#[test]
fn test_max_length_paste_truncated() {
    let mut state = LineInputState::with_value("abc").with_max_length(5);

    let output = LineInput::update(&mut state, LineInputMessage::Paste("defgh".to_string()));
    assert!(output.is_some());
    assert_eq!(state.value(), "abcde");
}

#[test]
fn test_max_length_paste_at_limit() {
    let mut state = LineInputState::with_value("abcde").with_max_length(5);

    let output = LineInput::update(&mut state, LineInputMessage::Paste("f".to_string()));
    assert_eq!(output, None);
    assert_eq!(state.value(), "abcde");
}

#[test]
fn test_max_length_set_value_truncated() {
    let mut state = LineInputState::new().with_max_length(3);

    let output = LineInput::update(&mut state, LineInputMessage::SetValue("abcdef".to_string()));
    assert!(output.is_some());
    assert_eq!(state.value(), "abc");
}

#[test]
fn test_max_length_unicode() {
    // Unicode chars: each is 1 char but multi-byte
    let mut state = LineInputState::new().with_max_length(3);

    LineInput::update(&mut state, LineInputMessage::Insert('e'));
    LineInput::update(&mut state, LineInputMessage::Insert('n'));
    LineInput::update(&mut state, LineInputMessage::Insert('u'));
    assert_eq!(state.value(), "enu");
    // Should reject 4th char
    let output = LineInput::update(&mut state, LineInputMessage::Insert('a'));
    assert_eq!(output, None);
    assert_eq!(state.value(), "enu");
}

#[test]
fn test_max_length_existing_content_not_truncated() {
    let mut state = LineInputState::with_value("abcdef");
    state.set_max_length(Some(3));
    // Existing content is NOT truncated
    assert_eq!(state.value(), "abcdef");
    // But new insertions are rejected

    let output = LineInput::update(&mut state, LineInputMessage::Insert('g'));
    assert_eq!(output, None);
}

#[test]
fn test_max_length_insert_with_selection_replacement() {
    // If text is selected, inserting replaces selection, so effective length may allow it
    let mut state = LineInputState::with_value("abcde").with_max_length(5);

    // Select all
    LineInput::update(&mut state, LineInputMessage::SelectAll);
    // Inserting 'x' should work: replaces all 5 chars with 1
    let output = LineInput::update(&mut state, LineInputMessage::Insert('x'));
    assert!(output.is_some());
    assert_eq!(state.value(), "x");
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = LineInputState::new();
    let (mut terminal, theme) = setup_render(30, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                LineInput::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::LineInput);
    assert_eq!(regions.len(), 1);
    assert!(!regions[0].annotation.focused);
    assert!(!regions[0].annotation.disabled);
}

#[test]
fn visual_rows_at_width_empty() {
    let state = LineInputState::new();
    assert_eq!(state.visual_rows_at_width(10), 1);
    assert_eq!(state.visual_rows_at_width(0), 1);
}

#[test]
fn visual_rows_at_width_fits_single_row() {
    let state = LineInputState::with_value("Hello");
    assert_eq!(state.visual_rows_at_width(10), 1);
    assert_eq!(state.visual_rows_at_width(5), 1);
}

#[test]
fn visual_rows_at_width_wraps() {
    let state = LineInputState::with_value("Hello, world!");
    // 13 chars at width 7 → 2 rows
    assert_eq!(state.visual_rows_at_width(7), 2);
    // 13 chars at width 5 → 3 rows
    assert_eq!(state.visual_rows_at_width(5), 3);
}

#[test]
fn visual_rows_at_width_cjk() {
    // Each CJK character takes 2 columns
    let state = LineInputState::with_value("世界你好");
    // 4 CJK chars = 8 columns, width 4 → 2 rows (2 chars per row)
    assert_eq!(state.visual_rows_at_width(4), 2);
}
