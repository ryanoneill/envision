use super::*;

// ========================================
// ItemState Tests
// ========================================

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

#[test]
fn test_spinner_animation_frames() {
    let state = ItemState::Loading;
    // Test all 10 spinner frames (Braille dots matching SpinnerStyle::Dots)
    let expected = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    for (i, &expected_frame) in expected.iter().enumerate() {
        assert_eq!(state.symbol(i), expected_frame);
    }
    // Frame 10 should wrap to frame 0
    assert_eq!(state.symbol(10), state.symbol(0));
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
