use super::*;
use crate::component::EventContext;
use ratatui::style::Color;

// ========================================
// UsageMetric Tests
// ========================================

#[test]
fn test_metric_new() {
    let metric = UsageMetric::new("CPU", "45%");
    assert_eq!(metric.label(), "CPU");
    assert_eq!(metric.value(), "45%");
    assert_eq!(metric.color(), None);
    assert_eq!(metric.icon(), None);
}

#[test]
fn test_metric_with_color() {
    let metric = UsageMetric::new("CPU", "45%").with_color(Color::Green);
    assert_eq!(metric.color(), Some(Color::Green));
}

#[test]
fn test_metric_with_icon() {
    let metric = UsageMetric::new("CPU", "45%").with_icon("*");
    assert_eq!(metric.icon(), Some("*"));
}

#[test]
fn test_metric_with_color_and_icon() {
    let metric = UsageMetric::new("CPU", "45%")
        .with_color(Color::Green)
        .with_icon("*");
    assert_eq!(metric.color(), Some(Color::Green));
    assert_eq!(metric.icon(), Some("*"));
}

#[test]
fn test_metric_setters() {
    let mut metric = UsageMetric::new("CPU", "45%");
    metric.set_label("Memory");
    metric.set_value("3.2 GB");
    metric.set_color(Some(Color::Yellow));
    metric.set_icon(Some("M".to_string()));

    assert_eq!(metric.label(), "Memory");
    assert_eq!(metric.value(), "3.2 GB");
    assert_eq!(metric.color(), Some(Color::Yellow));
    assert_eq!(metric.icon(), Some("M"));
}

#[test]
fn test_metric_clear_color() {
    let mut metric = UsageMetric::new("CPU", "45%").with_color(Color::Green);
    metric.set_color(None);
    assert_eq!(metric.color(), None);
}

#[test]
fn test_metric_clear_icon() {
    let mut metric = UsageMetric::new("CPU", "45%").with_icon("*");
    metric.set_icon(None);
    assert_eq!(metric.icon(), None);
}

#[test]
fn test_metric_clone() {
    let metric = UsageMetric::new("CPU", "45%")
        .with_color(Color::Green)
        .with_icon("*");
    let cloned = metric.clone();
    assert_eq!(cloned.label(), "CPU");
    assert_eq!(cloned.value(), "45%");
    assert_eq!(cloned.color(), Some(Color::Green));
    assert_eq!(cloned.icon(), Some("*"));
}

#[test]
fn test_metric_debug() {
    let metric = UsageMetric::new("CPU", "45%");
    let debug_str = format!("{:?}", metric);
    assert!(debug_str.contains("CPU"));
    assert!(debug_str.contains("45%"));
}

// ========================================
// UsageLayout Tests
// ========================================

#[test]
fn test_layout_default() {
    let layout = UsageLayout::default();
    assert_eq!(layout, UsageLayout::Horizontal);
}

#[test]
fn test_layout_horizontal() {
    let layout = UsageLayout::Horizontal;
    assert_eq!(layout, UsageLayout::Horizontal);
}

#[test]
fn test_layout_vertical() {
    let layout = UsageLayout::Vertical;
    assert_eq!(layout, UsageLayout::Vertical);
}

#[test]
fn test_layout_grid() {
    let layout = UsageLayout::Grid(3);
    assert_eq!(layout, UsageLayout::Grid(3));
}

#[test]
fn test_layout_clone() {
    let layout = UsageLayout::Grid(2);
    let cloned = layout;
    assert_eq!(cloned, UsageLayout::Grid(2));
}

// ========================================
// State Creation Tests
// ========================================

#[test]
fn test_state_new() {
    let state = UsageDisplayState::new();
    assert!(state.is_empty());
    assert_eq!(state.layout(), UsageLayout::Horizontal);
    assert_eq!(state.title(), None);
}

#[test]
fn test_state_default() {
    let state = UsageDisplayState::default();
    assert!(state.is_empty());
    assert_eq!(state.layout(), UsageLayout::Horizontal);
}

#[test]
fn test_state_with_metrics() {
    let metrics = vec![
        UsageMetric::new("CPU", "45%"),
        UsageMetric::new("Memory", "3.2 GB"),
    ];
    let state = UsageDisplayState::with_metrics(metrics);
    assert_eq!(state.len(), 2);
}

#[test]
fn test_state_with_layout() {
    let state = UsageDisplayState::new().with_layout(UsageLayout::Vertical);
    assert_eq!(state.layout(), UsageLayout::Vertical);
}

#[test]
fn test_state_with_grid_layout() {
    let state = UsageDisplayState::new().with_layout(UsageLayout::Grid(3));
    assert_eq!(state.layout(), UsageLayout::Grid(3));
}

#[test]
fn test_state_with_title() {
    let state = UsageDisplayState::new().with_title("System Metrics");
    assert_eq!(state.title(), Some("System Metrics"));
}

#[test]
fn test_state_with_separator() {
    let state = UsageDisplayState::new().with_separator(" | ");
    assert_eq!(state.separator(), " | ");
}

#[test]
fn test_state_builder_metric() {
    let state = UsageDisplayState::new()
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Memory", "3.2 GB"))
        .metric(UsageMetric::new("Disk", "120 GB"));
    assert_eq!(state.len(), 3);
}

// ========================================
// Accessor Tests
// ========================================

#[test]
fn test_metrics() {
    let state = UsageDisplayState::new()
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Memory", "3.2 GB"));
    assert_eq!(state.metrics().len(), 2);
    assert_eq!(state.metrics()[0].label(), "CPU");
    assert_eq!(state.metrics()[1].label(), "Memory");
}

#[test]
fn test_len_and_is_empty() {
    let state = UsageDisplayState::new();
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);

    let state = state.metric(UsageMetric::new("CPU", "45%"));
    assert!(!state.is_empty());
    assert_eq!(state.len(), 1);
}

#[test]
fn test_find() {
    let state = UsageDisplayState::new()
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Memory", "3.2 GB"));

    let found = state.find("CPU");
    assert!(found.is_some());
    assert_eq!(found.unwrap().value(), "45%");

    let not_found = state.find("Disk");
    assert!(not_found.is_none());
}

#[test]
fn test_find_mut() {
    let mut state = UsageDisplayState::new()
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Memory", "3.2 GB"));

    if let Some(metric) = state.find_mut("CPU") {
        metric.set_value("80%");
    }
    assert_eq!(state.find("CPU").unwrap().value(), "80%");
}

// ========================================
// Mutator Tests
// ========================================

#[test]
fn test_set_metrics() {
    let mut state = UsageDisplayState::new().metric(UsageMetric::new("old", "value"));
    state.set_metrics(vec![UsageMetric::new("new", "value")]);
    assert_eq!(state.len(), 1);
    assert_eq!(state.metrics()[0].label(), "new");
}

#[test]
fn test_add_metric() {
    let mut state = UsageDisplayState::new();
    state.add_metric(UsageMetric::new("CPU", "45%"));
    assert_eq!(state.len(), 1);
}

#[test]
fn test_remove_metric() {
    let mut state = UsageDisplayState::new()
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Memory", "3.2 GB"));
    state.remove_metric("CPU");
    assert_eq!(state.len(), 1);
    assert_eq!(state.metrics()[0].label(), "Memory");
}

#[test]
fn test_remove_metric_not_found() {
    let mut state = UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%"));
    state.remove_metric("Disk");
    assert_eq!(state.len(), 1);
}

#[test]
fn test_update_value() {
    let mut state = UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%"));
    let updated = state.update_value("CPU", "80%");
    assert!(updated);
    assert_eq!(state.find("CPU").unwrap().value(), "80%");
}

#[test]
fn test_update_value_not_found() {
    let mut state = UsageDisplayState::new();
    let updated = state.update_value("CPU", "80%");
    assert!(!updated);
}

#[test]
fn test_update_color() {
    let mut state = UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%"));
    let updated = state.update_color("CPU", Some(Color::Red));
    assert!(updated);
    assert_eq!(state.find("CPU").unwrap().color(), Some(Color::Red));
}

#[test]
fn test_update_color_not_found() {
    let mut state = UsageDisplayState::new();
    let updated = state.update_color("CPU", Some(Color::Red));
    assert!(!updated);
}

#[test]
fn test_update_color_clear() {
    let mut state =
        UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%").with_color(Color::Green));
    state.update_color("CPU", None);
    assert_eq!(state.find("CPU").unwrap().color(), None);
}

#[test]
fn test_set_layout() {
    let mut state = UsageDisplayState::new();
    state.set_layout(UsageLayout::Vertical);
    assert_eq!(state.layout(), UsageLayout::Vertical);
}

#[test]
fn test_set_title() {
    let mut state = UsageDisplayState::new();
    state.set_title(Some("System".to_string()));
    assert_eq!(state.title(), Some("System"));

    state.set_title(None);
    assert_eq!(state.title(), None);
}

#[test]
fn test_set_separator() {
    let mut state = UsageDisplayState::new();
    state.set_separator(" -- ");
    assert_eq!(state.separator(), " -- ");
}
#[test]
fn test_clear() {
    let mut state = UsageDisplayState::new()
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Memory", "3.2 GB"));
    state.clear();
    assert!(state.is_empty());
}

// ========================================
// Component Tests
// ========================================

#[test]
fn test_init() {
    let state = UsageDisplay::init();
    assert!(state.is_empty());
    assert_eq!(state.layout(), UsageLayout::Horizontal);
}

#[test]
fn test_update_set_metrics() {
    let mut state = UsageDisplay::init();
    UsageDisplay::update(
        &mut state,
        UsageDisplayMessage::SetMetrics(vec![UsageMetric::new("CPU", "45%")]),
    );
    assert_eq!(state.len(), 1);
}

#[test]
fn test_update_add_metric() {
    let mut state = UsageDisplay::init();
    UsageDisplay::update(
        &mut state,
        UsageDisplayMessage::AddMetric(UsageMetric::new("CPU", "45%")),
    );
    assert_eq!(state.len(), 1);
}

#[test]
fn test_update_remove_metric() {
    let mut state = UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%"));
    UsageDisplay::update(
        &mut state,
        UsageDisplayMessage::RemoveMetric("CPU".to_string()),
    );
    assert!(state.is_empty());
}

#[test]
fn test_update_value_msg() {
    let mut state = UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%"));
    UsageDisplay::update(
        &mut state,
        UsageDisplayMessage::UpdateValue {
            label: "CPU".to_string(),
            value: "80%".to_string(),
        },
    );
    assert_eq!(state.find("CPU").unwrap().value(), "80%");
}

#[test]
fn test_update_color_msg() {
    let mut state = UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%"));
    UsageDisplay::update(
        &mut state,
        UsageDisplayMessage::UpdateColor {
            label: "CPU".to_string(),
            color: Some(Color::Red),
        },
    );
    assert_eq!(state.find("CPU").unwrap().color(), Some(Color::Red));
}

#[test]
fn test_update_set_layout() {
    let mut state = UsageDisplay::init();
    UsageDisplay::update(
        &mut state,
        UsageDisplayMessage::SetLayout(UsageLayout::Vertical),
    );
    assert_eq!(state.layout(), UsageLayout::Vertical);
}

#[test]
fn test_update_set_title() {
    let mut state = UsageDisplay::init();
    UsageDisplay::update(
        &mut state,
        UsageDisplayMessage::SetTitle(Some("System".to_string())),
    );
    assert_eq!(state.title(), Some("System"));
}

#[test]
fn test_update_set_separator() {
    let mut state = UsageDisplay::init();
    UsageDisplay::update(
        &mut state,
        UsageDisplayMessage::SetSeparator(" -- ".to_string()),
    );
    assert_eq!(state.separator(), " -- ");
}

#[test]
fn test_update_clear() {
    let mut state = UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%"));
    UsageDisplay::update(&mut state, UsageDisplayMessage::Clear);
    assert!(state.is_empty());
}

#[test]
fn test_update_returns_none() {
    let mut state = UsageDisplay::init();
    let output = UsageDisplay::update(&mut state, UsageDisplayMessage::Clear);
    assert!(output.is_none());
}

// ========================================
// Disableable Trait Tests
// ========================================
// ========================================
// View Tests
// ========================================

#[test]
fn test_view_empty() {
    let state = UsageDisplayState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 3);
    terminal
        .draw(|frame| {
            UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_horizontal_single() {
    let state = UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_horizontal_multiple() {
    let state = UsageDisplayState::new()
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Memory", "3.2 GB"))
        .metric(UsageMetric::new("Disk", "120 GB"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_horizontal_with_color() {
    let state = UsageDisplayState::new()
        .metric(UsageMetric::new("CPU", "45%").with_color(Color::Green))
        .metric(UsageMetric::new("Memory", "3.2 GB").with_color(Color::Yellow));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_horizontal_with_icon() {
    let state = UsageDisplayState::new()
        .metric(UsageMetric::new("CPU", "45%").with_icon("*"))
        .metric(UsageMetric::new("Memory", "3.2 GB").with_icon("#"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_horizontal_custom_separator() {
    let state = UsageDisplayState::new()
        .with_separator(" | ")
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Memory", "3.2 GB"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_vertical() {
    let state = UsageDisplayState::new()
        .with_layout(UsageLayout::Vertical)
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Memory", "3.2 GB"))
        .metric(UsageMetric::new("Disk", "120 GB"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 5);
    terminal
        .draw(|frame| {
            UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_vertical_with_title() {
    let state = UsageDisplayState::new()
        .with_layout(UsageLayout::Vertical)
        .with_title("System")
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Memory", "3.2 GB"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 4);
    terminal
        .draw(|frame| {
            UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_grid_2_columns() {
    let state = UsageDisplayState::new()
        .with_layout(UsageLayout::Grid(2))
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Memory", "3.2 GB"))
        .metric(UsageMetric::new("Disk", "120 GB"))
        .metric(UsageMetric::new("Network", "1.5 Mbps"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(50, 4);
    terminal
        .draw(|frame| {
            UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_grid_3_columns() {
    let state = UsageDisplayState::new()
        .with_layout(UsageLayout::Grid(3))
        .with_title("Metrics")
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Mem", "3.2 GB"))
        .metric(UsageMetric::new("Disk", "120 GB"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(50, 3);
    terminal
        .draw(|frame| {
            UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_grid_odd_count() {
    let state = UsageDisplayState::new()
        .with_layout(UsageLayout::Grid(2))
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Memory", "3.2 GB"))
        .metric(UsageMetric::new("Disk", "120 GB"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(50, 4);
    terminal
        .draw(|frame| {
            UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_zero_width() {
    let state = UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 3);
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 0, 3);
            UsageDisplay::view(&state, &mut RenderContext::new(frame, area, &theme));
        })
        .unwrap();
    // Should not panic
}

#[test]
fn test_view_zero_height() {
    let state = UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 3);
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 60, 0);
            UsageDisplay::view(&state, &mut RenderContext::new(frame, area, &theme));
        })
        .unwrap();
    // Should not panic
}

// ========================================
// handle_event Tests
// ========================================

#[test]
fn test_handle_event_returns_none() {
    let state = UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%"));
    let event = crate::input::Event::key(crate::input::KeyCode::Char('q'));
    let msg = UsageDisplay::handle_event(&state, &event, &EventContext::default());
    assert!(msg.is_none());
}

#[test]
fn test_dispatch_event_returns_none() {
    let mut state = UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%"));
    let event = crate::input::Event::key(crate::input::KeyCode::Enter);
    let output = UsageDisplay::dispatch_event(&mut state, &event, &EventContext::default());
    assert!(output.is_none());
}

// ========================================
// Default Matches Init Tests
// ========================================

#[test]
fn test_default_matches_init() {
    let default_state = UsageDisplayState::default();
    let init_state = UsageDisplay::init();

    assert_eq!(default_state.is_empty(), init_state.is_empty());
    assert_eq!(default_state.len(), init_state.len());
    assert_eq!(default_state.layout(), init_state.layout());
    assert_eq!(default_state.title(), init_state.title());
    assert_eq!(default_state.is_disabled(), init_state.is_disabled());
}

// ========================================
// Annotation Tests
// ========================================

#[test]
fn test_annotation_emitted_horizontal() {
    use crate::annotation::with_annotations;

    let state = UsageDisplayState::new().metric(UsageMetric::new("CPU", "45%"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 1);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_annotation_emitted_vertical() {
    use crate::annotation::with_annotations;

    let state = UsageDisplayState::new()
        .with_layout(UsageLayout::Vertical)
        .metric(UsageMetric::new("CPU", "45%"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_annotation_emitted_grid() {
    use crate::annotation::with_annotations;

    let state = UsageDisplayState::new()
        .with_layout(UsageLayout::Grid(2))
        .metric(UsageMetric::new("CPU", "45%"))
        .metric(UsageMetric::new("Memory", "3.2 GB"));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(50, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                UsageDisplay::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
}
