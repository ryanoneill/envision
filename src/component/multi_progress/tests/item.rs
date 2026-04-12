use super::*;
use crate::theme::Theme;

// ========================================
// ProgressItemStatus Tests
// ========================================

#[test]
fn test_status_default() {
    let status = ProgressItemStatus::default();
    assert_eq!(status, ProgressItemStatus::Pending);
}

#[test]
fn test_status_styles() {
    let theme = Theme::default();
    assert_eq!(
        ProgressItemStatus::Pending.style(&theme),
        theme.disabled_style()
    );
    assert_eq!(ProgressItemStatus::Active.style(&theme), theme.info_style());
    assert_eq!(
        ProgressItemStatus::Completed.style(&theme),
        theme.success_style()
    );
    assert_eq!(
        ProgressItemStatus::Failed.style(&theme),
        theme.error_style()
    );
}

#[test]
fn test_status_symbols() {
    assert_eq!(ProgressItemStatus::Pending.symbol(), "○");
    assert_eq!(ProgressItemStatus::Active.symbol(), "●");
    assert_eq!(ProgressItemStatus::Completed.symbol(), "✓");
    assert_eq!(ProgressItemStatus::Failed.symbol(), "✗");
}

#[test]
fn test_status_equality() {
    assert_eq!(ProgressItemStatus::Pending, ProgressItemStatus::Pending);
    assert_eq!(ProgressItemStatus::Active, ProgressItemStatus::Active);
    assert_eq!(ProgressItemStatus::Completed, ProgressItemStatus::Completed);
    assert_eq!(ProgressItemStatus::Failed, ProgressItemStatus::Failed);
    assert_ne!(ProgressItemStatus::Pending, ProgressItemStatus::Active);
    assert_ne!(ProgressItemStatus::Completed, ProgressItemStatus::Failed);
}

#[test]
fn test_status_copy() {
    let status = ProgressItemStatus::Active;
    let copy = status;
    assert_eq!(status, copy);
}

#[test]
fn test_status_debug() {
    let debug_str = format!("{:?}", ProgressItemStatus::Pending);
    assert_eq!(debug_str, "Pending");

    let debug_str = format!("{:?}", ProgressItemStatus::Active);
    assert_eq!(debug_str, "Active");

    let debug_str = format!("{:?}", ProgressItemStatus::Completed);
    assert_eq!(debug_str, "Completed");

    let debug_str = format!("{:?}", ProgressItemStatus::Failed);
    assert_eq!(debug_str, "Failed");
}

// ========================================
// ProgressItem Tests
// ========================================

#[test]
fn test_item_new() {
    let item = ProgressItem::new("id1", "Label");
    assert_eq!(item.id(), "id1");
    assert_eq!(item.label(), "Label");
    assert_eq!(item.progress(), 0.0);
    assert_eq!(item.status(), ProgressItemStatus::Pending);
    assert!(item.message().is_none());
}

#[test]
fn test_item_new_with_string_types() {
    let item = ProgressItem::new(String::from("id1"), String::from("My Label"));
    assert_eq!(item.id(), "id1");
    assert_eq!(item.label(), "My Label");
}

#[test]
fn test_item_percentage() {
    let mut item = ProgressItem::new("id1", "Test");
    item.progress = 0.5;
    assert_eq!(item.percentage(), 50);

    item.progress = 0.0;
    assert_eq!(item.percentage(), 0);

    item.progress = 1.0;
    assert_eq!(item.percentage(), 100);
}

#[test]
fn test_item_percentage_rounding() {
    let mut item = ProgressItem::new("id1", "Test");

    item.progress = 0.334;
    assert_eq!(item.percentage(), 33);

    item.progress = 0.335;
    assert_eq!(item.percentage(), 34);

    item.progress = 0.999;
    assert_eq!(item.percentage(), 100);

    item.progress = 0.001;
    assert_eq!(item.percentage(), 0);

    item.progress = 0.005;
    assert_eq!(item.percentage(), 1);
}

#[test]
fn test_item_clone() {
    let mut item = ProgressItem::new("id1", "Label");
    item.progress = 0.75;
    item.status = ProgressItemStatus::Active;
    item.message = Some("Working...".to_string());

    let cloned = item.clone();
    assert_eq!(item, cloned);
    assert_eq!(cloned.id(), "id1");
    assert_eq!(cloned.label(), "Label");
    assert_eq!(cloned.progress(), 0.75);
    assert_eq!(cloned.status(), ProgressItemStatus::Active);
    assert_eq!(cloned.message(), Some("Working..."));
}

#[test]
fn test_item_debug() {
    let item = ProgressItem::new("id1", "Label");
    let debug_str = format!("{:?}", item);
    assert!(debug_str.contains("id1"));
    assert!(debug_str.contains("Label"));
}

#[test]
fn test_item_equality() {
    let item1 = ProgressItem::new("id1", "Label");
    let item2 = ProgressItem::new("id1", "Label");
    assert_eq!(item1, item2);

    let item3 = ProgressItem::new("id2", "Label");
    assert_ne!(item1, item3);

    let item4 = ProgressItem::new("id1", "Different");
    assert_ne!(item1, item4);
}

#[test]
fn test_item_message_accessor() {
    let mut item = ProgressItem::new("id1", "Label");
    assert!(item.message().is_none());

    item.message = Some("Test message".to_string());
    assert_eq!(item.message(), Some("Test message"));
}
