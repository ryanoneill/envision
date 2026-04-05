use super::*;
use crate::input::{Event, KeyCode};

// ========================================
// Construction Tests
// ========================================

#[test]
fn test_new() {
    let state = SliderState::new(0.0, 100.0);
    assert_eq!(state.value(), 0.0);
    assert_eq!(state.min(), 0.0);
    assert_eq!(state.max(), 100.0);
    assert_eq!(state.step(), 1.0);
    assert!(state.show_value());
    assert_eq!(state.label(), None);
}

#[test]
fn test_new_custom_range() {
    let state = SliderState::new(-50.0, 50.0);
    assert_eq!(state.value(), -50.0);
    assert_eq!(state.min(), -50.0);
    assert_eq!(state.max(), 50.0);
}

#[test]
fn test_default() {
    let state = SliderState::default();
    assert_eq!(state.value(), 0.0);
    assert_eq!(state.min(), 0.0);
    assert_eq!(state.max(), 100.0);
    assert_eq!(state.step(), 1.0);
}

#[test]
fn test_with_value() {
    let state = SliderState::new(0.0, 100.0).with_value(50.0);
    assert_eq!(state.value(), 50.0);
}

#[test]
fn test_with_value_clamped_above() {
    let state = SliderState::new(0.0, 100.0).with_value(200.0);
    assert_eq!(state.value(), 100.0);
}

#[test]
fn test_with_value_clamped_below() {
    let state = SliderState::new(0.0, 100.0).with_value(-50.0);
    assert_eq!(state.value(), 0.0);
}

#[test]
fn test_with_step() {
    let state = SliderState::new(0.0, 100.0).with_step(5.0);
    assert_eq!(state.step(), 5.0);
}

#[test]
fn test_with_label() {
    let state = SliderState::new(0.0, 100.0).with_label("Volume");
    assert_eq!(state.label(), Some("Volume"));
}

#[test]
fn test_with_orientation() {
    let state = SliderState::new(0.0, 100.0).with_orientation(SliderOrientation::Vertical);
    assert_eq!(state.orientation(), &SliderOrientation::Vertical);
}

#[test]
fn test_with_show_value() {
    let state = SliderState::new(0.0, 100.0).with_show_value(false);
    assert!(!state.show_value());
}
#[test]
fn test_builder_chaining() {
    let state = SliderState::new(0.0, 100.0)
        .with_value(50.0)
        .with_step(5.0)
        .with_label("Volume")
        .with_orientation(SliderOrientation::Horizontal)
        .with_show_value(true);
    assert_eq!(state.value(), 50.0);
    assert_eq!(state.step(), 5.0);
    assert_eq!(state.label(), Some("Volume"));
    assert_eq!(state.orientation(), &SliderOrientation::Horizontal);
    assert!(state.show_value());
}

// ========================================
// Value Operation Tests
// ========================================

#[test]
fn test_increment() {
    let mut state = SliderState::new(0.0, 100.0).with_value(50.0);
    let output = Slider::update(&mut state, SliderMessage::Increment);
    assert_eq!(output, Some(SliderOutput::ValueChanged(51.0)));
    assert_eq!(state.value(), 51.0);
}

#[test]
fn test_decrement() {
    let mut state = SliderState::new(0.0, 100.0).with_value(50.0);
    let output = Slider::update(&mut state, SliderMessage::Decrement);
    assert_eq!(output, Some(SliderOutput::ValueChanged(49.0)));
    assert_eq!(state.value(), 49.0);
}

#[test]
fn test_increment_page() {
    let mut state = SliderState::new(0.0, 100.0).with_value(50.0);
    let output = Slider::update(&mut state, SliderMessage::IncrementPage);
    assert_eq!(output, Some(SliderOutput::ValueChanged(60.0)));
    assert_eq!(state.value(), 60.0);
}

#[test]
fn test_decrement_page() {
    let mut state = SliderState::new(0.0, 100.0).with_value(50.0);
    let output = Slider::update(&mut state, SliderMessage::DecrementPage);
    assert_eq!(output, Some(SliderOutput::ValueChanged(40.0)));
    assert_eq!(state.value(), 40.0);
}

#[test]
fn test_increment_page_with_custom_step() {
    let mut state = SliderState::new(0.0, 100.0).with_value(50.0).with_step(2.0);
    let output = Slider::update(&mut state, SliderMessage::IncrementPage);
    assert_eq!(output, Some(SliderOutput::ValueChanged(70.0)));
    assert_eq!(state.value(), 70.0);
}

#[test]
fn test_set_value() {
    let mut state = SliderState::new(0.0, 100.0);
    let output = Slider::update(&mut state, SliderMessage::SetValue(42.0));
    assert_eq!(output, Some(SliderOutput::ValueChanged(42.0)));
    assert_eq!(state.value(), 42.0);
}

#[test]
fn test_set_value_clamped_above() {
    let mut state = SliderState::new(0.0, 100.0);
    let output = Slider::update(&mut state, SliderMessage::SetValue(200.0));
    assert_eq!(output, Some(SliderOutput::ValueChanged(100.0)));
    assert_eq!(state.value(), 100.0);
}

#[test]
fn test_set_value_clamped_below() {
    let mut state = SliderState::new(0.0, 100.0).with_value(50.0);
    let output = Slider::update(&mut state, SliderMessage::SetValue(-50.0));
    assert_eq!(output, Some(SliderOutput::ValueChanged(0.0)));
    assert_eq!(state.value(), 0.0);
}

#[test]
fn test_set_min() {
    let mut state = SliderState::new(0.0, 100.0).with_value(50.0);
    let output = Slider::update(&mut state, SliderMessage::SetMin);
    assert_eq!(output, Some(SliderOutput::ValueChanged(0.0)));
    assert_eq!(state.value(), 0.0);
}

#[test]
fn test_set_max() {
    let mut state = SliderState::new(0.0, 100.0).with_value(50.0);
    let output = Slider::update(&mut state, SliderMessage::SetMax);
    assert_eq!(output, Some(SliderOutput::ValueChanged(100.0)));
    assert_eq!(state.value(), 100.0);
}

#[test]
fn test_increment_at_max_no_change() {
    let mut state = SliderState::new(0.0, 100.0).with_value(100.0);
    let output = Slider::update(&mut state, SliderMessage::Increment);
    assert_eq!(output, None);
    assert_eq!(state.value(), 100.0);
}

#[test]
fn test_decrement_at_min_no_change() {
    let mut state = SliderState::new(0.0, 100.0).with_value(0.0);
    let output = Slider::update(&mut state, SliderMessage::Decrement);
    assert_eq!(output, None);
    assert_eq!(state.value(), 0.0);
}

#[test]
fn test_set_min_already_at_min() {
    let mut state = SliderState::new(0.0, 100.0);
    let output = Slider::update(&mut state, SliderMessage::SetMin);
    assert_eq!(output, None);
}

#[test]
fn test_set_max_already_at_max() {
    let mut state = SliderState::new(0.0, 100.0).with_value(100.0);
    let output = Slider::update(&mut state, SliderMessage::SetMax);
    assert_eq!(output, None);
}
#[test]
fn test_set_value_method() {
    let mut state = SliderState::new(0.0, 100.0);
    state.set_value(75.0);
    assert_eq!(state.value(), 75.0);
}

#[test]
fn test_set_value_method_clamped() {
    let mut state = SliderState::new(0.0, 100.0);
    state.set_value(150.0);
    assert_eq!(state.value(), 100.0);

    state.set_value(-10.0);
    assert_eq!(state.value(), 0.0);
}

// ========================================
// Percentage Tests
// ========================================

#[test]
fn test_percentage_at_min() {
    let state = SliderState::new(0.0, 100.0);
    assert!((state.percentage() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_percentage_at_max() {
    let state = SliderState::new(0.0, 100.0).with_value(100.0);
    assert!((state.percentage() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_percentage_at_midpoint() {
    let state = SliderState::new(0.0, 100.0).with_value(50.0);
    assert!((state.percentage() - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_percentage_custom_range() {
    let state = SliderState::new(10.0, 20.0).with_value(15.0);
    assert!((state.percentage() - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_percentage_negative_range() {
    let state = SliderState::new(-100.0, 100.0).with_value(0.0);
    assert!((state.percentage() - 0.5).abs() < f64::EPSILON);
}

// ========================================
// Edge Case Tests
// ========================================

#[test]
fn test_min_equals_max() {
    let state = SliderState::new(10.0, 10.0);
    assert_eq!(state.value(), 10.0);
    assert!((state.percentage() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_min_equals_max_increment() {
    let mut state = SliderState::new(10.0, 10.0);
    let output = Slider::update(&mut state, SliderMessage::Increment);
    assert_eq!(output, None);
    assert_eq!(state.value(), 10.0);
}

#[test]
fn test_step_larger_than_range() {
    let mut state = SliderState::new(0.0, 5.0).with_step(10.0);
    let output = Slider::update(&mut state, SliderMessage::Increment);
    assert_eq!(output, Some(SliderOutput::ValueChanged(5.0)));
    assert_eq!(state.value(), 5.0);
}

#[test]
fn test_value_at_boundary_increment() {
    let mut state = SliderState::new(0.0, 100.0).with_value(99.5).with_step(1.0);
    let output = Slider::update(&mut state, SliderMessage::Increment);
    assert_eq!(output, Some(SliderOutput::ValueChanged(100.0)));
    assert_eq!(state.value(), 100.0);
}

#[test]
fn test_value_at_boundary_decrement() {
    let mut state = SliderState::new(0.0, 100.0).with_value(0.5).with_step(1.0);
    let output = Slider::update(&mut state, SliderMessage::Decrement);
    assert_eq!(output, Some(SliderOutput::ValueChanged(0.0)));
    assert_eq!(state.value(), 0.0);
}

// ========================================
// Event Handling Tests - Horizontal
// ========================================

#[test]
fn test_handle_event_right_horizontal() {
    let state = SliderState::new(0.0, 100.0);
    let msg = Slider::handle_event(
        &state,
        &Event::key(KeyCode::Right),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SliderMessage::Increment));
}

#[test]
fn test_handle_event_left_horizontal() {
    let state = SliderState::new(0.0, 100.0);
    let msg = Slider::handle_event(
        &state,
        &Event::key(KeyCode::Left),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SliderMessage::Decrement));
}

#[test]
fn test_handle_event_l_horizontal() {
    let state = SliderState::new(0.0, 100.0);
    let msg = Slider::handle_event(&state, &Event::char('l'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(SliderMessage::Increment));
}

#[test]
fn test_handle_event_h_horizontal() {
    let state = SliderState::new(0.0, 100.0);
    let msg = Slider::handle_event(&state, &Event::char('h'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(SliderMessage::Decrement));
}

#[test]
fn test_handle_event_page_up() {
    let state = SliderState::new(0.0, 100.0);
    let msg = Slider::handle_event(
        &state,
        &Event::key(KeyCode::PageUp),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SliderMessage::IncrementPage));
}

#[test]
fn test_handle_event_page_down() {
    let state = SliderState::new(0.0, 100.0);
    let msg = Slider::handle_event(
        &state,
        &Event::key(KeyCode::PageDown),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SliderMessage::DecrementPage));
}

#[test]
fn test_handle_event_home() {
    let state = SliderState::new(0.0, 100.0);
    let msg = Slider::handle_event(
        &state,
        &Event::key(KeyCode::Home),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SliderMessage::SetMin));
}

#[test]
fn test_handle_event_end() {
    let state = SliderState::new(0.0, 100.0);
    let msg = Slider::handle_event(
        &state,
        &Event::key(KeyCode::End),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SliderMessage::SetMax));
}

// ========================================
// Event Handling Tests - Vertical
// ========================================

#[test]
fn test_handle_event_up_vertical() {
    let state = SliderState::new(0.0, 100.0).with_orientation(SliderOrientation::Vertical);
    let msg = Slider::handle_event(
        &state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SliderMessage::Increment));
}

#[test]
fn test_handle_event_down_vertical() {
    let state = SliderState::new(0.0, 100.0).with_orientation(SliderOrientation::Vertical);
    let msg = Slider::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SliderMessage::Decrement));
}

#[test]
fn test_handle_event_k_vertical() {
    let state = SliderState::new(0.0, 100.0).with_orientation(SliderOrientation::Vertical);
    let msg = Slider::handle_event(&state, &Event::char('k'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(SliderMessage::Increment));
}

#[test]
fn test_handle_event_j_vertical() {
    let state = SliderState::new(0.0, 100.0).with_orientation(SliderOrientation::Vertical);
    let msg = Slider::handle_event(&state, &Event::char('j'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(SliderMessage::Decrement));
}

#[test]
fn test_handle_event_page_up_vertical() {
    let state = SliderState::new(0.0, 100.0).with_orientation(SliderOrientation::Vertical);
    let msg = Slider::handle_event(
        &state,
        &Event::key(KeyCode::PageUp),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SliderMessage::IncrementPage));
}

#[test]
fn test_handle_event_home_vertical() {
    let state = SliderState::new(0.0, 100.0).with_orientation(SliderOrientation::Vertical);
    let msg = Slider::handle_event(
        &state,
        &Event::key(KeyCode::Home),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SliderMessage::SetMin));
}

#[test]
fn test_handle_event_end_vertical() {
    let state = SliderState::new(0.0, 100.0).with_orientation(SliderOrientation::Vertical);
    let msg = Slider::handle_event(
        &state,
        &Event::key(KeyCode::End),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SliderMessage::SetMax));
}

// ========================================
// Event Guard Tests
// ========================================

#[test]
fn test_handle_event_unfocused() {
    let state = SliderState::new(0.0, 100.0);
    let msg = Slider::handle_event(&state, &Event::key(KeyCode::Right), &ViewContext::default());
    assert_eq!(msg, None);
}
#[test]
fn test_handle_event_unrelated_key() {
    let state = SliderState::new(0.0, 100.0);
    let msg = Slider::handle_event(&state, &Event::char('q'), &ViewContext::new().focused(true));
    assert_eq!(msg, None);
}

// ========================================
// Dispatch Event Tests
// ========================================

#[test]
fn test_dispatch_event() {
    let mut state = SliderState::new(0.0, 100.0);
    let output = Slider::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Right),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(SliderOutput::ValueChanged(1.0)));
    assert_eq!(state.value(), 1.0);
}

#[test]
fn test_dispatch_event_unfocused() {
    let mut state = SliderState::new(0.0, 100.0);
    let output = Slider::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Right),
        &ViewContext::default(),
    );
    assert_eq!(output, None);
    assert_eq!(state.value(), 0.0);
}
#[test]
fn test_instance_update() {
    let mut state = SliderState::new(0.0, 100.0);
    let output = state.update(SliderMessage::Increment);
    assert_eq!(output, Some(SliderOutput::ValueChanged(1.0)));
    assert_eq!(state.value(), 1.0);
}
// ========================================
// Init Test
// ========================================

#[test]
fn test_init() {
    let state = Slider::init();
    assert_eq!(state.value(), 0.0);
    assert_eq!(state.min(), 0.0);
    assert_eq!(state.max(), 100.0);
    assert_eq!(state.step(), 1.0);
}

// ========================================
// View Snapshot Tests
// ========================================

#[test]
fn test_view_horizontal_empty() {
    let state = SliderState::new(0.0, 100.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);

    terminal
        .draw(|frame| {
            Slider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_horizontal_half() {
    let state = SliderState::new(0.0, 100.0).with_value(50.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);

    terminal
        .draw(|frame| {
            Slider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_horizontal_full() {
    let state = SliderState::new(0.0, 100.0).with_value(100.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);

    terminal
        .draw(|frame| {
            Slider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_horizontal_with_label() {
    let state = SliderState::new(0.0, 100.0)
        .with_value(42.0)
        .with_label("Volume");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);

    terminal
        .draw(|frame| {
            Slider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_horizontal_no_value_display() {
    let state = SliderState::new(0.0, 100.0)
        .with_value(50.0)
        .with_show_value(false);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);

    terminal
        .draw(|frame| {
            Slider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_horizontal_focused() {
    let state = SliderState::new(0.0, 100.0).with_value(50.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);

    terminal
        .draw(|frame| {
            Slider::view(
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
fn test_view_horizontal_disabled() {
    let state = SliderState::new(0.0, 100.0).with_value(50.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);

    terminal
        .draw(|frame| {
            Slider::view(
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
fn test_view_vertical() {
    let state = SliderState::new(0.0, 100.0)
        .with_value(50.0)
        .with_orientation(SliderOrientation::Vertical);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(10, 12);

    terminal
        .draw(|frame| {
            Slider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_vertical_with_label() {
    let state = SliderState::new(0.0, 100.0)
        .with_value(75.0)
        .with_orientation(SliderOrientation::Vertical)
        .with_label("Vol");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(10, 12);

    terminal
        .draw(|frame| {
            Slider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_zero_area() {
    let state = SliderState::new(0.0, 100.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(0, 0);

    // Should not panic
    terminal
        .draw(|frame| {
            Slider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// ========================================
// Format Value Tests
// ========================================

#[test]
fn test_format_value_integer() {
    assert_eq!(format_value(42.0), "42");
}

#[test]
fn test_format_value_fractional() {
    assert_eq!(format_value(42.5), "42.5");
}

#[test]
fn test_format_value_zero() {
    assert_eq!(format_value(0.0), "0");
}

#[test]
fn test_format_value_negative() {
    assert_eq!(format_value(-10.0), "-10");
}

// ========================================
// Annotation Tests
// ========================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{with_annotations, WidgetType};
    let state = SliderState::new(0.0, 100.0)
        .with_value(42.0)
        .with_label("Volume");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Slider::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Custom("Slider".to_string()));
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.label, Some("Volume".to_string()));
    assert_eq!(regions[0].annotation.value, Some("42".to_string()));
}

#[test]
fn test_annotation_focused() {
    use crate::annotation::with_annotations;
    let state = SliderState::new(0.0, 100.0).with_value(50.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Slider::view(
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
    let state = SliderState::new(0.0, 100.0).with_value(50.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Slider::view(
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
