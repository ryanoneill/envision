use super::*;
use crate::component::test_utils;

// ---------------------------------------------------------------------------
// Construction and builders
// ---------------------------------------------------------------------------

#[test]
fn test_new() {
    let state = ResourceGaugeState::new(100.0, 200.0, 500.0);
    assert_eq!(state.actual(), 100.0);
    assert_eq!(state.request(), 200.0);
    assert_eq!(state.limit(), 500.0);
    assert_eq!(state.label(), None);
    assert_eq!(state.units(), None);
    assert!(state.show_legend());
}

#[test]
fn test_default() {
    let state = ResourceGaugeState::default();
    assert_eq!(state.actual(), 0.0);
    assert_eq!(state.request(), 0.0);
    assert_eq!(state.limit(), 0.0);
}

#[test]
fn test_builder_chain() {
    let state = ResourceGaugeState::new(350.0, 500.0, 1000.0)
        .with_label("CPU")
        .with_units("m")
        .with_title("Container Resources")
        .with_show_legend(true)
        .with_orientation(GaugeOrientation::Horizontal)
        .with_disabled(false);

    assert_eq!(state.label(), Some("CPU"));
    assert_eq!(state.units(), Some("m"));
    assert_eq!(state.title(), Some("Container Resources"));
    assert!(state.show_legend());
    assert_eq!(state.orientation(), &GaugeOrientation::Horizontal);
    assert!(!state.is_disabled());
}

// ---------------------------------------------------------------------------
// Setters
// ---------------------------------------------------------------------------

#[test]
fn test_set_actual() {
    let mut state = ResourceGaugeState::new(0.0, 100.0, 200.0);
    state.set_actual(75.0);
    assert_eq!(state.actual(), 75.0);
}

#[test]
fn test_set_request() {
    let mut state = ResourceGaugeState::new(50.0, 100.0, 200.0);
    state.set_request(150.0);
    assert_eq!(state.request(), 150.0);
}

#[test]
fn test_set_limit() {
    let mut state = ResourceGaugeState::new(50.0, 100.0, 200.0);
    state.set_limit(500.0);
    assert_eq!(state.limit(), 500.0);
}

#[test]
fn test_set_values() {
    let mut state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    state.set_values(350.0, 500.0, 1000.0);
    assert_eq!(state.actual(), 350.0);
    assert_eq!(state.request(), 500.0);
    assert_eq!(state.limit(), 1000.0);
}

#[test]
fn test_set_label() {
    let mut state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    state.set_label(Some("MEM".to_string()));
    assert_eq!(state.label(), Some("MEM"));
    state.set_label(None);
    assert_eq!(state.label(), None);
}

#[test]
fn test_set_units() {
    let mut state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    state.set_units(Some("GB".to_string()));
    assert_eq!(state.units(), Some("GB"));
}

#[test]
fn test_set_title() {
    let mut state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    state.set_title(Some("Resources".to_string()));
    assert_eq!(state.title(), Some("Resources"));
}

// ---------------------------------------------------------------------------
// Computed properties
// ---------------------------------------------------------------------------

#[test]
fn test_utilization() {
    let state = ResourceGaugeState::new(250.0, 500.0, 1000.0);
    assert!((state.utilization() - 0.25).abs() < 0.001);
}

#[test]
fn test_utilization_zero_limit() {
    let state = ResourceGaugeState::new(100.0, 0.0, 0.0);
    assert_eq!(state.utilization(), 0.0);
}

#[test]
fn test_utilization_clamped() {
    let state = ResourceGaugeState::new(2000.0, 500.0, 1000.0);
    assert_eq!(state.utilization(), 1.0);
}

#[test]
fn test_request_ratio() {
    let state = ResourceGaugeState::new(250.0, 500.0, 1000.0);
    assert!((state.request_ratio() - 0.5).abs() < 0.001);
}

#[test]
fn test_request_ratio_zero() {
    let state = ResourceGaugeState::new(100.0, 0.0, 1000.0);
    assert_eq!(state.request_ratio(), 0.0);
}

#[test]
fn test_is_over_request() {
    let under = ResourceGaugeState::new(100.0, 500.0, 1000.0);
    assert!(!under.is_over_request());

    let at = ResourceGaugeState::new(500.0, 500.0, 1000.0);
    assert!(at.is_over_request());

    let over = ResourceGaugeState::new(600.0, 500.0, 1000.0);
    assert!(over.is_over_request());
}

#[test]
fn test_is_over_request_zero_request() {
    let state = ResourceGaugeState::new(100.0, 0.0, 1000.0);
    assert!(!state.is_over_request());
}

#[test]
fn test_is_near_limit() {
    let safe = ResourceGaugeState::new(500.0, 500.0, 1000.0);
    assert!(!safe.is_near_limit());

    let near = ResourceGaugeState::new(900.0, 500.0, 1000.0);
    assert!(near.is_near_limit());

    let at = ResourceGaugeState::new(1000.0, 500.0, 1000.0);
    assert!(at.is_near_limit());
}

#[test]
fn test_is_near_limit_zero_limit() {
    let state = ResourceGaugeState::new(100.0, 0.0, 0.0);
    assert!(!state.is_near_limit());
}

// ---------------------------------------------------------------------------
// Health color
// ---------------------------------------------------------------------------

#[test]
fn test_health_color_green() {
    let state = ResourceGaugeState::new(100.0, 500.0, 1000.0);
    assert_eq!(state.health_color(), Color::Green);
}

#[test]
fn test_health_color_yellow() {
    let state = ResourceGaugeState::new(600.0, 500.0, 1000.0);
    assert_eq!(state.health_color(), Color::Yellow);
}

#[test]
fn test_health_color_red() {
    let state = ResourceGaugeState::new(950.0, 500.0, 1000.0);
    assert_eq!(state.health_color(), Color::Red);
}

#[test]
fn test_health_color_zero_limit() {
    let state = ResourceGaugeState::new(100.0, 0.0, 0.0);
    assert_eq!(state.health_color(), Color::DarkGray);
}

// ---------------------------------------------------------------------------
// Legend text
// ---------------------------------------------------------------------------

#[test]
fn test_legend_text() {
    let state = ResourceGaugeState::new(350.0, 500.0, 1000.0).with_units("m");
    assert_eq!(state.legend_text(), "350m / 500m / 1000m");
}

#[test]
fn test_legend_text_no_units() {
    let state = ResourceGaugeState::new(1.5, 2.0, 4.0);
    assert_eq!(state.legend_text(), "1.5 / 2 / 4");
}

#[test]
fn test_legend_text_integers() {
    let state = ResourceGaugeState::new(100.0, 200.0, 500.0);
    assert_eq!(state.legend_text(), "100 / 200 / 500");
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

#[test]
fn test_message_set_actual() {
    let mut state = ResourceGaugeState::new(0.0, 100.0, 200.0);
    ResourceGauge::update(&mut state, ResourceGaugeMessage::SetActual(75.0));
    assert_eq!(state.actual(), 75.0);
}

#[test]
fn test_message_set_request() {
    let mut state = ResourceGaugeState::new(0.0, 100.0, 200.0);
    ResourceGauge::update(&mut state, ResourceGaugeMessage::SetRequest(150.0));
    assert_eq!(state.request(), 150.0);
}

#[test]
fn test_message_set_limit() {
    let mut state = ResourceGaugeState::new(0.0, 100.0, 200.0);
    ResourceGauge::update(&mut state, ResourceGaugeMessage::SetLimit(500.0));
    assert_eq!(state.limit(), 500.0);
}

#[test]
fn test_message_set_values() {
    let mut state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    ResourceGauge::update(
        &mut state,
        ResourceGaugeMessage::SetValues {
            actual: 350.0,
            request: 500.0,
            limit: 1000.0,
        },
    );
    assert_eq!(state.actual(), 350.0);
    assert_eq!(state.request(), 500.0);
    assert_eq!(state.limit(), 1000.0);
}

#[test]
fn test_message_set_label() {
    let mut state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    ResourceGauge::update(
        &mut state,
        ResourceGaugeMessage::SetLabel(Some("CPU".to_string())),
    );
    assert_eq!(state.label(), Some("CPU"));
}

#[test]
fn test_message_set_units() {
    let mut state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    ResourceGauge::update(
        &mut state,
        ResourceGaugeMessage::SetUnits(Some("Mi".to_string())),
    );
    assert_eq!(state.units(), Some("Mi"));
}

#[test]
fn test_instance_update() {
    let mut state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    state.update(ResourceGaugeMessage::SetActual(42.0));
    assert_eq!(state.actual(), 42.0);
}

// ---------------------------------------------------------------------------
// Component trait
// ---------------------------------------------------------------------------

#[test]
fn test_init() {
    let state = ResourceGauge::init();
    assert_eq!(state.actual(), 0.0);
}

#[test]
fn test_handle_event_returns_none() {
    let state = ResourceGaugeState::new(100.0, 200.0, 500.0);
    let event = crate::input::Event::char('x');
    let ctx = EventContext::new().focused(true);
    assert!(ResourceGauge::handle_event(&state, &event, &ctx).is_none());
}

// ---------------------------------------------------------------------------
// Snapshot tests
// ---------------------------------------------------------------------------

#[test]
fn test_snapshot_healthy() {
    let state = ResourceGaugeState::new(200.0, 500.0, 1000.0)
        .with_label("CPU")
        .with_units("m");
    let (mut terminal, theme) = test_utils::setup_render(50, 3);
    terminal
        .draw(|frame| {
            ResourceGauge::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_warning() {
    let state = ResourceGaugeState::new(600.0, 500.0, 1000.0)
        .with_label("CPU")
        .with_units("m");
    let (mut terminal, theme) = test_utils::setup_render(50, 3);
    terminal
        .draw(|frame| {
            ResourceGauge::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_critical() {
    let state = ResourceGaugeState::new(950.0, 500.0, 1000.0)
        .with_label("CPU")
        .with_units("m");
    let (mut terminal, theme) = test_utils::setup_render(50, 3);
    terminal
        .draw(|frame| {
            ResourceGauge::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_over_limit() {
    let state = ResourceGaugeState::new(1200.0, 500.0, 1000.0)
        .with_label("MEM")
        .with_units("Mi");
    let (mut terminal, theme) = test_utils::setup_render(50, 3);
    terminal
        .draw(|frame| {
            ResourceGauge::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_zero_limit() {
    let state = ResourceGaugeState::new(0.0, 0.0, 0.0).with_label("GPU");
    let (mut terminal, theme) = test_utils::setup_render(50, 3);
    terminal
        .draw(|frame| {
            ResourceGauge::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_title() {
    let state = ResourceGaugeState::new(350.0, 500.0, 1000.0)
        .with_title("Container CPU")
        .with_units("m");
    let (mut terminal, theme) = test_utils::setup_render(50, 3);
    terminal
        .draw(|frame| {
            ResourceGauge::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_no_legend() {
    let state = ResourceGaugeState::new(350.0, 500.0, 1000.0)
        .with_label("CPU")
        .with_show_legend(false);
    let (mut terminal, theme) = test_utils::setup_render(50, 3);
    terminal
        .draw(|frame| {
            ResourceGauge::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// ---------------------------------------------------------------------------
// Format value
// ---------------------------------------------------------------------------

#[test]
fn test_format_value_integer() {
    assert_eq!(format_value(100.0), "100");
    assert_eq!(format_value(0.0), "0");
}

#[test]
fn test_format_value_decimal() {
    assert_eq!(format_value(1.5), "1.5");
    assert_eq!(format_value(99.9), "99.9");
}
