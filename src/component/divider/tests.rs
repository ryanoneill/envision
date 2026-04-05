use super::*;

// ---- Construction tests ----

#[test]
fn test_new() {
    let state = DividerState::new();
    assert_eq!(state.orientation(), &DividerOrientation::Horizontal);
    assert!(state.label().is_none());
    assert!(state.color().is_none());
}

#[test]
fn test_default() {
    let state = DividerState::default();
    assert_eq!(state.orientation(), &DividerOrientation::Horizontal);
    assert!(state.label().is_none());
    assert!(state.color().is_none());
}

#[test]
fn test_horizontal() {
    let state = DividerState::horizontal();
    assert_eq!(state.orientation(), &DividerOrientation::Horizontal);
}

#[test]
fn test_vertical() {
    let state = DividerState::vertical();
    assert_eq!(state.orientation(), &DividerOrientation::Vertical);
}

#[test]
fn test_with_label() {
    let state = DividerState::new().with_label("Section");
    assert_eq!(state.label(), Some("Section"));
}

#[test]
fn test_with_color() {
    let state = DividerState::new().with_color(Color::Red);
    assert_eq!(state.color(), Some(Color::Red));
}

#[test]
fn test_with_orientation() {
    let state = DividerState::new().with_orientation(DividerOrientation::Vertical);
    assert_eq!(state.orientation(), &DividerOrientation::Vertical);
}

#[test]
fn test_builder_chaining() {
    let state = DividerState::new()
        .with_label("Title")
        .with_color(Color::Cyan)
        .with_orientation(DividerOrientation::Vertical);
    assert_eq!(state.label(), Some("Title"));
    assert_eq!(state.color(), Some(Color::Cyan));
    assert_eq!(state.orientation(), &DividerOrientation::Vertical);
}

// ---- Getter/setter tests ----

#[test]
fn test_set_label() {
    let mut state = DividerState::new();
    state.set_label(Some("New".to_string()));
    assert_eq!(state.label(), Some("New"));

    state.set_label(None);
    assert!(state.label().is_none());
}

#[test]
fn test_set_orientation() {
    let mut state = DividerState::new();
    assert_eq!(state.orientation(), &DividerOrientation::Horizontal);

    state.set_orientation(DividerOrientation::Vertical);
    assert_eq!(state.orientation(), &DividerOrientation::Vertical);

    state.set_orientation(DividerOrientation::Horizontal);
    assert_eq!(state.orientation(), &DividerOrientation::Horizontal);
}

// ---- Update tests ----

#[test]
fn test_update_set_label() {
    let mut state = DividerState::new();
    let output = Divider::update(
        &mut state,
        DividerMessage::SetLabel(Some("Test".to_string())),
    );
    assert!(output.is_none());
    assert_eq!(state.label(), Some("Test"));
}

#[test]
fn test_update_set_label_none() {
    let mut state = DividerState::new().with_label("Existing");
    let output = Divider::update(&mut state, DividerMessage::SetLabel(None));
    assert!(output.is_none());
    assert!(state.label().is_none());
}

#[test]
fn test_update_set_orientation() {
    let mut state = DividerState::new();
    let output = Divider::update(
        &mut state,
        DividerMessage::SetOrientation(DividerOrientation::Vertical),
    );
    assert!(output.is_none());
    assert_eq!(state.orientation(), &DividerOrientation::Vertical);
}

#[test]
fn test_init() {
    let state = Divider::init();
    assert_eq!(state.orientation(), &DividerOrientation::Horizontal);
    assert!(state.label().is_none());
}

#[test]
fn test_default_matches_init() {
    let default_state = DividerState::default();
    let init_state = Divider::init();

    assert_eq!(default_state.orientation(), init_state.orientation());
    assert_eq!(default_state.label(), init_state.label());
    assert_eq!(default_state.color(), init_state.color());
}

// ---- Instance method tests ----

#[test]
fn test_instance_update() {
    let mut state = DividerState::new();
    let output = state.update(DividerMessage::SetLabel(Some("Via Instance".to_string())));
    assert!(output.is_none());
    assert_eq!(state.label(), Some("Via Instance"));
}

// ---- Disableable trait tests ----

// ---- DividerOrientation tests ----

#[test]
fn test_orientation_default() {
    let orientation = DividerOrientation::default();
    assert_eq!(orientation, DividerOrientation::Horizontal);
}

#[test]
fn test_orientation_clone() {
    let orientation = DividerOrientation::Vertical;
    let cloned = orientation.clone();
    assert_eq!(orientation, cloned);
}

#[test]
fn test_orientation_debug() {
    let orientation = DividerOrientation::Horizontal;
    let debug = format!("{:?}", orientation);
    assert_eq!(debug, "Horizontal");
}

// ---- View snapshot tests ----

#[test]
fn test_view_horizontal_no_label() {
    let state = DividerState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);

    terminal
        .draw(|frame| {
            Divider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_horizontal_with_label() {
    let state = DividerState::new().with_label("Section");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);

    terminal
        .draw(|frame| {
            Divider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_vertical() {
    let state = DividerState::vertical();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(5, 5);

    terminal
        .draw(|frame| {
            Divider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_vertical_with_label() {
    let state = DividerState::vertical().with_label("X");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(5, 5);

    terminal
        .draw(|frame| {
            Divider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = DividerState::new().with_label("Disabled");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);

    terminal
        .draw(|frame| {
            Divider::view(
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
fn test_view_zero_width() {
    let state = DividerState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 0, 3);
            Divider::view(&state, frame, area, &theme, &ViewContext::default());
        })
        .unwrap();

    // Should not panic on zero-width area
}

#[test]
fn test_view_zero_height() {
    let state = DividerState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 40, 0);
            Divider::view(&state, frame, area, &theme, &ViewContext::default());
        })
        .unwrap();

    // Should not panic on zero-height area
}

#[test]
fn test_view_narrow_with_long_label() {
    let state = DividerState::new().with_label("Very Long Section Title");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(10, 3);

    terminal
        .draw(|frame| {
            Divider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ---- Annotation tests ----

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{with_annotations, WidgetType};

    let state = DividerState::new().with_label("Section");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Divider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });

    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Divider);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.label, Some("Section".to_string()));
}

#[test]
fn test_annotation_emitted_no_label() {
    use crate::annotation::{with_annotations, WidgetType};

    let state = DividerState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Divider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });

    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Divider);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.label, Some(String::new()));
}

#[test]
fn test_annotation_disabled() {
    use crate::annotation::{with_annotations, WidgetType};

    let state = DividerState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Divider::view(
                    &state,
                    frame,
                    frame.area(),
                    &theme,
                    &ViewContext::new().disabled(true),
                );
            })
            .unwrap();
    });

    let regions = registry.find_by_type(&WidgetType::Divider);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.disabled);
}
