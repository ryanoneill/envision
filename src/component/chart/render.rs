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

/// Renders a line, area, or scatter chart using ratatui's Chart widget with shared axes.
///
/// All non-bar chart types are rendered through this path, using braille
/// markers for high-resolution output with multi-series overlay.
pub(super) fn render_shared_axis_chart(
    state: &ChartState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    _focused: bool,
    disabled: bool,
) {
    if state.series.is_empty() && state.thresholds.is_empty() {
        return;
    }

    let effective_min = state.effective_min();
    let effective_max = state.effective_max();
    let scale = &state.y_scale;

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

    // Build data vectors, applying Y-axis scale transform
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
                .map(|(i, v)| (i as f64, scale.transform(*v)))
                .collect()
        })
        .collect();

    // Build threshold data vectors with scale transform
    let threshold_data: Vec<Vec<(f64, f64)>> = state
        .thresholds
        .iter()
        .map(|t| {
            let tv = scale.transform(t.value);
            vec![(0.0, tv), (max_x, tv)]
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

    // Build axes with tick labels
    let max_x_ticks = (area.width / 10).max(2) as usize;
    let max_y_ticks = (area.height / 3).max(2) as usize;

    let x_ticks = super::ticks::nice_ticks(0.0, max_x, max_x_ticks);

    // For logarithmic scales, generate ticks in data space then transform
    let (y_tick_positions, y_labels) = if scale.is_logarithmic() {
        let data_ticks = super::scale::log_ticks(effective_min, effective_max, max_y_ticks);
        let labels: Vec<String> = data_ticks
            .iter()
            .map(|v| super::scale::format_log_tick(*v))
            .collect();
        let positions: Vec<f64> = data_ticks.iter().map(|v| scale.transform(*v)).collect();
        (positions, labels)
    } else {
        let y_ticks = super::ticks::nice_ticks(effective_min, effective_max, max_y_ticks);
        let y_step = if y_ticks.len() >= 2 {
            y_ticks[1] - y_ticks[0]
        } else {
            1.0
        };
        let labels: Vec<String> = y_ticks
            .iter()
            .map(|v| super::ticks::format_tick(*v, y_step))
            .collect();
        (y_ticks, labels)
    };

    let x_step = if x_ticks.len() >= 2 {
        x_ticks[1] - x_ticks[0]
    } else {
        1.0
    };
    let x_labels: Vec<String> = x_ticks
        .iter()
        .map(|v| super::ticks::format_tick(*v, x_step))
        .collect();

    // Use the tick bounds for axis range (may extend slightly beyond data)
    let x_min_bound = x_ticks.first().copied().unwrap_or(0.0);
    let x_max_bound = x_ticks.last().copied().unwrap_or(max_x);
    let y_min_bound = y_tick_positions
        .first()
        .copied()
        .unwrap_or(scale.transform(effective_min));
    let y_max_bound = y_tick_positions
        .last()
        .copied()
        .unwrap_or(scale.transform(effective_max));

    let mut x_axis = RatatuiAxis::default()
        .bounds([x_min_bound, x_max_bound])
        .labels(x_labels);
    if let Some(ref label) = state.x_label {
        x_axis = x_axis.title(label.as_str());
    }

    let mut y_axis = RatatuiAxis::default()
        .bounds([y_min_bound, y_max_bound])
        .labels(y_labels);
    if let Some(ref label) = state.y_label {
        y_axis = y_axis.title(label.as_str());
    }

    let chart = RatatuiChart::new(datasets).x_axis(x_axis).y_axis(y_axis);

    frame.render_widget(chart, area);
}
