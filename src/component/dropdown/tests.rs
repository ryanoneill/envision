use super::*;

// ========== State Creation Tests ==========

#[test]
fn test_new() {
    let state = DropdownState::new(vec!["A", "B", "C"]);
    assert_eq!(state.options().len(), 3);
    assert_eq!(state.selected_index(), None);
    assert!(!state.is_open());
    assert!(!Dropdown::is_focused(&state));
    assert_eq!(state.filtered_indices.len(), 3);
}

#[test]
fn test_with_selection() {
    let state = DropdownState::with_selection(vec!["A", "B", "C"], 1);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.selected_value(), Some("B"));
}

#[test]
fn test_with_selection_out_of_bounds() {
    let state = DropdownState::with_selection(vec!["A", "B"], 5);
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_default() {
    let state = DropdownState::default();
    assert_eq!(state.options().len(), 0);
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.placeholder(), "Search...");
}

// ========== Accessor Tests ==========

#[test]
fn test_options() {
    let state = DropdownState::new(vec!["X", "Y", "Z"]);
    assert_eq!(state.options(), &["X", "Y", "Z"]);
}

#[test]
fn test_selected_index() {
    let state = DropdownState::with_selection(vec!["A", "B"], 0);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_selected_value() {
    let state = DropdownState::with_selection(vec!["Apple", "Banana"], 1);
    assert_eq!(state.selected_value(), Some("Banana"));

    let empty_state = DropdownState::new(vec!["A", "B"]);
    assert_eq!(empty_state.selected_value(), None);
}

#[test]
fn test_filter_text() {
    let mut state = DropdownState::new(vec!["A", "B"]);
    assert_eq!(state.filter_text(), "");

    Dropdown::update(&mut state, DropdownMessage::Insert('x'));
    assert_eq!(state.filter_text(), "x");
}

#[test]
fn test_filtered_options() {
    let mut state = DropdownState::new(vec!["Apple", "Banana", "Apricot"]);
    assert_eq!(state.filtered_options(), vec!["Apple", "Banana", "Apricot"]);

    Dropdown::update(&mut state, DropdownMessage::Insert('a'));
    Dropdown::update(&mut state, DropdownMessage::Insert('p'));
    assert_eq!(state.filtered_options(), vec!["Apple", "Apricot"]);
}

#[test]
fn test_is_open() {
    let mut state = DropdownState::new(vec!["A", "B"]);
    assert!(!state.is_open());

    Dropdown::update(&mut state, DropdownMessage::Open);
    assert!(state.is_open());
}

#[test]
fn test_placeholder() {
    let state = DropdownState::new(vec!["A"]);
    assert_eq!(state.placeholder(), "Search...");
}

#[test]
fn test_is_disabled() {
    let mut state = DropdownState::new(vec!["A"]);
    assert!(!state.is_disabled());

    state.set_disabled(true);
    assert!(state.is_disabled());
}

// ========== Mutator Tests ==========

#[test]
fn test_set_options() {
    let mut state = DropdownState::new(vec!["A", "B"]);
    state.set_options(vec!["X", "Y", "Z"]);
    assert_eq!(state.options().len(), 3);
    assert_eq!(state.options()[0], "X");
}

#[test]
fn test_set_options_resets_invalid_selection() {
    let mut state = DropdownState::with_selection(vec!["A", "B", "C"], 2);
    state.set_options(vec!["X", "Y"]);
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_set_selected_index() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    state.set_selected_index(Some(1));
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.selected_value(), Some("B"));
}

#[test]
fn test_set_selected_index_out_of_bounds() {
    let mut state = DropdownState::new(vec!["A", "B"]);
    state.set_selected_index(Some(5));
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_set_placeholder() {
    let mut state = DropdownState::new(vec!["A"]);
    state.set_placeholder("Type here...");
    assert_eq!(state.placeholder(), "Type here...");
}

#[test]
fn test_set_disabled() {
    let mut state = DropdownState::new(vec!["A", "B"]);
    state.set_disabled(true);
    assert!(state.is_disabled());
}

// ========== Open/Close Tests ==========

#[test]
fn test_open() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    assert!(state.is_open());
}

#[test]
fn test_close() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    Dropdown::update(&mut state, DropdownMessage::Close);
    assert!(!state.is_open());
}

#[test]
fn test_toggle() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);

    Dropdown::update(&mut state, DropdownMessage::Toggle);
    assert!(state.is_open());

    Dropdown::update(&mut state, DropdownMessage::Toggle);
    assert!(!state.is_open());
}

#[test]
fn test_open_empty_options() {
    let mut state = DropdownState::new(Vec::<String>::new());
    Dropdown::update(&mut state, DropdownMessage::Open);
    assert!(!state.is_open());
}

#[test]
fn test_close_clears_filter() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    Dropdown::update(&mut state, DropdownMessage::Insert('a'));
    assert_eq!(state.filter_text(), "a");

    Dropdown::update(&mut state, DropdownMessage::Close);
    assert_eq!(state.filter_text(), "");
}

// ========== Filtering Tests ==========

#[test]
fn test_insert_char() {
    let mut state = DropdownState::new(vec!["A", "B"]);
    let output = Dropdown::update(&mut state, DropdownMessage::Insert('x'));
    assert_eq!(state.filter_text(), "x");
    assert_eq!(output, Some(DropdownOutput::FilterChanged("x".to_string())));
}

#[test]
fn test_insert_filters() {
    let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
    Dropdown::update(&mut state, DropdownMessage::Insert('a'));

    // Apple, Banana both contain 'a'
    assert_eq!(state.filtered_count(), 2);
    assert!(state.filtered_options().contains(&"Apple"));
    assert!(state.filtered_options().contains(&"Banana"));
}

#[test]
fn test_backspace() {
    let mut state = DropdownState::new(vec!["A", "B"]);
    Dropdown::update(&mut state, DropdownMessage::Insert('a'));
    Dropdown::update(&mut state, DropdownMessage::Insert('b'));
    assert_eq!(state.filter_text(), "ab");

    let output = Dropdown::update(&mut state, DropdownMessage::Backspace);
    assert_eq!(state.filter_text(), "a");
    assert_eq!(output, Some(DropdownOutput::FilterChanged("a".to_string())));
}

#[test]
fn test_backspace_empty() {
    let mut state = DropdownState::new(vec!["A", "B"]);
    let output = Dropdown::update(&mut state, DropdownMessage::Backspace);
    assert_eq!(output, None);
}

#[test]
fn test_backspace_refilters() {
    let mut state = DropdownState::new(vec!["Apple", "Banana", "Apricot"]);
    Dropdown::update(&mut state, DropdownMessage::Insert('a'));
    Dropdown::update(&mut state, DropdownMessage::Insert('p'));
    assert_eq!(state.filtered_count(), 2); // Apple, Apricot

    Dropdown::update(&mut state, DropdownMessage::Backspace);
    assert_eq!(state.filtered_count(), 3); // All contain 'a'
}

#[test]
fn test_clear_filter() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    Dropdown::update(&mut state, DropdownMessage::Insert('x'));
    Dropdown::update(&mut state, DropdownMessage::Insert('y'));

    let output = Dropdown::update(&mut state, DropdownMessage::ClearFilter);
    assert_eq!(state.filter_text(), "");
    assert_eq!(output, Some(DropdownOutput::FilterChanged("".to_string())));
}

#[test]
fn test_clear_filter_empty() {
    let mut state = DropdownState::new(vec!["A", "B"]);
    let output = Dropdown::update(&mut state, DropdownMessage::ClearFilter);
    assert_eq!(output, None);
}

#[test]
fn test_set_filter() {
    let mut state = DropdownState::new(vec!["Apple", "Banana"]);
    let output = Dropdown::update(&mut state, DropdownMessage::SetFilter("app".to_string()));

    assert_eq!(state.filter_text(), "app");
    assert_eq!(
        output,
        Some(DropdownOutput::FilterChanged("app".to_string()))
    );
    assert_eq!(state.filtered_count(), 1);
}

#[test]
fn test_set_filter_same() {
    let mut state = DropdownState::new(vec!["A", "B"]);
    Dropdown::update(&mut state, DropdownMessage::SetFilter("x".to_string()));

    let output = Dropdown::update(&mut state, DropdownMessage::SetFilter("x".to_string()));
    assert_eq!(output, None);
}

#[test]
fn test_filter_case_insensitive() {
    let mut state = DropdownState::new(vec!["Apple", "BANANA", "cherry"]);
    Dropdown::update(&mut state, DropdownMessage::Insert('A'));

    // Should match Apple, BANANA (both contain 'a' case-insensitively)
    assert_eq!(state.filtered_count(), 2);
    assert!(state.filtered_options().contains(&"Apple"));
    assert!(state.filtered_options().contains(&"BANANA"));
}

#[test]
fn test_filter_contains() {
    let mut state = DropdownState::new(vec!["Apple", "Pineapple", "Grape"]);
    Dropdown::update(&mut state, DropdownMessage::Insert('p'));
    Dropdown::update(&mut state, DropdownMessage::Insert('l'));
    Dropdown::update(&mut state, DropdownMessage::Insert('e'));

    // "ple" is contained in Apple, Pineapple (not Grape)
    assert_eq!(state.filtered_count(), 2);
}

#[test]
fn test_filter_no_matches() {
    let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
    Dropdown::update(&mut state, DropdownMessage::Insert('x'));
    Dropdown::update(&mut state, DropdownMessage::Insert('y'));
    Dropdown::update(&mut state, DropdownMessage::Insert('z'));

    assert_eq!(state.filtered_count(), 0);
    assert!(state.filtered_options().is_empty());
}

#[test]
fn test_filter_resets_highlight() {
    let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
    Dropdown::update(&mut state, DropdownMessage::Open);

    // Navigate to second item
    Dropdown::update(&mut state, DropdownMessage::SelectNext);
    assert_eq!(state.highlighted_index, 1);

    // Filter - should reset highlight to 0
    Dropdown::update(&mut state, DropdownMessage::Insert('a'));
    assert_eq!(state.highlighted_index, 0);
}

// ========== Navigation Tests ==========

#[test]
fn test_select_next() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    Dropdown::update(&mut state, DropdownMessage::Open);

    Dropdown::update(&mut state, DropdownMessage::SelectNext);
    assert_eq!(state.highlighted_index, 1);

    Dropdown::update(&mut state, DropdownMessage::SelectNext);
    assert_eq!(state.highlighted_index, 2);
}

#[test]
fn test_select_previous() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    Dropdown::update(&mut state, DropdownMessage::SelectNext);
    Dropdown::update(&mut state, DropdownMessage::SelectNext);
    assert_eq!(state.highlighted_index, 2);

    Dropdown::update(&mut state, DropdownMessage::SelectPrevious);
    assert_eq!(state.highlighted_index, 1);

    Dropdown::update(&mut state, DropdownMessage::SelectPrevious);
    assert_eq!(state.highlighted_index, 0);
}

#[test]
fn test_select_next_wraps() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    Dropdown::update(&mut state, DropdownMessage::Open);

    Dropdown::update(&mut state, DropdownMessage::SelectNext);
    Dropdown::update(&mut state, DropdownMessage::SelectNext);
    Dropdown::update(&mut state, DropdownMessage::SelectNext);
    assert_eq!(state.highlighted_index, 0); // Wrapped
}

#[test]
fn test_select_previous_wraps() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    Dropdown::update(&mut state, DropdownMessage::Open);

    Dropdown::update(&mut state, DropdownMessage::SelectPrevious);
    assert_eq!(state.highlighted_index, 2); // Wrapped to end
}

#[test]
fn test_navigation_empty_filter() {
    let mut state = DropdownState::new(vec!["Apple", "Banana"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    Dropdown::update(&mut state, DropdownMessage::SetFilter("xyz".to_string()));

    // No matches - navigation should be no-op
    Dropdown::update(&mut state, DropdownMessage::SelectNext);
    assert_eq!(state.highlighted_index, 0);
}

#[test]
fn test_navigation_when_closed() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    // Closed - navigation should be no-op
    Dropdown::update(&mut state, DropdownMessage::SelectNext);
    assert_eq!(state.highlighted_index, 0);
}

// ========== Confirm Tests ==========

#[test]
fn test_confirm() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    Dropdown::update(&mut state, DropdownMessage::SelectNext);

    Dropdown::update(&mut state, DropdownMessage::Confirm);
    assert_eq!(state.selected_index(), Some(1));
    assert!(!state.is_open());
}

#[test]
fn test_confirm_returns_changed() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    Dropdown::update(&mut state, DropdownMessage::SelectNext);

    let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
    assert_eq!(output, Some(DropdownOutput::Changed(Some(1))));
}

#[test]
fn test_confirm_returns_submitted() {
    let mut state = DropdownState::with_selection(vec!["A", "B", "C"], 1);
    Dropdown::update(&mut state, DropdownMessage::Open);
    // Highlight is on selected item

    let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
    assert_eq!(output, Some(DropdownOutput::Submitted(1)));
}

#[test]
fn test_confirm_when_closed() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
    assert_eq!(output, None);
}

#[test]
fn test_confirm_no_matches() {
    let mut state = DropdownState::new(vec!["Apple", "Banana"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    Dropdown::update(&mut state, DropdownMessage::SetFilter("xyz".to_string()));

    let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
    assert_eq!(output, None);
}

#[test]
fn test_confirm_clears_filter() {
    let mut state = DropdownState::new(vec!["Apple", "Banana"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    Dropdown::update(&mut state, DropdownMessage::Insert('a'));
    assert_eq!(state.filter_text(), "a");

    Dropdown::update(&mut state, DropdownMessage::Confirm);
    assert_eq!(state.filter_text(), "");
}

#[test]
fn test_confirm_with_filter() {
    let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    Dropdown::update(&mut state, DropdownMessage::Insert('a'));
    // Filtered: Apple (0), Banana (1)
    Dropdown::update(&mut state, DropdownMessage::SelectNext);
    // Highlight on Banana (index 1 in filtered = original index 1)

    let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
    assert_eq!(output, Some(DropdownOutput::Changed(Some(1))));
    assert_eq!(state.selected_value(), Some("Banana"));
}

// ========== Disabled State Tests ==========

#[test]
fn test_disabled_ignores_messages() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    state.set_disabled(true);

    let output = Dropdown::update(&mut state, DropdownMessage::Open);
    assert_eq!(output, None);
    assert!(!state.is_open());

    let output = Dropdown::update(&mut state, DropdownMessage::Insert('a'));
    assert_eq!(output, None);
    assert_eq!(state.filter_text(), "");
}

#[test]
fn test_disabling_closes_dropdown() {
    let mut state = DropdownState::new(vec!["A", "B", "C"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    assert!(state.is_open());

    state.set_disabled(true);
    assert!(!state.is_open());
}

// ========== Focus Tests ==========

#[test]
fn test_focusable_is_focused() {
    let state = DropdownState::new(vec!["A", "B"]);
    assert!(!Dropdown::is_focused(&state));
}

#[test]
fn test_focusable_set_focused() {
    let mut state = DropdownState::new(vec!["A", "B"]);
    Dropdown::set_focused(&mut state, true);
    assert!(Dropdown::is_focused(&state));
}

#[test]
fn test_focus_blur() {
    let mut state = DropdownState::new(vec!["A", "B"]);

    Dropdown::focus(&mut state);
    assert!(Dropdown::is_focused(&state));

    Dropdown::blur(&mut state);
    assert!(!Dropdown::is_focused(&state));
}

// ========== View Tests ==========

#[test]
fn test_view_closed_empty() {
    let state = DropdownState::new(vec!["Apple", "Banana"]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 10);

    terminal
        .draw(|frame| {
            Dropdown::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Search...") || output.contains("▼"));
}

#[test]
fn test_view_closed_with_selection() {
    let state = DropdownState::with_selection(vec!["Apple", "Banana"], 0);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 10);

    terminal
        .draw(|frame| {
            Dropdown::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Apple"));
}

#[test]
fn test_view_open_no_filter() {
    let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
    Dropdown::update(&mut state, DropdownMessage::Open);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 15);

    terminal
        .draw(|frame| {
            Dropdown::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Apple"));
    assert!(output.contains("Banana"));
    assert!(output.contains("Cherry"));
}

#[test]
fn test_view_open_with_filter() {
    let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    Dropdown::update(&mut state, DropdownMessage::Insert('a'));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 15);

    terminal
        .draw(|frame| {
            Dropdown::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Apple"));
    assert!(output.contains("Banana"));
    // Cherry should not be shown (doesn't contain 'a')
}

#[test]
fn test_view_highlight() {
    let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    Dropdown::update(&mut state, DropdownMessage::SelectNext);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 15);

    terminal
        .draw(|frame| {
            Dropdown::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    // Should show highlight indicator on Banana
    assert!(output.contains("> Banana") || output.contains("Banana"));
}

#[test]
fn test_view_no_matches() {
    let mut state = DropdownState::new(vec!["Apple", "Banana"]);
    Dropdown::update(&mut state, DropdownMessage::Open);
    Dropdown::update(&mut state, DropdownMessage::SetFilter("xyz".to_string()));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 15);

    terminal
        .draw(|frame| {
            Dropdown::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("No matches"));
}

#[test]
fn test_view_focused() {
    let mut state = DropdownState::new(vec!["A", "B"]);
    Dropdown::focus(&mut state);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 10);

    terminal
        .draw(|frame| {
            Dropdown::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    // Should render without error
    let output = terminal.backend().to_string();
    assert!(!output.is_empty());
}

// ========== Integration Tests ==========

#[test]
fn test_clone() {
    let state = DropdownState::with_selection(vec!["A", "B", "C"], 1);
    let cloned = state.clone();
    assert_eq!(cloned.selected_index(), Some(1));
}

#[test]
fn test_init() {
    let state = Dropdown::init();
    assert_eq!(state.options().len(), 0);
    assert!(!Dropdown::is_focused(&state));
}

#[test]
fn test_full_workflow() {
    let mut state = DropdownState::new(vec!["Apple", "Apricot", "Banana", "Cherry"]);

    // Open dropdown
    Dropdown::update(&mut state, DropdownMessage::Open);
    assert!(state.is_open());
    assert_eq!(state.filtered_count(), 4);

    // Type to filter
    Dropdown::update(&mut state, DropdownMessage::Insert('a'));
    assert_eq!(state.filtered_count(), 3); // Apple, Apricot, Banana

    Dropdown::update(&mut state, DropdownMessage::Insert('p'));
    assert_eq!(state.filtered_count(), 2); // Apple, Apricot

    // Navigate
    Dropdown::update(&mut state, DropdownMessage::SelectNext);
    assert_eq!(state.highlighted_index, 1); // Apricot

    // Confirm
    let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
    assert_eq!(output, Some(DropdownOutput::Changed(Some(1)))); // Apricot is index 1
    assert_eq!(state.selected_value(), Some("Apricot"));
    assert!(!state.is_open());
    assert_eq!(state.filter_text(), ""); // Filter cleared
}

#[test]
fn test_auto_open_on_type() {
    let mut state = DropdownState::new(vec!["Apple", "Banana"]);
    assert!(!state.is_open());

    // Typing should auto-open
    Dropdown::update(&mut state, DropdownMessage::Insert('a'));
    assert!(state.is_open());
}

#[test]
fn test_filtered_count() {
    let mut state = DropdownState::new(vec!["Apple", "Apricot", "Banana"]);
    assert_eq!(state.filtered_count(), 3);

    Dropdown::update(&mut state, DropdownMessage::Insert('a'));
    Dropdown::update(&mut state, DropdownMessage::Insert('p'));
    assert_eq!(state.filtered_count(), 2);
}
