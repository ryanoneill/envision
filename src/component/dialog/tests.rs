    use super::*;
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    // ========================================
    // DialogButton Tests
    // ========================================

    #[test]
    fn test_dialog_button_new() {
        let button = DialogButton::new("ok", "OK");
        assert_eq!(button.id(), "ok");
        assert_eq!(button.label(), "OK");
    }

    #[test]
    fn test_dialog_button_clone() {
        let button = DialogButton::new("save", "Save");
        let cloned = button.clone();
        assert_eq!(cloned.id(), "save");
        assert_eq!(cloned.label(), "Save");
    }

    #[test]
    fn test_dialog_button_eq() {
        let button1 = DialogButton::new("ok", "OK");
        let button2 = DialogButton::new("ok", "OK");
        let button3 = DialogButton::new("cancel", "Cancel");
        assert_eq!(button1, button2);
        assert_ne!(button1, button3);
    }

    // ========================================
    // State Creation Tests
    // ========================================

    #[test]
    fn test_new() {
        let buttons = vec![
            DialogButton::new("ok", "OK"),
            DialogButton::new("cancel", "Cancel"),
        ];
        let state = DialogState::new("Title", "Message", buttons);
        assert_eq!(state.title(), "Title");
        assert_eq!(state.message(), "Message");
        assert_eq!(state.buttons().len(), 2);
        assert_eq!(state.primary_button(), 0);
        assert_eq!(state.focused_button(), 0);
        assert!(!Dialog::is_visible(&state));
    }

    #[test]
    fn test_with_primary() {
        let buttons = vec![
            DialogButton::new("cancel", "Cancel"),
            DialogButton::new("ok", "OK"),
        ];
        let state = DialogState::with_primary("Title", "Message", buttons, 1);
        assert_eq!(state.primary_button(), 1);
        assert_eq!(state.focused_button(), 1);
    }

    #[test]
    fn test_with_primary_clamps() {
        let buttons = vec![DialogButton::new("ok", "OK")];
        let state = DialogState::with_primary("Title", "Message", buttons, 10);
        assert_eq!(state.primary_button(), 0);
    }

    #[test]
    fn test_alert() {
        let state = DialogState::alert("Error", "Something went wrong.");
        assert_eq!(state.title(), "Error");
        assert_eq!(state.message(), "Something went wrong.");
        assert_eq!(state.buttons().len(), 1);
        assert_eq!(state.buttons()[0].id(), "ok");
        assert_eq!(state.buttons()[0].label(), "OK");
    }

    #[test]
    fn test_confirm() {
        let state = DialogState::confirm("Delete?", "This cannot be undone.");
        assert_eq!(state.title(), "Delete?");
        assert_eq!(state.buttons().len(), 2);
        assert_eq!(state.buttons()[0].id(), "cancel");
        assert_eq!(state.buttons()[1].id(), "ok");
        assert_eq!(state.primary_button(), 1);
    }

    #[test]
    fn test_default() {
        let state = DialogState::default();
        assert_eq!(state.title(), "");
        assert_eq!(state.message(), "");
        assert!(state.buttons().is_empty());
        assert!(!Dialog::is_visible(&state));
    }

    #[test]
    fn test_new_empty_buttons() {
        let state = DialogState::new("Title", "Message", vec![]);
        assert!(state.buttons().is_empty());
        assert_eq!(state.primary_button(), 0);
        assert_eq!(state.focused_button(), 0);
    }

    // ========================================
    // Accessor Tests
    // ========================================

    #[test]
    fn test_title() {
        let state = DialogState::alert("My Title", "Message");
        assert_eq!(state.title(), "My Title");
    }

    #[test]
    fn test_message() {
        let state = DialogState::alert("Title", "My Message");
        assert_eq!(state.message(), "My Message");
    }

    #[test]
    fn test_buttons() {
        let buttons = vec![DialogButton::new("a", "A"), DialogButton::new("b", "B")];
        let state = DialogState::new("T", "M", buttons);
        assert_eq!(state.buttons().len(), 2);
        assert_eq!(state.buttons()[0].id(), "a");
        assert_eq!(state.buttons()[1].id(), "b");
    }

    #[test]
    fn test_primary_button() {
        let buttons = vec![DialogButton::new("a", "A"), DialogButton::new("b", "B")];
        let state = DialogState::with_primary("T", "M", buttons, 1);
        assert_eq!(state.primary_button(), 1);
    }

    #[test]
    fn test_focused_button() {
        let state = DialogState::confirm("T", "M");
        assert_eq!(state.focused_button(), 1); // Primary is 1, so focus starts there
    }

    // ========================================
    // Mutator Tests
    // ========================================

    #[test]
    fn test_set_title() {
        let mut state = DialogState::alert("Old", "Message");
        state.set_title("New");
        assert_eq!(state.title(), "New");
    }

    #[test]
    fn test_set_message() {
        let mut state = DialogState::alert("Title", "Old");
        state.set_message("New");
        assert_eq!(state.message(), "New");
    }

    #[test]
    fn test_set_buttons() {
        let mut state = DialogState::alert("Title", "Message");
        let new_buttons = vec![
            DialogButton::new("yes", "Yes"),
            DialogButton::new("no", "No"),
        ];
        state.set_buttons(new_buttons);
        assert_eq!(state.buttons().len(), 2);
        assert_eq!(state.buttons()[0].id(), "yes");
    }

    #[test]
    fn test_set_buttons_resets_focus() {
        let mut state = DialogState::with_primary(
            "T",
            "M",
            vec![
                DialogButton::new("a", "A"),
                DialogButton::new("b", "B"),
                DialogButton::new("c", "C"),
            ],
            2,
        );
        assert_eq!(state.focused_button(), 2);

        // Set new buttons - focus should reset to clamped primary
        state.set_buttons(vec![DialogButton::new("x", "X")]);
        assert_eq!(state.primary_button(), 0);
        assert_eq!(state.focused_button(), 0);
    }

    #[test]
    fn test_set_primary_button() {
        let mut state = DialogState::confirm("T", "M");
        state.set_primary_button(0);
        assert_eq!(state.primary_button(), 0);
    }

    #[test]
    fn test_set_primary_clamps() {
        let mut state = DialogState::alert("T", "M");
        state.set_primary_button(10);
        assert_eq!(state.primary_button(), 0);
    }

    // ========================================
    // Visibility (Toggleable) Tests
    // ========================================

    #[test]
    fn test_is_visible() {
        let state = DialogState::alert("T", "M");
        assert!(!Dialog::is_visible(&state));
    }

    #[test]
    fn test_set_visible() {
        let mut state = DialogState::alert("T", "M");
        Dialog::set_visible(&mut state, true);
        assert!(Dialog::is_visible(&state));
        Dialog::set_visible(&mut state, false);
        assert!(!Dialog::is_visible(&state));
    }

    #[test]
    fn test_show() {
        let mut state = DialogState::alert("T", "M");
        Dialog::show(&mut state);
        assert!(Dialog::is_visible(&state));
    }

    #[test]
    fn test_hide() {
        let mut state = DialogState::alert("T", "M");
        Dialog::show(&mut state);
        Dialog::hide(&mut state);
        assert!(!Dialog::is_visible(&state));
    }

    #[test]
    fn test_toggle() {
        let mut state = DialogState::alert("T", "M");
        assert!(!Dialog::is_visible(&state));
        Dialog::toggle(&mut state);
        assert!(Dialog::is_visible(&state));
        Dialog::toggle(&mut state);
        assert!(!Dialog::is_visible(&state));
    }

    // ========================================
    // Focus (Focusable) Tests
    // ========================================

    #[test]
    fn test_is_focused() {
        let state = DialogState::alert("T", "M");
        assert!(!Dialog::is_focused(&state));
    }

    #[test]
    fn test_set_focused() {
        let mut state = DialogState::alert("T", "M");
        Dialog::set_focused(&mut state, true);
        assert!(Dialog::is_focused(&state));
        Dialog::set_focused(&mut state, false);
        assert!(!Dialog::is_focused(&state));
    }

    #[test]
    fn test_focus() {
        let mut state = DialogState::alert("T", "M");
        Dialog::focus(&mut state);
        assert!(Dialog::is_focused(&state));
    }

    #[test]
    fn test_blur() {
        let mut state = DialogState::alert("T", "M");
        Dialog::focus(&mut state);
        Dialog::blur(&mut state);
        assert!(!Dialog::is_focused(&state));
    }

    // ========================================
    // Navigation Tests
    // ========================================

    #[test]
    fn test_focus_next() {
        let mut state = DialogState::confirm("T", "M");
        Dialog::show(&mut state);
        // Start at primary (1 = OK)
        assert_eq!(state.focused_button(), 1);

        Dialog::update(&mut state, DialogMessage::FocusNext);
        assert_eq!(state.focused_button(), 0); // Wraps to Cancel
    }

    #[test]
    fn test_focus_next_wraps() {
        let buttons = vec![
            DialogButton::new("a", "A"),
            DialogButton::new("b", "B"),
            DialogButton::new("c", "C"),
        ];
        let mut state = DialogState::with_primary("T", "M", buttons, 2);
        Dialog::show(&mut state);
        assert_eq!(state.focused_button(), 2);

        Dialog::update(&mut state, DialogMessage::FocusNext);
        assert_eq!(state.focused_button(), 0);
    }

    #[test]
    fn test_focus_prev() {
        let mut state = DialogState::confirm("T", "M");
        Dialog::show(&mut state);
        assert_eq!(state.focused_button(), 1);

        Dialog::update(&mut state, DialogMessage::FocusPrev);
        assert_eq!(state.focused_button(), 0);
    }

    #[test]
    fn test_focus_prev_wraps() {
        let mut state = DialogState::confirm("T", "M");
        Dialog::show(&mut state);
        Dialog::update(&mut state, DialogMessage::FocusPrev); // 1 -> 0
        Dialog::update(&mut state, DialogMessage::FocusPrev); // 0 -> 1 (wrap)
        assert_eq!(state.focused_button(), 1);
    }

    #[test]
    fn test_focus_empty() {
        let mut state = DialogState::new("T", "M", vec![]);
        Dialog::show(&mut state);

        // Should not panic
        Dialog::update(&mut state, DialogMessage::FocusNext);
        Dialog::update(&mut state, DialogMessage::FocusPrev);
        assert_eq!(state.focused_button(), 0);
    }

    // ========================================
    // Button Press Tests
    // ========================================

    #[test]
    fn test_press() {
        let mut state = DialogState::alert("T", "M");
        Dialog::show(&mut state);

        let output = Dialog::update(&mut state, DialogMessage::Press);
        assert_eq!(output, Some(DialogOutput::ButtonPressed("ok".into())));
    }

    #[test]
    fn test_press_hides_dialog() {
        let mut state = DialogState::alert("T", "M");
        Dialog::show(&mut state);
        assert!(Dialog::is_visible(&state));

        Dialog::update(&mut state, DialogMessage::Press);
        assert!(!Dialog::is_visible(&state));
    }

    #[test]
    fn test_press_empty() {
        let mut state = DialogState::new("T", "M", vec![]);
        Dialog::show(&mut state);

        let output = Dialog::update(&mut state, DialogMessage::Press);
        assert_eq!(output, None);
    }

    // ========================================
    // Close Tests
    // ========================================

    #[test]
    fn test_close() {
        let mut state = DialogState::alert("T", "M");
        Dialog::show(&mut state);

        let output = Dialog::update(&mut state, DialogMessage::Close);
        assert_eq!(output, Some(DialogOutput::Closed));
    }

    #[test]
    fn test_close_hides_dialog() {
        let mut state = DialogState::alert("T", "M");
        Dialog::show(&mut state);
        assert!(Dialog::is_visible(&state));

        Dialog::update(&mut state, DialogMessage::Close);
        assert!(!Dialog::is_visible(&state));
    }

    // ========================================
    // Open Tests
    // ========================================

    #[test]
    fn test_open() {
        let mut state = DialogState::confirm("T", "M");
        assert!(!Dialog::is_visible(&state));

        Dialog::update(&mut state, DialogMessage::Open);
        assert!(Dialog::is_visible(&state));
        assert_eq!(state.focused_button(), 1); // Focuses primary
    }

    #[test]
    fn test_open_when_visible() {
        let mut state = DialogState::alert("T", "M");
        Dialog::show(&mut state);

        // Open when already visible should be a no-op
        let output = Dialog::update(&mut state, DialogMessage::Open);
        assert_eq!(output, None);
        assert!(Dialog::is_visible(&state));
    }

    // ========================================
    // Hidden State Tests
    // ========================================

    #[test]
    fn test_update_when_hidden() {
        let mut state = DialogState::confirm("T", "M");
        assert!(!Dialog::is_visible(&state));

        // All messages except Open should be ignored when hidden
        let output = Dialog::update(&mut state, DialogMessage::FocusNext);
        assert_eq!(output, None);

        let output = Dialog::update(&mut state, DialogMessage::FocusPrev);
        assert_eq!(output, None);

        let output = Dialog::update(&mut state, DialogMessage::Press);
        assert_eq!(output, None);

        let output = Dialog::update(&mut state, DialogMessage::Close);
        assert_eq!(output, None);
    }

    #[test]
    fn test_view_when_hidden() {
        let state = DialogState::alert("Title", "Message");
        let theme = Theme::default();

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dialog::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        // Should not contain dialog content when hidden
        assert!(!output.contains("Title"));
        assert!(!output.contains("Message"));
    }

    // ========================================
    // View Tests
    // ========================================

    #[test]
    fn test_view_renders() {
        let mut state = DialogState::alert("Test Title", "Test message content.");
        Dialog::show(&mut state);
        let theme = Theme::default();

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dialog::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Test Title"));
    }

    #[test]
    fn test_view_title() {
        let mut state = DialogState::alert("My Dialog Title", "Message");
        Dialog::show(&mut state);
        let theme = Theme::default();

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dialog::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("My Dialog Title"));
    }

    #[test]
    fn test_view_message() {
        let mut state = DialogState::alert("Title", "This is the message content.");
        Dialog::show(&mut state);
        let theme = Theme::default();

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dialog::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("This is the message content."));
    }

    #[test]
    fn test_view_buttons() {
        let mut state = DialogState::confirm("Title", "Message");
        Dialog::show(&mut state);
        let theme = Theme::default();

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dialog::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Cancel"));
        assert!(output.contains("OK"));
    }

    #[test]
    fn test_view_focused_button() {
        let mut state = DialogState::confirm("Title", "Message");
        Dialog::show(&mut state);
        Dialog::focus(&mut state);
        let theme = Theme::default();

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dialog::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        // Just verify it renders without panicking
        let output = terminal.backend().to_string();
        assert!(output.contains("OK"));
    }

    #[test]
    fn test_view_primary_button() {
        let mut state = DialogState::confirm("Title", "Message");
        Dialog::show(&mut state);
        let theme = Theme::default();

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dialog::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        // Just verify it renders without panicking
        let output = terminal.backend().to_string();
        assert!(output.contains("OK"));
    }

    #[test]
    fn test_view_multiline_message() {
        let mut state = DialogState::alert("Title", "Line 1\nLine 2\nLine 3");
        Dialog::show(&mut state);
        let theme = Theme::default();

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dialog::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Line 1"));
    }

    // ========================================
    // Integration Tests
    // ========================================

    #[test]
    fn test_clone() {
        let state = DialogState::confirm("Title", "Message");
        let cloned = state.clone();
        assert_eq!(cloned.title(), "Title");
        assert_eq!(cloned.message(), "Message");
        assert_eq!(cloned.buttons().len(), 2);
    }

    #[test]
    fn test_init() {
        let state = Dialog::init();
        assert_eq!(state.title(), "");
        assert_eq!(state.message(), "");
        assert!(state.buttons().is_empty());
    }

    #[test]
    fn test_alert_workflow() {
        let mut state = DialogState::alert("Error", "File not found.");
        Dialog::show(&mut state);
        Dialog::focus(&mut state);

        // Press OK
        let output = Dialog::update(&mut state, DialogMessage::Press);
        assert_eq!(output, Some(DialogOutput::ButtonPressed("ok".into())));
        assert!(!Dialog::is_visible(&state));
    }

    #[test]
    fn test_confirm_workflow() {
        let mut state = DialogState::confirm("Delete?", "This cannot be undone.");
        Dialog::show(&mut state);
        Dialog::focus(&mut state);

        // Start at OK (primary)
        assert_eq!(state.focused_button(), 1);

        // Navigate to Cancel
        Dialog::update(&mut state, DialogMessage::FocusPrev);
        assert_eq!(state.focused_button(), 0);

        // Press Cancel
        let output = Dialog::update(&mut state, DialogMessage::Press);
        assert_eq!(output, Some(DialogOutput::ButtonPressed("cancel".into())));
        assert!(!Dialog::is_visible(&state));
    }

    #[test]
    fn test_custom_workflow() {
        let buttons = vec![
            DialogButton::new("save", "Save"),
            DialogButton::new("discard", "Discard"),
            DialogButton::new("cancel", "Cancel"),
        ];
        let mut state = DialogState::with_primary("Unsaved Changes", "Save your work?", buttons, 0);
        Dialog::show(&mut state);
        Dialog::focus(&mut state);

        // Navigate to Discard
        Dialog::update(&mut state, DialogMessage::FocusNext);
        assert_eq!(state.focused_button(), 1);

        // Press Discard
        let output = Dialog::update(&mut state, DialogMessage::Press);
        assert_eq!(output, Some(DialogOutput::ButtonPressed("discard".into())));
    }

    #[test]
    fn test_show_resets_focus_to_primary() {
        let mut state = DialogState::confirm("T", "M");
        Dialog::show(&mut state);

        // Navigate away from primary
        Dialog::update(&mut state, DialogMessage::FocusPrev);
        assert_eq!(state.focused_button(), 0);

        // Close and reopen
        Dialog::hide(&mut state);
        Dialog::show(&mut state);

        // Focus should be back at primary
        assert_eq!(state.focused_button(), 1);
    }
