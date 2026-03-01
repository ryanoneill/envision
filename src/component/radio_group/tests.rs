use super::*;
use crate::input::{Event, KeyCode};

#[test]
fn test_new() {
    let state = RadioGroupState::new(vec!["A", "B", "C"]);
    assert_eq!(state.len(), 3);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected(), Some(&"A"));
    assert!(!state.is_disabled());
    assert!(!RadioGroup::<&str>::is_focused(&state));
}

#[test]
fn test_new_empty() {
    let state = RadioGroupState::<String>::new(vec![]);
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.selected(), None);
}

#[test]
fn test_with_selected() {
    let state = RadioGroupState::with_selected(vec!["A", "B", "C"], 1);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.selected(), Some(&"B"));
}

#[test]
fn test_with_selected_clamps() {
    let state = RadioGroupState::with_selected(vec!["A", "B", "C"], 10);
    assert_eq!(state.selected_index(), Some(2)); // Clamped to last
}

#[test]
fn test_with_selected_empty_options() {
    let state = RadioGroupState::<String>::with_selected(vec![], 5);
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.selected(), None);
}

#[test]
fn test_default() {
    let state = RadioGroupState::<String>::default();
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.selected(), None);
}

#[test]
fn test_selected_accessors() {
    let mut state = RadioGroupState::new(vec!["A", "B", "C"]);

    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected(), Some(&"A"));

    state.set_selected(2);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(state.selected(), Some(&"C"));

    // Out of bounds is ignored
    state.set_selected(100);
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_navigate_down() {
    let mut state = RadioGroupState::new(vec!["A", "B", "C"]);

    let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Down);
    assert_eq!(output, Some(RadioGroupOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));

    let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Down);
    assert_eq!(output, Some(RadioGroupOutput::SelectionChanged(2)));
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_navigate_up() {
    let mut state = RadioGroupState::with_selected(vec!["A", "B", "C"], 2);

    let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Up);
    assert_eq!(output, Some(RadioGroupOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));

    let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Up);
    assert_eq!(output, Some(RadioGroupOutput::SelectionChanged(0)));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_navigate_at_bounds() {
    let mut state = RadioGroupState::new(vec!["A", "B", "C"]);

    // At first, Up returns None
    let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Up);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));

    // Go to last
    state.set_selected(2);

    // At last, Down returns None
    let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Down);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_confirm() {
    let mut state = RadioGroupState::with_selected(vec!["A", "B", "C"], 1);

    let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Confirm);
    assert_eq!(output, Some(RadioGroupOutput::Confirmed("B")));
    // Selection unchanged
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_confirm_empty() {
    let mut state = RadioGroupState::<String>::new(vec![]);

    let output = RadioGroup::<String>::update(&mut state, RadioGroupMessage::Confirm);
    assert_eq!(output, None);
}

#[test]
fn test_disabled() {
    let mut state = RadioGroupState::new(vec!["A", "B", "C"]);
    state.set_disabled(true);

    let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Down);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));

    let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Up);
    assert_eq!(output, None);

    let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Confirm);
    assert_eq!(output, None);
}

#[test]
fn test_empty_navigation() {
    let mut state = RadioGroupState::<String>::new(vec![]);

    assert_eq!(
        RadioGroup::<String>::update(&mut state, RadioGroupMessage::Down),
        None
    );
    assert_eq!(
        RadioGroup::<String>::update(&mut state, RadioGroupMessage::Up),
        None
    );
    assert_eq!(
        RadioGroup::<String>::update(&mut state, RadioGroupMessage::Confirm),
        None
    );
}

#[test]
fn test_init() {
    let state = RadioGroup::<String>::init();
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
    assert!(!state.is_disabled());
    assert!(!RadioGroup::<String>::is_focused(&state));
}

#[test]
fn test_view_renders_indicators() {
    let mut state = RadioGroupState::with_selected(vec!["Option A", "Option B", "Option C"], 1);
    state.focused = true;
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            RadioGroup::<&str>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let mut state = RadioGroupState::new(vec!["Test"]);
    state.set_disabled(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            RadioGroup::<&str>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_multiple_navigations() {
    let mut state = RadioGroupState::new(vec!["1", "2", "3", "4", "5"]);

    // Navigate down multiple times
    RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Down);
    RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Down);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(state.selected(), Some(&"3"));

    // Navigate up
    RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Up);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.selected(), Some(&"2"));
}

#[test]
fn test_view_unfocused() {
    let mut state = RadioGroupState::with_selected(vec!["A", "B", "C"], 1);
    state.focused = false; // Explicitly unfocused
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            RadioGroup::<&str>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused_not_selected() {
    // Render with focused but rendering non-selected items
    let mut state = RadioGroupState::with_selected(vec!["First", "Second", "Third"], 0);
    state.focused = true;
    let (mut terminal, theme) = crate::component::test_utils::setup_render(50, 10);

    terminal
        .draw(|frame| {
            RadioGroup::<&str>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_navigate_down_outputs_selection_changed() {
    let mut state = RadioGroupState::new(vec!["Red", "Green", "Blue"]);

    let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Down);
    assert_eq!(output, Some(RadioGroupOutput::SelectionChanged(1)));
    assert_eq!(state.selected(), Some(&"Green"));
}

#[test]
fn test_navigate_up_outputs_selection_changed() {
    let mut state = RadioGroupState::with_selected(vec!["Red", "Green", "Blue"], 2);

    let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Up);
    assert_eq!(output, Some(RadioGroupOutput::SelectionChanged(1)));
    assert_eq!(state.selected(), Some(&"Green"));
}

#[test]
fn test_large_radio_group_navigation() {
    let options: Vec<String> = (0..100).map(|i| format!("Option {}", i)).collect();
    let mut state = RadioGroupState::new(options);

    // Navigate to middle
    for _ in 0..50 {
        RadioGroup::<String>::update(&mut state, RadioGroupMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(50));

    // Navigate back to start
    for _ in 0..50 {
        RadioGroup::<String>::update(&mut state, RadioGroupMessage::Up);
    }
    assert_eq!(state.selected_index(), Some(0));

    // Up from 0 stays at 0 (no wrapping)
    let output = RadioGroup::<String>::update(&mut state, RadioGroupMessage::Up);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));

    // Navigate to last
    for _ in 0..99 {
        RadioGroup::<String>::update(&mut state, RadioGroupMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(99));

    // Down from last stays at last (no wrapping)
    let output = RadioGroup::<String>::update(&mut state, RadioGroupMessage::Down);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(99));
}

// handle_event tests

#[test]
fn test_handle_event_up() {
    let mut state =
        RadioGroupState::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    state.set_focused(true);
    let msg = RadioGroup::<String>::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, Some(RadioGroupMessage::Up));
}

#[test]
fn test_handle_event_down() {
    let mut state =
        RadioGroupState::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    state.set_focused(true);
    let msg = RadioGroup::<String>::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, Some(RadioGroupMessage::Down));
}

#[test]
fn test_handle_event_k() {
    let mut state =
        RadioGroupState::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    state.set_focused(true);
    let msg = RadioGroup::<String>::handle_event(&state, &Event::char('k'));
    assert_eq!(msg, Some(RadioGroupMessage::Up));
}

#[test]
fn test_handle_event_j() {
    let mut state =
        RadioGroupState::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    state.set_focused(true);
    let msg = RadioGroup::<String>::handle_event(&state, &Event::char('j'));
    assert_eq!(msg, Some(RadioGroupMessage::Down));
}

#[test]
fn test_handle_event_enter() {
    let mut state =
        RadioGroupState::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    state.set_focused(true);
    let msg = RadioGroup::<String>::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(RadioGroupMessage::Confirm));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = RadioGroupState::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    let msg = RadioGroup::<String>::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_ignored_when_disabled() {
    let mut state =
        RadioGroupState::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    state.set_focused(true);
    state.set_disabled(true);
    let msg = RadioGroup::<String>::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, None);
}

#[test]
fn test_dispatch_event_radio() {
    let mut state =
        RadioGroupState::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    state.set_focused(true);
    let output = RadioGroup::<String>::dispatch_event(&mut state, &Event::key(KeyCode::Down));
    assert_eq!(output, Some(RadioGroupOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_instance_is_focused() {
    let mut state =
        RadioGroupState::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
}

#[test]
fn test_instance_handle_event() {
    let mut state =
        RadioGroupState::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    state.set_focused(true);
    let msg = state.handle_event(&Event::key(KeyCode::Down));
    assert_eq!(msg, Some(RadioGroupMessage::Down));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state =
        RadioGroupState::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    state.set_focused(true);
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, Some(RadioGroupOutput::SelectionChanged(1)));
}

#[test]
fn test_instance_update() {
    let mut state =
        RadioGroupState::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    let output = state.update(RadioGroupMessage::Down);
    assert_eq!(output, Some(RadioGroupOutput::SelectionChanged(1)));
}
