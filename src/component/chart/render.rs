//! Rendering functions for the Chart component.
//!
//! Extracted from the main chart module to keep file sizes manageable.
//! Contains renderers for bar charts and shared-axis charts (line, area,
//! scatter) using braille markers, as well as legend and threshold rendering.

use ratatui::prelude::*;
use ratatui::widgets::{
    Axis as RatatuiAxis, Bar, BarChart, BarGroup, Chart as RatatuiChart, Dataset, GraphType,
    Paragraph,
};

use super::{ChartKind, ChartState};
use crate::theme::Theme;

/// Renders the legend showing series labels and colors.
pub(super) fn render_legend(state: &ChartState, frame: &mut Frame, area: Rect) {
    let spans: Vec<Span> = state
        .series
        .iter()
        .enumerate()
        .flat_map(|(i, s)| {
            let marker = if i == state.active_series {
                "●"
            } else {
                "○"
            };
            let separator = if i < state.series.len() - 1 { "  " } else { "" };
            vec![Span::styled(
                format!("{} {}{}", marker, s.label(), separator),
                Style::default().fg(s.color()),
            )]
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

/// Renders a bar chart.
pub(super) fn render_bar_chart(
    state: &ChartState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    horizontal: bool,
    _focused: bool,
    disabled: bool,
) {
    if state.series.is_empty() {
        return;
    }

    // For bar charts, use the first series (or active series)
    let series = &state.series[state.active_series];
    if series.is_empty() {
        return;
    }

    let style = if disabled {
        theme.disabled_style()
    } else {
        Style::default().fg(series.color())
    };

    // Create bars from the series values
    let bars: Vec<Bar> = series
        .values()
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let label = format!("{}", i + 1);
            Bar::default()
                .value(v.max(0.0) as u64)
                .label(Line::from(label))
                .style(style)
        })
        .collect();

    let group = BarGroup::default().bars(&bars);

    let mut bar_chart = BarChart::default()
        .data(group)
        .bar_width(state.bar_width)
        .bar_gap(state.bar_gap)
        .bar_style(style);

    if horizontal {
        bar_chart = bar_chart.direction(Direction::Horizontal);
    }

    frame.render_widget(bar_chart, area);
}

/// Renders an area or scatter chart using ratatui's Chart widget with shared axes.
///
/// This is used for `ChartKind::Area` and `ChartKind::Scatter`, and renders all
/// series overlaid on shared X and Y axes.
pub(super) fn render_shared_axis_chart(
    state: &ChartState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    _focused: bool,
    disabled: bool,
) {
    if state.series.is_empty() && state.thresholds.is_empty() && state.vertical_lines.is_empty() {
        return;
    }

    let effective_min = state.effective_min();
    let effective_max = state.effective_max();

    // Compute max x value across all series
    let max_x = state
        .series
        .iter()
        .map(|s| {
            let len = s.values().len();
            if len > state.max_display_points {
                state.max_display_points
            } else {
                len
            }
        })
        .max()
        .unwrap_or(1)
        .max(1) as f64
        - 1.0;
    let max_x = max_x.max(1.0);

    let graph_type = match state.kind {
        ChartKind::Scatter => GraphType::Scatter,
        _ => GraphType::Line,
    };

    // Build data vectors that outlive the datasets
    let series_data: Vec<Vec<(f64, f64)>> = state
        .series
        .iter()
        .map(|s| {
            let values = if s.values().len() > state.max_display_points {
                &s.values()[s.values().len() - state.max_display_points..]
            } else {
                s.values()
            };
            values
                .iter()
                .enumerate()
                .map(|(i, v)| (i as f64, *v))
                .collect()
        })
        .collect();

    // Build threshold data vectors (horizontal reference lines)
    let threshold_data: Vec<Vec<(f64, f64)>> = state
        .thresholds
        .iter()
        .map(|t| vec![(0.0, t.value), (max_x, t.value)])
        .collect();

    // Build vertical reference line data vectors
    let vline_data: Vec<Vec<(f64, f64)>> = state
        .vertical_lines
        .iter()
        .map(|v| vec![(v.x_value, effective_min), (v.x_value, effective_max)])
        .collect();

    // Build datasets referencing the data vectors
    let mut datasets: Vec<Dataset> = state
        .series
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if disabled {
                theme.disabled_style()
            } else if i == state.active_series {
                Style::default().fg(s.color()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(s.color())
            };
            Dataset::default()
                .name(s.label())
                .data(&series_data[i])
                .marker(symbols::Marker::Braille)
                .graph_type(graph_type)
                .style(style)
        })
        .collect();

    // Add threshold lines as additional datasets
    for (i, threshold) in state.thresholds.iter().enumerate() {
        let style = Style::default().fg(threshold.color);
        datasets.push(
            Dataset::default()
                .name(threshold.label.as_str())
                .data(&threshold_data[i])
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(style),
        );
    }

    // Add vertical reference lines as additional datasets
    for (i, vline) in state.vertical_lines.iter().enumerate() {
        let style = Style::default().fg(vline.color);
        datasets.push(
            Dataset::default()
                .name(vline.label.as_str())
                .data(&vline_data[i])
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(style),
        );
    }

    // Build axes
    let x_axis = if let Some(ref label) = state.x_label {
        RatatuiAxis::default()
            .bounds([0.0, max_x])
            .title(label.as_str())
            .labels(["0".into(), format!("{:.0}", max_x)])
    } else {
        RatatuiAxis::default()
            .bounds([0.0, max_x])
            .labels(["0".into(), format!("{:.0}", max_x)])
    };

    let y_axis = if let Some(ref label) = state.y_label {
        RatatuiAxis::default()
            .bounds([effective_min, effective_max])
            .title(label.as_str())
            .labels([
                format!("{:.0}", effective_min),
                format!("{:.0}", effective_max),
            ])
    } else {
        RatatuiAxis::default()
            .bounds([effective_min, effective_max])
            .labels([
                format!("{:.0}", effective_min),
                format!("{:.0}", effective_max),
            ])
    };

    let chart = RatatuiChart::new(datasets).x_axis(x_axis).y_axis(y_axis);

    frame.render_widget(chart, area);
}
