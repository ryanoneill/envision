use super::*;

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
fn test_item_percentage() {
    let mut item = ProgressItem::new("id1", "Test");
    item.progress = 0.5;
    assert_eq!(item.percentage(), 50);

    item.progress = 0.0;
    assert_eq!(item.percentage(), 0);

    item.progress = 1.0;
    assert_eq!(item.percentage(), 100);
}
