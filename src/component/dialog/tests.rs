use super::*;

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
// Mutator Tests
// ========================================

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
fn test_set_primary_clamps() {
    let mut state = DialogState::alert("T", "M");
    state.set_primary_button(10);
    assert_eq!(state.primary_button(), 0);
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
    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Dialog::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========================================
// View Tests
// ========================================

#[test]
fn test_view_renders() {
    let mut state = DialogState::alert("Test Title", "Test message content.");
    Dialog::show(&mut state);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Dialog::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_title() {
    let mut state = DialogState::alert("My Dialog Title", "Message");
    Dialog::show(&mut state);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Dialog::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_message() {
    let mut state = DialogState::alert("Title", "This is the message content.");
    Dialog::show(&mut state);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Dialog::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_buttons() {
    let mut state = DialogState::confirm("Title", "Message");
    Dialog::show(&mut state);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Dialog::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused_button() {
    let mut state = DialogState::confirm("Title", "Message");
    Dialog::show(&mut state);
    Dialog::focus(&mut state);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Dialog::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_primary_button() {
    let mut state = DialogState::confirm("Title", "Message");
    Dialog::show(&mut state);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Dialog::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_multiline_message() {
    let mut state = DialogState::alert("Title", "Line 1\nLine 2\nLine 3");
    Dialog::show(&mut state);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Dialog::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========================================
// Integration Tests
// ========================================

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
