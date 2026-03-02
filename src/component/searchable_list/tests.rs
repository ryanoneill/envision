use super::*;
use crate::component::test_utils;

fn sample_items() -> Vec<String> {
    vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
        "Date".to_string(),
        "Elderberry".to_string(),
    ]
}

fn focused_state() -> SearchableListState<String> {
    let mut state = SearchableListState::new(sample_items());
    SearchableList::set_focused(&mut state, true);
    state
}

// =============================================================================
// Construction and defaults
// =============================================================================

#[test]
fn test_new_creates_state_with_all_items_visible() {
    let state = SearchableListState::new(sample_items());
    assert_eq!(state.items().len(), 5);
    assert_eq!(state.filtered_items().len(), 5);
    assert_eq!(state.filtered_count(), 5);
}

#[test]
fn test_new_selects_first_item() {
    let state = SearchableListState::new(sample_items());
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_item(), Some(&"Apple".to_string()));
}

#[test]
fn test_new_empty_list_has_no_selection() {
    let state = SearchableListState::<String>::new(vec![]);
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.selected_item(), None);
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
}

#[test]
fn test_default_state() {
    let state = SearchableListState::<String>::default();
    assert!(state.items().is_empty());
    assert!(state.filtered_items().is_empty());
    assert_eq!(state.filter_text(), "");
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert!(state.is_filter_focused());
}

#[test]
fn test_default_placeholder() {
    let state = SearchableListState::<String>::new(vec![]);
    assert_eq!(state.placeholder(), "Type to filter...");
}

// =============================================================================
// Builder methods
// =============================================================================

#[test]
fn test_with_placeholder() {
    let state = SearchableListState::<String>::new(vec![])
        .with_placeholder("Search...");
    assert_eq!(state.placeholder(), "Search...");
}

#[test]
fn test_set_placeholder() {
    let mut state = SearchableListState::<String>::new(vec![]);
    state.set_placeholder("Find items");
    assert_eq!(state.placeholder(), "Find items");
}

#[test]
fn test_with_disabled() {
    let state = SearchableListState::<String>::new(vec![])
        .with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// Focus management
// =============================================================================

#[test]
fn test_initial_internal_focus_is_filter() {
    let state = SearchableListState::new(sample_items());
    assert!(state.is_filter_focused());
    assert!(!state.is_list_focused());
}

#[test]
fn test_toggle_focus_switches_to_list() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);
    assert!(state.is_list_focused());
    assert!(!state.is_filter_focused());
}

#[test]
fn test_toggle_focus_switches_back_to_filter() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);
    assert!(state.is_filter_focused());
}

#[test]
fn test_set_focused_and_is_focused() {
    let mut state = SearchableListState::new(sample_items());
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
    state.set_focused(false);
    assert!(!state.is_focused());
}

// =============================================================================
// Filtering
// =============================================================================

#[test]
fn test_filter_narrows_items() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("an".into()));
    assert_eq!(state.filtered_count(), 1); // "Banana"
    assert_eq!(state.filtered_items(), vec![&"Banana".to_string()]);
}

#[test]
fn test_filter_is_case_insensitive() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("APPLE".into()));
    assert_eq!(state.filtered_count(), 1);
    assert_eq!(state.filtered_items(), vec![&"Apple".to_string()]);
}

#[test]
fn test_filter_matches_substring() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("err".into()));
    assert_eq!(state.filtered_count(), 2); // Cherry, Elderberry
}

#[test]
fn test_filter_no_matches() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("xyz".into()));
    assert_eq!(state.filtered_count(), 0);
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.selected_item(), None);
}

#[test]
fn test_filter_changed_resets_selection_to_first() {
    let mut state = focused_state();
    // Move to third item
    SearchableList::update(&mut state, SearchableListMessage::Down);
    SearchableList::update(&mut state, SearchableListMessage::Down);
    assert_eq!(state.selected_index(), Some(2));

    // Filter resets selection to 0
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("a".into()));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_filter_changed_returns_output() {
    let mut state = focused_state();
    let output = SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("test".into()),
    );
    assert_eq!(
        output,
        Some(SearchableListOutput::FilterChanged("test".into()))
    );
}

#[test]
fn test_empty_filter_shows_all_items() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("app".into()));
    assert_eq!(state.filtered_count(), 1);

    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("".into()));
    assert_eq!(state.filtered_count(), 5);
}

// =============================================================================
// Filter char-by-char input
// =============================================================================

#[test]
fn test_filter_char_appends_to_filter() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::FilterChar('b'));
    assert_eq!(state.filter_text(), "b");
    SearchableList::update(&mut state, SearchableListMessage::FilterChar('a'));
    assert_eq!(state.filter_text(), "ba");
    assert_eq!(state.filtered_count(), 1); // "Banana"
}

#[test]
fn test_filter_char_returns_filter_changed_output() {
    let mut state = focused_state();
    let output = SearchableList::update(&mut state, SearchableListMessage::FilterChar('x'));
    assert_eq!(
        output,
        Some(SearchableListOutput::FilterChanged("x".into()))
    );
}

#[test]
fn test_filter_char_from_list_switches_focus_to_filter() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);
    assert!(state.is_list_focused());

    SearchableList::update(&mut state, SearchableListMessage::FilterChar('a'));
    assert!(state.is_filter_focused());
    assert_eq!(state.filter_text(), "a");
}

// =============================================================================
// Filter backspace and clear
// =============================================================================

#[test]
fn test_filter_backspace_removes_last_char() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("abc".into()));
    SearchableList::update(&mut state, SearchableListMessage::FilterBackspace);
    assert_eq!(state.filter_text(), "ab");
}

#[test]
fn test_filter_backspace_on_empty_returns_none() {
    let mut state = focused_state();
    let output = SearchableList::update(&mut state, SearchableListMessage::FilterBackspace);
    assert_eq!(output, None);
}

#[test]
fn test_filter_clear_empties_filter() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("abc".into()));
    let output = SearchableList::update(&mut state, SearchableListMessage::FilterClear);
    assert_eq!(state.filter_text(), "");
    assert_eq!(state.filtered_count(), 5);
    assert_eq!(
        output,
        Some(SearchableListOutput::FilterChanged(String::new()))
    );
}

#[test]
fn test_filter_clear_on_empty_returns_none() {
    let mut state = focused_state();
    let output = SearchableList::update(&mut state, SearchableListMessage::FilterClear);
    assert_eq!(output, None);
}

// =============================================================================
// List navigation
// =============================================================================

#[test]
fn test_down_moves_selection() {
    let mut state = focused_state();
    let output = SearchableList::update(&mut state, SearchableListMessage::Down);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(output, Some(SearchableListOutput::SelectionChanged(1)));
}

#[test]
fn test_up_moves_selection() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::Down);
    SearchableList::update(&mut state, SearchableListMessage::Down);
    let output = SearchableList::update(&mut state, SearchableListMessage::Up);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(output, Some(SearchableListOutput::SelectionChanged(1)));
}

#[test]
fn test_up_at_top_returns_none() {
    let mut state = focused_state();
    let output = SearchableList::update(&mut state, SearchableListMessage::Up);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, None);
}

#[test]
fn test_down_at_bottom_returns_none() {
    let mut state = focused_state();
    for _ in 0..4 {
        SearchableList::update(&mut state, SearchableListMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(4));
    let output = SearchableList::update(&mut state, SearchableListMessage::Down);
    assert_eq!(state.selected_index(), Some(4));
    assert_eq!(output, None);
}

#[test]
fn test_first_jumps_to_top() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::Down);
    SearchableList::update(&mut state, SearchableListMessage::Down);
    let output = SearchableList::update(&mut state, SearchableListMessage::First);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, Some(SearchableListOutput::SelectionChanged(0)));
}

#[test]
fn test_first_at_top_returns_none() {
    let mut state = focused_state();
    let output = SearchableList::update(&mut state, SearchableListMessage::First);
    assert_eq!(output, None);
}

#[test]
fn test_last_jumps_to_bottom() {
    let mut state = focused_state();
    let output = SearchableList::update(&mut state, SearchableListMessage::Last);
    assert_eq!(state.selected_index(), Some(4));
    assert_eq!(output, Some(SearchableListOutput::SelectionChanged(4)));
}

#[test]
fn test_last_at_bottom_returns_none() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::Last);
    let output = SearchableList::update(&mut state, SearchableListMessage::Last);
    assert_eq!(output, None);
}

#[test]
fn test_page_up() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::Last);
    let output = SearchableList::update(&mut state, SearchableListMessage::PageUp(3));
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(output, Some(SearchableListOutput::SelectionChanged(1)));
}

#[test]
fn test_page_up_clamps_to_zero() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::Down);
    let output = SearchableList::update(&mut state, SearchableListMessage::PageUp(10));
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, Some(SearchableListOutput::SelectionChanged(0)));
}

#[test]
fn test_page_down() {
    let mut state = focused_state();
    let output = SearchableList::update(&mut state, SearchableListMessage::PageDown(3));
    assert_eq!(state.selected_index(), Some(3));
    assert_eq!(output, Some(SearchableListOutput::SelectionChanged(3)));
}

#[test]
fn test_page_down_clamps_to_last() {
    let mut state = focused_state();
    let output = SearchableList::update(&mut state, SearchableListMessage::PageDown(100));
    assert_eq!(state.selected_index(), Some(4));
    assert_eq!(output, Some(SearchableListOutput::SelectionChanged(4)));
}

// =============================================================================
// Selection
// =============================================================================

#[test]
fn test_select_returns_selected_item() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::Down);
    let output = SearchableList::update(&mut state, SearchableListMessage::Select);
    assert_eq!(
        output,
        Some(SearchableListOutput::Selected("Banana".to_string()))
    );
}

#[test]
fn test_select_with_filter_returns_correct_item() {
    let mut state = focused_state();
    // Filter to show only "Cherry" and "Elderberry"
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("err".into()));
    assert_eq!(state.filtered_count(), 2);

    // Select first filtered item (Cherry)
    let output = SearchableList::update(&mut state, SearchableListMessage::Select);
    assert_eq!(
        output,
        Some(SearchableListOutput::Selected("Cherry".to_string()))
    );
}

#[test]
fn test_select_with_filter_second_item() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("err".into()));
    SearchableList::update(&mut state, SearchableListMessage::Down);
    let output = SearchableList::update(&mut state, SearchableListMessage::Select);
    assert_eq!(
        output,
        Some(SearchableListOutput::Selected("Elderberry".to_string()))
    );
}

#[test]
fn test_select_on_empty_filtered_list_returns_none() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("xyz".into()));
    let output = SearchableList::update(&mut state, SearchableListMessage::Select);
    assert_eq!(output, None);
}

#[test]
fn test_select_on_empty_list_returns_none() {
    let mut state = SearchableListState::<String>::new(vec![]);
    SearchableList::set_focused(&mut state, true);
    let output = SearchableList::update(&mut state, SearchableListMessage::Select);
    assert_eq!(output, None);
}

// =============================================================================
// Navigation on filtered list
// =============================================================================

#[test]
fn test_navigation_respects_filtered_bounds() {
    let mut state = focused_state();
    // Filter to 2 items
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("err".into()));
    assert_eq!(state.filtered_count(), 2);

    // Down should work
    let output = SearchableList::update(&mut state, SearchableListMessage::Down);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(output, Some(SearchableListOutput::SelectionChanged(1)));

    // Down at end should not move
    let output = SearchableList::update(&mut state, SearchableListMessage::Down);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(output, None);
}

// =============================================================================
// Disabled state
// =============================================================================

#[test]
fn test_disabled_ignores_all_messages() {
    let mut state = focused_state();
    state.set_disabled(true);

    let output = SearchableList::update(&mut state, SearchableListMessage::Down);
    assert_eq!(output, None);

    let output = SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("a".into()),
    );
    assert_eq!(output, None);

    let output = SearchableList::update(&mut state, SearchableListMessage::Select);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_state();
    state.set_disabled(true);

    let msg = SearchableList::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, None);
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = SearchableListState::new(sample_items());
    assert!(!state.is_focused());

    let msg = SearchableList::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_tab_maps_to_toggle_focus() {
    let state = focused_state();
    let msg = SearchableList::handle_event(&state, &Event::key(KeyCode::Tab));
    assert_eq!(msg, Some(SearchableListMessage::ToggleFocus));
}

#[test]
fn test_backtab_maps_to_toggle_focus() {
    let state = focused_state();
    let msg = SearchableList::handle_event(&state, &Event::key(KeyCode::BackTab));
    assert_eq!(msg, Some(SearchableListMessage::ToggleFocus));
}

#[test]
fn test_esc_maps_to_filter_clear() {
    let state = focused_state();
    let msg = SearchableList::handle_event(&state, &Event::key(KeyCode::Esc));
    assert_eq!(msg, Some(SearchableListMessage::FilterClear));
}

#[test]
fn test_char_in_filter_mode_maps_to_filter_char() {
    let state = focused_state();
    assert!(state.is_filter_focused());
    let msg = SearchableList::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, Some(SearchableListMessage::FilterChar('a')));
}

#[test]
fn test_enter_in_filter_mode_maps_to_toggle_focus() {
    let state = focused_state();
    assert!(state.is_filter_focused());
    let msg = SearchableList::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(SearchableListMessage::ToggleFocus));
}

#[test]
fn test_backspace_in_filter_mode_maps_to_filter_backspace() {
    let state = focused_state();
    let msg = SearchableList::handle_event(&state, &Event::key(KeyCode::Backspace));
    assert_eq!(msg, Some(SearchableListMessage::FilterBackspace));
}

#[test]
fn test_ctrl_j_in_filter_maps_to_down() {
    let state = focused_state();
    let msg = SearchableList::handle_event(&state, &Event::ctrl('j'));
    assert_eq!(msg, Some(SearchableListMessage::Down));
}

#[test]
fn test_ctrl_k_in_filter_maps_to_up() {
    let state = focused_state();
    let msg = SearchableList::handle_event(&state, &Event::ctrl('k'));
    assert_eq!(msg, Some(SearchableListMessage::Up));
}

#[test]
fn test_arrow_keys_in_list_mode() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);
    assert!(state.is_list_focused());

    let msg = SearchableList::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, Some(SearchableListMessage::Up));

    let msg = SearchableList::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, Some(SearchableListMessage::Down));
}

#[test]
fn test_vim_keys_in_list_mode() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);

    let msg = SearchableList::handle_event(&state, &Event::char('k'));
    assert_eq!(msg, Some(SearchableListMessage::Up));

    let msg = SearchableList::handle_event(&state, &Event::char('j'));
    assert_eq!(msg, Some(SearchableListMessage::Down));
}

#[test]
fn test_home_end_in_list_mode() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);

    let msg = SearchableList::handle_event(&state, &Event::key(KeyCode::Home));
    assert_eq!(msg, Some(SearchableListMessage::First));

    let msg = SearchableList::handle_event(&state, &Event::key(KeyCode::End));
    assert_eq!(msg, Some(SearchableListMessage::Last));
}

#[test]
fn test_g_and_shift_g_in_list_mode() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);

    let msg = SearchableList::handle_event(&state, &Event::char('g'));
    assert_eq!(msg, Some(SearchableListMessage::First));

    let msg = SearchableList::handle_event(&state, &Event::char('G'));
    assert_eq!(msg, Some(SearchableListMessage::Last));
}

#[test]
fn test_page_keys_in_list_mode() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);

    let msg = SearchableList::handle_event(&state, &Event::key(KeyCode::PageUp));
    assert_eq!(msg, Some(SearchableListMessage::PageUp(10)));

    let msg = SearchableList::handle_event(&state, &Event::key(KeyCode::PageDown));
    assert_eq!(msg, Some(SearchableListMessage::PageDown(10)));
}

#[test]
fn test_enter_in_list_mode_maps_to_select() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);

    let msg = SearchableList::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(SearchableListMessage::Select));
}

#[test]
fn test_char_in_list_mode_maps_to_filter_char() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);
    assert!(state.is_list_focused());

    // Typing in list mode should redirect to filter
    let msg = SearchableList::handle_event(&state, &Event::char('x'));
    assert_eq!(msg, Some(SearchableListMessage::FilterChar('x')));
}

// =============================================================================
// dispatch_event
// =============================================================================

#[test]
fn test_dispatch_event_filters_and_selects() {
    let mut state = focused_state();

    // Type a character via dispatch_event
    let output = state.dispatch_event(&Event::char('b'));
    assert_eq!(
        output,
        Some(SearchableListOutput::FilterChanged("b".into()))
    );
    assert_eq!(state.filter_text(), "b");

    // Filtered to Banana and Elderberry
    let output = state.dispatch_event(&Event::char('a'));
    assert_eq!(
        output,
        Some(SearchableListOutput::FilterChanged("ba".into()))
    );
    // Only Banana matches "ba"
    assert_eq!(state.filtered_count(), 1);
}

// =============================================================================
// set_items
// =============================================================================

#[test]
fn test_set_items_refilters() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("a".into()));
    let count_before = state.filtered_count();

    state.set_items(vec![
        "Avocado".to_string(),
        "Artichoke".to_string(),
        "Zucchini".to_string(),
    ]);
    // "a" now matches Avocado and Artichoke
    assert_eq!(state.filtered_count(), 2);
    assert_ne!(count_before, state.filtered_count());
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_state();
    let msg = state.handle_event(&Event::char('x'));
    assert_eq!(msg, Some(SearchableListMessage::FilterChar('x')));
}

#[test]
fn test_instance_update() {
    let mut state = focused_state();
    let output = state.update(SearchableListMessage::Down);
    assert_eq!(output, Some(SearchableListOutput::SelectionChanged(1)));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_state();
    let output = state.dispatch_event(&Event::char('a'));
    assert!(matches!(output, Some(SearchableListOutput::FilterChanged(_))));
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_unfocused() {
    let state = SearchableListState::new(sample_items());
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_focused_filter() {
    let mut state = SearchableListState::new(sample_items());
    SearchableList::set_focused(&mut state, true);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_focused_list() {
    let mut state = SearchableListState::new(sample_items());
    SearchableList::set_focused(&mut state, true);
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_filter() {
    let mut state = SearchableListState::new(sample_items());
    SearchableList::set_focused(&mut state, true);
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("an".into()));
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let mut state = SearchableListState::new(sample_items());
    state.set_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_empty_list() {
    let state = SearchableListState::<String>::new(vec![]);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// =============================================================================
// Focusable trait
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = SearchableList::<String>::init();
    assert!(!SearchableList::is_focused(&state));

    SearchableList::focus(&mut state);
    assert!(SearchableList::is_focused(&state));

    SearchableList::blur(&mut state);
    assert!(!SearchableList::is_focused(&state));
}

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_partial_eq() {
    let state1 = SearchableListState::new(sample_items());
    let state2 = SearchableListState::new(sample_items());
    assert_eq!(state1, state2);
}

#[test]
fn test_partial_eq_different_filter() {
    let mut state1 = SearchableListState::new(sample_items());
    let state2 = SearchableListState::new(sample_items());
    SearchableList::set_focused(&mut state1, true);
    SearchableList::update(&mut state1, SearchableListMessage::FilterChanged("a".into()));
    assert_ne!(state1, state2);
}
