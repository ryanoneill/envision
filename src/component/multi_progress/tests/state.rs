use super::*;

// ========================================
// State Creation Tests
// ========================================

#[test]
fn test_state_new() {
    let state = MultiProgressState::new();
    assert!(state.is_empty());
    assert_eq!(state.max_visible(), 8);
    assert!(!state.auto_remove_completed());
    assert!(state.show_percentages());
}

#[test]
fn test_state_with_max_visible() {
    let state = MultiProgressState::new().with_max_visible(5);
    assert_eq!(state.max_visible(), 5);
}

#[test]
fn test_state_with_auto_remove() {
    let state = MultiProgressState::new().with_auto_remove(true);
    assert!(state.auto_remove_completed());
}

#[test]
fn test_state_with_title() {
    let state = MultiProgressState::new().with_title("Progress");
    assert_eq!(state.title(), Some("Progress"));
}

#[test]
fn test_state_with_percentages() {
    let state = MultiProgressState::new().with_percentages(false);
    assert!(!state.show_percentages());
}

// ========================================
// State Manipulation Tests
// ========================================

#[test]
fn test_add_item() {
    let mut state = MultiProgressState::new();
    assert!(state.add("id1", "Item 1"));
    assert_eq!(state.len(), 1);
    assert!(!state.is_empty());
}

#[test]
fn test_add_duplicate_id() {
    let mut state = MultiProgressState::new();
    assert!(state.add("id1", "Item 1"));
    assert!(!state.add("id1", "Item 1 again")); // Duplicate
    assert_eq!(state.len(), 1);
}

#[test]
fn test_find() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    assert!(state.find("id1").is_some());
    assert!(state.find("nonexistent").is_none());
}

#[test]
fn test_find_mut() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    if let Some(item) = state.find_mut("id1") {
        item.progress = 0.5;
    }

    assert_eq!(state.find("id1").unwrap().progress(), 0.5);
}

#[test]
fn test_remove() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");

    assert!(state.remove("id1"));
    assert_eq!(state.len(), 1);
    assert!(!state.remove("id1")); // Already removed
}

#[test]
fn test_clear() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");

    state.clear();
    assert!(state.is_empty());
}

// ========================================
// Progress Counting Tests
// ========================================

#[test]
fn test_completed_count() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");
    state.add("id3", "Item 3");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id1".to_string()),
    );
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id2".to_string()),
    );

    assert_eq!(state.completed_count(), 2);
}

#[test]
fn test_failed_count() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Fail {
            id: "id1".to_string(),
            message: Some("Error".to_string()),
        },
    );

    assert_eq!(state.failed_count(), 1);
}

#[test]
fn test_active_count() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetStatus {
            id: "id1".to_string(),
            status: ProgressItemStatus::Active,
        },
    );

    assert_eq!(state.active_count(), 1);
}

#[test]
fn test_overall_progress() {
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
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id2".to_string(),
            progress: 1.0,
        },
    );

    assert_eq!(state.overall_progress(), 0.75);
}

#[test]
fn test_overall_progress_empty() {
    let state = MultiProgressState::new();
    assert_eq!(state.overall_progress(), 0.0);
}

// ========================================
// Additional State Tests
// ========================================

#[test]
fn test_set_scroll_offset_clamped() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");

    state.set_scroll_offset(100); // Too large
    assert_eq!(state.scroll_offset(), 1); // Clamped to last valid
}

#[test]
fn test_set_title() {
    let mut state = MultiProgressState::new();
    assert!(state.title().is_none());

    state.set_title(Some("New Title".to_string()));
    assert_eq!(state.title(), Some("New Title"));

    state.set_title(None);
    assert!(state.title().is_none());
}

#[test]
fn test_set_show_percentages() {
    let mut state = MultiProgressState::new();
    assert!(state.show_percentages());

    state.set_show_percentages(false);
    assert!(!state.show_percentages());
}

#[test]
fn test_set_auto_remove_completed() {
    let mut state = MultiProgressState::new();
    assert!(!state.auto_remove_completed());

    state.set_auto_remove_completed(true);
    assert!(state.auto_remove_completed());
}

#[test]
fn test_set_max_visible() {
    let mut state = MultiProgressState::new();
    assert_eq!(state.max_visible(), 8);

    state.set_max_visible(5);
    assert_eq!(state.max_visible(), 5);
}
