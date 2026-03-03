use super::*;

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

// Invalid index tests (messages targeting out-of-bounds indices)

#[test]
fn test_start_timer_invalid_index() {
    let mut state = StatusBarState::new();
    // No items, so index 0 is invalid
    StatusBar::update(
        &mut state,
        StatusBarMessage::StartTimer {
            section: Section::Left,
            index: 0,
        },
    );
    // Should not panic
    assert!(state.is_empty());
}

#[test]
fn test_stop_timer_invalid_index() {
    let mut state = StatusBarState::new();
    StatusBar::update(
        &mut state,
        StatusBarMessage::StopTimer {
            section: Section::Left,
            index: 0,
        },
    );
    assert!(state.is_empty());
}

#[test]
fn test_reset_timer_invalid_index() {
    let mut state = StatusBarState::new();
    StatusBar::update(
        &mut state,
        StatusBarMessage::ResetTimer {
            section: Section::Center,
            index: 0,
        },
    );
    assert!(state.is_empty());
}

#[test]
fn test_increment_counter_invalid_index() {
    let mut state = StatusBarState::new();
    StatusBar::update(
        &mut state,
        StatusBarMessage::IncrementCounter {
            section: Section::Right,
            index: 5,
        },
    );
    assert!(state.is_empty());
}

#[test]
fn test_decrement_counter_invalid_index() {
    let mut state = StatusBarState::new();
    StatusBar::update(
        &mut state,
        StatusBarMessage::DecrementCounter {
            section: Section::Right,
            index: 0,
        },
    );
    assert!(state.is_empty());
}

#[test]
fn test_set_counter_invalid_index() {
    let mut state = StatusBarState::new();
    StatusBar::update(
        &mut state,
        StatusBarMessage::SetCounter {
            section: Section::Left,
            index: 0,
            value: 100,
        },
    );
    assert!(state.is_empty());
}

#[test]
fn test_activate_heartbeat_invalid_index() {
    let mut state = StatusBarState::new();
    StatusBar::update(
        &mut state,
        StatusBarMessage::ActivateHeartbeat {
            section: Section::Left,
            index: 0,
        },
    );
    assert!(state.is_empty());
}

#[test]
fn test_deactivate_heartbeat_invalid_index() {
    let mut state = StatusBarState::new();
    StatusBar::update(
        &mut state,
        StatusBarMessage::DeactivateHeartbeat {
            section: Section::Left,
            index: 0,
        },
    );
    assert!(state.is_empty());
}

#[test]
fn test_pulse_heartbeat_invalid_index() {
    let mut state = StatusBarState::new();
    StatusBar::update(
        &mut state,
        StatusBarMessage::PulseHeartbeat {
            section: Section::Left,
            index: 0,
        },
    );
    assert!(state.is_empty());
}

// Wrong content type tests (message targets item of different type)

#[test]
fn test_start_timer_on_static_item() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("Static"));

    StatusBar::update(
        &mut state,
        StatusBarMessage::StartTimer {
            section: Section::Left,
            index: 0,
        },
    );
    // Should not panic; item remains unchanged
    assert_eq!(state.left()[0].text(), "Static");
}

#[test]
fn test_stop_timer_on_counter() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::counter());

    StatusBar::update(
        &mut state,
        StatusBarMessage::StopTimer {
            section: Section::Left,
            index: 0,
        },
    );
    // Should not panic
    assert_eq!(state.left()[0].text(), "0");
}

#[test]
fn test_increment_counter_on_static_item() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("Not a counter"));

    StatusBar::update(
        &mut state,
        StatusBarMessage::IncrementCounter {
            section: Section::Left,
            index: 0,
        },
    );
    // Should not panic; item remains unchanged
    assert_eq!(state.left()[0].text(), "Not a counter");
}

#[test]
fn test_increment_counter_on_heartbeat() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::heartbeat());

    StatusBar::update(
        &mut state,
        StatusBarMessage::IncrementCounter {
            section: Section::Right,
            index: 0,
        },
    );
    // Should not panic; heartbeat remains as is
    assert_eq!(state.right()[0].text(), "\u{2661}");
}

#[test]
fn test_activate_heartbeat_on_static_item() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("Not a heartbeat"));

    StatusBar::update(
        &mut state,
        StatusBarMessage::ActivateHeartbeat {
            section: Section::Left,
            index: 0,
        },
    );
    assert_eq!(state.left()[0].text(), "Not a heartbeat");
}

#[test]
fn test_pulse_heartbeat_on_counter() {
    let mut state = StatusBarState::new();
    state.push_center(StatusBarItem::counter().with_label("Count"));

    StatusBar::update(
        &mut state,
        StatusBarMessage::PulseHeartbeat {
            section: Section::Center,
            index: 0,
        },
    );
    // Should not panic; counter unchanged
    assert_eq!(state.center()[0].text(), "Count: 0");
}

// Tick with multiple sections

#[test]
fn test_tick_affects_all_sections() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::elapsed_time());
    state.push_center(StatusBarItem::elapsed_time());
    state.push_right(StatusBarItem::elapsed_time());

    // Start all timers
    for section in [Section::Left, Section::Center, Section::Right] {
        StatusBar::update(
            &mut state,
            StatusBarMessage::StartTimer {
                section,
                index: 0,
            },
        );
    }

    // Tick once
    StatusBar::update(&mut state, StatusBarMessage::Tick(5000));

    assert_eq!(state.left()[0].text(), "00:05");
    assert_eq!(state.center()[0].text(), "00:05");
    assert_eq!(state.right()[0].text(), "00:05");
}

#[test]
fn test_tick_static_items_unchanged() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("Static"));

    StatusBar::update(&mut state, StatusBarMessage::Tick(5000));
    assert_eq!(state.left()[0].text(), "Static");
}

// Heartbeat pulse frame wrapping

#[test]
fn test_pulse_heartbeat_frame_wraps_at_four() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::heartbeat());

    // Pulse 5 times: frames should be 1, 2, 3, 0, 1
    for _ in 0..5 {
        StatusBar::update(
            &mut state,
            StatusBarMessage::PulseHeartbeat {
                section: Section::Left,
                index: 0,
            },
        );
    }

    if let StatusBarItemContent::Heartbeat { frame, active } = state.left()[0].content() {
        assert_eq!(*frame, 1); // (0+5) % 4 = 1
        assert!(*active);
    } else {
        panic!("Expected Heartbeat content");
    }
}

// Counter operations on center section

#[test]
fn test_counter_operations_center_section() {
    let mut state = StatusBarState::new();
    state.push_center(StatusBarItem::counter().with_label("Total"));

    StatusBar::update(
        &mut state,
        StatusBarMessage::SetCounter {
            section: Section::Center,
            index: 0,
            value: 10,
        },
    );
    assert_eq!(state.center()[0].text(), "Total: 10");

    StatusBar::update(
        &mut state,
        StatusBarMessage::IncrementCounter {
            section: Section::Center,
            index: 0,
        },
    );
    assert_eq!(state.center()[0].text(), "Total: 11");

    StatusBar::update(
        &mut state,
        StatusBarMessage::DecrementCounter {
            section: Section::Center,
            index: 0,
        },
    );
    assert_eq!(state.center()[0].text(), "Total: 10");
}

// Timer operations on right section

#[test]
fn test_timer_operations_right_section() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::elapsed_time());

    StatusBar::update(
        &mut state,
        StatusBarMessage::StartTimer {
            section: Section::Right,
            index: 0,
        },
    );

    StatusBar::update(&mut state, StatusBarMessage::Tick(30000));
    assert_eq!(state.right()[0].text(), "00:30");

    StatusBar::update(
        &mut state,
        StatusBarMessage::StopTimer {
            section: Section::Right,
            index: 0,
        },
    );

    // After stopping, tick should not advance
    StatusBar::update(&mut state, StatusBarMessage::Tick(5000));
    assert_eq!(state.right()[0].text(), "00:30");
}

// Message clone and debug

#[test]
fn test_message_clone() {
    let msg = StatusBarMessage::SetLeftItems(vec![StatusBarItem::new("A")]);
    let cloned = msg.clone();
    assert_eq!(msg, cloned);
}

#[test]
fn test_message_debug() {
    let msg = StatusBarMessage::Clear;
    let debug = format!("{:?}", msg);
    assert!(debug.contains("Clear"));
}

#[test]
fn test_message_partial_eq() {
    assert_eq!(StatusBarMessage::Clear, StatusBarMessage::Clear);
    assert_eq!(StatusBarMessage::ClearLeft, StatusBarMessage::ClearLeft);
    assert_ne!(StatusBarMessage::ClearLeft, StatusBarMessage::ClearRight);
    assert_eq!(StatusBarMessage::Tick(100), StatusBarMessage::Tick(100));
    assert_ne!(StatusBarMessage::Tick(100), StatusBarMessage::Tick(200));
}

// All update variants return None

#[test]
fn test_all_update_variants_return_none() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::elapsed_time());
    state.push_center(StatusBarItem::counter());
    state.push_right(StatusBarItem::heartbeat());

    let messages = vec![
        StatusBarMessage::SetLeftItems(vec![StatusBarItem::new("A")]),
        StatusBarMessage::SetCenterItems(vec![]),
        StatusBarMessage::SetRightItems(vec![]),
        StatusBarMessage::Clear,
        StatusBarMessage::ClearLeft,
        StatusBarMessage::ClearCenter,
        StatusBarMessage::ClearRight,
        StatusBarMessage::Tick(100),
        StatusBarMessage::StartTimer {
            section: Section::Left,
            index: 0,
        },
        StatusBarMessage::StopTimer {
            section: Section::Left,
            index: 0,
        },
        StatusBarMessage::ResetTimer {
            section: Section::Left,
            index: 0,
        },
        StatusBarMessage::IncrementCounter {
            section: Section::Center,
            index: 0,
        },
        StatusBarMessage::DecrementCounter {
            section: Section::Center,
            index: 0,
        },
        StatusBarMessage::SetCounter {
            section: Section::Center,
            index: 0,
            value: 5,
        },
        StatusBarMessage::ActivateHeartbeat {
            section: Section::Right,
            index: 0,
        },
        StatusBarMessage::DeactivateHeartbeat {
            section: Section::Right,
            index: 0,
        },
        StatusBarMessage::PulseHeartbeat {
            section: Section::Right,
            index: 0,
        },
    ];

    // Re-create state for each message since some clear it
    for msg in messages {
        let mut s = StatusBarState::new();
        s.push_left(StatusBarItem::elapsed_time());
        s.push_center(StatusBarItem::counter());
        s.push_right(StatusBarItem::heartbeat());
        let output = StatusBar::update(&mut s, msg);
        assert!(output.is_none());
    }
}
