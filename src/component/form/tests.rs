use super::*;
use crate::component::test_utils;

fn sample_form() -> FormState {
    FormState::new(vec![
        FormField::text("name", "Name"),
        FormField::checkbox("agree", "I agree"),
        FormField::select("color", "Color", vec!["Red", "Green", "Blue"]),
    ])
}

fn focused_form() -> FormState {
    let mut state = sample_form();
    Form::set_focused(&mut state, true);
    state
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new_creates_form_with_fields() {
    let state = sample_form();
    assert_eq!(state.field_count(), 3);
}

#[test]
fn test_new_focuses_first_field() {
    let state = focused_form();
    assert_eq!(state.focused_field_id(), Some("name"));
    assert_eq!(state.focused_field_index(), 0);
}

#[test]
fn test_empty_form() {
    let state = FormState::new(vec![]);
    assert_eq!(state.field_count(), 0);
    assert_eq!(state.focused_field_id(), None);
}

#[test]
fn test_default() {
    let state = FormState::default();
    assert_eq!(state.field_count(), 0);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
}

// =============================================================================
// FormField construction
// =============================================================================

#[test]
fn test_form_field_text() {
    let field = FormField::text("email", "Email");
    assert_eq!(field.id(), "email");
    assert_eq!(field.label(), "Email");
    assert!(matches!(field.kind(), FormFieldKind::Text));
}

#[test]
fn test_form_field_text_with_placeholder() {
    let field = FormField::text_with_placeholder("email", "Email", "user@example.com");
    assert_eq!(field.id(), "email");
    assert!(matches!(
        field.kind(),
        FormFieldKind::TextWithPlaceholder(_)
    ));
}

#[test]
fn test_form_field_checkbox() {
    let field = FormField::checkbox("agree", "I agree");
    assert_eq!(field.id(), "agree");
    assert!(matches!(field.kind(), FormFieldKind::Checkbox));
}

#[test]
fn test_form_field_select() {
    let field = FormField::select("role", "Role", vec!["Admin", "User"]);
    assert_eq!(field.id(), "role");
    assert!(matches!(field.kind(), FormFieldKind::Select(_)));
}

// =============================================================================
// Initial values
// =============================================================================

#[test]
fn test_initial_text_value_is_empty() {
    let state = sample_form();
    assert_eq!(state.value("name"), Some(FormValue::Text(String::new())));
}

#[test]
fn test_initial_checkbox_value_is_false() {
    let state = sample_form();
    assert_eq!(state.value("agree"), Some(FormValue::Bool(false)));
}

#[test]
fn test_initial_select_value_is_none() {
    let state = sample_form();
    assert_eq!(state.value("color"), Some(FormValue::Selected(None)));
}

#[test]
fn test_value_unknown_field_returns_none() {
    let state = sample_form();
    assert_eq!(state.value("unknown"), None);
}

#[test]
fn test_values_returns_all_pairs() {
    let state = sample_form();
    let values = state.values();
    assert_eq!(values.len(), 3);
    assert_eq!(values[0].0, "name");
    assert_eq!(values[1].0, "agree");
    assert_eq!(values[2].0, "color");
}

// =============================================================================
// Text input
// =============================================================================

#[test]
fn test_input_char_to_text_field() {
    let mut state = focused_form();
    let output = Form::update(&mut state, FormMessage::Input('H'));
    assert_eq!(state.value("name"), Some(FormValue::Text("H".into())));
    assert_eq!(
        output,
        Some(FormOutput::FieldChanged(
            "name".into(),
            FormValue::Text("H".into())
        ))
    );
}

#[test]
fn test_input_multiple_chars() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::Input('H'));
    Form::update(&mut state, FormMessage::Input('i'));
    assert_eq!(state.value("name"), Some(FormValue::Text("Hi".into())));
}

#[test]
fn test_backspace_in_text_field() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::Input('A'));
    Form::update(&mut state, FormMessage::Input('B'));
    let output = Form::update(&mut state, FormMessage::Backspace);
    assert_eq!(state.value("name"), Some(FormValue::Text("A".into())));
    assert_eq!(
        output,
        Some(FormOutput::FieldChanged(
            "name".into(),
            FormValue::Text("A".into())
        ))
    );
}

#[test]
fn test_delete_in_text_field() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::Input('A'));
    Form::update(&mut state, FormMessage::Input('B'));
    Form::update(&mut state, FormMessage::Home);
    let output = Form::update(&mut state, FormMessage::Delete);
    assert_eq!(state.value("name"), Some(FormValue::Text("B".into())));
    assert!(matches!(output, Some(FormOutput::FieldChanged(_, _))));
}

#[test]
fn test_clear_text_field() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::Input('X'));
    let output = Form::update(&mut state, FormMessage::Clear);
    assert_eq!(state.value("name"), Some(FormValue::Text(String::new())));
    assert_eq!(
        output,
        Some(FormOutput::FieldChanged(
            "name".into(),
            FormValue::Text(String::new())
        ))
    );
}

#[test]
fn test_cursor_movement_left_right() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::Input('A'));
    Form::update(&mut state, FormMessage::Input('B'));

    // Left then right should not change value, only cursor
    let output = Form::update(&mut state, FormMessage::Left);
    assert_eq!(output, None);
    let output = Form::update(&mut state, FormMessage::Right);
    assert_eq!(output, None);
}

#[test]
fn test_cursor_movement_home_end() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::Input('A'));
    let output = Form::update(&mut state, FormMessage::Home);
    assert_eq!(output, None);
    let output = Form::update(&mut state, FormMessage::End);
    assert_eq!(output, None);
}

// =============================================================================
// Checkbox toggle
// =============================================================================

#[test]
fn test_toggle_checkbox() {
    let mut state = focused_form();
    // Focus checkbox field
    Form::update(&mut state, FormMessage::FocusNext);
    assert_eq!(state.focused_field_id(), Some("agree"));

    let output = Form::update(&mut state, FormMessage::Toggle);
    assert_eq!(state.value("agree"), Some(FormValue::Bool(true)));
    assert_eq!(
        output,
        Some(FormOutput::FieldChanged(
            "agree".into(),
            FormValue::Bool(true)
        ))
    );
}

#[test]
fn test_toggle_checkbox_twice() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::FocusNext);
    Form::update(&mut state, FormMessage::Toggle);
    Form::update(&mut state, FormMessage::Toggle);
    assert_eq!(state.value("agree"), Some(FormValue::Bool(false)));
}

// =============================================================================
// Select field
// =============================================================================

#[test]
fn test_select_open_and_choose() {
    let mut state = focused_form();
    // Focus select field
    Form::update(&mut state, FormMessage::FocusNext);
    Form::update(&mut state, FormMessage::FocusNext);
    assert_eq!(state.focused_field_id(), Some("color"));

    // Open, navigate down, confirm
    Form::update(&mut state, FormMessage::Toggle);
    Form::update(&mut state, FormMessage::SelectDown);
    let output = Form::update(&mut state, FormMessage::SelectConfirm);

    assert!(matches!(
        output,
        Some(FormOutput::FieldChanged(_, FormValue::Selected(Some(_))))
    ));
}

#[test]
fn test_select_up_down() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::FocusNext);
    Form::update(&mut state, FormMessage::FocusNext);

    // Open the select
    Form::update(&mut state, FormMessage::Toggle);
    Form::update(&mut state, FormMessage::SelectDown);
    Form::update(&mut state, FormMessage::SelectUp);
    // Should be back at top, confirm
    let output = Form::update(&mut state, FormMessage::SelectConfirm);
    assert!(matches!(output, Some(FormOutput::FieldChanged(_, _))));
}

// =============================================================================
// Focus navigation
// =============================================================================

#[test]
fn test_focus_next_cycles_through_fields() {
    let mut state = focused_form();
    assert_eq!(state.focused_field_id(), Some("name"));

    Form::update(&mut state, FormMessage::FocusNext);
    assert_eq!(state.focused_field_id(), Some("agree"));

    Form::update(&mut state, FormMessage::FocusNext);
    assert_eq!(state.focused_field_id(), Some("color"));

    // Wraps around
    Form::update(&mut state, FormMessage::FocusNext);
    assert_eq!(state.focused_field_id(), Some("name"));
}

#[test]
fn test_focus_prev_cycles_backward() {
    let mut state = focused_form();
    assert_eq!(state.focused_field_id(), Some("name"));

    // Wraps to last
    Form::update(&mut state, FormMessage::FocusPrev);
    assert_eq!(state.focused_field_id(), Some("color"));

    Form::update(&mut state, FormMessage::FocusPrev);
    assert_eq!(state.focused_field_id(), Some("agree"));

    Form::update(&mut state, FormMessage::FocusPrev);
    assert_eq!(state.focused_field_id(), Some("name"));
}

#[test]
fn test_focus_next_returns_no_output() {
    let mut state = focused_form();
    let output = Form::update(&mut state, FormMessage::FocusNext);
    assert_eq!(output, None);
}

// =============================================================================
// Submit
// =============================================================================

#[test]
fn test_submit_collects_all_values() {
    let mut state = focused_form();
    // Set some values
    Form::update(&mut state, FormMessage::Input('J'));
    Form::update(&mut state, FormMessage::FocusNext);
    Form::update(&mut state, FormMessage::Toggle);

    let output = Form::update(&mut state, FormMessage::Submit);
    match output {
        Some(FormOutput::Submitted(values)) => {
            assert_eq!(values.len(), 3);
            assert_eq!(values[0], ("name".into(), FormValue::Text("J".into())));
            assert_eq!(values[1], ("agree".into(), FormValue::Bool(true)));
            assert_eq!(values[2], ("color".into(), FormValue::Selected(None)));
        }
        _ => panic!("Expected Submitted output"),
    }
}

// =============================================================================
// Disabled state
// =============================================================================

#[test]
fn test_disabled_ignores_messages() {
    let mut state = focused_form();
    state.set_disabled(true);

    let output = Form::update(&mut state, FormMessage::Input('X'));
    assert_eq!(output, None);
    assert_eq!(state.value("name"), Some(FormValue::Text(String::new())));
}

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_form();
    state.set_disabled(true);

    let msg = Form::handle_event(
        &state,
        &Event::char('a'),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_with_disabled_builder() {
    let state = FormState::new(vec![FormField::text("a", "A")]).with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = sample_form();
    assert!(!state.is_focused());
    let msg = Form::handle_event(&state, &Event::char('a'), &ViewContext::default());
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_tab_maps_to_focus_next() {
    let state = focused_form();
    let msg = Form::handle_event(
        &state,
        &Event::key(KeyCode::Tab),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(FormMessage::FocusNext));
}

#[test]
fn test_backtab_maps_to_focus_prev() {
    let state = focused_form();
    let msg = Form::handle_event(
        &state,
        &Event::key(KeyCode::BackTab),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(FormMessage::FocusPrev));
}

#[test]
fn test_ctrl_enter_maps_to_submit() {
    let state = focused_form();
    let _msg = Form::handle_event(
        &state,
        &Event::ctrl('\n'),
        &ViewContext::new().focused(true),
    );
    // Ctrl+Enter on terminal may send ctrl('\n'). Test via explicit key_with.
    let msg = Form::handle_event(
        &state,
        &Event::key_with(KeyCode::Enter, crate::input::KeyModifiers::CONTROL),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(FormMessage::Submit));
}

#[test]
fn test_char_in_text_field_maps_to_input() {
    let state = focused_form();
    let msg = Form::handle_event(&state, &Event::char('x'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(FormMessage::Input('x')));
}

#[test]
fn test_backspace_in_text_field_maps() {
    let state = focused_form();
    let msg = Form::handle_event(
        &state,
        &Event::key(KeyCode::Backspace),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(FormMessage::Backspace));
}

#[test]
fn test_space_in_checkbox_maps_to_toggle() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::FocusNext); // Move to checkbox
    let msg = Form::handle_event(&state, &Event::char(' '), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(FormMessage::Toggle));
}

#[test]
fn test_enter_in_checkbox_maps_to_toggle() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::FocusNext);
    let msg = Form::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(FormMessage::Toggle));
}

#[test]
fn test_enter_in_closed_select_maps_to_toggle() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::FocusNext);
    Form::update(&mut state, FormMessage::FocusNext);
    let msg = Form::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(FormMessage::Toggle));
}

#[test]
fn test_arrow_keys_in_open_select() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::FocusNext);
    Form::update(&mut state, FormMessage::FocusNext);
    Form::update(&mut state, FormMessage::Toggle); // Open select

    let msg = Form::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(FormMessage::SelectDown));

    let msg = Form::handle_event(
        &state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(FormMessage::SelectUp));

    let msg = Form::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(FormMessage::SelectConfirm));
}

#[test]
fn test_esc_in_open_select_maps_to_toggle() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::FocusNext);
    Form::update(&mut state, FormMessage::FocusNext);
    Form::update(&mut state, FormMessage::Toggle);

    let msg = Form::handle_event(
        &state,
        &Event::key(KeyCode::Esc),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(FormMessage::Toggle));
}

// =============================================================================
// dispatch_event
// =============================================================================

#[test]
fn test_dispatch_event_types_text() {
    let mut state = focused_form();
    let output = state.dispatch_event(&Event::char('A'));
    assert_eq!(
        output,
        Some(FormOutput::FieldChanged(
            "name".into(),
            FormValue::Text("A".into())
        ))
    );
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_form();
    let msg = state.handle_event(&Event::char('x'));
    assert_eq!(msg, Some(FormMessage::Input('x')));
}

#[test]
fn test_instance_update() {
    let mut state = focused_form();
    let output = state.update(FormMessage::Input('Z'));
    assert!(matches!(output, Some(FormOutput::FieldChanged(_, _))));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_form();
    let output = state.dispatch_event(&Event::char('A'));
    assert!(matches!(output, Some(FormOutput::FieldChanged(_, _))));
}

// =============================================================================
// Field accessors
// =============================================================================

#[test]
fn test_fields_returns_descriptors() {
    let state = sample_form();
    let fields = state.fields();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].id(), "name");
    assert_eq!(fields[1].id(), "agree");
    assert_eq!(fields[2].id(), "color");
}

#[test]
fn test_field_label() {
    let state = sample_form();
    assert_eq!(state.field_label(0), Some("Name"));
    assert_eq!(state.field_label(1), Some("I agree"));
    assert_eq!(state.field_label(2), Some("Color"));
    assert_eq!(state.field_label(3), None);
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_unfocused() {
    let state = sample_form();
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
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
}

#[test]
fn test_render_focused() {
    let state = focused_form();
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
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
}

#[test]
fn test_render_with_values() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::Input('J'));
    Form::update(&mut state, FormMessage::FocusNext);
    Form::update(&mut state, FormMessage::Toggle);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
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
}

#[test]
fn test_render_disabled() {
    let state = sample_form().with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            Form::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_empty_form() {
    let state = FormState::new(vec![]);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
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
}

// =============================================================================
// Focusable trait
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = Form::init();
    assert!(!Form::is_focused(&state));

    Form::focus(&mut state);
    assert!(Form::is_focused(&state));

    Form::blur(&mut state);
    assert!(!Form::is_focused(&state));
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_input_on_non_text_field_does_nothing() {
    let mut state = focused_form();
    Form::update(&mut state, FormMessage::FocusNext); // checkbox
    let output = Form::update(&mut state, FormMessage::Input('X'));
    assert_eq!(output, None);
}

#[test]
fn test_toggle_on_text_field_does_nothing() {
    let mut state = focused_form();
    let output = Form::update(&mut state, FormMessage::Toggle);
    assert_eq!(output, None);
}

#[test]
fn test_select_confirm_on_non_select_does_nothing() {
    let mut state = focused_form();
    let output = Form::update(&mut state, FormMessage::SelectConfirm);
    assert_eq!(output, None);
}

#[test]
fn test_empty_form_ignores_all_messages() {
    let mut state = FormState::new(vec![]);
    Form::set_focused(&mut state, true);

    let output = Form::update(&mut state, FormMessage::Input('X'));
    assert_eq!(output, None);

    let output = Form::update(&mut state, FormMessage::Submit);
    assert_eq!(output, None);
}

#[test]
fn test_empty_form_ignores_events() {
    let mut state = FormState::new(vec![]);
    Form::set_focused(&mut state, true);

    let msg = Form::handle_event(&state, &Event::char('a'), &ViewContext::new().focused(true));
    assert_eq!(msg, None);
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{with_annotations, WidgetType};
    let state = sample_form();
    let (mut terminal, theme) = test_utils::setup_render(40, 20);
    let registry = with_annotations(|| {
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
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Form);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.has_id("form"));
}
