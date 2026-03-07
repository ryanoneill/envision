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

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_default_empty() {
    let state = SearchableListState::<String>::new(vec![]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_populated() {
    let state = SearchableListState::new(sample_items());
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused_filter() {
    let mut state = SearchableListState::new(sample_items());
    SearchableList::set_focused(&mut state, true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused_list() {
    let mut state = SearchableListState::new(sample_items());
    SearchableList::set_focused(&mut state, true);
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_filtered() {
    let mut state = SearchableListState::new(sample_items());
    SearchableList::set_focused(&mut state, true);
    SearchableList::update(&mut state, SearchableListMessage::FilterChar('a'));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_no_matches() {
    let mut state = SearchableListState::new(sample_items());
    SearchableList::set_focused(&mut state, true);
    SearchableList::update(
        &mut state,
        SearchableListMessage::FilterChanged("xyz".into()),
    );
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_custom_placeholder() {
    let state = SearchableListState::new(sample_items()).with_placeholder("Search fruits...");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            SearchableList::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
