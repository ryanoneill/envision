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

// StatusBarItem accessor tests

#[test]
fn test_item_content_accessor() {
    let item = StatusBarItem::new("Hello");
    match item.content() {
        StatusBarItemContent::Static(text) => assert_eq!(text, "Hello"),
        _ => panic!("Expected Static content"),
    }
}

#[test]
fn test_item_content_accessor_elapsed_time() {
    let item = StatusBarItem::elapsed_time();
    match item.content() {
        StatusBarItemContent::ElapsedTime {
            elapsed_ms,
            running,
            long_format,
        } => {
            assert_eq!(*elapsed_ms, 0);
            assert!(!*running);
            assert!(!*long_format);
        }
        _ => panic!("Expected ElapsedTime content"),
    }
}

#[test]
fn test_item_content_mut_accessor() {
    let mut item = StatusBarItem::new("Hello");
    if let StatusBarItemContent::Static(text) = item.content_mut() {
        *text = "World".to_string();
    }
    assert_eq!(item.text(), "World");
}

#[test]
fn test_item_set_text() {
    let mut item = StatusBarItem::new("Original");
    assert_eq!(item.text(), "Original");

    item.set_text("Updated");
    assert_eq!(item.text(), "Updated");
}

#[test]
fn test_item_set_text_converts_dynamic_to_static() {
    let mut item = StatusBarItem::elapsed_time();
    assert!(item.is_dynamic());

    item.set_text("Static now");
    assert!(!item.is_dynamic());
    assert_eq!(item.text(), "Static now");
}

#[test]
fn test_item_set_style() {
    let mut item = StatusBarItem::new("Test");
    assert_eq!(item.style(), StatusBarStyle::Default);

    item.set_style(StatusBarStyle::Error);
    assert_eq!(item.style(), StatusBarStyle::Error);
}

#[test]
fn test_item_set_separator() {
    let mut item = StatusBarItem::new("Test");
    assert!(item.has_separator());

    item.set_separator(false);
    assert!(!item.has_separator());

    item.set_separator(true);
    assert!(item.has_separator());
}

// is_dynamic tests

#[test]
fn test_item_is_dynamic_static() {
    let item = StatusBarItem::new("Static");
    assert!(!item.is_dynamic());
}

#[test]
fn test_item_is_dynamic_counter() {
    let item = StatusBarItem::counter();
    assert!(item.is_dynamic());
}

#[test]
fn test_item_is_dynamic_heartbeat() {
    let item = StatusBarItem::heartbeat();
    assert!(item.is_dynamic());
}

// with_label on non-counter (no-op)

#[test]
fn test_with_label_on_static_item_is_noop() {
    let item = StatusBarItem::new("Static").with_label("Label");
    // with_label should not change static items
    assert_eq!(item.text(), "Static");
}

#[test]
fn test_with_label_on_heartbeat_is_noop() {
    let item = StatusBarItem::heartbeat().with_label("Label");
    assert_eq!(item.text(), "\u{2661}"); // heartbeat symbol
}

// with_long_format on non-elapsed-time (no-op)

#[test]
fn test_with_long_format_on_static_item_is_noop() {
    let item = StatusBarItem::new("Static").with_long_format(true);
    assert_eq!(item.text(), "Static");
}

#[test]
fn test_with_long_format_on_counter_is_noop() {
    let item = StatusBarItem::counter().with_long_format(true);
    assert_eq!(item.text(), "0");
}

// StatusBarStyle tests

#[test]
fn test_style_default_variant() {
    assert_eq!(StatusBarStyle::default(), StatusBarStyle::Default);
}

#[test]
fn test_style_clone() {
    let style = StatusBarStyle::Info;
    let cloned = style;
    assert_eq!(style, cloned);
}

#[test]
fn test_style_debug() {
    assert_eq!(format!("{:?}", StatusBarStyle::Default), "Default");
    assert_eq!(format!("{:?}", StatusBarStyle::Info), "Info");
    assert_eq!(format!("{:?}", StatusBarStyle::Success), "Success");
    assert_eq!(format!("{:?}", StatusBarStyle::Warning), "Warning");
    assert_eq!(format!("{:?}", StatusBarStyle::Error), "Error");
    assert_eq!(format!("{:?}", StatusBarStyle::Muted), "Muted");
}

#[test]
fn test_style_all_variants_differ_from_each_other() {
    let variants = [
        StatusBarStyle::Default,
        StatusBarStyle::Info,
        StatusBarStyle::Success,
        StatusBarStyle::Warning,
        StatusBarStyle::Error,
        StatusBarStyle::Muted,
    ];
    for i in 0..variants.len() {
        for j in (i + 1)..variants.len() {
            assert_ne!(variants[i], variants[j]);
        }
    }
}

// StatusBarItemContent tests

#[test]
fn test_content_is_dynamic_static() {
    let content = StatusBarItemContent::static_text("Hello");
    assert!(!content.is_dynamic());
}

#[test]
fn test_content_is_dynamic_elapsed_time() {
    let content = StatusBarItemContent::elapsed_time();
    assert!(content.is_dynamic());
}

#[test]
fn test_content_is_dynamic_counter() {
    let content = StatusBarItemContent::counter();
    assert!(content.is_dynamic());
}

#[test]
fn test_content_is_dynamic_heartbeat() {
    let content = StatusBarItemContent::heartbeat();
    assert!(content.is_dynamic());
}

#[test]
fn test_content_clone() {
    let content = StatusBarItemContent::Counter {
        value: 42,
        label: Some("Items".to_string()),
    };
    let cloned = content.clone();
    assert_eq!(cloned.display_text(), "Items: 42");
}

#[test]
fn test_content_debug() {
    let content = StatusBarItemContent::static_text("Test");
    let debug = format!("{:?}", content);
    assert!(debug.contains("Static"));
}

// StatusBarItem clone and debug

#[test]
fn test_item_clone() {
    let item = StatusBarItem::new("Test")
        .with_style(StatusBarStyle::Error)
        .with_separator(false);
    let cloned = item.clone();
    assert_eq!(cloned.text(), "Test");
    assert_eq!(cloned.style(), StatusBarStyle::Error);
    assert!(!cloned.has_separator());
}

#[test]
fn test_item_debug() {
    let item = StatusBarItem::new("Test");
    let debug = format!("{:?}", item);
    assert!(debug.contains("StatusBarItem"));
}

// Counter edge cases

#[test]
fn test_content_counter_zero_with_label() {
    let content = StatusBarItemContent::Counter {
        value: 0,
        label: Some("Count".to_string()),
    };
    assert_eq!(content.display_text(), "Count: 0");
}

#[test]
fn test_content_counter_large_value() {
    let content = StatusBarItemContent::Counter {
        value: 999_999,
        label: None,
    };
    assert_eq!(content.display_text(), "999999");
}

// ElapsedTime edge cases

#[test]
fn test_content_elapsed_time_zero() {
    let content = StatusBarItemContent::ElapsedTime {
        elapsed_ms: 0,
        running: true,
        long_format: false,
    };
    assert_eq!(content.display_text(), "00:00");
}

#[test]
fn test_content_elapsed_time_just_under_hour() {
    let content = StatusBarItemContent::ElapsedTime {
        elapsed_ms: 3_599_000, // 59 min 59 sec
        running: false,
        long_format: false,
    };
    assert_eq!(content.display_text(), "59:59");
}

#[test]
fn test_content_elapsed_time_exactly_one_hour_short_format() {
    // When hours > 0 and long_format is false, auto-switches to long
    let content = StatusBarItemContent::ElapsedTime {
        elapsed_ms: 3_600_000, // 1 hour exactly
        running: false,
        long_format: false,
    };
    assert_eq!(content.display_text(), "01:00:00");
}

#[test]
fn test_content_elapsed_time_long_format_zero() {
    let content = StatusBarItemContent::ElapsedTime {
        elapsed_ms: 0,
        running: false,
        long_format: true,
    };
    assert_eq!(content.display_text(), "00:00:00");
}

// Heartbeat frame wrapping

#[test]
fn test_content_heartbeat_frame_wrapping() {
    // Frame values cycle through 0..4
    let content = StatusBarItemContent::Heartbeat {
        active: true,
        frame: 4, // Should wrap to 0
    };
    assert_eq!(content.display_text(), "\u{2661}"); // Frame 0 = heart outline
}

#[test]
fn test_content_heartbeat_all_inactive_frames_same() {
    // When inactive, all frames show the same symbol
    for frame in 0..4 {
        let content = StatusBarItemContent::Heartbeat {
            active: false,
            frame,
        };
        assert_eq!(content.display_text(), "\u{2661}");
    }
}
