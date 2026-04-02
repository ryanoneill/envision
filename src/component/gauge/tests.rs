use super::*;
use ratatui::style::Color;

// =========================================================================
// Construction tests
// =========================================================================

#[test]
fn test_new() {
    let state = GaugeState::new(50.0, 100.0);
    assert_eq!(state.value(), 50.0);
    assert_eq!(state.max(), 100.0);
    assert!(!state.is_disabled());
}

#[test]
fn test_default() {
    let state = GaugeState::default();
    assert_eq!(state.value(), 0.0);
    assert_eq!(state.max(), 100.0);
    assert_eq!(state.display_percentage(), 0);
}

#[test]
fn test_with_label() {
    let state = GaugeState::new(50.0, 100.0).with_label("Custom Label");
    assert_eq!(state.label_text(), "Custom Label");
}

#[test]
fn test_with_units() {
    let state = GaugeState::new(512.0, 1024.0).with_units("MB");
    assert_eq!(state.label_text(), "512.0 / 1024.0 MB");
}

#[test]
fn test_with_variant_full() {
    let state = GaugeState::new(50.0, 100.0).with_variant(GaugeVariant::Full);
    assert_eq!(state.variant, GaugeVariant::Full);
}

#[test]
fn test_with_variant_line() {
    let state = GaugeState::new(50.0, 100.0).with_variant(GaugeVariant::Line);
    assert_eq!(state.variant, GaugeVariant::Line);
}

#[test]
fn test_with_thresholds() {
    let state = GaugeState::new(50.0, 100.0).with_thresholds(vec![
        ThresholdZone {
            above: 0.0,
            color: Color::Blue,
        },
        ThresholdZone {
            above: 0.5,
            color: Color::Cyan,
        },
    ]);
    assert_eq!(state.thresholds.len(), 2);
    assert_eq!(state.thresholds[0].color, Color::Blue);
    assert_eq!(state.thresholds[1].color, Color::Cyan);
}

#[test]
fn test_with_thresholds_sorts() {
    let state = GaugeState::new(50.0, 100.0).with_thresholds(vec![
        ThresholdZone {
            above: 0.8,
            color: Color::Red,
        },
        ThresholdZone {
            above: 0.0,
            color: Color::Green,
        },
        ThresholdZone {
            above: 0.5,
            color: Color::Yellow,
        },
    ]);
    assert_eq!(state.thresholds[0].above, 0.0);
    assert_eq!(state.thresholds[1].above, 0.5);
    assert_eq!(state.thresholds[2].above, 0.8);
}

#[test]
fn test_with_title() {
    let state = GaugeState::new(50.0, 100.0).with_title("CPU Usage");
    assert_eq!(state.title, Some("CPU Usage".to_string()));
}

#[test]
fn test_with_disabled() {
    let state = GaugeState::new(50.0, 100.0).with_disabled(true);
    assert!(state.is_disabled());
}

#[test]
fn test_with_disabled_false() {
    let state = GaugeState::new(50.0, 100.0).with_disabled(false);
    assert!(!state.is_disabled());
}

// =========================================================================
// Value operation tests
// =========================================================================

#[test]
fn test_set_value() {
    let mut state = GaugeState::new(0.0, 100.0);
    state.set_value(75.0);
    assert_eq!(state.value(), 75.0);
}

#[test]
fn test_set_max() {
    let mut state = GaugeState::new(50.0, 100.0);
    state.set_max(200.0);
    assert_eq!(state.max(), 200.0);
    assert_eq!(state.display_percentage(), 25);
}

#[test]
fn test_percentage_normal() {
    let state = GaugeState::new(75.0, 100.0);
    assert!((state.percentage() - 0.75).abs() < f64::EPSILON);
}

#[test]
fn test_percentage_half() {
    let state = GaugeState::new(50.0, 100.0);
    assert!((state.percentage() - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_percentage_clamped_above() {
    let state = GaugeState::new(150.0, 100.0);
    assert!((state.percentage() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_percentage_clamped_below() {
    let state = GaugeState::new(-50.0, 100.0);
    assert!((state.percentage() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_display_percentage() {
    let state = GaugeState::new(75.0, 100.0);
    assert_eq!(state.display_percentage(), 75);
}

#[test]
fn test_display_percentage_zero() {
    let state = GaugeState::new(0.0, 100.0);
    assert_eq!(state.display_percentage(), 0);
}

#[test]
fn test_display_percentage_full() {
    let state = GaugeState::new(100.0, 100.0);
    assert_eq!(state.display_percentage(), 100);
}

#[test]
fn test_display_percentage_over_max() {
    let state = GaugeState::new(200.0, 100.0);
    assert_eq!(state.display_percentage(), 100);
}

// =========================================================================
// Threshold / color tests
// =========================================================================

#[test]
fn test_current_color_green_zone() {
    let state = GaugeState::new(50.0, 100.0);
    assert_eq!(state.current_color(), Color::Green);
}

#[test]
fn test_current_color_yellow_zone() {
    let state = GaugeState::new(75.0, 100.0);
    assert_eq!(state.current_color(), Color::Yellow);
}

#[test]
fn test_current_color_red_zone() {
    let state = GaugeState::new(95.0, 100.0);
    assert_eq!(state.current_color(), Color::Red);
}

#[test]
fn test_current_color_at_threshold_boundary_70() {
    let state = GaugeState::new(70.0, 100.0);
    assert_eq!(state.current_color(), Color::Yellow);
}

#[test]
fn test_current_color_at_threshold_boundary_90() {
    let state = GaugeState::new(90.0, 100.0);
    assert_eq!(state.current_color(), Color::Red);
}

#[test]
fn test_current_color_just_below_70() {
    let state = GaugeState::new(69.9, 100.0);
    assert_eq!(state.current_color(), Color::Green);
}

#[test]
fn test_current_color_just_below_90() {
    let state = GaugeState::new(89.9, 100.0);
    assert_eq!(state.current_color(), Color::Yellow);
}

#[test]
fn test_current_color_at_zero() {
    let state = GaugeState::new(0.0, 100.0);
    assert_eq!(state.current_color(), Color::Green);
}

#[test]
fn test_current_color_at_max() {
    let state = GaugeState::new(100.0, 100.0);
    assert_eq!(state.current_color(), Color::Red);
}

#[test]
fn test_current_color_custom_thresholds() {
    let state = GaugeState::new(60.0, 100.0).with_thresholds(vec![
        ThresholdZone {
            above: 0.0,
            color: Color::Blue,
        },
        ThresholdZone {
            above: 0.5,
            color: Color::Cyan,
        },
        ThresholdZone {
            above: 0.8,
            color: Color::Magenta,
        },
    ]);
    assert_eq!(state.current_color(), Color::Cyan);
}

#[test]
fn test_current_color_empty_thresholds() {
    let state = GaugeState::new(50.0, 100.0).with_thresholds(vec![]);
    assert_eq!(state.current_color(), Color::Green);
}

// =========================================================================
// Label formatting tests
// =========================================================================

#[test]
fn test_label_with_units() {
    let state = GaugeState::new(512.0, 1024.0).with_units("MB");
    assert_eq!(state.label_text(), "512.0 / 1024.0 MB");
}

#[test]
fn test_label_without_units() {
    let state = GaugeState::new(75.0, 100.0);
    assert_eq!(state.label_text(), "75%");
}

#[test]
fn test_label_custom() {
    let state = GaugeState::new(75.0, 100.0).with_label("Three quarters");
    assert_eq!(state.label_text(), "Three quarters");
}

#[test]
fn test_label_custom_overrides_units() {
    let state = GaugeState::new(512.0, 1024.0)
        .with_units("MB")
        .with_label("Half used");
    assert_eq!(state.label_text(), "Half used");
}

#[test]
fn test_label_percent_units() {
    let state = GaugeState::new(45.0, 100.0).with_units("%");
    assert_eq!(state.label_text(), "45.0 / 100.0 %");
}

#[test]
fn test_label_zero_value() {
    let state = GaugeState::new(0.0, 100.0);
    assert_eq!(state.label_text(), "0%");
}

#[test]
fn test_label_full_value() {
    let state = GaugeState::new(100.0, 100.0);
    assert_eq!(state.label_text(), "100%");
}

// =========================================================================
// Update message tests
// =========================================================================

#[test]
fn test_update_set_value() {
    let mut state = GaugeState::new(0.0, 100.0);
    let output = Gauge::update(&mut state, GaugeMessage::SetValue(42.0));
    assert_eq!(output, None);
    assert_eq!(state.value(), 42.0);
}

#[test]
fn test_update_set_max() {
    let mut state = GaugeState::new(50.0, 100.0);
    let output = Gauge::update(&mut state, GaugeMessage::SetMax(200.0));
    assert_eq!(output, None);
    assert_eq!(state.max(), 200.0);
}

#[test]
fn test_update_set_label() {
    let mut state = GaugeState::new(50.0, 100.0);
    let output = Gauge::update(
        &mut state,
        GaugeMessage::SetLabel(Some("New label".to_string())),
    );
    assert_eq!(output, None);
    assert_eq!(state.label_text(), "New label");
}

#[test]
fn test_update_set_label_none() {
    let mut state = GaugeState::new(50.0, 100.0).with_label("Old");
    Gauge::update(&mut state, GaugeMessage::SetLabel(None));
    assert_eq!(state.label_text(), "50%");
}

#[test]
fn test_update_set_units() {
    let mut state = GaugeState::new(512.0, 1024.0);
    Gauge::update(&mut state, GaugeMessage::SetUnits(Some("MB".to_string())));
    assert_eq!(state.label_text(), "512.0 / 1024.0 MB");
}

#[test]
fn test_update_set_units_none() {
    let mut state = GaugeState::new(512.0, 1024.0).with_units("MB");
    Gauge::update(&mut state, GaugeMessage::SetUnits(None));
    assert_eq!(state.label_text(), "50%");
}

// =========================================================================
// Instance method tests
// =========================================================================

#[test]
fn test_instance_update() {
    let mut state = GaugeState::new(0.0, 100.0);
    state.update(GaugeMessage::SetValue(42.0));
    assert_eq!(state.value(), 42.0);
}

#[test]
fn test_instance_handle_event() {
    let state = GaugeState::new(50.0, 100.0);
    let event = crate::input::Event::key(crate::input::KeyCode::Enter);
    assert_eq!(state.handle_event(&event), None);
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = GaugeState::new(50.0, 100.0);
    let event = crate::input::Event::key(crate::input::KeyCode::Enter);
    assert_eq!(state.dispatch_event(&event), None);
}

// =========================================================================
// Init tests
// =========================================================================

#[test]
fn test_init() {
    let state = Gauge::init();
    assert_eq!(state.value(), 0.0);
    assert_eq!(state.max(), 100.0);
    assert_eq!(state.display_percentage(), 0);
}

// =========================================================================
// Disabled tests
// =========================================================================

#[test]
fn test_disabled_default_is_false() {
    let state = GaugeState::new(50.0, 100.0);
    assert!(!state.is_disabled());
}

#[test]
fn test_set_disabled() {
    let mut state = GaugeState::new(50.0, 100.0);
    state.set_disabled(true);
    assert!(state.is_disabled());
    state.set_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_disableable_trait() {
    let mut state = GaugeState::new(50.0, 100.0);
    assert!(!Gauge::is_disabled(&state));
    Gauge::set_disabled(&mut state, true);
    assert!(Gauge::is_disabled(&state));
}

// =========================================================================
// Edge case tests
// =========================================================================

#[test]
fn test_value_greater_than_max() {
    let state = GaugeState::new(200.0, 100.0);
    assert_eq!(state.display_percentage(), 100);
    assert!((state.percentage() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_value_zero() {
    let state = GaugeState::new(0.0, 100.0);
    assert_eq!(state.display_percentage(), 0);
    assert!((state.percentage() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_max_zero() {
    let state = GaugeState::new(50.0, 0.0);
    assert_eq!(state.display_percentage(), 0);
    assert!((state.percentage() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_negative_value() {
    let state = GaugeState::new(-10.0, 100.0);
    assert_eq!(state.display_percentage(), 0);
    assert!((state.percentage() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_negative_max() {
    let state = GaugeState::new(50.0, -100.0);
    assert_eq!(state.display_percentage(), 0);
    assert!((state.percentage() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_both_zero() {
    let state = GaugeState::new(0.0, 0.0);
    assert_eq!(state.display_percentage(), 0);
}

// =========================================================================
// GaugeVariant tests
// =========================================================================

#[test]
fn test_gauge_variant_default_is_full() {
    assert_eq!(GaugeVariant::default(), GaugeVariant::Full);
}

#[test]
fn test_gauge_variant_clone() {
    let variant = GaugeVariant::Line;
    let cloned = variant.clone();
    assert_eq!(variant, cloned);
}

#[test]
fn test_gauge_variant_debug() {
    let variant = GaugeVariant::Full;
    assert_eq!(format!("{:?}", variant), "Full");
}

// =========================================================================
// ThresholdZone tests
// =========================================================================

#[test]
fn test_threshold_zone_clone() {
    let zone = ThresholdZone {
        above: 0.5,
        color: Color::Yellow,
    };
    let cloned = zone.clone();
    assert_eq!(zone, cloned);
}

#[test]
fn test_threshold_zone_debug() {
    let zone = ThresholdZone {
        above: 0.5,
        color: Color::Yellow,
    };
    let debug = format!("{:?}", zone);
    assert!(debug.contains("0.5"));
    assert!(debug.contains("Yellow"));
}

// =========================================================================
// View snapshot tests
// =========================================================================

#[test]
fn test_view_full_gauge() {
    let state = GaugeState::new(50.0, 100.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Gauge::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_full_gauge_with_title() {
    let state = GaugeState::new(75.0, 100.0).with_title("CPU Usage");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Gauge::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_full_gauge_with_units() {
    let state = GaugeState::new(512.0, 1024.0)
        .with_units("MB")
        .with_title("Memory");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Gauge::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_line_gauge() {
    let state = GaugeState::new(50.0, 100.0).with_variant(GaugeVariant::Line);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Gauge::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_line_gauge_with_title() {
    let state = GaugeState::new(75.0, 100.0)
        .with_variant(GaugeVariant::Line)
        .with_title("Disk");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Gauge::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = GaugeState::new(50.0, 100.0).with_disabled(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Gauge::view(
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
fn test_view_green_zone() {
    let state = GaugeState::new(30.0, 100.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Gauge::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_yellow_zone() {
    let state = GaugeState::new(80.0, 100.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Gauge::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_red_zone() {
    let state = GaugeState::new(95.0, 100.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Gauge::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_zero_percent() {
    let state = GaugeState::new(0.0, 100.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Gauge::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_full_percent() {
    let state = GaugeState::new(100.0, 100.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Gauge::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// =========================================================================
// Annotation tests
// =========================================================================

#[test]
fn test_annotation_emitted_full() {
    use crate::annotation::{with_annotations, WidgetType};
    let state = GaugeState::new(50.0, 100.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Gauge::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Custom("Gauge".into()));
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.value, Some("50%".to_string()));
}

#[test]
fn test_annotation_emitted_line() {
    use crate::annotation::{with_annotations, WidgetType};
    let state = GaugeState::new(75.0, 100.0).with_variant(GaugeVariant::Line);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Gauge::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Custom("Gauge".into()));
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.value, Some("75%".to_string()));
}
