use super::*;
use crate::backend::CaptureBackend;
use ratatui::Terminal;

#[derive(Clone, Debug, PartialEq)]
struct TestItem {
    id: u32,
    name: String,
}

fn make_items() -> Vec<TestItem> {
    vec![
        TestItem {
            id: 1,
            name: "Item One".to_string(),
        },
        TestItem {
            id: 2,
            name: "Item Two".to_string(),
        },
        TestItem {
            id: 3,
            name: "Item Three".to_string(),
        },
    ]
}

// ========================================
// ItemState Tests
// ========================================

#[test]
fn test_item_state_default() {
    let state = ItemState::default();
    assert!(state.is_ready());
}

#[test]
fn test_item_state_is_loading() {
    let state = ItemState::Loading;
    assert!(state.is_loading());
    assert!(!state.is_ready());
    assert!(!state.is_error());
}

#[test]
fn test_item_state_is_error() {
    let state = ItemState::Error("Test error".to_string());
    assert!(state.is_error());
    assert!(!state.is_loading());
    assert!(!state.is_ready());
}

#[test]
fn test_item_state_error_message() {
    let state = ItemState::Error("Test error".to_string());
    assert_eq!(state.error_message(), Some("Test error"));

    let ready = ItemState::Ready;
    assert!(ready.error_message().is_none());
}

#[test]
fn test_item_state_symbols() {
    assert_eq!(ItemState::Ready.symbol(0), " ");
    assert_eq!(ItemState::Error("".to_string()).symbol(0), "✗");
    // Loading has animated symbols
    assert!(!ItemState::Loading.symbol(0).is_empty());
}

#[test]
fn test_item_state_styles() {
    let theme = Theme::default();
    assert_eq!(ItemState::Ready.style(&theme), theme.normal_style());
    assert_eq!(ItemState::Loading.style(&theme), theme.warning_style());
    assert_eq!(
        ItemState::Error("".to_string()).style(&theme),
        theme.error_style()
    );
}

// ========================================
// LoadingListItem Tests
// ========================================

#[test]
fn test_list_item_new() {
    let item = LoadingListItem::new("data", "Label");
    assert_eq!(item.data(), &"data");
    assert_eq!(item.label(), "Label");
    assert!(item.is_ready());
}

#[test]
fn test_list_item_set_label() {
    let mut item = LoadingListItem::new("data", "Old");
    item.set_label("New");
    assert_eq!(item.label(), "New");
}

#[test]
fn test_list_item_set_state() {
    let mut item = LoadingListItem::new("data", "Label");
    item.set_state(ItemState::Loading);
    assert!(item.is_loading());

    item.set_state(ItemState::Error("err".to_string()));
    assert!(item.is_error());
}

#[test]
fn test_list_item_data_mut() {
    let mut item = LoadingListItem::new("original", "Label");
    *item.data_mut() = "modified";
    assert_eq!(item.data(), &"modified");
}

// ========================================
// LoadingListState Tests
// ========================================

#[test]
fn test_state_new() {
    let state: LoadingListState<String> = LoadingListState::new();
    assert!(state.is_empty());
    assert!(state.selected().is_none());
    assert!(state.show_indicators());
}

#[test]
fn test_state_with_items() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());

    assert_eq!(state.len(), 3);
    assert_eq!(state.items()[0].label(), "Item One");
    assert_eq!(state.items()[1].label(), "Item Two");
    assert_eq!(state.items()[2].label(), "Item Three");
}

#[test]
fn test_state_with_title() {
    let state: LoadingListState<String> = LoadingListState::new().with_title("My List");
    assert_eq!(state.title(), Some("My List"));
}

#[test]
fn test_state_with_indicators() {
    let state: LoadingListState<String> = LoadingListState::new().with_indicators(false);
    assert!(!state.show_indicators());
}

#[test]
fn test_state_set_loading() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_loading(0);
    assert!(state.items()[0].is_loading());
}

#[test]
fn test_state_set_ready() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_loading(0);
    state.set_ready(0);
    assert!(state.items()[0].is_ready());
}

#[test]
fn test_state_set_error() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_error(0, "Failed to load");
    assert!(state.items()[0].is_error());
    assert_eq!(
        state.items()[0].state().error_message(),
        Some("Failed to load")
    );
}

#[test]
fn test_state_counts() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_loading(0);
    state.set_loading(1);
    state.set_error(2, "Error");

    assert_eq!(state.loading_count(), 2);
    assert_eq!(state.error_count(), 1);
    assert!(state.has_loading());
    assert!(state.has_errors());
}

#[test]
fn test_state_selected() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_selected(Some(1));
    assert_eq!(state.selected(), Some(1));
    assert_eq!(state.selected_item().unwrap().label(), "Item Two");
    assert_eq!(state.selected_data().unwrap().id, 2);
}

#[test]
fn test_state_selected_clamped() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_selected(Some(100)); // Too high
    assert_eq!(state.selected(), Some(2)); // Clamped to last
}

#[test]
fn test_state_get() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());

    assert!(state.get(0).is_some());
    assert!(state.get(100).is_none());
}

#[test]
fn test_state_clear() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(0));

    state.clear();
    assert!(state.is_empty());
    assert!(state.selected().is_none());
}

// ========================================
// Component Tests
// ========================================

#[test]
fn test_init() {
    let state: LoadingListState<String> = LoadingList::init();
    assert!(state.is_empty());
}

#[test]
fn test_update_set_loading() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    let output = LoadingList::update(&mut state, LoadingListMessage::SetLoading(0));

    assert!(matches!(
        output,
        Some(LoadingListOutput::ItemStateChanged {
            index: 0,
            state: ItemState::Loading
        })
    ));
    assert!(state.items()[0].is_loading());
}

#[test]
fn test_update_set_ready() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_loading(0);

    let output = LoadingList::update(&mut state, LoadingListMessage::SetReady(0));

    assert!(matches!(
        output,
        Some(LoadingListOutput::ItemStateChanged {
            index: 0,
            state: ItemState::Ready
        })
    ));
}

#[test]
fn test_update_set_error() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    let output = LoadingList::update(
        &mut state,
        LoadingListMessage::SetError {
            index: 0,
            message: "Failed".to_string(),
        },
    );

    assert!(matches!(
        output,
        Some(LoadingListOutput::ItemStateChanged {
            index: 0,
            state: ItemState::Error(_)
        })
    ));
}

#[test]
fn test_update_clear_error() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_error(0, "Error");

    let output = LoadingList::update(&mut state, LoadingListMessage::ClearError(0));

    assert!(matches!(
        output,
        Some(LoadingListOutput::ItemStateChanged {
            index: 0,
            state: ItemState::Ready
        })
    ));
    assert!(state.items()[0].is_ready());
}

#[test]
fn test_update_clear_error_not_error() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    let output = LoadingList::update(&mut state, LoadingListMessage::ClearError(0));
    assert!(output.is_none()); // Was already ready
}

// ========================================
// Navigation Tests
// ========================================

#[test]
fn test_update_down() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    LoadingList::update(&mut state, LoadingListMessage::Down);
    assert_eq!(state.selected(), Some(0));

    LoadingList::update(&mut state, LoadingListMessage::Down);
    assert_eq!(state.selected(), Some(1));
}

#[test]
fn test_update_down_wrap() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(2)); // Last item

    LoadingList::update(&mut state, LoadingListMessage::Down);
    assert_eq!(state.selected(), Some(0)); // Wraps
}

#[test]
fn test_update_up() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(2));

    LoadingList::update(&mut state, LoadingListMessage::Up);
    assert_eq!(state.selected(), Some(1));
}

#[test]
fn test_update_up_wrap() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(0));

    LoadingList::update(&mut state, LoadingListMessage::Up);
    assert_eq!(state.selected(), Some(2)); // Wraps
}

#[test]
fn test_update_first() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(2));

    LoadingList::update(&mut state, LoadingListMessage::First);
    assert_eq!(state.selected(), Some(0));
}

#[test]
fn test_update_last() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    LoadingList::update(&mut state, LoadingListMessage::Last);
    assert_eq!(state.selected(), Some(2));
}

#[test]
fn test_update_select() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items.clone(), |i| i.name.clone());
    state.set_selected(Some(1));

    let output = LoadingList::update(&mut state, LoadingListMessage::Select);

    assert!(matches!(output, Some(LoadingListOutput::Selected(item)) if item.id == 2));
}

#[test]
fn test_update_select_nothing() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    let output = LoadingList::update(&mut state, LoadingListMessage::Select);
    assert!(output.is_none()); // Nothing selected
}

#[test]
fn test_update_tick() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    assert_eq!(state.spinner_frame(), 0);
    LoadingList::update(&mut state, LoadingListMessage::Tick);
    assert_eq!(state.spinner_frame(), 1);
}

#[test]
fn test_navigation_empty_list() {
    let mut state: LoadingListState<TestItem> = LoadingListState::new();

    let output = LoadingList::update(&mut state, LoadingListMessage::Down);
    assert!(output.is_none());

    let output = LoadingList::update(&mut state, LoadingListMessage::Up);
    assert!(output.is_none());
}

// ========================================
// Focusable Tests
// ========================================

#[test]
fn test_focusable() {
    let mut state: LoadingListState<String> = LoadingListState::new();
    assert!(!LoadingList::is_focused(&state));

    LoadingList::focus(&mut state);
    assert!(LoadingList::is_focused(&state));

    LoadingList::blur(&mut state);
    assert!(!LoadingList::is_focused(&state));
}

// ========================================
// View Tests
// ========================================

#[test]
fn test_view_empty() {
    let state: LoadingListState<String> = LoadingListState::new();
    let backend = CaptureBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| LoadingList::view(&state, frame, frame.area(), &Theme::default()))
        .unwrap();

    // Should render border only
    let output = terminal.backend().to_string();
    assert!(output.contains("─") || output.contains("│"));
}

#[test]
fn test_view_with_items() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(1));

    let backend = CaptureBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| LoadingList::view(&state, frame, frame.area(), &Theme::default()))
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Item One"));
    assert!(output.contains("Item Two"));
    assert!(output.contains("Item Three"));
    assert!(output.contains("▸")); // Selection marker
}

#[test]
fn test_view_with_title() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone()).with_title("My Items");

    let backend = CaptureBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| LoadingList::view(&state, frame, frame.area(), &Theme::default()))
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("My Items"));
}

#[test]
fn test_view_with_error() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_error(0, "Connection failed");

    let backend = CaptureBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| LoadingList::view(&state, frame, frame.area(), &Theme::default()))
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Error"));
    assert!(output.contains("Connection failed"));
}

#[test]
fn test_clone() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(1));

    let cloned = state.clone();
    assert_eq!(cloned.len(), 3);
    assert_eq!(cloned.selected(), Some(1));
}

// ========================================
// Additional Coverage Tests
// ========================================

#[test]
fn test_view_zero_size_area() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    let backend = CaptureBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    // Test with zero width
    terminal
        .draw(|frame| {
            LoadingList::view(&state, frame, Rect::new(0, 0, 0, 10), &Theme::default());
        })
        .unwrap();

    // Test with zero height
    terminal
        .draw(|frame| {
            LoadingList::view(&state, frame, Rect::new(0, 0, 60, 0), &Theme::default());
        })
        .unwrap();
}

#[test]
fn test_view_without_indicators() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone()).with_indicators(false);
    state.set_selected(Some(0));

    let backend = CaptureBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| LoadingList::view(&state, frame, frame.area(), &Theme::default()))
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Item One"));
}

#[test]
fn test_view_without_indicators_with_error() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone()).with_indicators(false);
    state.set_error(0, "Failed");

    let backend = CaptureBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| LoadingList::view(&state, frame, frame.area(), &Theme::default()))
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Error"));
    assert!(output.contains("Failed"));
}

#[test]
fn test_update_set_items() {
    let mut state: LoadingListState<TestItem> = LoadingListState::new();
    state.set_selected(Some(0));

    let items = make_items();
    LoadingList::update(&mut state, LoadingListMessage::SetItems(items));

    assert_eq!(state.len(), 3);
    assert!(state.selected().is_none()); // Selection cleared
    assert_eq!(state.items()[0].label(), "Item 1"); // Uses default labeling
}

#[test]
fn test_update_invalid_index() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    // SetLoading with invalid index
    let output = LoadingList::update(&mut state, LoadingListMessage::SetLoading(100));
    assert!(output.is_none());

    // SetReady with invalid index
    let output = LoadingList::update(&mut state, LoadingListMessage::SetReady(100));
    assert!(output.is_none());

    // SetError with invalid index
    let output = LoadingList::update(
        &mut state,
        LoadingListMessage::SetError {
            index: 100,
            message: "Error".to_string(),
        },
    );
    assert!(output.is_none());

    // ClearError with invalid index
    let output = LoadingList::update(&mut state, LoadingListMessage::ClearError(100));
    assert!(output.is_none());
}

#[test]
fn test_up_no_selection() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    // No selection set

    let output = LoadingList::update(&mut state, LoadingListMessage::Up);
    assert_eq!(state.selected(), Some(2)); // Goes to last item
    assert!(matches!(
        output,
        Some(LoadingListOutput::SelectionChanged(2))
    ));
}

#[test]
fn test_down_no_selection() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    // No selection set

    let output = LoadingList::update(&mut state, LoadingListMessage::Down);
    assert_eq!(state.selected(), Some(0)); // Goes to first item
    assert!(matches!(
        output,
        Some(LoadingListOutput::SelectionChanged(0))
    ));
}

#[test]
fn test_first_empty_list() {
    let mut state: LoadingListState<TestItem> = LoadingListState::new();

    let output = LoadingList::update(&mut state, LoadingListMessage::First);
    assert!(output.is_none());
}

#[test]
fn test_last_empty_list() {
    let mut state: LoadingListState<TestItem> = LoadingListState::new();

    let output = LoadingList::update(&mut state, LoadingListMessage::Last);
    assert!(output.is_none());
}

#[test]
fn test_get_mut() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    if let Some(item) = state.get_mut(0) {
        item.set_label("Modified");
    }
    assert_eq!(state.items()[0].label(), "Modified");

    assert!(state.get_mut(100).is_none());
}

#[test]
fn test_items_mut() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.items_mut().push(LoadingListItem::new(
        TestItem {
            id: 4,
            name: "Item Four".to_string(),
        },
        "Item Four",
    ));

    assert_eq!(state.len(), 4);
}

#[test]
fn test_set_title() {
    let mut state: LoadingListState<String> = LoadingListState::new();
    assert!(state.title().is_none());

    state.set_title(Some("New Title".to_string()));
    assert_eq!(state.title(), Some("New Title"));

    state.set_title(None);
    assert!(state.title().is_none());
}

#[test]
fn test_set_show_indicators() {
    let mut state: LoadingListState<String> = LoadingListState::new();
    assert!(state.show_indicators());

    state.set_show_indicators(false);
    assert!(!state.show_indicators());
}

#[test]
fn test_set_loading_invalid_index() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    // This should not panic
    state.set_loading(100);
    state.set_ready(100);
    state.set_error(100, "Error");
}

#[test]
fn test_spinner_animation_frames() {
    let state = ItemState::Loading;
    // Test all 4 spinner frames
    let frame0 = state.symbol(0);
    let frame1 = state.symbol(1);
    let frame2 = state.symbol(2);
    let frame3 = state.symbol(3);
    let frame4 = state.symbol(4); // Should wrap to frame 0

    assert_eq!(frame0, frame4);
    assert_ne!(frame0, frame1);
    assert_ne!(frame1, frame2);
    assert_ne!(frame2, frame3);
}
