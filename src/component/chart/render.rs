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

    // Save label strings for graph area computation before axes consume them
    let x_labels_for_layout = x_labels.clone();
    let y_labels_for_layout = y_labels.clone();

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

    // For area charts, fill below the curve after the line has been rendered.
    if state.kind == ChartKind::Area && !state.series.is_empty() {
        let graph_area = compute_graph_area(area, &y_labels_for_layout, &x_labels_for_layout);
        if graph_area.width > 0 && graph_area.height > 0 {
            fill_area_below_curve(
                state,
                frame,
                graph_area,
                &series_data,
                x_bound_min,
                x_bound_max,
                y_axis_min,
                y_axis_max,
                disabled,
                theme,
            );
        }
    }

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

/// Computes the graph area by replicating ratatui's internal Chart layout logic.
fn compute_graph_area(area: Rect, y_labels: &[String], x_labels: &[String]) -> Rect {
    if area.height == 0 || area.width == 0 {
        return Rect::default();
    }
    let mut x = area.left();
    let mut y = area.bottom().saturating_sub(1);
    let has_x_labels = !x_labels.is_empty();
    if has_x_labels && y > area.top() {
        y = y.saturating_sub(1);
    }
    let has_y_labels = !y_labels.is_empty();
    let y_label_max_width = y_labels.iter().map(|l| l.len() as u16).max().unwrap_or(0);
    let first_x_label_width = x_labels.first().map(|l| l.len() as u16).unwrap_or(0);
    let x_label_left_width = if has_y_labels {
        first_x_label_width.saturating_sub(1)
    } else {
        first_x_label_width
    };
    let max_label_width = y_label_max_width.max(x_label_left_width);
    let capped_label_width = max_label_width.min(area.width / 3);
    x += capped_label_width;
    if has_x_labels && y > area.top() {
        y = y.saturating_sub(1);
    }
    if has_y_labels && x + 1 < area.right() {
        x += 1;
    }
    let graph_width = area.right().saturating_sub(x);
    let graph_height = y.saturating_sub(area.top()).saturating_add(1);
    Rect::new(x, area.top(), graph_width, graph_height)
}

/// Fills the area below each series curve for area charts.
#[allow(clippy::too_many_arguments)]
fn fill_area_below_curve(
    state: &ChartState,
    frame: &mut Frame,
    graph_area: Rect,
    series_data: &[Vec<(f64, f64)>],
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    disabled: bool,
    theme: &Theme,
) {
    let x_range = x_max - x_min;
    let y_range = y_max - y_min;
    if x_range <= 0.0 || y_range <= 0.0 {
        return;
    }
    let buf = frame.buffer_mut();
    for (series_idx, series) in state.series.iter().enumerate() {
        let color = if disabled {
            theme.disabled_style().fg.unwrap_or(Color::DarkGray)
        } else {
            series.color()
        };
        let data = &series_data[series_idx];
        if data.len() < 2 {
            if let Some(&(dx, dy)) = data.first() {
                let x_frac = (dx - x_min) / x_range;
                let screen_x = graph_area.x + (x_frac * (graph_area.width as f64 - 1.0)) as u16;
                let y_frac = (dy - y_min) / y_range;
                let line_y = graph_area
                    .bottom()
                    .saturating_sub(1)
                    .saturating_sub((y_frac * (graph_area.height as f64 - 1.0)) as u16);
                fill_column(buf, screen_x, line_y + 1, graph_area.bottom(), color);
            }
            continue;
        }
        for screen_x in graph_area.x..graph_area.right() {
            let x_frac =
                (screen_x - graph_area.x) as f64 / (graph_area.width as f64 - 1.0).max(1.0);
            let data_x = x_min + x_frac * x_range;
            let data_y = interpolate_y(data, data_x);
            if let Some(dy) = data_y {
                let y_frac = ((dy - y_min) / y_range).clamp(0.0, 1.0);
                let line_y = graph_area
                    .bottom()
                    .saturating_sub(1)
                    .saturating_sub((y_frac * (graph_area.height as f64 - 1.0)) as u16);
                fill_column(buf, screen_x, line_y + 1, graph_area.bottom(), color);
            }
        }
    }
}

/// Fills a single column of cells with the area fill character.
fn fill_column(buf: &mut Buffer, x: u16, y_start: u16, y_end: u16, color: Color) {
    for y in y_start..y_end {
        if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
            if cell.symbol() == " " {
                cell.set_char('\u{2591}');
                cell.set_fg(color);
            }
        }
    }
}

/// Linearly interpolates the Y value for a given X within a sorted data series.
fn interpolate_y(data: &[(f64, f64)], x: f64) -> Option<f64> {
    if data.is_empty() {
        return None;
    }
    let (first_x, _) = data[0];
    let (last_x, _) = data[data.len() - 1];
    if x < first_x || x > last_x {
        return None;
    }
    for window in data.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];
        if x >= x0 && x <= x1 {
            if (x1 - x0).abs() < f64::EPSILON {
                return Some(y0);
            }
            let t = (x - x0) / (x1 - x0);
            return Some(y0 + t * (y1 - y0));
        }
    }
    Some(data[data.len() - 1].1)
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
