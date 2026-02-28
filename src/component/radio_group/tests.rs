use super::*;

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

    let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
    assert_eq!(output, Some(RadioOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));

    let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
    assert_eq!(output, Some(RadioOutput::SelectionChanged(2)));
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_navigate_up() {
    let mut state = RadioGroupState::with_selected(vec!["A", "B", "C"], 2);

    let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Up);
    assert_eq!(output, Some(RadioOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));

    let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Up);
    assert_eq!(output, Some(RadioOutput::SelectionChanged(0)));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_navigate_at_bounds() {
    let mut state = RadioGroupState::new(vec!["A", "B", "C"]);

    // At first, Up returns None
    let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Up);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));

    // Go to last
    state.set_selected(2);

    // At last, Down returns None
    let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_confirm() {
    let mut state = RadioGroupState::with_selected(vec!["A", "B", "C"], 1);

    let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Confirm);
    assert_eq!(output, Some(RadioOutput::Confirmed("B")));
    // Selection unchanged
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_confirm_empty() {
    let mut state = RadioGroupState::<String>::new(vec![]);

    let output = RadioGroup::<String>::update(&mut state, RadioMessage::Confirm);
    assert_eq!(output, None);
}

#[test]
fn test_disabled() {
    let mut state = RadioGroupState::new(vec!["A", "B", "C"]);
    state.set_disabled(true);

    let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));

    let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Up);
    assert_eq!(output, None);

    let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Confirm);
    assert_eq!(output, None);
}

#[test]
fn test_empty_navigation() {
    let mut state = RadioGroupState::<String>::new(vec![]);

    assert_eq!(
        RadioGroup::<String>::update(&mut state, RadioMessage::Down),
        None
    );
    assert_eq!(
        RadioGroup::<String>::update(&mut state, RadioMessage::Up),
        None
    );
    assert_eq!(
        RadioGroup::<String>::update(&mut state, RadioMessage::Confirm),
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
    RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
    RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(state.selected(), Some(&"3"));

    // Navigate up
    RadioGroup::<&str>::update(&mut state, RadioMessage::Up);
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

    let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
    assert_eq!(output, Some(RadioOutput::SelectionChanged(1)));
    assert_eq!(state.selected(), Some(&"Green"));
}

#[test]
fn test_navigate_up_outputs_selection_changed() {
    let mut state = RadioGroupState::with_selected(vec!["Red", "Green", "Blue"], 2);

    let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Up);
    assert_eq!(output, Some(RadioOutput::SelectionChanged(1)));
    assert_eq!(state.selected(), Some(&"Green"));
}
