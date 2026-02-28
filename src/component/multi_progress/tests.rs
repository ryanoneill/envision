    use super::*;
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    // ========================================
    // ProgressItemStatus Tests
    // ========================================

    #[test]
    fn test_status_default() {
        let status = ProgressItemStatus::default();
        assert_eq!(status, ProgressItemStatus::Pending);
    }

    #[test]
    fn test_status_styles() {
        let theme = Theme::default();
        assert_eq!(
            ProgressItemStatus::Pending.style(&theme),
            theme.disabled_style()
        );
        assert_eq!(ProgressItemStatus::Active.style(&theme), theme.info_style());
        assert_eq!(
            ProgressItemStatus::Completed.style(&theme),
            theme.success_style()
        );
        assert_eq!(
            ProgressItemStatus::Failed.style(&theme),
            theme.error_style()
        );
    }

    #[test]
    fn test_status_symbols() {
        assert_eq!(ProgressItemStatus::Pending.symbol(), "○");
        assert_eq!(ProgressItemStatus::Active.symbol(), "●");
        assert_eq!(ProgressItemStatus::Completed.symbol(), "✓");
        assert_eq!(ProgressItemStatus::Failed.symbol(), "✗");
    }

    // ========================================
    // ProgressItem Tests
    // ========================================

    #[test]
    fn test_item_new() {
        let item = ProgressItem::new("id1", "Label");
        assert_eq!(item.id(), "id1");
        assert_eq!(item.label(), "Label");
        assert_eq!(item.progress(), 0.0);
        assert_eq!(item.status(), ProgressItemStatus::Pending);
        assert!(item.message().is_none());
    }

    #[test]
    fn test_item_percentage() {
        let mut item = ProgressItem::new("id1", "Test");
        item.progress = 0.5;
        assert_eq!(item.percentage(), 50);

        item.progress = 0.0;
        assert_eq!(item.percentage(), 0);

        item.progress = 1.0;
        assert_eq!(item.percentage(), 100);
    }

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

        let output =
            MultiProgress::update(&mut state, MultiProgressMessage::Remove("id1".to_string()));

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

    // ========================================
    // Focusable Tests
    // ========================================

    #[test]
    fn test_focusable() {
        let mut state = MultiProgressState::new();
        assert!(!MultiProgress::is_focused(&state));

        MultiProgress::focus(&mut state);
        assert!(MultiProgress::is_focused(&state));

        MultiProgress::blur(&mut state);
        assert!(!MultiProgress::is_focused(&state));
    }

    // ========================================
    // View Tests
    // ========================================

    #[test]
    fn test_view_empty() {
        let state = MultiProgressState::new();
        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        // Should render border only
        let output = terminal.backend().to_string();
        assert!(output.contains("─") || output.contains("│"));
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

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Item 1"));
        assert!(output.contains("Item 2"));
        assert!(output.contains("50%"));
    }

    #[test]
    fn test_view_with_title() {
        let state = MultiProgressState::new().with_title("Downloads");
        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Downloads"));
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

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Error"));
        assert!(output.contains("Connection lost"));
    }

    #[test]
    fn test_view_completed_item() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        MultiProgress::update(
            &mut state,
            MultiProgressMessage::Complete("id1".to_string()),
        );

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("100%"));
        assert!(output.contains("✓"));
    }

    #[test]
    fn test_clone() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        let cloned = state.clone();
        assert_eq!(cloned.len(), 1);
    }

    // ========================================
    // Additional Coverage Tests
    // ========================================

    #[test]
    fn test_view_zero_size_area() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");
        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        // Test with zero width
        terminal
            .draw(|frame| {
                MultiProgress::view(&state, frame, Rect::new(0, 0, 0, 10), &Theme::default());
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

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Item 1"));
        assert!(!output.contains("%")); // No percentage shown
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

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Error"));
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
