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

// StatusBarState tests

#[test]
fn test_state_new() {
    let state = StatusBarState::new();
    assert!(state.left().is_empty());
    assert!(state.center().is_empty());
    assert!(state.right().is_empty());
    assert_eq!(state.separator(), " | ");
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
}

#[test]
fn test_state_default() {
    let state = StatusBarState::default();
    assert!(state.is_empty());
}

#[test]
fn test_state_with_separator() {
    let state = StatusBarState::with_separator(" :: ");
    assert_eq!(state.separator(), " :: ");
}

#[test]
fn test_state_push_left() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("A"));
    state.push_left(StatusBarItem::new("B"));
    assert_eq!(state.left().len(), 2);
    assert_eq!(state.left()[0].text(), "A");
    assert_eq!(state.left()[1].text(), "B");
}

#[test]
fn test_state_push_center() {
    let mut state = StatusBarState::new();
    state.push_center(StatusBarItem::new("Center"));
    assert_eq!(state.center().len(), 1);
}

#[test]
fn test_state_push_right() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::new("Right"));
    assert_eq!(state.right().len(), 1);
}

#[test]
fn test_state_clear() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L"));
    state.push_center(StatusBarItem::new("C"));
    state.push_right(StatusBarItem::new("R"));
    assert_eq!(state.len(), 3);

    state.clear();
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
}

// StatusBar component tests

#[test]
fn test_init() {
    let state = StatusBar::init();
    assert!(state.is_empty());
}

#[test]
fn test_set_left_items() {
    let mut state = StatusBarState::new();
    let items = vec![StatusBarItem::new("A"), StatusBarItem::new("B")];

    StatusBar::update(&mut state, StatusBarMessage::SetLeftItems(items));
    assert_eq!(state.left().len(), 2);
}

#[test]
fn test_set_center_items() {
    let mut state = StatusBarState::new();
    let items = vec![StatusBarItem::new("Center")];

    StatusBar::update(&mut state, StatusBarMessage::SetCenterItems(items));
    assert_eq!(state.center().len(), 1);
}

#[test]
fn test_set_right_items() {
    let mut state = StatusBarState::new();
    let items = vec![StatusBarItem::new("Right")];

    StatusBar::update(&mut state, StatusBarMessage::SetRightItems(items));
    assert_eq!(state.right().len(), 1);
}

#[test]
fn test_clear_message() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L"));
    state.push_center(StatusBarItem::new("C"));
    state.push_right(StatusBarItem::new("R"));

    StatusBar::update(&mut state, StatusBarMessage::Clear);
    assert!(state.is_empty());
}

#[test]
fn test_clear_left_message() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L"));
    state.push_center(StatusBarItem::new("C"));

    StatusBar::update(&mut state, StatusBarMessage::ClearLeft);
    assert!(state.left().is_empty());
    assert_eq!(state.center().len(), 1);
}

#[test]
fn test_clear_center_message() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L"));
    state.push_center(StatusBarItem::new("C"));

    StatusBar::update(&mut state, StatusBarMessage::ClearCenter);
    assert_eq!(state.left().len(), 1);
    assert!(state.center().is_empty());
}

#[test]
fn test_clear_right_message() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::new("R"));
    state.push_center(StatusBarItem::new("C"));

    StatusBar::update(&mut state, StatusBarMessage::ClearRight);
    assert!(state.right().is_empty());
    assert_eq!(state.center().len(), 1);
}

#[test]
fn test_update_returns_none() {
    let mut state = StatusBarState::new();
    let output = StatusBar::update(&mut state, StatusBarMessage::Clear);
    assert!(output.is_none());
}

// View tests

#[test]
fn test_view_empty() {
    let state = StatusBarState::new();

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_left_only() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("LEFT"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_right_only() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::new("RIGHT"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_center_only() {
    let mut state = StatusBarState::new();
    state.push_center(StatusBarItem::new("CENTER"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_all_sections() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("LEFT"));
    state.push_center(StatusBarItem::new("CENTER"));
    state.push_right(StatusBarItem::new("RIGHT"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_separator() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("A"));
    state.push_left(StatusBarItem::new("B"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_custom_separator() {
    let mut state = StatusBarState::with_separator(" :: ");
    state.push_left(StatusBarItem::new("A"));
    state.push_left(StatusBarItem::new("B"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_no_separator_on_last_item() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("A").with_separator(false));
    state.push_left(StatusBarItem::new("B"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_styled_items() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("INFO").with_style(StatusBarStyle::Info));
    state.push_left(StatusBarItem::new("ERROR").with_style(StatusBarStyle::Error));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// Integration tests

#[test]
fn test_typical_editor_status_bar() {
    let mut state = StatusBarState::new();

    // Left: mode indicator
    state.push_left(StatusBarItem::new("NORMAL").with_style(StatusBarStyle::Info));

    // Center: filename
    state.push_center(StatusBarItem::new("main.rs"));
    state.push_center(StatusBarItem::new("[+]").with_style(StatusBarStyle::Warning));

    // Right: position info
    state.push_right(StatusBarItem::new("UTF-8").with_style(StatusBarStyle::Muted));
    state.push_right(StatusBarItem::new("Ln 42, Col 8"));

    assert_eq!(state.left().len(), 1);
    assert_eq!(state.center().len(), 2);
    assert_eq!(state.right().len(), 2);
    assert_eq!(state.len(), 5);
}

#[test]
fn test_update_mode_indicator() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("NORMAL").with_style(StatusBarStyle::Info));

    // Simulate mode change
    StatusBar::update(
        &mut state,
        StatusBarMessage::SetLeftItems(vec![
            StatusBarItem::new("INSERT").with_style(StatusBarStyle::Success)
        ]),
    );

    assert_eq!(state.left().len(), 1);
    assert_eq!(state.left()[0].text(), "INSERT");
    assert_eq!(state.left()[0].style(), StatusBarStyle::Success);
}

#[test]
fn test_render_section_empty() {
    let theme = Theme::default();
    let spans = StatusBar::render_section(&[], " | ", &theme);
    assert!(spans.is_empty());
}

#[test]
fn test_render_section_single_item() {
    let theme = Theme::default();
    let items = vec![StatusBarItem::new("Test")];
    let spans = StatusBar::render_section(&items, " | ", &theme);
    assert_eq!(spans.len(), 1);
}

#[test]
fn test_render_section_multiple_items() {
    let theme = Theme::default();
    let items = vec![StatusBarItem::new("A"), StatusBarItem::new("B")];
    let spans = StatusBar::render_section(&items, " | ", &theme);
    // A + separator + B = 3 spans
    assert_eq!(spans.len(), 3);
}

// ========================================
// Dynamic Content Tests
// ========================================

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

// Section tests

#[test]
fn test_section_enum() {
    assert_ne!(Section::Left, Section::Center);
    assert_ne!(Section::Center, Section::Right);
    assert_ne!(Section::Left, Section::Right);
}

#[test]
fn test_state_section() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L"));
    state.push_center(StatusBarItem::new("C"));
    state.push_right(StatusBarItem::new("R"));

    assert_eq!(state.section(Section::Left).len(), 1);
    assert_eq!(state.section(Section::Center).len(), 1);
    assert_eq!(state.section(Section::Right).len(), 1);
}

#[test]
fn test_state_section_mut() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L"));

    state
        .section_mut(Section::Left)
        .push(StatusBarItem::new("L2"));
    assert_eq!(state.section(Section::Left).len(), 2);
}

#[test]
fn test_state_get_item_mut() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("Test"));

    let item = state.get_item_mut(Section::Left, 0);
    assert!(item.is_some());
    item.unwrap().set_text("Updated");
    assert_eq!(state.left()[0].text(), "Updated");
}

#[test]
fn test_state_get_item_mut_invalid_index() {
    let mut state = StatusBarState::new();
    assert!(state.get_item_mut(Section::Left, 0).is_none());
}

// Timer message tests

#[test]
fn test_tick_message() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::elapsed_time());

    // Start the timer
    StatusBar::update(
        &mut state,
        StatusBarMessage::StartTimer {
            section: Section::Left,
            index: 0,
        },
    );

    // Tick 5 seconds
    StatusBar::update(&mut state, StatusBarMessage::Tick(5000));
    assert_eq!(state.left()[0].text(), "00:05");

    // Tick another 65 seconds
    StatusBar::update(&mut state, StatusBarMessage::Tick(65000));
    assert_eq!(state.left()[0].text(), "01:10");
}

#[test]
fn test_start_timer() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::elapsed_time());

    StatusBar::update(
        &mut state,
        StatusBarMessage::StartTimer {
            section: Section::Left,
            index: 0,
        },
    );

    if let StatusBarItemContent::ElapsedTime { running, .. } = state.left()[0].content() {
        assert!(*running);
    } else {
        panic!("Expected ElapsedTime content");
    }
}

#[test]
fn test_stop_timer() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::elapsed_time());

    // Start then stop
    StatusBar::update(
        &mut state,
        StatusBarMessage::StartTimer {
            section: Section::Left,
            index: 0,
        },
    );
    StatusBar::update(
        &mut state,
        StatusBarMessage::StopTimer {
            section: Section::Left,
            index: 0,
        },
    );

    if let StatusBarItemContent::ElapsedTime { running, .. } = state.left()[0].content() {
        assert!(!*running);
    } else {
        panic!("Expected ElapsedTime content");
    }
}

#[test]
fn test_reset_timer() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::elapsed_time());

    // Start, tick, then reset
    StatusBar::update(
        &mut state,
        StatusBarMessage::StartTimer {
            section: Section::Left,
            index: 0,
        },
    );
    StatusBar::update(&mut state, StatusBarMessage::Tick(10000));
    assert_eq!(state.left()[0].text(), "00:10");

    StatusBar::update(
        &mut state,
        StatusBarMessage::ResetTimer {
            section: Section::Left,
            index: 0,
        },
    );
    assert_eq!(state.left()[0].text(), "00:00");
}

#[test]
fn test_timer_stopped_no_tick() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::elapsed_time());

    // Timer not started, ticking should not change time
    StatusBar::update(&mut state, StatusBarMessage::Tick(5000));
    assert_eq!(state.left()[0].text(), "00:00");
}

// Counter message tests

#[test]
fn test_increment_counter() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::counter());

    StatusBar::update(
        &mut state,
        StatusBarMessage::IncrementCounter {
            section: Section::Right,
            index: 0,
        },
    );
    assert_eq!(state.right()[0].text(), "1");

    StatusBar::update(
        &mut state,
        StatusBarMessage::IncrementCounter {
            section: Section::Right,
            index: 0,
        },
    );
    assert_eq!(state.right()[0].text(), "2");
}

#[test]
fn test_decrement_counter() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::counter());

    // Set to 5, then decrement
    StatusBar::update(
        &mut state,
        StatusBarMessage::SetCounter {
            section: Section::Right,
            index: 0,
            value: 5,
        },
    );
    StatusBar::update(
        &mut state,
        StatusBarMessage::DecrementCounter {
            section: Section::Right,
            index: 0,
        },
    );
    assert_eq!(state.right()[0].text(), "4");
}

#[test]
fn test_decrement_counter_no_underflow() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::counter());

    // Try to decrement below 0
    StatusBar::update(
        &mut state,
        StatusBarMessage::DecrementCounter {
            section: Section::Right,
            index: 0,
        },
    );
    assert_eq!(state.right()[0].text(), "0");
}

#[test]
fn test_set_counter() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::counter().with_label("Items"));

    StatusBar::update(
        &mut state,
        StatusBarMessage::SetCounter {
            section: Section::Right,
            index: 0,
            value: 42,
        },
    );
    assert_eq!(state.right()[0].text(), "Items: 42");
}

// Heartbeat message tests

#[test]
fn test_activate_heartbeat() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::heartbeat());

    StatusBar::update(
        &mut state,
        StatusBarMessage::ActivateHeartbeat {
            section: Section::Left,
            index: 0,
        },
    );

    if let StatusBarItemContent::Heartbeat { active, .. } = state.left()[0].content() {
        assert!(*active);
    } else {
        panic!("Expected Heartbeat content");
    }
}

#[test]
fn test_deactivate_heartbeat() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::heartbeat());

    StatusBar::update(
        &mut state,
        StatusBarMessage::ActivateHeartbeat {
            section: Section::Left,
            index: 0,
        },
    );
    StatusBar::update(
        &mut state,
        StatusBarMessage::DeactivateHeartbeat {
            section: Section::Left,
            index: 0,
        },
    );

    if let StatusBarItemContent::Heartbeat { active, .. } = state.left()[0].content() {
        assert!(!*active);
    } else {
        panic!("Expected Heartbeat content");
    }
}

#[test]
fn test_pulse_heartbeat() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::heartbeat());

    // First pulse
    StatusBar::update(
        &mut state,
        StatusBarMessage::PulseHeartbeat {
            section: Section::Left,
            index: 0,
        },
    );

    if let StatusBarItemContent::Heartbeat { active, frame } = state.left()[0].content() {
        assert!(*active);
        assert_eq!(*frame, 1);
    } else {
        panic!("Expected Heartbeat content");
    }
}

#[test]
fn test_heartbeat_tick() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::heartbeat());

    // Activate and tick
    StatusBar::update(
        &mut state,
        StatusBarMessage::ActivateHeartbeat {
            section: Section::Left,
            index: 0,
        },
    );

    StatusBar::update(&mut state, StatusBarMessage::Tick(100));

    if let StatusBarItemContent::Heartbeat { frame, .. } = state.left()[0].content() {
        assert_eq!(*frame, 1);
    } else {
        panic!("Expected Heartbeat content");
    }
}

// View tests for dynamic content

#[test]
fn test_view_elapsed_time() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::elapsed_time());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_counter_with_label() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::counter().with_label("Files"));

    StatusBar::update(
        &mut state,
        StatusBarMessage::SetCounter {
            section: Section::Right,
            index: 0,
            value: 15,
        },
    );

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_heartbeat() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::heartbeat());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// Integration tests for dynamic content

#[test]
fn test_media_player_status_bar() {
    let mut state = StatusBarState::new();

    // Left: elapsed time
    state.push_left(StatusBarItem::elapsed_time().with_style(StatusBarStyle::Info));

    // Center: file name
    state.push_center(StatusBarItem::new("song.mp3"));

    // Right: heartbeat for activity
    state.push_right(StatusBarItem::heartbeat());

    assert_eq!(state.len(), 3);
    assert!(state.left()[0].is_dynamic());
}

#[test]
fn test_file_processor_status_bar() {
    let mut state = StatusBarState::new();

    // Left: timer for processing
    state.push_left(StatusBarItem::elapsed_time_long());

    // Center: file count
    state.push_center(StatusBarItem::counter().with_label("Processed"));

    // Right: remaining count
    state.push_right(StatusBarItem::counter().with_label("Remaining"));

    // Simulate processing
    StatusBar::update(
        &mut state,
        StatusBarMessage::StartTimer {
            section: Section::Left,
            index: 0,
        },
    );
    StatusBar::update(
        &mut state,
        StatusBarMessage::SetCounter {
            section: Section::Right,
            index: 0,
            value: 100,
        },
    );

    // Process one file
    StatusBar::update(
        &mut state,
        StatusBarMessage::IncrementCounter {
            section: Section::Center,
            index: 0,
        },
    );
    StatusBar::update(
        &mut state,
        StatusBarMessage::DecrementCounter {
            section: Section::Right,
            index: 0,
        },
    );

    assert_eq!(state.center()[0].text(), "Processed: 1");
    assert_eq!(state.right()[0].text(), "Remaining: 99");
}
