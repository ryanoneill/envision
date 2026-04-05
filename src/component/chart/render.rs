//! Rendering functions for the Chart component.
//!
//! Extracted from the main chart module to keep file sizes manageable.
//! Contains renderers for bar charts and shared-axis charts (line, area, scatter),
//! as well as legend, axis labels, and threshold line rendering.

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

/// Renders a line, area, or scatter chart using ratatui's Chart widget with shared axes.
///
/// This is used for `ChartKind::Line`, `ChartKind::Area`, and `ChartKind::Scatter`,
/// and renders all series overlaid on shared X and Y axes. Uses LTTB downsampling
/// for large datasets and applies scale transforms for logarithmic axes.
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

    // Compute max x value across all series (using full series length, not truncated)
    let max_x = state
        .series
        .iter()
        .map(|s| s.values().len())
        .max()
        .unwrap_or(1)
        .max(1) as f64
        - 1.0;
    let max_x = max_x.max(1.0);

    // Compute effective max points based on render area width
    let effective_max_points = (area.width as usize * 2).min(state.max_display_points);

    let graph_type = match state.kind {
        ChartKind::Scatter => GraphType::Scatter,
        _ => GraphType::Line,
    };

    let is_log = state.y_scale.is_logarithmic();

    // Build data vectors with LTTB downsampling and scale transforms
    let series_data: Vec<Vec<(f64, f64)>> = state
        .series
        .iter()
        .map(|s| {
            // Convert to (x, y) pairs
            let points: Vec<(f64, f64)> = s
                .values()
                .iter()
                .enumerate()
                .map(|(i, v)| (i as f64, *v))
                .collect();

            // Apply LTTB downsampling
            let downsampled = if points.len() > effective_max_points {
                super::downsample::lttb(&points, effective_max_points)
            } else {
                points
            };

            // Apply scale transform to Y values
            if is_log {
                downsampled
                    .into_iter()
                    .map(|(x, y)| (x, state.y_scale.transform(y)))
                    .collect()
            } else {
                downsampled
            }
        })
        .collect();

    // Transform effective min/max through the scale
    let (y_bound_min, y_bound_max) = if is_log {
        (
            state
                .y_scale
                .transform(effective_min.max(f64::MIN_POSITIVE)),
            state
                .y_scale
                .transform(effective_max.max(f64::MIN_POSITIVE)),
        )
    } else {
        (effective_min, effective_max)
    };

    // Build threshold data vectors (with scale transforms)
    let threshold_data: Vec<Vec<(f64, f64)>> = state
        .thresholds
        .iter()
        .map(|t| {
            let y = if is_log {
                state.y_scale.transform(t.value.max(f64::MIN_POSITIVE))
            } else {
                t.value
            };
            vec![(0.0, y), (max_x, y)]
        })
        .collect();

    // Build vertical line data vectors (with scale transforms)
    let scale = &state.y_scale;
    let vline_data: Vec<Vec<(f64, f64)>> = state
        .vertical_lines
        .iter()
        .map(|v| {
            let tv_min = scale.transform(effective_min);
            let tv_max = scale.transform(effective_max);
            vec![(v.x_value, tv_min), (v.x_value, tv_max)]
        })
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

    // Generate tick labels
    let max_x_ticks = (area.width / 10).max(2) as usize;
    let max_y_ticks = (area.height / 3).max(2) as usize;

    let x_ticks = super::ticks::nice_ticks(0.0, max_x, max_x_ticks);
    let x_labels: Vec<String> = x_ticks
        .iter()
        .map(|&v| {
            super::ticks::format_tick(
                v,
                x_ticks.get(1).copied().unwrap_or(1.0) - x_ticks.first().copied().unwrap_or(0.0),
            )
        })
        .collect();

    let x_bound_min = x_ticks.first().copied().unwrap_or(0.0);
    let x_bound_max = x_ticks.last().copied().unwrap_or(max_x);

    // Generate Y ticks
    let (y_ticks_values, y_labels) = if is_log {
        let ticks = super::scale::log_ticks(
            effective_min.max(f64::MIN_POSITIVE),
            effective_max.max(f64::MIN_POSITIVE),
            max_y_ticks,
        );
        let labels: Vec<String> = ticks
            .iter()
            .map(|&v| super::scale::format_log_tick(v))
            .collect();
        let transformed: Vec<f64> = ticks.iter().map(|&v| state.y_scale.transform(v)).collect();
        (transformed, labels)
    } else {
        let ticks = super::ticks::nice_ticks(effective_min, effective_max, max_y_ticks);
        let step = ticks.get(1).copied().unwrap_or(effective_max)
            - ticks.first().copied().unwrap_or(effective_min);
        let labels: Vec<String> = ticks
            .iter()
            .map(|&v| super::ticks::format_tick(v, step))
            .collect();
        (ticks, labels)
    };

    let y_axis_min = y_ticks_values.first().copied().unwrap_or(y_bound_min);
    let y_axis_max = y_ticks_values.last().copied().unwrap_or(y_bound_max);

    // Build axes with tick labels
    let x_axis = if let Some(ref label) = state.x_label {
        RatatuiAxis::default()
            .bounds([x_bound_min, x_bound_max])
            .title(label.as_str())
            .labels(x_labels)
    } else {
        RatatuiAxis::default()
            .bounds([x_bound_min, x_bound_max])
            .labels(x_labels)
    };

    let y_axis = if let Some(ref label) = state.y_label {
        RatatuiAxis::default()
            .bounds([y_axis_min, y_axis_max])
            .title(label.as_str())
            .labels(y_labels)
    } else {
        RatatuiAxis::default()
            .bounds([y_axis_min, y_axis_max])
            .labels(y_labels)
    };

    let chart = RatatuiChart::new(datasets).x_axis(x_axis).y_axis(y_axis);

    frame.render_widget(chart, area);
}
