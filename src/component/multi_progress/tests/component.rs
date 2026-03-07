use super::*;

// ========================================
// Component Tests
// ========================================

#[test]
fn test_init() {
    let state = MultiProgress::init();
    assert!(state.is_empty());
}

#[test]
fn test_update_add() {
    let mut state = MultiProgress::init();
    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::Add {
            id: "id1".to_string(),
            label: "Item 1".to_string(),
        },
    );
    assert_eq!(output, Some(MultiProgressOutput::Added("id1".to_string())));
    assert_eq!(state.len(), 1);
}

#[test]
fn test_update_set_progress() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: 0.5,
        },
    );

    assert_eq!(state.find("id1").unwrap().progress(), 0.5);
    // Should auto-activate
    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Active
    );
}

#[test]
fn test_update_set_progress_clamped() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: 1.5, // Should be clamped to 1.0
        },
    );

    assert_eq!(state.find("id1").unwrap().progress(), 1.0);

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: -0.5, // Should be clamped to 0.0
        },
    );

    assert_eq!(state.find("id1").unwrap().progress(), 0.0);
}

#[test]
fn test_update_complete() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id1".to_string()),
    );

    assert_eq!(
        output,
        Some(MultiProgressOutput::Completed("id1".to_string()))
    );
    let item = state.find("id1").unwrap();
    assert_eq!(item.progress(), 1.0);
    assert_eq!(item.status(), ProgressItemStatus::Completed);
}

#[test]
fn test_update_complete_auto_remove() {
    let mut state = MultiProgressState::new().with_auto_remove(true);
    state.add("id1", "Item 1");

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id1".to_string()),
    );

    assert_eq!(
        output,
        Some(MultiProgressOutput::Removed("id1".to_string()))
    );
    assert!(state.is_empty());
}

#[test]
fn test_update_fail() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::Fail {
            id: "id1".to_string(),
            message: Some("Timeout".to_string()),
        },
    );

    assert_eq!(output, Some(MultiProgressOutput::Failed("id1".to_string())));
    let item = state.find("id1").unwrap();
    assert_eq!(item.status(), ProgressItemStatus::Failed);
    assert_eq!(item.message(), Some("Timeout"));
}

#[test]
fn test_update_remove() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    let output = MultiProgress::update(&mut state, MultiProgressMessage::Remove("id1".to_string()));

    assert_eq!(
        output,
        Some(MultiProgressOutput::Removed("id1".to_string()))
    );
    assert!(state.is_empty());
}

#[test]
fn test_update_clear() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");

    let output = MultiProgress::update(&mut state, MultiProgressMessage::Clear);

    assert_eq!(output, Some(MultiProgressOutput::Cleared));
    assert!(state.is_empty());
}

#[test]
fn test_update_clear_empty() {
    let mut state = MultiProgress::init();
    let output = MultiProgress::update(&mut state, MultiProgressMessage::Clear);
    assert!(output.is_none());
}

// ========================================
// Scroll Tests
// ========================================

#[test]
fn test_scroll_down() {
    let mut state = MultiProgressState::new();
    for i in 0..10 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up() {
    let mut state = MultiProgressState::new();
    for i in 0..10 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }
    state.set_scroll_offset(5);

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 4);
}

#[test]
fn test_scroll_to_top() {
    let mut state = MultiProgressState::new();
    for i in 0..10 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }
    state.set_scroll_offset(5);

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollToTop);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_to_bottom() {
    let mut state = MultiProgressState::new();
    for i in 0..10 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollToBottom);
    assert_eq!(state.scroll_offset(), 9);
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    assert_eq!(state.scroll_offset(), 0);

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0); // Should stay at 0
}

#[test]
fn test_scroll_down_at_bottom() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");
    state.set_scroll_offset(1); // At the last item

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1); // Should stay at 1
}

// ========================================
// View Tests
// ========================================

#[test]
fn test_view_empty() {
    let state = MultiProgressState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &theme))
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_items() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: 0.5,
        },
    );

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &theme))
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_title() {
    let state = MultiProgressState::new().with_title("Downloads");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &theme))
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_failed_item() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Fail {
            id: "id1".to_string(),
            message: Some("Connection lost".to_string()),
        },
    );

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &theme))
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_completed_item() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id1".to_string()),
    );

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &theme))
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========================================
// Additional Coverage Tests
// ========================================

#[test]
fn test_view_zero_size_area() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    // Test with zero width
    terminal
        .draw(|frame| {
            MultiProgress::view(&state, frame, Rect::new(0, 0, 0, 10), &theme);
        })
        .unwrap();

    // Test with zero height
    terminal
        .draw(|frame| {
            MultiProgress::view(&state, frame, Rect::new(0, 0, 60, 0), &Theme::default());
        })
        .unwrap();
}

#[test]
fn test_view_without_percentages() {
    let mut state = MultiProgressState::new().with_percentages(false);
    state.add("id1", "Item 1");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &theme))
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_failed_without_message() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Fail {
            id: "id1".to_string(),
            message: None, // No message
        },
    );

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &theme))
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_update_add_duplicate() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::Add {
            id: "id1".to_string(),
            label: "Duplicate".to_string(),
        },
    );

    assert!(output.is_none());
    assert_eq!(state.len(), 1);
}

#[test]
fn test_update_set_progress_nonexistent() {
    let mut state = MultiProgressState::new();

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "nonexistent".to_string(),
            progress: 0.5,
        },
    );

    assert!(output.is_none());
}

#[test]
fn test_update_set_status_nonexistent() {
    let mut state = MultiProgressState::new();

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetStatus {
            id: "nonexistent".to_string(),
            status: ProgressItemStatus::Active,
        },
    );

    assert!(output.is_none());
}

#[test]
fn test_update_set_message() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetMessage {
            id: "id1".to_string(),
            message: Some("Processing...".to_string()),
        },
    );

    assert_eq!(state.find("id1").unwrap().message(), Some("Processing..."));
}

#[test]
fn test_update_set_message_nonexistent() {
    let mut state = MultiProgressState::new();

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetMessage {
            id: "nonexistent".to_string(),
            message: Some("Message".to_string()),
        },
    );

    assert!(output.is_none());
}

#[test]
fn test_update_complete_nonexistent() {
    let mut state = MultiProgressState::new();

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("nonexistent".to_string()),
    );

    assert!(output.is_none());
}

#[test]
fn test_update_fail_nonexistent() {
    let mut state = MultiProgressState::new();

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::Fail {
            id: "nonexistent".to_string(),
            message: None,
        },
    );

    assert!(output.is_none());
}

#[test]
fn test_update_remove_nonexistent() {
    let mut state = MultiProgressState::new();

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::Remove("nonexistent".to_string()),
    );

    assert!(output.is_none());
}

#[test]
fn test_set_progress_no_auto_activate_if_already_active() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    // First set to Active
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetStatus {
            id: "id1".to_string(),
            status: ProgressItemStatus::Active,
        },
    );

    // Now set progress
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: 0.5,
        },
    );

    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Active
    );
}

#[test]
fn test_set_progress_no_auto_activate_if_zero() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    // Set progress to 0 (should not auto-activate)
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: 0.0,
        },
    );

    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Pending
    );
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{with_annotations, WidgetType};
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                MultiProgress::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::MultiProgress);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.has_id("multi_progress"));
}
