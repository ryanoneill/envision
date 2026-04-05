use super::*;
use crate::input::KeyCode;

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

#[test]
fn test_tab_maps_to_toggle_focus() {
    let state = focused_state();
    let msg = SearchableList::handle_event(
        &state,
        &Event::key(KeyCode::Tab),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SearchableListMessage::ToggleFocus));
}

#[test]
fn test_backtab_maps_to_toggle_focus() {
    let state = focused_state();
    let msg = SearchableList::handle_event(
        &state,
        &Event::key(KeyCode::BackTab),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SearchableListMessage::ToggleFocus));
}

#[test]
fn test_esc_maps_to_filter_clear() {
    let state = focused_state();
    let msg = SearchableList::handle_event(
        &state,
        &Event::key(KeyCode::Esc),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SearchableListMessage::FilterClear));
}

#[test]
fn test_char_in_filter_mode_maps_to_filter_char() {
    let state = focused_state();
    assert!(state.is_filter_focused());
    let msg =
        SearchableList::handle_event(&state, &Event::char('a'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(SearchableListMessage::FilterChar('a')));
}

#[test]
fn test_enter_in_filter_mode_maps_to_toggle_focus() {
    let state = focused_state();
    assert!(state.is_filter_focused());
    let msg = SearchableList::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SearchableListMessage::ToggleFocus));
}

#[test]
fn test_backspace_in_filter_mode_maps_to_filter_backspace() {
    let state = focused_state();
    let msg = SearchableList::handle_event(
        &state,
        &Event::key(KeyCode::Backspace),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SearchableListMessage::FilterBackspace));
}

#[test]
fn test_ctrl_j_in_filter_maps_to_down() {
    let state = focused_state();
    let msg =
        SearchableList::handle_event(&state, &Event::ctrl('j'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(SearchableListMessage::Down));
}

#[test]
fn test_ctrl_k_in_filter_maps_to_up() {
    let state = focused_state();
    let msg =
        SearchableList::handle_event(&state, &Event::ctrl('k'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(SearchableListMessage::Up));
}

#[test]
fn test_arrow_keys_in_list_mode() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);
    assert!(state.is_list_focused());

    let msg = SearchableList::handle_event(
        &state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SearchableListMessage::Up));

    let msg = SearchableList::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SearchableListMessage::Down));
}

#[test]
fn test_vim_keys_in_list_mode() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);

    let msg =
        SearchableList::handle_event(&state, &Event::char('k'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(SearchableListMessage::Up));

    let msg =
        SearchableList::handle_event(&state, &Event::char('j'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(SearchableListMessage::Down));
}

#[test]
fn test_home_end_in_list_mode() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);

    let msg = SearchableList::handle_event(
        &state,
        &Event::key(KeyCode::Home),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SearchableListMessage::First));

    let msg = SearchableList::handle_event(
        &state,
        &Event::key(KeyCode::End),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SearchableListMessage::Last));
}

#[test]
fn test_g_and_shift_g_in_list_mode() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);

    let msg =
        SearchableList::handle_event(&state, &Event::char('g'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(SearchableListMessage::First));

    let msg =
        SearchableList::handle_event(&state, &Event::char('G'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(SearchableListMessage::Last));
}

#[test]
fn test_page_keys_in_list_mode() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);

    let msg = SearchableList::handle_event(
        &state,
        &Event::key(KeyCode::PageUp),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SearchableListMessage::PageUp(10)));

    let msg = SearchableList::handle_event(
        &state,
        &Event::key(KeyCode::PageDown),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SearchableListMessage::PageDown(10)));
}

#[test]
fn test_enter_in_list_mode_maps_to_select() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);

    let msg = SearchableList::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SearchableListMessage::Select));
}

#[test]
fn test_char_in_list_mode_maps_to_filter_char() {
    let mut state = focused_state();
    SearchableList::update(&mut state, SearchableListMessage::ToggleFocus);
    assert!(state.is_list_focused());

    // Typing in list mode should redirect to filter
    let msg =
        SearchableList::handle_event(&state, &Event::char('x'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(SearchableListMessage::FilterChar('x')));
}
