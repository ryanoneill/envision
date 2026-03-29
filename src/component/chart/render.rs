//! Rendering functions for the Chart component.
//!
//! Extracted from the main chart module to keep file sizes manageable.
//! Contains renderers for line (sparkline), bar, area, and scatter charts,
//! as well as legend, axis labels, and threshold line rendering.

use ratatui::prelude::*;
use ratatui::widgets::{
    Axis as RatatuiAxis, Bar, BarChart, BarGroup, Chart as RatatuiChart, Dataset, GraphType,
    Paragraph, Sparkline,
};

use super::{ChartKind, ChartState, DataSeries};
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

/// Renders a line chart using sparkline.
pub(super) fn render_line_chart(state: &ChartState, frame: &mut Frame, area: Rect, theme: &Theme) {
    if state.series.is_empty() {
        return;
    }

    // Show y-axis labels on the left
    let y_label_width = if state.y_label.is_some() { 8u16 } else { 0 };

    let (y_area, chart_area) = if y_label_width > 0 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(y_label_width), Constraint::Min(1)])
            .split(area);
        (Some(chunks[0]), chunks[1])
    } else {
        (None, area)
    };

    // Render y-axis min/max labels
    if let Some(y_area) = y_area {
        let global_max = state.effective_max();
        let global_min = state.effective_min();
        let max_text = format!("{:.1}", global_max);
        let min_text = format!("{:.1}", global_min);

        if y_area.height >= 2 {
            let p_max = Paragraph::new(max_text)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Right);
            frame.render_widget(p_max, Rect::new(y_area.x, y_area.y, y_area.width, 1));

            let p_min = Paragraph::new(min_text)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Right);
            frame.render_widget(
                p_min,
                Rect::new(y_area.x, y_area.y + y_area.height - 1, y_area.width, 1),
            );
        }
    }

    // For multi-series, stack sparklines vertically
    if state.series.len() == 1 || chart_area.height < 2 {
        // Single series: full area sparkline
        let series = &state.series[state.active_series];
        let data = series_to_sparkline_data(series, state.max_display_points);
        let style = if state.disabled {
            theme.disabled_style()
        } else {
            Style::default().fg(series.color())
        };
        let sparkline = Sparkline::default().data(&data).style(style);
        frame.render_widget(sparkline, chart_area);
    } else {
        // Multi-series: divide height
        let count = state.series.len() as u16;
        let constraints: Vec<Constraint> = (0..count)
            .map(|_| Constraint::Ratio(1, count as u32))
            .collect();

        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(chart_area);

        for (i, series) in state.series.iter().enumerate() {
            if let Some(sparkline_area) = areas.get(i) {
                let data = series_to_sparkline_data(series, state.max_display_points);
                let style = if state.disabled {
                    theme.disabled_style()
                } else if i == state.active_series {
                    Style::default()
                        .fg(series.color())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(series.color())
                };
                let sparkline = Sparkline::default().data(&data).style(style);
                frame.render_widget(sparkline, *sparkline_area);
            }
        }
    }
}

/// Converts a data series to sparkline-compatible u64 data.
pub(super) fn series_to_sparkline_data(series: &DataSeries, max_points: usize) -> Vec<u64> {
    let values = if series.values().len() > max_points {
        &series.values()[series.values().len() - max_points..]
    } else {
        series.values()
    };

    if values.is_empty() {
        return Vec::new();
    }

    let min = values.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let max = values.iter().copied().reduce(f64::max).unwrap_or(0.0);
    let range = max - min;

    if range == 0.0 {
        return values.iter().map(|_| 50).collect();
    }

    values
        .iter()
        .map(|v| ((v - min) / range * 100.0) as u64)
        .collect()
}

/// Renders a bar chart.
pub(super) fn render_bar_chart(
    state: &ChartState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    horizontal: bool,
) {
    if state.series.is_empty() {
        return;
    }

    // For bar charts, use the first series (or active series)
    let series = &state.series[state.active_series];
    if series.is_empty() {
        return;
    }

    let style = if state.disabled {
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
) {
    if state.series.is_empty() && state.thresholds.is_empty() {
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

    // Build threshold data vectors
    let threshold_data: Vec<Vec<(f64, f64)>> = state
        .thresholds
        .iter()
        .map(|t| vec![(0.0, t.value), (max_x, t.value)])
        .collect();

    // Build datasets referencing the data vectors
    let mut datasets: Vec<Dataset> = state
        .series
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if state.disabled {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_to_sparkline_data() {
        let s = DataSeries::new("Test", vec![0.0, 50.0, 100.0]);
        let data = series_to_sparkline_data(&s, 50);
        assert_eq!(data, vec![0, 50, 100]);
    }

    #[test]
    fn test_series_to_sparkline_data_constant() {
        let s = DataSeries::new("Test", vec![5.0, 5.0, 5.0]);
        let data = series_to_sparkline_data(&s, 50);
        assert_eq!(data, vec![50, 50, 50]);
    }

    #[test]
    fn test_series_to_sparkline_data_bounded() {
        let s = DataSeries::new("Test", vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let data = series_to_sparkline_data(&s, 3);
        assert_eq!(data.len(), 3); // Only last 3 points
    }

    #[test]
    fn test_series_to_sparkline_data_empty() {
        let s = DataSeries::new("Test", vec![]);
        let data = series_to_sparkline_data(&s, 50);
        assert!(data.is_empty());
    }
}
