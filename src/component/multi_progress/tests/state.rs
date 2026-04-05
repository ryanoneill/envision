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

#[test]
fn test_default_matches_init() {
    let default_state = MultiProgressState::default();
    let init_state = MultiProgress::init();

    assert_eq!(default_state.is_empty(), init_state.is_empty());
    assert_eq!(default_state.len(), init_state.len());
    assert_eq!(default_state.max_visible(), init_state.max_visible());
    assert_eq!(default_state.scroll_offset(), init_state.scroll_offset());
    assert_eq!(
        default_state.auto_remove_completed(),
        init_state.auto_remove_completed()
    );
    assert_eq!(default_state.title(), init_state.title());
    assert_eq!(
        default_state.show_percentages(),
        init_state.show_percentages()
    );
}

// ========================================
// Items Accessor Tests
// ========================================

#[test]
fn test_items_accessor_empty() {
    let state = MultiProgressState::new();
    assert!(state.items().is_empty());
}

#[test]
fn test_items_accessor_with_items() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");

    let items = state.items();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].id(), "id1");
    assert_eq!(items[0].label(), "Item 1");
    assert_eq!(items[1].id(), "id2");
    assert_eq!(items[1].label(), "Item 2");
}

#[test]
fn test_len_and_is_empty() {
    let mut state = MultiProgressState::new();
    assert_eq!(state.len(), 0);
    assert!(state.is_empty());

    state.add("id1", "Item 1");
    assert_eq!(state.len(), 1);
    assert!(!state.is_empty());

    state.add("id2", "Item 2");
    assert_eq!(state.len(), 2);

    state.remove("id1");
    assert_eq!(state.len(), 1);

    state.remove("id2");
    assert_eq!(state.len(), 0);
    assert!(state.is_empty());
}

// ========================================
// Builder Chaining Tests
// ========================================

#[test]
fn test_builder_chaining() {
    let state = MultiProgressState::new()
        .with_max_visible(5)
        .with_auto_remove(true)
        .with_title("Downloads")
        .with_percentages(false);

    assert_eq!(state.max_visible(), 5);
    assert!(state.auto_remove_completed());
    assert_eq!(state.title(), Some("Downloads"));
    assert!(!state.show_percentages());
}

// ========================================
// Scroll Offset Edge Cases
// ========================================

#[test]
fn test_scroll_offset_empty_list() {
    let mut state = MultiProgressState::new();
    state.set_scroll_offset(5);
    // With no items, saturating_sub(1) on 0 is 0
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_offset_single_item() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.set_scroll_offset(5);
    assert_eq!(state.scroll_offset(), 0); // Only valid offset is 0
}

#[test]
fn test_scroll_offset_exact_boundary() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");
    state.add("id3", "Item 3");

    state.set_scroll_offset(2); // Last valid index
    assert_eq!(state.scroll_offset(), 2);

    state.set_scroll_offset(3); // Out of bounds
    assert_eq!(state.scroll_offset(), 2);
}

// ========================================
// Count Tests with Mixed States
// ========================================

#[test]
fn test_counts_with_mixed_statuses() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");
    state.add("id3", "Item 3");
    state.add("id4", "Item 4");
    state.add("id5", "Item 5");

    // Item 1: Active
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetStatus {
            id: "id1".to_string(),
            status: ProgressItemStatus::Active,
        },
    );
    // Item 2: Completed
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id2".to_string()),
    );
    // Item 3: Failed
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Fail {
            id: "id3".to_string(),
            message: None,
        },
    );
    // Item 4: Completed
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id4".to_string()),
    );
    // Item 5: stays Pending

    assert_eq!(state.active_count(), 1);
    assert_eq!(state.completed_count(), 2);
    assert_eq!(state.failed_count(), 1);
    assert_eq!(state.len(), 5);
}

#[test]
fn test_counts_empty() {
    let state = MultiProgressState::new();
    assert_eq!(state.active_count(), 0);
    assert_eq!(state.completed_count(), 0);
    assert_eq!(state.failed_count(), 0);
}

// ========================================
// Add Item with Various Types
// ========================================

#[test]
fn test_add_with_string_types() {
    let mut state = MultiProgressState::new();
    assert!(state.add(String::from("id1"), String::from("Label 1")));
    assert!(state.add("id2", "Label 2"));
    assert_eq!(state.len(), 2);
}

#[test]
fn test_add_multiple_items() {
    let mut state = MultiProgressState::new();
    for i in 0..20 {
        assert!(state.add(format!("id{}", i), format!("Item {}", i)));
    }
    assert_eq!(state.len(), 20);
}

// ========================================
// Clear Resets Scroll
// ========================================

#[test]
fn test_clear_resets_scroll_offset() {
    let mut state = MultiProgressState::new();
    for i in 0..10 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }
    state.set_scroll_offset(5);
    assert_eq!(state.scroll_offset(), 5);

    state.clear();
    assert_eq!(state.scroll_offset(), 0);
    assert!(state.is_empty());
}

// ========================================
// Clone and Debug Tests
// ========================================

#[test]
fn test_state_clone() {
    let mut state = MultiProgressState::new()
        .with_title("Test")
        .with_max_visible(5);
    state.add("id1", "Item 1");

    let cloned = state.clone();
    assert_eq!(cloned.title(), Some("Test"));
    assert_eq!(cloned.max_visible(), 5);
    assert_eq!(cloned.len(), 1);
}

#[test]
fn test_state_debug() {
    let state = MultiProgressState::new().with_title("Test");
    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("Test"));
}

// ========================================
// Overall Progress Edge Cases
// ========================================

#[test]
fn test_overall_progress_single_item() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: 0.75,
        },
    );

    assert_eq!(state.overall_progress(), 0.75);
}

#[test]
fn test_overall_progress_all_complete() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");

    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id1".to_string()),
    );
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id2".to_string()),
    );

    assert_eq!(state.overall_progress(), 1.0);
}
