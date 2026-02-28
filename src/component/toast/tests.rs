use super::*;

// ========================================
// ToastLevel Tests
// ========================================

#[test]
fn test_toast_level_default() {
    let level = ToastLevel::default();
    assert_eq!(level, ToastLevel::Info);
}

#[test]
fn test_toast_level_clone() {
    let level = ToastLevel::Success;
    let cloned = level;
    assert_eq!(cloned, ToastLevel::Success);
}

#[test]
fn test_toast_level_eq() {
    assert_eq!(ToastLevel::Info, ToastLevel::Info);
    assert_ne!(ToastLevel::Info, ToastLevel::Error);
    assert_eq!(ToastLevel::Warning, ToastLevel::Warning);
}

// ========================================
// ToastItem Tests
// ========================================

#[test]
fn test_toast_item_accessors() {
    let mut state = ToastState::new();
    state.push("Test message".into(), ToastLevel::Success, Some(1000));

    let toast = &state.toasts()[0];
    assert_eq!(toast.id(), 0);
    assert_eq!(toast.message(), "Test message");
    assert_eq!(toast.level(), ToastLevel::Success);
    assert_eq!(toast.remaining_ms(), Some(1000));
}

#[test]
fn test_toast_item_is_persistent() {
    let mut state = ToastState::new();
    state.push("Persistent".into(), ToastLevel::Info, None);
    state.push("Timed".into(), ToastLevel::Info, Some(1000));

    assert!(state.toasts()[0].is_persistent());
    assert!(!state.toasts()[1].is_persistent());
}

#[test]
fn test_toast_item_clone() {
    let mut state = ToastState::new();
    state.push("Test".into(), ToastLevel::Info, Some(1000));

    let toast = state.toasts()[0].clone();
    assert_eq!(toast.message(), "Test");
}

// ========================================
// State Creation Tests
// ========================================

#[test]
fn test_new() {
    let state = ToastState::new();
    assert!(state.is_empty());
    assert_eq!(state.default_duration(), None);
    assert_eq!(state.max_visible(), DEFAULT_MAX_VISIBLE);
}

#[test]
fn test_with_duration() {
    let state = ToastState::with_duration(3000);
    assert_eq!(state.default_duration(), Some(3000));
}

#[test]
fn test_with_max_visible() {
    let state = ToastState::with_max_visible(3);
    assert_eq!(state.max_visible(), 3);
}

#[test]
fn test_default() {
    let state = ToastState::default();
    assert!(state.is_empty());
    assert_eq!(state.default_duration(), None);
}

// ========================================
// Accessor Tests
// ========================================

#[test]
fn test_toasts() {
    let mut state = ToastState::new();
    state.info("One");
    state.info("Two");

    assert_eq!(state.toasts().len(), 2);
    assert_eq!(state.toasts()[0].message(), "One");
    assert_eq!(state.toasts()[1].message(), "Two");
}

#[test]
fn test_len() {
    let mut state = ToastState::new();
    assert_eq!(state.len(), 0);

    state.info("Test");
    assert_eq!(state.len(), 1);

    state.info("Test 2");
    assert_eq!(state.len(), 2);
}

#[test]
fn test_is_empty() {
    let mut state = ToastState::new();
    assert!(state.is_empty());

    state.info("Test");
    assert!(!state.is_empty());
}

#[test]
fn test_default_duration() {
    let state = ToastState::new();
    assert_eq!(state.default_duration(), None);

    let state = ToastState::with_duration(5000);
    assert_eq!(state.default_duration(), Some(5000));
}

#[test]
fn test_max_visible() {
    let state = ToastState::new();
    assert_eq!(state.max_visible(), DEFAULT_MAX_VISIBLE);

    let state = ToastState::with_max_visible(10);
    assert_eq!(state.max_visible(), 10);
}

// ========================================
// Convenience Method Tests
// ========================================

#[test]
fn test_info() {
    let mut state = ToastState::new();
    let id = state.info("Info message");

    assert_eq!(state.len(), 1);
    assert_eq!(state.toasts()[0].id(), id);
    assert_eq!(state.toasts()[0].level(), ToastLevel::Info);
    assert_eq!(state.toasts()[0].message(), "Info message");
}

#[test]
fn test_success() {
    let mut state = ToastState::new();
    let id = state.success("Success message");

    assert_eq!(state.toasts()[0].id(), id);
    assert_eq!(state.toasts()[0].level(), ToastLevel::Success);
}

#[test]
fn test_warning() {
    let mut state = ToastState::new();
    let id = state.warning("Warning message");

    assert_eq!(state.toasts()[0].id(), id);
    assert_eq!(state.toasts()[0].level(), ToastLevel::Warning);
}

#[test]
fn test_error() {
    let mut state = ToastState::new();
    let id = state.error("Error message");

    assert_eq!(state.toasts()[0].id(), id);
    assert_eq!(state.toasts()[0].level(), ToastLevel::Error);
}

#[test]
fn test_convenience_returns_id() {
    let mut state = ToastState::new();
    let id1 = state.info("One");
    let id2 = state.info("Two");
    let id3 = state.info("Three");

    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    assert_eq!(id3, 2);
}

#[test]
fn test_convenience_uses_default_duration() {
    let mut state = ToastState::with_duration(3000);
    state.info("Test");

    assert_eq!(state.toasts()[0].remaining_ms(), Some(3000));
}

// ========================================
// Push Message Tests
// ========================================

#[test]
fn test_push() {
    let mut state = ToastState::new();

    Toast::update(
        &mut state,
        ToastMessage::Push {
            message: "Test".into(),
            level: ToastLevel::Success,
            duration_ms: Some(5000),
        },
    );

    assert_eq!(state.len(), 1);
    assert_eq!(state.toasts()[0].message(), "Test");
    assert_eq!(state.toasts()[0].level(), ToastLevel::Success);
}

#[test]
fn test_push_returns_added() {
    let mut state = ToastState::new();

    let output = Toast::update(
        &mut state,
        ToastMessage::Push {
            message: "Test".into(),
            level: ToastLevel::Info,
            duration_ms: None,
        },
    );

    assert_eq!(output, Some(ToastOutput::Added(0)));
}

#[test]
fn test_push_increments_id() {
    let mut state = ToastState::new();

    let out1 = Toast::update(
        &mut state,
        ToastMessage::Push {
            message: "One".into(),
            level: ToastLevel::Info,
            duration_ms: None,
        },
    );
    let out2 = Toast::update(
        &mut state,
        ToastMessage::Push {
            message: "Two".into(),
            level: ToastLevel::Info,
            duration_ms: None,
        },
    );

    assert_eq!(out1, Some(ToastOutput::Added(0)));
    assert_eq!(out2, Some(ToastOutput::Added(1)));
}

#[test]
fn test_push_custom_duration() {
    let mut state = ToastState::with_duration(3000);

    Toast::update(
        &mut state,
        ToastMessage::Push {
            message: "Custom".into(),
            level: ToastLevel::Info,
            duration_ms: Some(10000),
        },
    );

    assert_eq!(state.toasts()[0].remaining_ms(), Some(10000));
}

#[test]
fn test_push_persistent() {
    let mut state = ToastState::new();

    Toast::update(
        &mut state,
        ToastMessage::Push {
            message: "Persistent".into(),
            level: ToastLevel::Info,
            duration_ms: None,
        },
    );

    assert!(state.toasts()[0].is_persistent());
}

// ========================================
// Dismiss Message Tests
// ========================================

#[test]
fn test_dismiss() {
    let mut state = ToastState::new();
    let id = state.info("Test");

    Toast::update(&mut state, ToastMessage::Dismiss(id));

    assert!(state.is_empty());
}

#[test]
fn test_dismiss_returns_dismissed() {
    let mut state = ToastState::new();
    let id = state.info("Test");

    let output = Toast::update(&mut state, ToastMessage::Dismiss(id));

    assert_eq!(output, Some(ToastOutput::Dismissed(id)));
}

#[test]
fn test_dismiss_nonexistent() {
    let mut state = ToastState::new();
    state.info("Test");

    let output = Toast::update(&mut state, ToastMessage::Dismiss(999));

    assert_eq!(output, None);
    assert_eq!(state.len(), 1);
}

#[test]
fn test_dismiss_preserves_others() {
    let mut state = ToastState::new();
    let id1 = state.info("One");
    let _id2 = state.info("Two");
    let id3 = state.info("Three");

    Toast::update(&mut state, ToastMessage::Dismiss(id1));

    assert_eq!(state.len(), 2);
    assert_eq!(state.toasts()[0].message(), "Two");
    assert_eq!(state.toasts()[1].id(), id3);
}

// ========================================
// Clear Message Tests
// ========================================

#[test]
fn test_clear() {
    let mut state = ToastState::new();
    state.info("One");
    state.info("Two");
    state.info("Three");

    Toast::update(&mut state, ToastMessage::Clear);

    assert!(state.is_empty());
}

#[test]
fn test_clear_returns_cleared() {
    let mut state = ToastState::new();
    state.info("Test");

    let output = Toast::update(&mut state, ToastMessage::Clear);

    assert_eq!(output, Some(ToastOutput::Cleared));
}

#[test]
fn test_clear_empty() {
    let mut state = ToastState::new();

    let output = Toast::update(&mut state, ToastMessage::Clear);

    assert_eq!(output, None);
}

// ========================================
// Tick Message Tests
// ========================================

#[test]
fn test_tick_decrements() {
    let mut state = ToastState::with_duration(3000);
    state.info("Test");

    Toast::update(&mut state, ToastMessage::Tick(1000));

    assert_eq!(state.toasts()[0].remaining_ms(), Some(2000));
}

#[test]
fn test_tick_expires() {
    let mut state = ToastState::with_duration(1000);
    state.info("Test");

    Toast::update(&mut state, ToastMessage::Tick(1000));

    assert!(state.is_empty());
}

#[test]
fn test_tick_returns_expired() {
    let mut state = ToastState::with_duration(1000);
    let id = state.info("Test");

    let output = Toast::update(&mut state, ToastMessage::Tick(1000));

    assert_eq!(output, Some(ToastOutput::Expired(id)));
}

#[test]
fn test_tick_persistent() {
    let mut state = ToastState::new();
    state.info("Persistent");

    Toast::update(&mut state, ToastMessage::Tick(10000));

    // Persistent toast should not be affected
    assert_eq!(state.len(), 1);
    assert!(state.toasts()[0].is_persistent());
}

#[test]
fn test_tick_multiple_expire() {
    let mut state = ToastState::with_duration(1000);
    state.info("One");
    state.info("Two");

    let output = Toast::update(&mut state, ToastMessage::Tick(1000));

    // Both should expire, but we only return the first
    assert!(state.is_empty());
    assert!(matches!(output, Some(ToastOutput::Expired(_))));
}

#[test]
fn test_tick_no_expire() {
    let mut state = ToastState::with_duration(3000);
    state.info("Test");

    let output = Toast::update(&mut state, ToastMessage::Tick(100));

    assert_eq!(output, None);
    assert_eq!(state.len(), 1);
}

// ========================================
// View Tests
// ========================================

#[test]
fn test_view_empty() {
    let state = ToastState::new();

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Toast::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_single() {
    let mut state = ToastState::new();
    state.info("Hello, world!");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Toast::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_multiple() {
    let mut state = ToastState::new();
    state.info("Message 1");
    state.success("Message 2");
    state.error("Message 3");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Toast::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_max_visible() {
    let mut state = ToastState::with_max_visible(2);
    state.info("Message 1");
    state.info("Message 2");
    state.info("Message 3");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Toast::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_info_style() {
    let mut state = ToastState::new();
    state.info("Info message");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Toast::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_success_style() {
    let mut state = ToastState::new();
    state.success("Success message");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Toast::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_warning_style() {
    let mut state = ToastState::new();
    state.warning("Warning message");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Toast::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_error_style() {
    let mut state = ToastState::new();
    state.error("Error message");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Toast::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========================================
// Integration Tests
// ========================================

#[test]
fn test_clone() {
    let mut state = ToastState::with_duration(3000);
    state.info("Test");
    state.success("Test 2");

    let cloned = state.clone();
    assert_eq!(cloned.len(), 2);
    assert_eq!(cloned.default_duration(), Some(3000));
}

#[test]
fn test_init() {
    let state = Toast::init();
    assert!(state.is_empty());
    assert_eq!(state.default_duration(), None);
}

#[test]
fn test_full_workflow() {
    let mut state = ToastState::with_duration(3000);

    // Add some toasts
    let id1 = state.success("File saved!");
    let id2 = state.info("Processing...");

    assert_eq!(state.len(), 2);

    // Tick some time
    Toast::update(&mut state, ToastMessage::Tick(1000));
    assert_eq!(state.toasts()[0].remaining_ms(), Some(2000));

    // Dismiss one
    Toast::update(&mut state, ToastMessage::Dismiss(id1));
    assert_eq!(state.len(), 1);
    assert_eq!(state.toasts()[0].id(), id2);

    // Tick until expire
    let output = Toast::update(&mut state, ToastMessage::Tick(2000));
    assert_eq!(output, Some(ToastOutput::Expired(id2)));
    assert!(state.is_empty());
}

#[test]
fn test_mixed_durations() {
    let mut state = ToastState::new();

    // Add persistent toast
    let persistent_id = state.info("Persistent");

    // Add timed toast via message
    Toast::update(
        &mut state,
        ToastMessage::Push {
            message: "Timed".into(),
            level: ToastLevel::Warning,
            duration_ms: Some(1000),
        },
    );

    assert_eq!(state.len(), 2);

    // Tick past timed duration
    Toast::update(&mut state, ToastMessage::Tick(1000));

    // Only persistent should remain
    assert_eq!(state.len(), 1);
    assert_eq!(state.toasts()[0].id(), persistent_id);
    assert!(state.toasts()[0].is_persistent());
}

#[test]
fn test_set_default_duration() {
    let mut state = ToastState::new();
    assert_eq!(state.default_duration(), None);

    state.set_default_duration(Some(5000));
    assert_eq!(state.default_duration(), Some(5000));

    state.info("Test");
    assert_eq!(state.toasts()[0].remaining_ms(), Some(5000));
}

#[test]
fn test_set_max_visible() {
    let mut state = ToastState::new();
    assert_eq!(state.max_visible(), DEFAULT_MAX_VISIBLE);

    state.set_max_visible(3);
    assert_eq!(state.max_visible(), 3);
}
