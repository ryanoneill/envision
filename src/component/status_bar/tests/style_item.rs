use super::*;

// StatusBarStyle tests

#[test]
fn test_style_method() {
    let theme = Theme::default();
    assert_eq!(StatusBarStyle::Default.style(&theme), theme.normal_style());
    assert_eq!(StatusBarStyle::Info.style(&theme), theme.info_style());
    assert_eq!(StatusBarStyle::Success.style(&theme), theme.success_style());
    assert_eq!(StatusBarStyle::Warning.style(&theme), theme.warning_style());
    assert_eq!(StatusBarStyle::Error.style(&theme), theme.error_style());
    assert_eq!(StatusBarStyle::Muted.style(&theme), theme.disabled_style());
}

// StatusBarItem tests

#[test]
fn test_item_new() {
    let item = StatusBarItem::new("Test");
    assert_eq!(item.text(), "Test");
    assert_eq!(item.style(), StatusBarStyle::Default);
    assert!(item.has_separator());
}

#[test]
fn test_item_with_style() {
    let item = StatusBarItem::new("Error").with_style(StatusBarStyle::Error);
    assert_eq!(item.style(), StatusBarStyle::Error);
}

#[test]
fn test_item_with_separator() {
    let item = StatusBarItem::new("Last").with_separator(false);
    assert!(!item.has_separator());
}

// StatusBarItemContent tests

#[test]
fn test_content_static_text() {
    let content = StatusBarItemContent::static_text("Hello");
    assert_eq!(content.display_text(), "Hello");
    assert!(!content.is_dynamic());
}

#[test]
fn test_content_elapsed_time_default() {
    let content = StatusBarItemContent::elapsed_time();
    assert_eq!(content.display_text(), "00:00");
    assert!(content.is_dynamic());
}

#[test]
fn test_content_elapsed_time_formatting() {
    let content = StatusBarItemContent::ElapsedTime {
        elapsed_ms: 65_000, // 1 min 5 sec
        running: false,
        long_format: false,
    };
    assert_eq!(content.display_text(), "01:05");
}

#[test]
fn test_content_elapsed_time_long_format() {
    let content = StatusBarItemContent::ElapsedTime {
        elapsed_ms: 3_665_000, // 1 hr 1 min 5 sec
        running: false,
        long_format: true,
    };
    assert_eq!(content.display_text(), "01:01:05");
}

#[test]
fn test_content_elapsed_time_auto_long_format() {
    // When hours > 0, should auto-switch to long format
    let content = StatusBarItemContent::ElapsedTime {
        elapsed_ms: 3_665_000, // 1 hr 1 min 5 sec
        running: false,
        long_format: false, // Not explicit, but should show hours
    };
    assert_eq!(content.display_text(), "01:01:05");
}

#[test]
fn test_content_counter_default() {
    let content = StatusBarItemContent::counter();
    assert_eq!(content.display_text(), "0");
}

#[test]
fn test_content_counter_with_value() {
    let content = StatusBarItemContent::Counter {
        value: 42,
        label: None,
    };
    assert_eq!(content.display_text(), "42");
}

#[test]
fn test_content_counter_with_label() {
    let content = StatusBarItemContent::Counter {
        value: 5,
        label: Some("Items".to_string()),
    };
    assert_eq!(content.display_text(), "Items: 5");
}

#[test]
fn test_content_heartbeat_inactive() {
    let content = StatusBarItemContent::Heartbeat {
        active: false,
        frame: 0,
    };
    assert_eq!(content.display_text(), "♡");
}

#[test]
fn test_content_heartbeat_active_frames() {
    // Frame 0
    let content0 = StatusBarItemContent::Heartbeat {
        active: true,
        frame: 0,
    };
    assert_eq!(content0.display_text(), "♡");

    // Frame 1
    let content1 = StatusBarItemContent::Heartbeat {
        active: true,
        frame: 1,
    };
    assert_eq!(content1.display_text(), "♥");

    // Frame 2
    let content2 = StatusBarItemContent::Heartbeat {
        active: true,
        frame: 2,
    };
    assert_eq!(content2.display_text(), "♥");

    // Frame 3
    let content3 = StatusBarItemContent::Heartbeat {
        active: true,
        frame: 3,
    };
    assert_eq!(content3.display_text(), "♡");
}

// StatusBarItem factory method tests

#[test]
fn test_item_elapsed_time() {
    let item = StatusBarItem::elapsed_time();
    assert_eq!(item.text(), "00:00");
    assert!(item.is_dynamic());
}

#[test]
fn test_item_elapsed_time_long() {
    let item = StatusBarItem::elapsed_time_long();
    assert_eq!(item.text(), "00:00:00");
}

#[test]
fn test_item_counter() {
    let item = StatusBarItem::counter();
    assert_eq!(item.text(), "0");
}

#[test]
fn test_item_counter_with_label() {
    let item = StatusBarItem::counter().with_label("Count");
    assert_eq!(item.text(), "Count: 0");
}

#[test]
fn test_item_heartbeat() {
    let item = StatusBarItem::heartbeat();
    assert_eq!(item.text(), "♡");
}

#[test]
fn test_item_with_long_format() {
    let item = StatusBarItem::elapsed_time().with_long_format(true);
    assert_eq!(item.text(), "00:00:00");
}
