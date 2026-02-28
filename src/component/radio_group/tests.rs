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
fn test_options_accessor() {
    let state = RadioGroupState::new(vec!["X", "Y", "Z"]);
    assert_eq!(state.options(), &["X", "Y", "Z"]);
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
fn test_disabled_accessors() {
    let mut state = RadioGroupState::new(vec!["A", "B"]);
    assert!(!state.is_disabled());

    state.set_disabled(true);
    assert!(state.is_disabled());

    state.set_disabled(false);
    assert!(!state.is_disabled());
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
fn test_focusable() {
    let mut state = RadioGroupState::new(vec!["A", "B"]);

    assert!(!RadioGroup::<&str>::is_focused(&state));

    RadioGroup::<&str>::set_focused(&mut state, true);
    assert!(RadioGroup::<&str>::is_focused(&state));

    RadioGroup::<&str>::blur(&mut state);
    assert!(!RadioGroup::<&str>::is_focused(&state));

    RadioGroup::<&str>::focus(&mut state);
    assert!(RadioGroup::<&str>::is_focused(&state));
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
fn test_clone() {
    let state = RadioGroupState::with_selected(vec!["A", "B", "C"], 1);
    let cloned = state.clone();

    assert_eq!(cloned.options(), &["A", "B", "C"]);
    assert_eq!(cloned.selected_index(), Some(1));
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

    let output = terminal.backend().to_string();
    assert!(output.contains("( ) Option A"));
    assert!(output.contains("(•) Option B")); // Selected
    assert!(output.contains("( ) Option C"));
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

    let output = terminal.backend().to_string();
    assert!(output.contains("(•) Test"));
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

    let output = terminal.backend().to_string();
    assert!(output.contains("( ) A"));
    assert!(output.contains("(•) B")); // Selected
    assert!(output.contains("( ) C"));
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

    let output = terminal.backend().to_string();
    // First is selected + focused (yellow)
    assert!(output.contains("(•) First"));
    // Others are unselected (default style)
    assert!(output.contains("( ) Second"));
    assert!(output.contains("( ) Third"));
}

#[test]
fn test_debug_impl() {
    let state = RadioGroupState::new(vec!["x", "y"]);
    let debug = format!("{:?}", state);
    assert!(debug.contains("RadioGroupState"));
}

#[test]
fn test_radio_message_eq() {
    assert_eq!(RadioMessage::Up, RadioMessage::Up);
    assert_eq!(RadioMessage::Down, RadioMessage::Down);
    assert_eq!(RadioMessage::Confirm, RadioMessage::Confirm);
    assert_ne!(RadioMessage::Up, RadioMessage::Down);
}

#[test]
fn test_radio_message_debug() {
    let debug = format!("{:?}", RadioMessage::Confirm);
    assert_eq!(debug, "Confirm");
}

#[test]
fn test_radio_output_eq() {
    let out1: RadioOutput<&str> = RadioOutput::Selected("a");
    let out2: RadioOutput<&str> = RadioOutput::Selected("a");
    assert_eq!(out1, out2);

    let out3: RadioOutput<i32> = RadioOutput::Confirmed(42);
    let out4: RadioOutput<i32> = RadioOutput::Confirmed(42);
    assert_eq!(out3, out4);
}

#[test]
fn test_radio_output_debug() {
    let out: RadioOutput<&str> = RadioOutput::Selected("test");
    let debug = format!("{:?}", out);
    assert!(debug.contains("Selected"));
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
