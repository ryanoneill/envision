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
