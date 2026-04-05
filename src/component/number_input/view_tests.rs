use super::*;

// ========================================
// View Snapshot Tests
// ========================================

#[test]
fn test_view_normal() {
    let state = NumberInputState::new(42.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            NumberInput::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let state = NumberInputState::new(42.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            NumberInput::view(
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
fn test_view_with_label() {
    let state = NumberInputState::new(42.0).with_label("Qty");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            NumberInput::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_editing() {
    let mut state = NumberInputState::new(42.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            NumberInput::view(
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
fn test_view_editing_with_label() {
    let mut state = NumberInputState::new(42.0).with_label("Qty");
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            NumberInput::view(
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
fn test_view_disabled() {
    let state = NumberInputState::new(42.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            NumberInput::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_negative_value() {
    let state = NumberInputState::new(-42.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            NumberInput::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_float_precision() {
    let state = NumberInputState::new(3.75).with_precision(2);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            NumberInput::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_zero_area() {
    let state = NumberInputState::new(42.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(0, 0);

    // Should not panic
    terminal
        .draw(|frame| {
            NumberInput::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// ========================================
// Annotation Tests
// ========================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{with_annotations, WidgetType};
    let state = NumberInputState::new(42.0).with_label("Quantity");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                NumberInput::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Custom("NumberInput".to_string()));
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.label, Some("Quantity".to_string()));
    assert_eq!(regions[0].annotation.value, Some("42".to_string()));
}

#[test]
fn test_annotation_focused() {
    use crate::annotation::with_annotations;
    let state = NumberInputState::new(42.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                NumberInput::view(
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
    let regions = registry.regions();
    assert!(regions[0].annotation.focused);
}

#[test]
fn test_annotation_disabled() {
    use crate::annotation::with_annotations;
    let state = NumberInputState::new(42.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                NumberInput::view(
                    &state,
                    frame,
                    frame.area(),
                    &theme,
                    &ViewContext::new().disabled(true),
                );
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.regions();
    assert!(regions[0].annotation.disabled);
}
