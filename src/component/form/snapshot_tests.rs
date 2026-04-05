use super::*;
use crate::component::test_utils;

fn sample_form() -> FormState {
    FormState::new(vec![
        FormField::text("name", "Name"),
        FormField::checkbox("agree", "I agree"),
        FormField::select("color", "Color", vec!["Red", "Green", "Blue"]),
    ])
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_default_empty() {
    let state = FormState::default();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Form::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_populated() {
    let mut state = sample_form();

    // Fill in some values
    Form::update(&mut state, FormMessage::Input('J'));
    Form::update(&mut state, FormMessage::Input('o'));
    Form::update(&mut state, FormMessage::Input('e'));
    // Tab to checkbox and toggle
    Form::update(&mut state, FormMessage::FocusNext);
    Form::update(&mut state, FormMessage::Toggle);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Form::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused_text_field() {
    let state = sample_form();

    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Form::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused_checkbox() {
    let mut state = sample_form();

    Form::update(&mut state, FormMessage::FocusNext);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Form::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused_select() {
    let mut state = sample_form();

    Form::update(&mut state, FormMessage::FocusNext);
    Form::update(&mut state, FormMessage::FocusNext);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Form::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_unfocused() {
    let state = sample_form();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Form::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_placeholder() {
    let state = FormState::new(vec![
        FormField::text_with_placeholder("email", "Email", "user@example.com"),
        FormField::text("name", "Name"),
    ]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Form::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
