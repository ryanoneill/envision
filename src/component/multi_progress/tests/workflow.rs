use super::*;

// ========================================
// SetStatus Message Tests
// ========================================

#[test]
fn test_update_set_status() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetStatus {
            id: "id1".to_string(),
            status: ProgressItemStatus::Active,
        },
    );

    assert!(output.is_none());
    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Active
    );
}

#[test]
fn test_update_set_status_to_completed() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetStatus {
            id: "id1".to_string(),
            status: ProgressItemStatus::Completed,
        },
    );

    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Completed
    );
}

#[test]
fn test_update_set_status_to_failed() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetStatus {
            id: "id1".to_string(),
            status: ProgressItemStatus::Failed,
        },
    );

    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Failed
    );
}

// ========================================
// SetMessage Clear Tests
// ========================================

#[test]
fn test_update_set_message_then_clear() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    // Set a message
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetMessage {
            id: "id1".to_string(),
            message: Some("Processing...".to_string()),
        },
    );
    assert_eq!(state.find("id1").unwrap().message(), Some("Processing..."));

    // Clear the message
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetMessage {
            id: "id1".to_string(),
            message: None,
        },
    );
    assert!(state.find("id1").unwrap().message().is_none());
}

// ========================================
// Fail Without Message Tests
// ========================================

#[test]
fn test_update_fail_without_message() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::Fail {
            id: "id1".to_string(),
            message: None,
        },
    );

    assert_eq!(output, Some(MultiProgressOutput::Failed("id1".to_string())));
    let item = state.find("id1").unwrap();
    assert_eq!(item.status(), ProgressItemStatus::Failed);
    assert!(item.message().is_none());
}

// ========================================
// Auto-Remove with Multiple Items
// ========================================

#[test]
fn test_auto_remove_preserves_other_items() {
    let mut state = MultiProgressState::new().with_auto_remove(true);
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");
    state.add("id3", "Item 3");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id2".to_string()),
    );

    // id2 should be removed, others remain
    assert_eq!(state.len(), 2);
    assert!(state.find("id1").is_some());
    assert!(state.find("id2").is_none());
    assert!(state.find("id3").is_some());
}

#[test]
fn test_auto_remove_all_items_sequentially() {
    let mut state = MultiProgressState::new().with_auto_remove(true);
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id1".to_string()),
    );
    assert_eq!(state.len(), 1);

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id2".to_string()),
    );
    assert_eq!(state.len(), 0);
    assert!(state.is_empty());
}

#[test]
fn test_auto_remove_does_not_affect_failed() {
    let mut state = MultiProgressState::new().with_auto_remove(true);
    state.add("id1", "Item 1");

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::Fail {
            id: "id1".to_string(),
            message: None,
        },
    );

    // Failed items should NOT be auto-removed
    assert_eq!(output, Some(MultiProgressOutput::Failed("id1".to_string())));
    assert_eq!(state.len(), 1);
    assert!(state.find("id1").is_some());
}

// ========================================
// Scroll with Empty List
// ========================================

#[test]
fn test_scroll_down_empty_list() {
    let mut state = MultiProgressState::new();

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_up_empty_list() {
    let mut state = MultiProgressState::new();

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_to_top_empty_list() {
    let mut state = MultiProgressState::new();

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollToTop);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_to_bottom_empty_list() {
    let mut state = MultiProgressState::new();

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollToBottom);
    assert_eq!(state.scroll_offset(), 0);
}

// ========================================
// SetProgress Auto-Activate Edge Cases
// ========================================

#[test]
fn test_set_progress_auto_activates_from_pending() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    // Verify starts as Pending
    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Pending
    );

    // Set nonzero progress -> should auto-activate
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: 0.01,
        },
    );

    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Active
    );
}

#[test]
fn test_set_progress_does_not_change_completed_status() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    // Complete the item
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id1".to_string()),
    );
    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Completed
    );

    // Set progress -- should NOT change status since it's not Pending
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: 0.5,
        },
    );
    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Completed
    );
}

#[test]
fn test_set_progress_does_not_change_failed_status() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    // Fail the item
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Fail {
            id: "id1".to_string(),
            message: None,
        },
    );

    // Set progress -- should NOT change status since it's not Pending
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: 0.5,
        },
    );
    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Failed
    );
}

// ========================================
// Full Workflow Tests
// ========================================

#[test]
fn test_full_workflow_add_progress_complete() {
    let mut state = MultiProgressState::new();

    // Add items
    let output = state.update(MultiProgressMessage::Add {
        id: "ch1".to_string(),
        label: "Chapter 1".to_string(),
    });
    assert_eq!(output, Some(MultiProgressOutput::Added("ch1".to_string())));

    state.update(MultiProgressMessage::Add {
        id: "ch2".to_string(),
        label: "Chapter 2".to_string(),
    });

    assert_eq!(state.len(), 2);
    assert_eq!(state.completed_count(), 0);

    // Progress on ch1
    state.update(MultiProgressMessage::SetProgress {
        id: "ch1".to_string(),
        progress: 0.5,
    });
    assert_eq!(state.find("ch1").unwrap().progress(), 0.5);
    assert_eq!(
        state.find("ch1").unwrap().status(),
        ProgressItemStatus::Active
    );

    // Complete ch1
    let output = state.update(MultiProgressMessage::Complete("ch1".to_string()));
    assert_eq!(
        output,
        Some(MultiProgressOutput::Completed("ch1".to_string()))
    );
    assert_eq!(state.find("ch1").unwrap().progress(), 1.0);
    assert_eq!(state.completed_count(), 1);

    // Fail ch2
    let output = state.update(MultiProgressMessage::Fail {
        id: "ch2".to_string(),
        message: Some("Timeout".to_string()),
    });
    assert_eq!(output, Some(MultiProgressOutput::Failed("ch2".to_string())));
    assert_eq!(state.failed_count(), 1);

    // Remove ch2
    let output = state.update(MultiProgressMessage::Remove("ch2".to_string()));
    assert_eq!(
        output,
        Some(MultiProgressOutput::Removed("ch2".to_string()))
    );
    assert_eq!(state.len(), 1);

    // Clear all
    let output = state.update(MultiProgressMessage::Clear);
    assert_eq!(output, Some(MultiProgressOutput::Cleared));
    assert!(state.is_empty());
}

// ========================================
// Additional View Tests
// ========================================

#[test]
fn test_view_multiple_items_mixed_states() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Download");
    state.add("id2", "Upload");
    state.add("id3", "Process");
    state.add("id4", "Verify");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: 0.75,
        },
    );
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id2".to_string()),
    );
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Fail {
            id: "id3".to_string(),
            message: Some("Timeout".to_string()),
        },
    );
    // id4 remains pending

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| {
            MultiProgress::view(&state, frame, frame.area(), &theme, &ViewContext::default())
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled_state() {
    let mut state = MultiProgressState::new().with_disabled(true);
    state.add("id1", "Item 1");

    // Need to temporarily enable to add progress
    state.set_disabled(false);
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: 0.5,
        },
    );
    state.set_disabled(true);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| {
            MultiProgress::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().disabled(true),
            )
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_single_item_full_progress() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Complete");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: 1.0,
        },
    );

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| {
            MultiProgress::view(&state, frame, frame.area(), &theme, &ViewContext::default())
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}
