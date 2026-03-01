use super::*;

#[test]
fn test_new() {
    let state = SelectState::new(vec!["A", "B", "C"]);
    assert_eq!(state.options().len(), 3);
    assert_eq!(state.selected_index(), None);
    assert!(!state.is_open());
    assert!(!Select::is_focused(&state));
}

#[test]
fn test_with_selection() {
    let state = SelectState::with_selection(vec!["A", "B", "C"], 1);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.selected_value(), Some("B"));
}

#[test]
fn test_with_selection_out_of_bounds() {
    let state = SelectState::with_selection(vec!["A", "B"], 5);
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_default() {
    let state = SelectState::default();
    assert_eq!(state.options().len(), 0);
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_set_options() {
    let mut state = SelectState::new(vec!["A", "B"]);
    state.set_options(vec!["X", "Y", "Z"]);
    assert_eq!(state.options().len(), 3);
    assert_eq!(state.options()[0], "X");
}

#[test]
fn test_set_options_resets_invalid_selection() {
    let mut state = SelectState::with_selection(vec!["A", "B", "C"], 2);
    state.set_options(vec!["X", "Y"]);
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_set_selected_index() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    state.set_selected_index(Some(1));
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.selected_value(), Some("B"));
}

#[test]
fn test_set_selected_index_out_of_bounds() {
    let mut state = SelectState::new(vec!["A", "B"]);
    state.set_selected_index(Some(5));
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_open_close() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);

    Select::update(&mut state, SelectMessage::Open);
    assert!(state.is_open());

    Select::update(&mut state, SelectMessage::Close);
    assert!(!state.is_open());
}

#[test]
fn test_toggle() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);

    Select::update(&mut state, SelectMessage::Toggle);
    assert!(state.is_open());

    Select::update(&mut state, SelectMessage::Toggle);
    assert!(!state.is_open());
}

#[test]
fn test_open_empty_options() {
    let mut state = SelectState::new(Vec::<String>::new());

    Select::update(&mut state, SelectMessage::Open);
    assert!(!state.is_open());
}

#[test]
fn test_select_next() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    Select::update(&mut state, SelectMessage::Open);

    let output = Select::update(&mut state, SelectMessage::Down);
    assert_eq!(output, Some(SelectOutput::SelectionChanged(1)));
    assert_eq!(state.highlighted_index, 1);

    let output = Select::update(&mut state, SelectMessage::Down);
    assert_eq!(output, Some(SelectOutput::SelectionChanged(2)));
    assert_eq!(state.highlighted_index, 2);

    // Wrap around
    let output = Select::update(&mut state, SelectMessage::Down);
    assert_eq!(output, Some(SelectOutput::SelectionChanged(0)));
    assert_eq!(state.highlighted_index, 0);
}

#[test]
fn test_select_previous() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    Select::update(&mut state, SelectMessage::Open);

    // Wrap around from start
    let output = Select::update(&mut state, SelectMessage::Up);
    assert_eq!(output, Some(SelectOutput::SelectionChanged(2)));
    assert_eq!(state.highlighted_index, 2);

    let output = Select::update(&mut state, SelectMessage::Up);
    assert_eq!(output, Some(SelectOutput::SelectionChanged(1)));
    assert_eq!(state.highlighted_index, 1);

    let output = Select::update(&mut state, SelectMessage::Up);
    assert_eq!(output, Some(SelectOutput::SelectionChanged(0)));
    assert_eq!(state.highlighted_index, 0);
}

#[test]
fn test_confirm_selection() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    Select::update(&mut state, SelectMessage::Open);
    Select::update(&mut state, SelectMessage::Down);

    let output = Select::update(&mut state, SelectMessage::Confirm);
    assert_eq!(output, Some(SelectOutput::Selected("B".to_string())));
    assert_eq!(state.selected_index(), Some(1));
    assert!(!state.is_open());
}

#[test]
fn test_confirm_same_selection() {
    let mut state = SelectState::with_selection(vec!["A", "B", "C"], 1);
    Select::update(&mut state, SelectMessage::Open);

    let output = Select::update(&mut state, SelectMessage::Confirm);
    assert_eq!(output, Some(SelectOutput::Submitted(1)));
    assert!(!state.is_open());
}

#[test]
fn test_confirm_when_closed() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);

    let output = Select::update(&mut state, SelectMessage::Confirm);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_messages() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    state.set_disabled(true);

    let output = Select::update(&mut state, SelectMessage::Open);
    assert_eq!(output, None);
    assert!(!state.is_open());

    let output = Select::update(&mut state, SelectMessage::Down);
    assert_eq!(output, None);
}

#[test]
fn test_disabling_closes_dropdown() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    Select::update(&mut state, SelectMessage::Open);
    assert!(state.is_open());

    state.set_disabled(true);
    assert!(!state.is_open());
}

#[test]
fn test_init() {
    let state = Select::init();
    assert_eq!(state.options().len(), 0);
    assert!(!Select::is_focused(&state));
}

#[test]
fn test_view_closed() {
    let state = SelectState::new(vec!["Red", "Green", "Blue"]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 10);

    terminal
        .draw(|frame| {
            Select::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_open() {
    let mut state = SelectState::new(vec!["Red", "Green", "Blue"]);
    Select::update(&mut state, SelectMessage::Open);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 15);

    terminal
        .draw(|frame| {
            Select::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_selection() {
    let state = SelectState::with_selection(vec!["Small", "Medium", "Large"], 1);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 10);

    terminal
        .draw(|frame| {
            Select::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let mut state = SelectState::new(vec!["A", "B"]);
    Select::focus(&mut state);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 10);

    terminal
        .draw(|frame| {
            Select::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_large_select_navigation() {
    let options: Vec<String> = (0..100).map(|i| format!("Option {}", i)).collect();
    let mut state = SelectState::new(options);

    // Open the select
    Select::update(&mut state, SelectMessage::Open);
    assert!(state.is_open());

    // Navigate to middle
    for _ in 0..50 {
        Select::update(&mut state, SelectMessage::Down);
    }
    assert_eq!(state.highlighted_index, 50);

    // Navigate 50 more to wrap back to 0
    for _ in 0..50 {
        Select::update(&mut state, SelectMessage::Down);
    }
    assert_eq!(state.highlighted_index, 0);

    // Up from 0 wraps to last
    Select::update(&mut state, SelectMessage::Up);
    assert_eq!(state.highlighted_index, 99);

    // Confirm selection at index 99
    let output = Select::update(&mut state, SelectMessage::Confirm);
    assert_eq!(
        output,
        Some(SelectOutput::Selected("Option 99".to_string()))
    );
    assert_eq!(state.selected_index(), Some(99));
    assert!(!state.is_open());
}

// ========================================
// handle_event Tests
// ========================================

use crate::input::{Event, KeyCode};

#[test]
fn test_handle_event_toggle_when_closed() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    Select::focus(&mut state);

    // Enter when closed -> Toggle
    let msg = Select::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(SelectMessage::Toggle));

    // Space when closed -> Toggle
    let msg = Select::handle_event(&state, &Event::char(' '));
    assert_eq!(msg, Some(SelectMessage::Toggle));
}

#[test]
fn test_handle_event_confirm_when_open() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    Select::focus(&mut state);
    Select::update(&mut state, SelectMessage::Open);

    let msg = Select::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(SelectMessage::Confirm));
}

#[test]
fn test_handle_event_close_when_open() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    Select::focus(&mut state);
    Select::update(&mut state, SelectMessage::Open);

    let msg = Select::handle_event(&state, &Event::key(KeyCode::Esc));
    assert_eq!(msg, Some(SelectMessage::Close));
}

#[test]
fn test_handle_event_up_when_open() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    Select::focus(&mut state);
    Select::update(&mut state, SelectMessage::Open);

    let msg = Select::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, Some(SelectMessage::Up));

    // Vim 'k' also maps to Up
    let msg = Select::handle_event(&state, &Event::char('k'));
    assert_eq!(msg, Some(SelectMessage::Up));
}

#[test]
fn test_handle_event_down_when_open() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    Select::focus(&mut state);
    Select::update(&mut state, SelectMessage::Open);

    let msg = Select::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, Some(SelectMessage::Down));

    // Vim 'j' also maps to Down
    let msg = Select::handle_event(&state, &Event::char('j'));
    assert_eq!(msg, Some(SelectMessage::Down));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = SelectState::new(vec!["A", "B", "C"]);
    let msg = Select::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_ignored_when_disabled() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    Select::focus(&mut state);
    state.set_disabled(true);
    let msg = Select::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, None);
}

// ========================================
// dispatch_event Tests
// ========================================

#[test]
fn test_dispatch_event() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    Select::focus(&mut state);
    Select::update(&mut state, SelectMessage::Open);
    Select::update(&mut state, SelectMessage::Down);

    // Enter when open dispatches Confirm, which selects the item
    let output = Select::dispatch_event(&mut state, &Event::key(KeyCode::Enter));
    assert_eq!(output, Some(SelectOutput::Selected("B".to_string())));
    assert!(!state.is_open());
}

// ========================================
// Instance Method Tests
// ========================================

#[test]
fn test_instance_methods() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    state.set_focused(true);

    // instance handle_event
    let msg = state.handle_event(&Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(SelectMessage::Toggle));

    // instance update
    let output = state.update(SelectMessage::Toggle);
    assert!(output.is_none()); // Toggle just opens, returns None
    assert!(state.is_open());

    // instance dispatch_event
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, Some(SelectOutput::SelectionChanged(1)));
}
