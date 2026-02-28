    use super::*;
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    // ========================================
    // StatusLogLevel Tests
    // ========================================

    #[test]
    fn test_level_default() {
        let level = StatusLogLevel::default();
        assert_eq!(level, StatusLogLevel::Info);
    }

    #[test]
    fn test_level_colors() {
        assert_eq!(StatusLogLevel::Info.color(), Color::Cyan);
        assert_eq!(StatusLogLevel::Success.color(), Color::Green);
        assert_eq!(StatusLogLevel::Warning.color(), Color::Yellow);
        assert_eq!(StatusLogLevel::Error.color(), Color::Red);
    }

    #[test]
    fn test_level_prefixes() {
        assert_eq!(StatusLogLevel::Info.prefix(), "ℹ");
        assert_eq!(StatusLogLevel::Success.prefix(), "✓");
        assert_eq!(StatusLogLevel::Warning.prefix(), "⚠");
        assert_eq!(StatusLogLevel::Error.prefix(), "✗");
    }

    // ========================================
    // StatusLogEntry Tests
    // ========================================

    #[test]
    fn test_entry_new() {
        let entry = StatusLogEntry::new(1, "Test message", StatusLogLevel::Info);
        assert_eq!(entry.id(), 1);
        assert_eq!(entry.message(), "Test message");
        assert_eq!(entry.level(), StatusLogLevel::Info);
        assert!(entry.timestamp().is_none());
    }

    #[test]
    fn test_entry_with_timestamp() {
        let entry =
            StatusLogEntry::with_timestamp(2, "Message", StatusLogLevel::Success, "12:34:56");
        assert_eq!(entry.id(), 2);
        assert_eq!(entry.timestamp(), Some("12:34:56"));
    }

    // ========================================
    // State Creation Tests
    // ========================================

    #[test]
    fn test_state_new() {
        let state = StatusLogState::new();
        assert!(state.is_empty());
        assert_eq!(state.max_entries(), 50);
        assert!(!state.show_timestamps());
    }

    #[test]
    fn test_state_with_max_entries() {
        let state = StatusLogState::new().with_max_entries(100);
        assert_eq!(state.max_entries(), 100);
    }

    #[test]
    fn test_state_with_timestamps() {
        let state = StatusLogState::new().with_timestamps(true);
        assert!(state.show_timestamps());
    }

    #[test]
    fn test_state_with_title() {
        let state = StatusLogState::new().with_title("Log");
        assert_eq!(state.title(), Some("Log"));
    }

    #[test]
    fn test_state_default() {
        let state = StatusLogState::default();
        assert!(state.is_empty());
    }

    // ========================================
    // Convenience Method Tests
    // ========================================

    #[test]
    fn test_info() {
        let mut state = StatusLogState::new();
        let id = state.info("Info message");
        assert_eq!(state.len(), 1);
        assert_eq!(state.entries()[0].level(), StatusLogLevel::Info);
        assert_eq!(id, 0);
    }

    #[test]
    fn test_success() {
        let mut state = StatusLogState::new();
        state.success("Success message");
        assert_eq!(state.entries()[0].level(), StatusLogLevel::Success);
    }

    #[test]
    fn test_warning() {
        let mut state = StatusLogState::new();
        state.warning("Warning message");
        assert_eq!(state.entries()[0].level(), StatusLogLevel::Warning);
    }

    #[test]
    fn test_error() {
        let mut state = StatusLogState::new();
        state.error("Error message");
        assert_eq!(state.entries()[0].level(), StatusLogLevel::Error);
    }

    #[test]
    fn test_info_with_timestamp() {
        let mut state = StatusLogState::new();
        state.info_with_timestamp("Message", "10:00:00");
        assert_eq!(state.entries()[0].timestamp(), Some("10:00:00"));
    }

    #[test]
    fn test_success_with_timestamp() {
        let mut state = StatusLogState::new();
        state.success_with_timestamp("Message", "10:00:01");
        assert_eq!(state.entries()[0].timestamp(), Some("10:00:01"));
    }

    #[test]
    fn test_warning_with_timestamp() {
        let mut state = StatusLogState::new();
        state.warning_with_timestamp("Message", "10:00:02");
        assert_eq!(state.entries()[0].timestamp(), Some("10:00:02"));
    }

    #[test]
    fn test_error_with_timestamp() {
        let mut state = StatusLogState::new();
        state.error_with_timestamp("Message", "10:00:03");
        assert_eq!(state.entries()[0].timestamp(), Some("10:00:03"));
    }

    // ========================================
    // ID Generation Tests
    // ========================================

    #[test]
    fn test_id_increment() {
        let mut state = StatusLogState::new();
        let id1 = state.info("First");
        let id2 = state.info("Second");
        let id3 = state.info("Third");

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
    }

    // ========================================
    // Max Entries Tests
    // ========================================

    #[test]
    fn test_max_entries_enforcement() {
        let mut state = StatusLogState::new().with_max_entries(3);

        state.info("One");
        state.info("Two");
        state.info("Three");
        assert_eq!(state.len(), 3);

        // Adding fourth should evict first via update
        let output = StatusLog::update(
            &mut state,
            StatusLogMessage::Push {
                message: "Four".to_string(),
                level: StatusLogLevel::Info,
                timestamp: None,
            },
        );

        assert_eq!(state.len(), 3);
        assert_eq!(output, Some(StatusLogOutput::Evicted(0)));
    }

    #[test]
    fn test_set_max_entries() {
        let mut state = StatusLogState::new();
        state.set_max_entries(10);
        assert_eq!(state.max_entries(), 10);
    }

    // ========================================
    // Accessor Tests
    // ========================================

    #[test]
    fn test_entries() {
        let mut state = StatusLogState::new();
        state.info("A");
        state.info("B");
        assert_eq!(state.entries().len(), 2);
    }

    #[test]
    fn test_entries_newest_first() {
        let mut state = StatusLogState::new();
        state.info("First");
        state.info("Second");
        state.info("Third");

        let messages: Vec<_> = state.entries_newest_first().map(|e| e.message()).collect();
        assert_eq!(messages, vec!["Third", "Second", "First"]);
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut state = StatusLogState::new();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);

        state.info("Message");
        assert!(!state.is_empty());
        assert_eq!(state.len(), 1);
    }

    // ========================================
    // Mutator Tests
    // ========================================

    #[test]
    fn test_remove() {
        let mut state = StatusLogState::new();
        let id = state.info("To remove");
        assert!(state.remove(id));
        assert!(state.is_empty());
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut state = StatusLogState::new();
        state.info("Message");
        assert!(!state.remove(999));
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut state = StatusLogState::new();
        state.info("A");
        state.info("B");
        state.clear();
        assert!(state.is_empty());
    }

    #[test]
    fn test_set_show_timestamps() {
        let mut state = StatusLogState::new();
        state.set_show_timestamps(true);
        assert!(state.show_timestamps());
    }

    #[test]
    fn test_set_title() {
        let mut state = StatusLogState::new();
        state.set_title(Some("New Title".to_string()));
        assert_eq!(state.title(), Some("New Title"));

        state.set_title(None);
        assert!(state.title().is_none());
    }

    // ========================================
    // Scroll Tests
    // ========================================

    #[test]
    fn test_scroll_offset() {
        let mut state = StatusLogState::new();
        for i in 0..10 {
            state.info(format!("Message {}", i));
        }

        assert_eq!(state.scroll_offset(), 0);

        state.set_scroll_offset(5);
        assert_eq!(state.scroll_offset(), 5);
    }

    #[test]
    fn test_scroll_offset_clamped() {
        let mut state = StatusLogState::new();
        state.info("A");
        state.info("B");

        state.set_scroll_offset(100);
        assert_eq!(state.scroll_offset(), 1); // Clamped to max
    }

    // ========================================
    // Component Tests
    // ========================================

    #[test]
    fn test_init() {
        let state = StatusLog::init();
        assert!(state.is_empty());
    }

    #[test]
    fn test_update_push() {
        let mut state = StatusLog::init();
        let output = StatusLog::update(
            &mut state,
            StatusLogMessage::Push {
                message: "Test".to_string(),
                level: StatusLogLevel::Info,
                timestamp: None,
            },
        );
        assert_eq!(state.len(), 1);
        assert_eq!(output, Some(StatusLogOutput::Added(0)));
    }

    #[test]
    fn test_update_push_with_timestamp() {
        let mut state = StatusLog::init();
        StatusLog::update(
            &mut state,
            StatusLogMessage::Push {
                message: "Test".to_string(),
                level: StatusLogLevel::Success,
                timestamp: Some("12:00".to_string()),
            },
        );
        assert_eq!(state.entries()[0].timestamp(), Some("12:00"));
    }

    #[test]
    fn test_update_clear() {
        let mut state = StatusLogState::new();
        state.info("A");

        let output = StatusLog::update(&mut state, StatusLogMessage::Clear);
        assert!(state.is_empty());
        assert_eq!(output, Some(StatusLogOutput::Cleared));
    }

    #[test]
    fn test_update_clear_empty() {
        let mut state = StatusLog::init();
        let output = StatusLog::update(&mut state, StatusLogMessage::Clear);
        assert!(output.is_none());
    }

    #[test]
    fn test_update_remove() {
        let mut state = StatusLogState::new();
        let id = state.info("To remove");

        let output = StatusLog::update(&mut state, StatusLogMessage::Remove(id));
        assert!(state.is_empty());
        assert_eq!(output, Some(StatusLogOutput::Removed(id)));
    }

    #[test]
    fn test_update_remove_nonexistent() {
        let mut state = StatusLogState::new();
        state.info("Keep");

        let output = StatusLog::update(&mut state, StatusLogMessage::Remove(999));
        assert!(output.is_none());
    }

    #[test]
    fn test_update_scroll_up() {
        let mut state = StatusLogState::new();
        for i in 0..5 {
            state.info(format!("Msg {}", i));
        }
        state.set_scroll_offset(3);

        StatusLog::update(&mut state, StatusLogMessage::ScrollUp);
        assert_eq!(state.scroll_offset(), 2);
    }

    #[test]
    fn test_update_scroll_up_at_top() {
        let mut state = StatusLogState::new();
        state.info("A");

        StatusLog::update(&mut state, StatusLogMessage::ScrollUp);
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_update_scroll_down() {
        let mut state = StatusLogState::new();
        for i in 0..5 {
            state.info(format!("Msg {}", i));
        }

        StatusLog::update(&mut state, StatusLogMessage::ScrollDown);
        assert_eq!(state.scroll_offset(), 1);
    }

    #[test]
    fn test_update_scroll_down_at_bottom() {
        let mut state = StatusLogState::new();
        state.info("A");
        state.info("B");
        state.set_scroll_offset(1);

        StatusLog::update(&mut state, StatusLogMessage::ScrollDown);
        assert_eq!(state.scroll_offset(), 1); // Can't go further
    }

    #[test]
    fn test_update_scroll_to_top() {
        let mut state = StatusLogState::new();
        for i in 0..5 {
            state.info(format!("Msg {}", i));
        }
        state.set_scroll_offset(3);

        StatusLog::update(&mut state, StatusLogMessage::ScrollToTop);
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_update_scroll_to_bottom() {
        let mut state = StatusLogState::new();
        for i in 0..5 {
            state.info(format!("Msg {}", i));
        }

        StatusLog::update(&mut state, StatusLogMessage::ScrollToBottom);
        assert_eq!(state.scroll_offset(), 4);
    }

    // ========================================
    // Focusable Tests
    // ========================================

    #[test]
    fn test_focusable_is_focused() {
        let state = StatusLogState::new();
        assert!(!StatusLog::is_focused(&state));
    }

    #[test]
    fn test_focusable_set_focused() {
        let mut state = StatusLogState::new();
        StatusLog::set_focused(&mut state, true);
        assert!(StatusLog::is_focused(&state));
    }

    #[test]
    fn test_focusable_focus_blur() {
        let mut state = StatusLogState::new();
        StatusLog::focus(&mut state);
        assert!(StatusLog::is_focused(&state));

        StatusLog::blur(&mut state);
        assert!(!StatusLog::is_focused(&state));
    }

    // ========================================
    // View Tests
    // ========================================

    #[test]
    fn test_view_empty() {
        let state = StatusLogState::new();
        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| StatusLog::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        // Should render border only
        let output = terminal.backend().to_string();
        assert!(output.contains("─") || output.contains("│"));
    }

    #[test]
    fn test_view_with_messages() {
        let mut state = StatusLogState::new();
        state.info("Info message");
        state.success("Success message");

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| StatusLog::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        // Newest first, so success should appear before info
        assert!(output.contains("Success message"));
        assert!(output.contains("Info message"));
    }

    #[test]
    fn test_view_with_title() {
        let mut state = StatusLogState::new().with_title("Status");
        state.info("Test");

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| StatusLog::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Status"));
    }

    #[test]
    fn test_view_with_timestamps() {
        let mut state = StatusLogState::new().with_timestamps(true);
        state.info_with_timestamp("Message", "12:34");

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| StatusLog::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("12:34"));
        assert!(output.contains("Message"));
    }

    #[test]
    fn test_view_all_levels() {
        let mut state = StatusLogState::new();
        state.info("Info");
        state.success("Success");
        state.warning("Warning");
        state.error("Error");

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| StatusLog::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Info"));
        assert!(output.contains("Success"));
        assert!(output.contains("Warning"));
        assert!(output.contains("Error"));
    }

    #[test]
    fn test_clone() {
        let mut state = StatusLogState::new();
        state.info("Test");

        let cloned = state.clone();
        assert_eq!(cloned.len(), 1);
    }
