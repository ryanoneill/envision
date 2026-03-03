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

// ========================================
// ItemState Default Tests
// ========================================

#[test]
fn test_item_state_default_is_ready() {
    let state = ItemState::default();
    assert!(state.is_ready());
    assert!(!state.is_loading());
    assert!(!state.is_error());
}

// ========================================
// ItemState PartialEq Tests
// ========================================

#[test]
fn test_item_state_partial_eq_ready() {
    assert_eq!(ItemState::Ready, ItemState::Ready);
    assert_ne!(ItemState::Ready, ItemState::Loading);
    assert_ne!(ItemState::Ready, ItemState::Error("err".to_string()));
}

#[test]
fn test_item_state_partial_eq_loading() {
    assert_eq!(ItemState::Loading, ItemState::Loading);
    assert_ne!(ItemState::Loading, ItemState::Ready);
    assert_ne!(ItemState::Loading, ItemState::Error("err".to_string()));
}

#[test]
fn test_item_state_partial_eq_error() {
    assert_eq!(
        ItemState::Error("msg".to_string()),
        ItemState::Error("msg".to_string())
    );
    assert_ne!(
        ItemState::Error("a".to_string()),
        ItemState::Error("b".to_string())
    );
    assert_ne!(ItemState::Error("msg".to_string()), ItemState::Ready);
    assert_ne!(ItemState::Error("msg".to_string()), ItemState::Loading);
}

// ========================================
// ItemState Error Message Edge Cases
// ========================================

#[test]
fn test_item_state_error_message_loading() {
    assert!(ItemState::Loading.error_message().is_none());
}

#[test]
fn test_item_state_error_message_empty_string() {
    let state = ItemState::Error(String::new());
    assert_eq!(state.error_message(), Some(""));
}

// ========================================
// ItemState Symbol Edge Cases
// ========================================

#[test]
fn test_item_state_ready_symbol_ignores_frame() {
    // Ready symbol is always the same regardless of spinner frame
    for frame in 0..20 {
        assert_eq!(ItemState::Ready.symbol(frame), " ");
    }
}

#[test]
fn test_item_state_error_symbol_ignores_frame() {
    // Error symbol is always the same regardless of spinner frame
    let state = ItemState::Error("err".to_string());
    for frame in 0..20 {
        assert_eq!(state.symbol(frame), "✗");
    }
}

#[test]
fn test_spinner_large_frame_number_wraps() {
    let state = ItemState::Loading;
    // Large frame numbers should wrap around mod 10
    assert_eq!(state.symbol(100), state.symbol(0));
    assert_eq!(state.symbol(103), state.symbol(3));
    assert_eq!(state.symbol(999), state.symbol(9));
}

// ========================================
// LoadingListItem PartialEq Tests
// ========================================

#[test]
fn test_list_item_partial_eq_same() {
    let item_a = LoadingListItem::new("data", "Label");
    let item_b = LoadingListItem::new("data", "Label");
    assert_eq!(item_a, item_b);
}

#[test]
fn test_list_item_partial_eq_different_data() {
    let item_a = LoadingListItem::new("data_a", "Label");
    let item_b = LoadingListItem::new("data_b", "Label");
    assert_ne!(item_a, item_b);
}

#[test]
fn test_list_item_partial_eq_different_label() {
    let item_a = LoadingListItem::new("data", "Label A");
    let item_b = LoadingListItem::new("data", "Label B");
    assert_ne!(item_a, item_b);
}

#[test]
fn test_list_item_partial_eq_different_state() {
    let mut item_a = LoadingListItem::new("data", "Label");
    let item_b = LoadingListItem::new("data", "Label");

    item_a.set_state(ItemState::Loading);
    assert_ne!(item_a, item_b);
}

// ========================================
// LoadingListItem State Accessor Tests
// ========================================

#[test]
fn test_list_item_state_accessor() {
    let item = LoadingListItem::new("data", "Label");
    assert_eq!(*item.state(), ItemState::Ready);

    let mut item = LoadingListItem::new("data", "Label");
    item.set_state(ItemState::Loading);
    assert_eq!(*item.state(), ItemState::Loading);

    item.set_state(ItemState::Error("fail".to_string()));
    assert_eq!(*item.state(), ItemState::Error("fail".to_string()));
}

#[test]
fn test_list_item_is_ready_default() {
    let item = LoadingListItem::new(42, "Number");
    assert!(item.is_ready());
    assert!(!item.is_loading());
    assert!(!item.is_error());
}

#[test]
fn test_list_item_is_loading() {
    let mut item = LoadingListItem::new(42, "Number");
    item.set_state(ItemState::Loading);
    assert!(!item.is_ready());
    assert!(item.is_loading());
    assert!(!item.is_error());
}

#[test]
fn test_list_item_is_error() {
    let mut item = LoadingListItem::new(42, "Number");
    item.set_state(ItemState::Error("oops".to_string()));
    assert!(!item.is_ready());
    assert!(!item.is_loading());
    assert!(item.is_error());
}

// ========================================
// LoadingListItem with Complex Types
// ========================================

#[test]
fn test_list_item_with_struct_data() {
    let item = LoadingListItem::new(
        TestItem {
            id: 1,
            name: "Test".to_string(),
        },
        "Test",
    );
    assert_eq!(item.data().id, 1);
    assert_eq!(item.data().name, "Test");
}

#[test]
fn test_list_item_label_from_string() {
    let item = LoadingListItem::new(1, String::from("String Label"));
    assert_eq!(item.label(), "String Label");
}

#[test]
fn test_list_item_label_from_str() {
    let item = LoadingListItem::new(1, "Str Label");
    assert_eq!(item.label(), "Str Label");
}
