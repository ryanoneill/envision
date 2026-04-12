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
    SearchableListState::new(sample_items())
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
    let state = SearchableListState::<String>::new(vec![]).with_placeholder("Search...");
    assert_eq!(state.placeholder(), "Search...");
}

#[test]
fn test_set_placeholder() {
    let mut state = SearchableListState::<String>::new(vec![]);
    state.set_placeholder("Find items");
    assert_eq!(state.placeholder(), "Find items");
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

// =============================================================================
// Filtering
// =============================================================================

#[test]
fn test_filter_narrows_items() {
    let mut state = focused_state();
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("an".into()),
    );
    assert_eq!(state.filtered_count(), 1); // "Banana"
    assert_eq!(state.filtered_items(), vec![&"Banana".to_string()]);
}

#[test]
fn test_filter_is_case_insensitive() {
    let mut state = focused_state();
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("APPLE".into()),
    );
    assert_eq!(state.filtered_count(), 1);
    assert_eq!(state.filtered_items(), vec![&"Apple".to_string()]);
}

#[test]
fn test_filter_matches_substring() {
    let mut state = focused_state();
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("err".into()),
    );
    assert_eq!(state.filtered_count(), 2); // Cherry, Elderberry
}

#[test]
fn test_filter_no_matches() {
    let mut state = focused_state();
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("xyz".into()),
    );
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
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("app".into()),
    );
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
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("abc".into()),
    );
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
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("abc".into()),
    );
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
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("err".into()),
    );
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
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("err".into()),
    );
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
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("xyz".into()),
    );
    let output = SearchableList::update(&mut state, SearchableListMessage::Select);
    assert_eq!(output, None);
}

#[test]
fn test_select_on_empty_list_returns_none() {
    let mut state = SearchableListState::<String>::new(vec![]);
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
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("err".into()),
    );
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
fn test_disabled_ignores_events() {
    let state = focused_state();

    let msg = SearchableList::handle_event(
        &state,
        &Event::char('a'),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = SearchableListState::new(sample_items());

    let msg = SearchableList::handle_event(&state, &Event::char('a'), &EventContext::default());
    assert_eq!(msg, None);
}

// =============================================================================
// dispatch_event
// =============================================================================

#[test]
fn test_dispatch_event_filters_and_selects() {
    let mut state = focused_state();

    // Type a character via dispatch_event
    let output = SearchableList::dispatch_event(
        &mut state,
        &Event::char('b'),
        &EventContext::new().focused(true),
    );
    assert_eq!(
        output,
        Some(SearchableListOutput::FilterChanged("b".into()))
    );
    assert_eq!(state.filter_text(), "b");

    // Filtered to Banana and Elderberry
    let output = SearchableList::dispatch_event(
        &mut state,
        &Event::char('a'),
        &EventContext::new().focused(true),
    );
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
#[test]
fn test_instance_update() {
    let mut state = focused_state();
    let output = state.update(SearchableListMessage::Down);
    assert_eq!(output, Some(SearchableListOutput::SelectionChanged(1)));
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
            SearchableList::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_focused_filter() {
    let state = SearchableListState::new(sample_items());
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            SearchableList::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_focused_list() {
    let mut state = SearchableListState::new(sample_items());
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            SearchableList::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_with_filter() {
    let mut state = SearchableListState::new(sample_items());
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("an".into()),
    );
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            SearchableList::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = SearchableListState::new(sample_items());
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            SearchableList::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_empty_list() {
    let state = SearchableListState::<String>::new(vec![]);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
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
    SearchableList::update(
        &mut state1,
        SearchableListMessage::FilterChanged("a".into()),
    );
    assert_ne!(state1, state2);
}

// =============================================================================
// Custom matcher
// =============================================================================

#[test]
fn test_default_substring_matching_still_works() {
    let mut state = SearchableListState::new(sample_items());
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("an".into()),
    );
    assert_eq!(state.filtered_count(), 1);
    assert_eq!(state.filtered_items(), vec![&"Banana".to_string()]);
}

#[test]
fn test_custom_matcher_filters_correctly() {
    let mut state = SearchableListState::new(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Apricot".to_string(),
        "Avocado".to_string(),
    ])
    .with_matcher(|query, item| {
        let item_lower = item.to_lowercase();
        let query_lower = query.to_lowercase();
        if item_lower.starts_with(&query_lower) {
            Some(0)
        } else {
            None
        }
    });
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("ap".into()),
    );
    assert_eq!(state.filtered_count(), 2);
    let filtered: Vec<&String> = state.filtered_items();
    assert!(filtered.contains(&&"Apple".to_string()));
    assert!(filtered.contains(&&"Apricot".to_string()));
}

#[test]
fn test_scored_matcher_sorts_by_score_descending() {
    let mut state = SearchableListState::new(vec![
        "Banana".to_string(),
        "Apple".to_string(),
        "Cantaloupe".to_string(),
        "Date".to_string(),
    ])
    .with_matcher(|query, item| {
        let item_lower = item.to_lowercase();
        let query_lower = query.to_lowercase();
        item_lower.find(&query_lower).map(|pos| -(pos as i64))
    });
    SearchableList::update(&mut state, SearchableListMessage::FilterChanged("a".into()));
    assert_eq!(state.filtered_count(), 4);
    // Apple should be first (score 0), others have score -1
    let filtered = state.filtered_items();
    assert_eq!(*filtered[0], "Apple");
}

#[test]
fn test_none_scores_filter_items_out() {
    let mut state = SearchableListState::new(vec![
        "Strawberry".to_string(),
        "Apple".to_string(),
        "Blueberry".to_string(),
        "Banana".to_string(),
    ])
    .with_matcher(|_query, item| {
        if item.to_lowercase().contains("berry") {
            Some(0)
        } else {
            None
        }
    });
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("anything".into()),
    );
    assert_eq!(state.filtered_count(), 2);
    let filtered = state.filtered_items();
    assert!(filtered.contains(&&"Strawberry".to_string()));
    assert!(filtered.contains(&&"Blueberry".to_string()));
    assert!(!filtered.contains(&&"Apple".to_string()));
    assert!(!filtered.contains(&&"Banana".to_string()));
}

#[test]
fn test_custom_matcher_empty_filter_shows_all() {
    let state = SearchableListState::new(vec!["Apple".to_string(), "Banana".to_string()])
        .with_matcher(|_query, _item| None);
    assert_eq!(state.filtered_count(), 2);
    assert_eq!(state.filter_text(), "");
}

#[test]
fn test_custom_matcher_receives_original_query() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let received_uppercase = Arc::new(AtomicBool::new(false));
    let received_uppercase_clone = received_uppercase.clone();
    let mut state =
        SearchableListState::new(vec!["Test".to_string()]).with_matcher(move |query, _item| {
            if query.chars().any(|c| c.is_uppercase()) {
                received_uppercase_clone.store(true, Ordering::Relaxed);
            }
            Some(0)
        });
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("ABC".into()),
    );
    assert!(received_uppercase.load(Ordering::Relaxed));
}

// =============================================================================
// Clone preserves matcher
// =============================================================================

#[test]
fn test_clone_preserves_custom_matcher() {
    let state = SearchableListState::new(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Apricot".to_string(),
    ])
    .with_matcher(|query, item| {
        let item_lower = item.to_lowercase();
        let query_lower = query.to_lowercase();
        if item_lower.starts_with(&query_lower) {
            Some(0)
        } else {
            None
        }
    });

    let mut cloned = state.clone();
    SearchableList::update(
        &mut cloned,
        SearchableListMessage::FilterChanged("ap".into()),
    );
    // The prefix matcher should only match "Apple" and "Apricot", not "Banana"
    assert_eq!(cloned.filtered_count(), 2);
    let filtered = cloned.filtered_items();
    assert!(filtered.contains(&&"Apple".to_string()));
    assert!(filtered.contains(&&"Apricot".to_string()));
    assert!(!filtered.contains(&&"Banana".to_string()));
}

#[test]
fn test_clone_without_matcher_uses_default_substring_match() {
    let state = SearchableListState::new(vec!["Apple".to_string(), "Banana".to_string()]);

    let mut cloned = state.clone();
    SearchableList::update(
        &mut cloned,
        SearchableListMessage::FilterChanged("an".into()),
    );
    // Default substring match should find "Banana"
    assert_eq!(cloned.filtered_count(), 1);
    assert_eq!(cloned.filtered_items(), vec![&"Banana".to_string()]);
}

// =============================================================================
// Debug
// =============================================================================

#[test]
fn test_debug_with_matcher() {
    let state =
        SearchableListState::new(vec!["Apple".to_string()]).with_matcher(|_query, _item| Some(0));
    let debug_output = format!("{:?}", state);
    assert!(debug_output.contains("SearchableListState"));
    assert!(debug_output.contains("matcher"));
}

#[test]
fn test_debug_without_matcher() {
    let state = SearchableListState::new(vec!["Apple".to_string()]);
    let debug_output = format!("{:?}", state);
    assert!(debug_output.contains("SearchableListState"));
    assert!(debug_output.contains("matcher"));
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = SearchableListState::new(sample_items());
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                SearchableList::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::SearchableList);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.has_id("searchable_list"));
}

#[test]
fn searchable_list_state_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SearchableListState<String>>();
}
