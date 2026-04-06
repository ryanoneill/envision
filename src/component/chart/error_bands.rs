//! Error band / confidence interval rendering for charts.

use ratatui::prelude::*;

use super::ChartState;
use super::render::interpolate_y;
use crate::theme::Theme;

/// Fills the shaded region between upper and lower bounds for error bands.
#[allow(clippy::too_many_arguments)]
pub(super) fn fill_error_bands(
    state: &ChartState,
    frame: &mut Frame,
    graph_area: Rect,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    disabled: bool,
    theme: &Theme,
    is_log: bool,
) {
    let x_range = x_max - x_min;
    let y_range = y_max - y_min;
    if x_range <= 0.0 || y_range <= 0.0 {
        return;
    }
    let buf = frame.buffer_mut();
    for series in &state.series {
        let upper = series.upper_bound();
        let lower = series.lower_bound();
        if upper.is_none() && lower.is_none() {
            continue;
        }
        let color = if disabled {
            theme.disabled_style().fg.unwrap_or(Color::DarkGray)
        } else {
            dim_color(series.color())
        };
        let upper_data: Vec<(f64, f64)> = upper.map_or_else(Vec::new, |ub| {
            build_bound_data(series, ub, is_log, &state.y_scale)
        });
        let lower_data: Vec<(f64, f64)> = lower.map_or_else(Vec::new, |lb| {
            build_bound_data(series, lb, is_log, &state.y_scale)
        });
        let main_data: Vec<(f64, f64)> =
            build_bound_data(series, series.values(), is_log, &state.y_scale);
        for screen_x in graph_area.x..graph_area.right() {
            let x_frac =
                (screen_x - graph_area.x) as f64 / (graph_area.width as f64 - 1.0).max(1.0);
            let data_x = x_min + x_frac * x_range;
            let upper_y = if !upper_data.is_empty() {
                interpolate_y(&upper_data, data_x)
            } else {
                None
            };
            let lower_y = if !lower_data.is_empty() {
                interpolate_y(&lower_data, data_x)
            } else {
                None
            };
            let (band_upper, band_lower) = match (upper_y, lower_y) {
                (Some(u), Some(l)) => (u, l),
                (Some(u), None) => match interpolate_y(&main_data, data_x) {
                    Some(m) => (u, m),
                    None => continue,
                },
                (None, Some(l)) => match interpolate_y(&main_data, data_x) {
                    Some(m) => (m, l),
                    None => continue,
                },
                (None, None) => continue,
            };
            if band_upper <= band_lower {
                continue;
            }
            let upper_frac = f64::clamp((band_upper - y_min) / y_range, 0.0, 1.0);
            let lower_frac = f64::clamp((band_lower - y_min) / y_range, 0.0, 1.0);
            let upper_screen_y = graph_area
                .bottom()
                .saturating_sub(1)
                .saturating_sub((upper_frac * (graph_area.height as f64 - 1.0)) as u16);
            let lower_screen_y = graph_area
                .bottom()
                .saturating_sub(1)
                .saturating_sub((lower_frac * (graph_area.height as f64 - 1.0)) as u16);
            for y in upper_screen_y..=lower_screen_y.min(graph_area.bottom()) {
                if let Some(cell) = buf.cell_mut(Position::new(screen_x, y)) {
                    if cell.symbol() == " " {
                        cell.set_char('\u{2591}');
                        cell.set_fg(color);
                    }
                }
            }
        }
    }
}

fn build_bound_data(
    series: &super::DataSeries,
    bound_values: &[f64],
    is_log: bool,
    scale: &super::Scale,
) -> Vec<(f64, f64)> {
    let points: Vec<(f64, f64)> = if let Some(x_vals) = series.x_values() {
        x_vals
            .iter()
            .zip(bound_values)
            .map(|(&x, &y)| (x, y))
            .collect()
    } else {
        bound_values
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v))
            .collect()
    };
    if is_log {
        points
            .into_iter()
            .map(|(x, y)| (x, scale.transform(y)))
            .collect()
    } else {
        points
    }
}

fn dim_color(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(r / 2, g / 2, b / 2),
        _ => Color::DarkGray,
    }
}
