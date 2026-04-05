use super::*;
use crate::component::test_utils;

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_empty() {
    let state = BoxPlotState::default().with_title("Empty Box Plot");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_single_vertical() {
    let state = BoxPlotState::new(vec![BoxPlotData::new("API", 10.0, 20.0, 35.0, 45.0, 55.0)])
        .with_title("Latency Distribution");
    let (mut terminal, theme) = test_utils::setup_render(40, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_multiple_vertical() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("SvcA", 10.0, 20.0, 30.0, 40.0, 50.0).with_color(Color::Cyan),
        BoxPlotData::new("SvcB", 15.0, 25.0, 35.0, 45.0, 55.0).with_color(Color::Green),
        BoxPlotData::new("SvcC", 5.0, 15.0, 25.0, 35.0, 60.0).with_color(Color::Yellow),
    ])
    .with_title("Service Comparison");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_horizontal() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("SvcA", 10.0, 20.0, 30.0, 40.0, 50.0),
        BoxPlotData::new("SvcB", 15.0, 25.0, 35.0, 45.0, 55.0),
    ])
    .with_orientation(BoxPlotOrientation::Horizontal)
    .with_title("Horizontal Comparison");
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_outliers() {
    let state = BoxPlotState::new(vec![BoxPlotData::new(
        "Response", 10.0, 25.0, 35.0, 45.0, 55.0,
    )
    .with_outliers(vec![2.0, 70.0, 80.0])])
    .with_title("With Outliers");
    let (mut terminal, theme) = test_utils::setup_render(40, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let state = BoxPlotState::new(vec![BoxPlotData::new(
        "Disabled", 10.0, 20.0, 30.0, 40.0, 50.0,
    )])
    .with_title("Disabled Box Plot");
    let (mut terminal, theme) = test_utils::setup_render(40, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(
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

// =============================================================================
// Annotation tests
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = BoxPlotState::new(vec![BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                BoxPlot::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert!(registry.get_by_id("box_plot").is_some());
}

#[test]
fn test_annotation_with_focus() {
    use crate::annotation::with_annotations;
    let state = BoxPlotState::new(vec![BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                BoxPlot::view(
                    &state,
                    frame,
                    frame.area(),
                    &theme,
                    &ViewContext::new().focused(true),
                );
            })
            .unwrap();
    });
    let region = registry.get_by_id("box_plot").unwrap();
    assert!(region.annotation.focused);
}

#[test]
fn test_annotation_with_disabled() {
    use crate::annotation::with_annotations;
    let state = BoxPlotState::new(vec![BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                BoxPlot::view(
                    &state,
                    frame,
                    frame.area(),
                    &theme,
                    &ViewContext::new().disabled(true),
                );
            })
            .unwrap();
    });
    let region = registry.get_by_id("box_plot").unwrap();
    assert!(region.annotation.disabled);
}
