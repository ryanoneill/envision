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
fn test_placeholder() {
    let mut state = SelectState::new(vec!["A", "B"]);
    assert_eq!(state.placeholder(), "Select...");

    state.set_placeholder("Choose one");
    assert_eq!(state.placeholder(), "Choose one");
}

#[test]
fn test_disabled() {
    let mut state = SelectState::new(vec!["A", "B"]);
    assert!(!state.is_disabled());

    state.set_disabled(true);
    assert!(state.is_disabled());
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

    Select::update(&mut state, SelectMessage::SelectNext);
    assert_eq!(state.highlighted_index, 1);

    Select::update(&mut state, SelectMessage::SelectNext);
    assert_eq!(state.highlighted_index, 2);

    // Wrap around
    Select::update(&mut state, SelectMessage::SelectNext);
    assert_eq!(state.highlighted_index, 0);
}

#[test]
fn test_select_previous() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    Select::update(&mut state, SelectMessage::Open);

    // Wrap around from start
    Select::update(&mut state, SelectMessage::SelectPrevious);
    assert_eq!(state.highlighted_index, 2);

    Select::update(&mut state, SelectMessage::SelectPrevious);
    assert_eq!(state.highlighted_index, 1);

    Select::update(&mut state, SelectMessage::SelectPrevious);
    assert_eq!(state.highlighted_index, 0);
}

#[test]
fn test_confirm_selection() {
    let mut state = SelectState::new(vec!["A", "B", "C"]);
    Select::update(&mut state, SelectMessage::Open);
    Select::update(&mut state, SelectMessage::SelectNext);

    let output = Select::update(&mut state, SelectMessage::Confirm);
    assert_eq!(output, Some(SelectOutput::Changed(Some(1))));
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

    let output = Select::update(&mut state, SelectMessage::SelectNext);
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
fn test_focusable() {
    let mut state = SelectState::new(vec!["A", "B"]);

    assert!(!Select::is_focused(&state));

    Select::focus(&mut state);
    assert!(Select::is_focused(&state));

    Select::blur(&mut state);
    assert!(!Select::is_focused(&state));
}

#[test]
fn test_init() {
    let state = Select::init();
    assert_eq!(state.options().len(), 0);
    assert!(!Select::is_focused(&state));
}

#[test]
fn test_clone() {
    let state = SelectState::with_selection(vec!["A", "B", "C"], 1);
    let cloned = state.clone();
    assert_eq!(cloned.selected_index(), Some(1));
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

    let output = terminal.backend().to_string();
    assert!(output.contains("Select...") || output.contains("▼"));
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

    let output = terminal.backend().to_string();
    assert!(output.contains("Red") || output.contains("Green") || output.contains("Blue"));
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

    let output = terminal.backend().to_string();
    assert!(output.contains("Medium"));
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

    // Should render without error
    let output = terminal.backend().to_string();
    assert!(!output.is_empty());
}
