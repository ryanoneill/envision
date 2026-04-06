use super::*;
use crate::component::test_utils;

fn sample_series() -> Vec<DataSeries> {
    vec![
        DataSeries::new("Series A", vec![10.0, 20.0, 30.0, 25.0, 15.0]),
        DataSeries::new("Series B", vec![5.0, 15.0, 10.0, 20.0, 25.0]).with_color(Color::Red),
    ]
}

#[test]
fn test_area_chart_renders_differently_from_line_chart() {
    let series = vec![DataSeries::new(
        "CPU",
        vec![45.0, 52.0, 48.0, 65.0, 72.0, 58.0, 61.0],
    )];

    // Render as line chart
    let line_state = ChartState::line(series.clone());
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(
                &line_state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::default(),
            );
        })
        .unwrap();
    let line_output = terminal.backend().to_string();

    // Render as area chart
    let area_state = ChartState::area(series);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(
                &area_state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::default(),
            );
        })
        .unwrap();
    let area_output = terminal.backend().to_string();

    // Area chart should have fill characters that line chart does not
    assert_ne!(
        line_output, area_output,
        "Area chart should render differently from line chart"
    );
    assert!(
        area_output.contains('\u{2591}'),
        "Area chart should contain light shade fill characters"
    );
    assert!(
        !line_output.contains('\u{2591}'),
        "Line chart should not contain light shade fill characters"
    );
}

#[test]
fn test_area_fill_does_not_overwrite_braille_dots() {
    let series = vec![DataSeries::new(
        "CPU",
        vec![45.0, 52.0, 48.0, 65.0, 72.0, 58.0, 61.0],
    )];

    // Render as line chart to get the braille dot positions
    let line_state = ChartState::line(series.clone());
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(
                &line_state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::default(),
            );
        })
        .unwrap();
    let line_output = terminal.backend().to_string();

    // Render as area chart
    let area_state = ChartState::area(series);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(
                &area_state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::default(),
            );
        })
        .unwrap();
    let area_output = terminal.backend().to_string();

    // Both outputs should have the same number of characters (same rendering grid)
    let line_chars: Vec<char> = line_output.chars().collect();
    let area_chars: Vec<char> = area_output.chars().collect();
    assert_eq!(line_chars.len(), area_chars.len());

    // Every non-space, non-fill character in the line chart should also appear
    // in the area chart at the same position, proving fill doesn't overwrite
    // the line data
    for (line_char, area_char) in line_chars.iter().zip(area_chars.iter()) {
        if *line_char != ' ' && *line_char != '\u{2591}' {
            assert_eq!(
                line_char, area_char,
                "Area fill should not overwrite existing content: expected '{}', got '{}'",
                line_char, area_char
            );
        }
    }
}

#[test]
fn test_scatter_chart_has_no_area_fill() {
    let series = vec![DataSeries::new(
        "Points",
        vec![10.0, 25.0, 15.0, 30.0, 20.0],
    )];
    let state = ChartState::scatter(series);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    let output = terminal.backend().to_string();
    assert!(
        !output.contains('\u{2591}'),
        "Scatter chart should not contain area fill characters"
    );
}

#[test]
fn test_area_chart_with_two_data_points() {
    // Test area fill with minimal data (2 points is the minimum for a line segment)
    let state = ChartState::area(vec![DataSeries::new("Solo", vec![30.0, 70.0])]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    let output = terminal.backend().to_string();
    assert!(
        output.contains('\u{2591}'),
        "Area chart with two data points should contain fill characters"
    );
}

#[test]
fn test_area_chart_fill_with_y_range() {
    // With y_range 0-100, the fill should extend further down
    let state = ChartState::area(vec![DataSeries::new(
        "CPU",
        vec![80.0, 85.0, 90.0, 75.0, 80.0],
    )])
    .with_y_range(0.0, 100.0);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    let output = terminal.backend().to_string();
    assert!(
        output.contains('\u{2591}'),
        "Area chart with y_range should contain fill characters"
    );
}

#[test]
fn test_area_chart_multi_series_fill() {
    let state = ChartState::area(sample_series());
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    let output = terminal.backend().to_string();
    assert!(
        output.contains('\u{2591}'),
        "Multi-series area chart should contain fill characters"
    );
}

#[test]
fn test_line_chart_has_no_area_fill() {
    let series = vec![DataSeries::new(
        "CPU",
        vec![45.0, 52.0, 48.0, 65.0, 72.0, 58.0, 61.0],
    )];
    let state = ChartState::line(series);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    let output = terminal.backend().to_string();
    assert!(
        !output.contains('\u{2591}'),
        "Line chart should not contain area fill characters"
    );
}
