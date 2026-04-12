use super::*;
use crate::input::Event;

// Construction tests

#[test]
fn test_new() {
    let state = ConfirmDialogState::new("Title", "Message");
    assert_eq!(state.title(), "Title");
    assert_eq!(state.message(), "Message");
    assert_eq!(state.button_config(), &ButtonConfig::Ok);
    assert!(!state.is_visible());
}

#[test]
fn test_ok() {
    let state = ConfirmDialogState::ok("Info", "Done.");
    assert_eq!(state.button_config(), &ButtonConfig::Ok);
}

#[test]
fn test_ok_cancel() {
    let state = ConfirmDialogState::ok_cancel("Confirm", "Proceed?");
    assert_eq!(state.button_config(), &ButtonConfig::OkCancel);
}

#[test]
fn test_yes_no() {
    let state = ConfirmDialogState::yes_no("Delete?", "Sure?");
    assert_eq!(state.button_config(), &ButtonConfig::YesNo);
}

#[test]
fn test_yes_no_cancel() {
    let state = ConfirmDialogState::yes_no_cancel("Save?", "Save changes?");
    assert_eq!(state.button_config(), &ButtonConfig::YesNoCancel);
}

#[test]
fn test_destructive() {
    let state = ConfirmDialogState::destructive("Delete?", "Sure?", ButtonConfig::YesNo, 0);
    assert_eq!(state.destructive_button(), Some(0));
}

#[test]
fn test_with_button_config() {
    let state = ConfirmDialogState::new("T", "M").with_button_config(ButtonConfig::YesNo);
    assert_eq!(state.button_config(), &ButtonConfig::YesNo);
}

#[test]
fn test_with_destructive_button() {
    let state = ConfirmDialogState::new("T", "M").with_destructive_button(Some(1));
    assert_eq!(state.destructive_button(), Some(1));
}

#[test]
fn test_default() {
    let state = ConfirmDialogState::default();
    assert_eq!(state.title(), "");
    assert_eq!(state.message(), "");
    assert_eq!(state.button_config(), &ButtonConfig::Ok);
}

// Focus cycling tests

#[test]
fn test_focus_next_ok_cancel() {
    let mut state = ConfirmDialogState::ok_cancel("T", "M").with_visible(true);
    assert_eq!(state.focused_button(), 0);

    ConfirmDialog::update(&mut state, ConfirmDialogMessage::FocusNext);
    assert_eq!(state.focused_button(), 1);

    // Wraps around
    ConfirmDialog::update(&mut state, ConfirmDialogMessage::FocusNext);
    assert_eq!(state.focused_button(), 0);
}

#[test]
fn test_focus_prev_ok_cancel() {
    let mut state = ConfirmDialogState::ok_cancel("T", "M").with_visible(true);
    assert_eq!(state.focused_button(), 0);

    // Wraps to last
    ConfirmDialog::update(&mut state, ConfirmDialogMessage::FocusPrev);
    assert_eq!(state.focused_button(), 1);

    ConfirmDialog::update(&mut state, ConfirmDialogMessage::FocusPrev);
    assert_eq!(state.focused_button(), 0);
}

#[test]
fn test_focus_cycling_yes_no_cancel() {
    let mut state = ConfirmDialogState::yes_no_cancel("T", "M").with_visible(true);

    ConfirmDialog::update(&mut state, ConfirmDialogMessage::FocusNext);
    assert_eq!(state.focused_button(), 1);

    ConfirmDialog::update(&mut state, ConfirmDialogMessage::FocusNext);
    assert_eq!(state.focused_button(), 2);

    ConfirmDialog::update(&mut state, ConfirmDialogMessage::FocusNext);
    assert_eq!(state.focused_button(), 0);
}

// Press tests

#[test]
fn test_press_ok() {
    let mut state = ConfirmDialogState::ok("T", "M").with_visible(true);
    let output = ConfirmDialog::update(&mut state, ConfirmDialogMessage::Press);
    assert_eq!(
        output,
        Some(ConfirmDialogOutput::Confirmed(ConfirmDialogResult::Ok))
    );
    assert!(!state.is_visible());
}

#[test]
fn test_press_yes_no() {
    let mut state = ConfirmDialogState::yes_no("T", "M").with_visible(true);

    // First button is Yes
    let output = ConfirmDialog::update(&mut state, ConfirmDialogMessage::Press);
    assert_eq!(
        output,
        Some(ConfirmDialogOutput::Confirmed(ConfirmDialogResult::Yes))
    );
}

#[test]
fn test_press_no() {
    let mut state = ConfirmDialogState::yes_no("T", "M").with_visible(true);
    ConfirmDialog::update(&mut state, ConfirmDialogMessage::FocusNext);

    let output = ConfirmDialog::update(&mut state, ConfirmDialogMessage::Press);
    assert_eq!(
        output,
        Some(ConfirmDialogOutput::Confirmed(ConfirmDialogResult::No))
    );
}

#[test]
fn test_press_cancel_in_ok_cancel() {
    let mut state = ConfirmDialogState::ok_cancel("T", "M").with_visible(true);
    ConfirmDialog::update(&mut state, ConfirmDialogMessage::FocusNext);

    let output = ConfirmDialog::update(&mut state, ConfirmDialogMessage::Press);
    assert_eq!(
        output,
        Some(ConfirmDialogOutput::Confirmed(ConfirmDialogResult::Cancel))
    );
}

// Keyboard shortcut tests

#[test]
fn test_y_shortcut_yes_no() {
    let state = ConfirmDialogState::yes_no("T", "M").with_visible(true);

    let msg = ConfirmDialog::handle_event(
        &state,
        &Event::char('y'),
        &EventContext::new().focused(true),
    );
    assert_eq!(
        msg,
        Some(ConfirmDialogMessage::SelectResult(ConfirmDialogResult::Yes))
    );
}

#[test]
fn test_n_shortcut_yes_no() {
    let state = ConfirmDialogState::yes_no("T", "M").with_visible(true);

    let msg = ConfirmDialog::handle_event(
        &state,
        &Event::char('n'),
        &EventContext::new().focused(true),
    );
    assert_eq!(
        msg,
        Some(ConfirmDialogMessage::SelectResult(ConfirmDialogResult::No))
    );
}

#[test]
fn test_y_shortcut_uppercase() {
    let state = ConfirmDialogState::yes_no("T", "M").with_visible(true);

    let msg = ConfirmDialog::handle_event(
        &state,
        &Event::char('Y'),
        &EventContext::new().focused(true),
    );
    assert_eq!(
        msg,
        Some(ConfirmDialogMessage::SelectResult(ConfirmDialogResult::Yes))
    );
}

#[test]
fn test_y_shortcut_not_in_ok_config() {
    let state = ConfirmDialogState::ok("T", "M").with_visible(true);

    let msg = ConfirmDialog::handle_event(
        &state,
        &Event::char('y'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_tab_key() {
    let state = ConfirmDialogState::ok_cancel("T", "M").with_visible(true);

    let msg = ConfirmDialog::handle_event(
        &state,
        &Event::key(KeyCode::Tab),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(ConfirmDialogMessage::FocusNext));
}

#[test]
fn test_backtab_key() {
    let state = ConfirmDialogState::ok_cancel("T", "M").with_visible(true);

    let msg = ConfirmDialog::handle_event(
        &state,
        &Event::key(KeyCode::BackTab),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(ConfirmDialogMessage::FocusPrev));
}

#[test]
fn test_enter_key() {
    let state = ConfirmDialogState::ok("T", "M").with_visible(true);

    let msg = ConfirmDialog::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(ConfirmDialogMessage::Press));
}

#[test]
fn test_esc_key() {
    let state = ConfirmDialogState::ok("T", "M").with_visible(true);

    let msg = ConfirmDialog::handle_event(
        &state,
        &Event::key(KeyCode::Esc),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(ConfirmDialogMessage::Close));
}

// Close tests

#[test]
fn test_close() {
    let mut state = ConfirmDialogState::ok("T", "M").with_visible(true);
    let output = ConfirmDialog::update(&mut state, ConfirmDialogMessage::Close);
    assert_eq!(output, Some(ConfirmDialogOutput::Closed));
    assert!(!state.is_visible());
}

// Open tests

#[test]
fn test_open() {
    let mut state = ConfirmDialogState::ok("T", "M");
    assert!(!state.is_visible());

    ConfirmDialog::update(&mut state, ConfirmDialogMessage::Open);
    assert!(state.is_visible());
    assert_eq!(state.focused_button(), 0);
}

#[test]
fn test_open_resets_focus() {
    let mut state = ConfirmDialogState::ok_cancel("T", "M").with_visible(true);
    ConfirmDialog::update(&mut state, ConfirmDialogMessage::FocusNext);
    assert_eq!(state.focused_button(), 1);

    // Close and reopen
    ConfirmDialog::update(&mut state, ConfirmDialogMessage::Close);
    ConfirmDialog::update(&mut state, ConfirmDialogMessage::Open);
    assert_eq!(state.focused_button(), 0);
}

// Guard tests

#[test]
fn test_not_visible_ignores_events() {
    let state = ConfirmDialogState::ok("T", "M");
    let msg = ConfirmDialog::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &EventContext::default(),
    );
    assert_eq!(msg, None);
}

// Toggleable/Focusable trait tests

#[test]
fn test_toggleable_show() {
    let mut state = ConfirmDialogState::ok("T", "M");
    ConfirmDialog::show(&mut state);
    assert!(state.is_visible());
}

#[test]
fn test_toggleable_hide() {
    let mut state = ConfirmDialogState::ok("T", "M").with_visible(true);
    ConfirmDialog::hide(&mut state);
    assert!(!state.is_visible());
}

#[test]
fn test_toggleable_toggle() {
    let mut state = ConfirmDialogState::ok("T", "M");
    ConfirmDialog::toggle(&mut state);
    assert!(state.is_visible());
    ConfirmDialog::toggle(&mut state);
    assert!(!state.is_visible());
}

// SelectResult tests

#[test]
fn test_select_result_directly() {
    let mut state = ConfirmDialogState::yes_no("T", "M").with_visible(true);
    let output = ConfirmDialog::update(
        &mut state,
        ConfirmDialogMessage::SelectResult(ConfirmDialogResult::No),
    );
    assert_eq!(
        output,
        Some(ConfirmDialogOutput::Confirmed(ConfirmDialogResult::No))
    );
    assert!(!state.is_visible());
}

// Instance method tests

#[test]
fn test_instance_update() {
    let mut state = ConfirmDialogState::yes_no("T", "M").with_visible(true);
    let output = state.update(ConfirmDialogMessage::Press);
    assert_eq!(
        output,
        Some(ConfirmDialogOutput::Confirmed(ConfirmDialogResult::Yes))
    );
}

// Rendering tests

#[test]
fn test_view_not_visible() {
    let state = ConfirmDialogState::ok("T", "M");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 20);

    terminal
        .draw(|frame| {
            ConfirmDialog::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    // Should be empty when not visible
    let output = terminal.backend().to_string();
    assert!(!output.contains("T"));
}

#[test]
fn test_view_ok_dialog() {
    let mut state = ConfirmDialogState::ok("Info", "Operation complete.");
    state.set_visible(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 20);

    terminal
        .draw(|frame| {
            ConfirmDialog::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_yes_no_dialog() {
    let mut state = ConfirmDialogState::yes_no("Delete?", "This cannot be undone.");
    state.set_visible(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 20);

    terminal
        .draw(|frame| {
            ConfirmDialog::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{WidgetType, with_annotations};
    let mut state = ConfirmDialogState::yes_no("Delete?", "Sure?");
    state.set_visible(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 20);

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                ConfirmDialog::view(
                    &state,
                    &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
                );
            })
            .unwrap();
    });

    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::ConfirmDialog);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.label, Some("Delete?".to_string()));
}

// ButtonConfig tests

#[test]
fn test_button_config_ok_count() {
    assert_eq!(ButtonConfig::Ok.button_count(), 1);
}

#[test]
fn test_button_config_ok_cancel_count() {
    assert_eq!(ButtonConfig::OkCancel.button_count(), 2);
}

#[test]
fn test_button_config_yes_no_count() {
    assert_eq!(ButtonConfig::YesNo.button_count(), 2);
}

#[test]
fn test_button_config_yes_no_cancel_count() {
    assert_eq!(ButtonConfig::YesNoCancel.button_count(), 3);
}

#[test]
fn test_button_config_has_yes_no() {
    assert!(!ButtonConfig::Ok.has_yes_no());
    assert!(!ButtonConfig::OkCancel.has_yes_no());
    assert!(ButtonConfig::YesNo.has_yes_no());
    assert!(ButtonConfig::YesNoCancel.has_yes_no());
}
