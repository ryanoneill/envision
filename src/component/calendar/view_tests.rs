use super::*;

// ========== View Tests ==========

#[test]
fn test_view_march_2026() {
    let state = CalendarState::new(2026, 3);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(34, 12);
    terminal
        .draw(|frame| {
            Calendar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_selected_day() {
    let state = CalendarState::new(2026, 3).with_selected_day(20);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(34, 12);
    terminal
        .draw(|frame| {
            Calendar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_events() {
    let state = CalendarState::new(2026, 3)
        .with_selected_day(20)
        .with_event(2026, 3, 10, Color::Green)
        .with_event(2026, 3, 24, Color::Red);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(34, 12);
    terminal
        .draw(|frame| {
            Calendar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_title() {
    let state = CalendarState::new(2026, 3).with_title("Events");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 12);
    terminal
        .draw(|frame| {
            Calendar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(15);
    Calendar::focus(&mut state);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(34, 12);
    terminal
        .draw(|frame| {
            Calendar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = CalendarState::new(2026, 3)
        .with_selected_day(15)
        .with_disabled(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(34, 12);
    terminal
        .draw(|frame| {
            Calendar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_february_leap_year() {
    let state = CalendarState::new(2024, 2);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(34, 12);
    terminal
        .draw(|frame| {
            Calendar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_february_non_leap_year() {
    let state = CalendarState::new(2026, 2);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(34, 12);
    terminal
        .draw(|frame| {
            Calendar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_zero_area() {
    let state = CalendarState::new(2026, 3);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(34, 12);
    terminal
        .draw(|frame| {
            Calendar::view(&state, frame, Rect::new(0, 0, 0, 0), &theme);
        })
        .unwrap();
    // Should not panic
}

#[test]
fn test_view_narrow_area() {
    let state = CalendarState::new(2026, 3);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(10, 5);
    terminal
        .draw(|frame| {
            Calendar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========== Annotation Tests ==========

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;

    let state = CalendarState::new(2026, 3);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(34, 12);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Calendar::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.regions();
    assert_eq!(regions.len(), 1);
    let annotation = &regions[0].annotation;
    assert!(!annotation.focused);
    assert!(!annotation.disabled);
}

#[test]
fn test_annotation_reflects_state() {
    use crate::annotation::with_annotations;

    let mut state = CalendarState::new(2026, 3).with_disabled(true);
    Calendar::focus(&mut state);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(34, 12);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Calendar::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    });
    let regions = registry.regions();
    let annotation = &regions[0].annotation;
    assert!(annotation.focused);
    assert!(annotation.disabled);
}
