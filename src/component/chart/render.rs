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

use super::format::smart_format;
use super::{ChartKind, ChartState};
use crate::theme::Theme;

/// Renders the legend showing series labels and colors.
pub(super) fn render_legend(state: &ChartState, frame: &mut Frame, area: Rect) {
    let total_entries = state.series.len() + state.thresholds.len() + state.vertical_lines.len();
    let mut entry_index = 0;

    let mut spans: Vec<Span> = state
        .series
        .iter()
        .enumerate()
        .flat_map(|(i, s)| {
            let marker = if i == state.active_series {
                "●"
            } else {
                "○"
            };
            entry_index += 1;
            let separator = if entry_index < total_entries {
                "  "
            } else {
                ""
            };
            vec![Span::styled(
                format!("{} {}{}", marker, s.label(), separator),
                Style::default().fg(s.color()),
            )]
        })
        .collect();

    for threshold in &state.thresholds {
        entry_index += 1;
        let separator = if entry_index < total_entries {
            "  "
        } else {
            ""
        };
        spans.push(Span::styled(
            format!("── {}{}", threshold.label, separator),
            Style::default().fg(threshold.color),
        ));
    }

    for vline in &state.vertical_lines {
        entry_index += 1;
        let separator = if entry_index < total_entries {
            "  "
        } else {
            ""
        };
        spans.push(Span::styled(
            format!("│ {}{}", vline.label, separator),
            Style::default().fg(vline.color),
        ));
    }

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

    // Create bars from the series values, using category labels when available
    let bars: Vec<Bar> = series
        .values()
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let label = if i < state.categories().len() {
                state.categories()[i].clone()
            } else {
                format!("{}", i + 1)
            };
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

    // Compute X-axis bounds across all series.
    // For series with explicit x_values, use those min/max values.
    // For series with implicit indices, use 0..len-1.
    let (min_x_data, max_x_data) = {
        let mut overall_min = f64::INFINITY;
        let mut overall_max = f64::NEG_INFINITY;
        for s in &state.series {
            if let Some(x_vals) = s.x_values() {
                if let Some(&x_min) = x_vals.iter().reduce(|a, b| if a < b { a } else { b }) {
                    overall_min = overall_min.min(x_min);
                }
                if let Some(&x_max) = x_vals.iter().reduce(|a, b| if a > b { a } else { b }) {
                    overall_max = overall_max.max(x_max);
                }
            } else {
                let len = s.values().len();
                if len > 0 {
                    overall_min = overall_min.min(0.0);
                    overall_max = overall_max.max((len - 1) as f64);
                }
            }
        }
        if overall_min.is_infinite() {
            (0.0, 1.0)
        } else {
            (overall_min, overall_max.max(overall_min + 1.0))
        }
    };
    let max_x = max_x_data;

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
            // Convert to (x, y) pairs, using explicit x_values if available
            let points: Vec<(f64, f64)> = if let Some(x_vals) = s.x_values() {
                x_vals
                    .iter()
                    .zip(s.values())
                    .map(|(&x, &y)| (x, y))
                    .collect()
            } else {
                s.values()
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| (i as f64, v))
                    .collect()
            };

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
            // Use empty name to suppress ratatui's internal legend box;
            // our custom legend below the chart handles all entries.
            Dataset::default()
                .name("")
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
                .name("")
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
                .name("")
                .data(&vline_data[i])
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(style),
        );
    }

    // Add crosshair cursor as a vertical line dataset
    let crosshair_data = if state.show_crosshair {
        state.cursor_position.map(|pos| {
            let x = pos as f64;
            let y_min = scale.transform(effective_min);
            let y_max = scale.transform(effective_max);
            vec![(x, y_min), (x, y_max)]
        })
    } else {
        None
    };
    if let Some(ref data) = crosshair_data {
        datasets.push(
            Dataset::default()
                .name("")
                .data(data)
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::White)),
        );
    }

    // Generate tick labels
    let max_x_ticks = (area.width / 10).max(2) as usize;
    let max_y_ticks = (area.height / 3).max(2) as usize;

    let x_ticks = super::ticks::nice_ticks(min_x_data, max_x, max_x_ticks);
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
    let x_axis = RatatuiAxis::default()
        .bounds([x_bound_min, x_bound_max])
        .labels(x_labels);

    let y_axis = RatatuiAxis::default()
        .bounds([y_axis_min, y_axis_max])
        .labels(y_labels)
        .labels_alignment(Alignment::Right);

    let chart = RatatuiChart::new(datasets).x_axis(x_axis).y_axis(y_axis);

    frame.render_widget(chart, area);

    if state.show_grid {
        render_grid_lines(frame, area, &y_ticks_values, y_axis_min, y_axis_max);
    }
}

/// Renders the crosshair value readout overlay at the top of the chart area.
pub(super) fn render_crosshair_readout(
    state: &ChartState,
    frame: &mut Frame,
    area: Rect,
    cursor_pos: usize,
) {
    if area.height < 2 || area.width < 10 {
        return;
    }

    let mut spans = vec![Span::styled(
        format!("x:{}", cursor_pos),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )];

    for series in &state.series {
        if let Some(&value) = series.values().get(cursor_pos) {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                format!("{}: {}", series.label(), smart_format(value, None)),
                Style::default().fg(series.color()),
            ));
        }
    }

    let line = Line::from(spans);
    let readout = Paragraph::new(line)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .alignment(Alignment::Left);

    let readout_area = Rect::new(area.x, area.y, area.width, 1);
    frame.render_widget(readout, readout_area);
}

/// Renders horizontal grid lines at Y-axis tick positions.
fn render_grid_lines(
    frame: &mut Frame,
    area: Rect,
    y_ticks_values: &[f64],
    y_axis_min: f64,
    y_axis_max: f64,
) {
    let y_range = y_axis_max - y_axis_min;
    if y_range <= 0.0 || area.height < 2 {
        return;
    }
    for &tick_val in y_ticks_values {
        let y_frac = (tick_val - y_axis_min) / y_range;
        let screen_y =
            area.bottom().saturating_sub(1) - (y_frac * (area.height as f64 - 1.0)) as u16;
        if screen_y > area.y && screen_y < area.bottom().saturating_sub(1) {
            for x in area.x..area.right() {
                let cell = frame.buffer_mut().cell_mut(Position::new(x, screen_y));
                if let Some(cell) = cell {
                    if cell.symbol() == " " {
                        cell.set_char('·');
                        cell.set_fg(Color::DarkGray);
                    }
                }
            }
        }
    }
}
