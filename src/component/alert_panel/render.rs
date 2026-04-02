//! Rendering functions for the AlertPanel component.
//!
//! Renders a grid of metric cards with state indicators, values, and
//! optional sparklines inside an outer border with an aggregate title.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};

use super::{AlertPanelState, AlertState};
use crate::theme::Theme;

/// State indicator symbols for each alert level.
const OK_INDICATOR: &str = "\u{25cf}"; // ●
const WARN_INDICATOR: &str = "\u{25b2}"; // ▲
const CRIT_INDICATOR: &str = "\u{2716}"; // ✖
const UNKNOWN_INDICATOR: &str = "?";

/// Renders the full alert panel.
pub(super) fn render_alert_panel(
    state: &AlertPanelState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    if area.height < 3 || area.width < 3 {
        return;
    }

    crate::annotation::with_registry(|reg| {
        reg.register(
            area,
            crate::annotation::Annotation::container("alert_panel")
                .with_focus(focused)
                .with_disabled(disabled),
        );
    });

    // Outer border with title showing aggregate counts
    let outer_border_style = if disabled {
        theme.disabled_style()
    } else if focused {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let title = state.title_with_counts();
    let outer_block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(outer_border_style);

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    if state.metrics().is_empty() || inner.height == 0 || inner.width == 0 {
        return;
    }

    let rows = state.rows();
    let cols = state.columns();

    // Compute row heights
    let row_constraints: Vec<Constraint> = (0..rows)
        .map(|_| Constraint::Ratio(1, rows as u32))
        .collect();

    let row_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(inner);

    // Compute column widths
    let col_constraints: Vec<Constraint> = (0..cols)
        .map(|_| Constraint::Ratio(1, cols as u32))
        .collect();

    for (row_idx, row_area) in row_areas.iter().enumerate() {
        let col_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints.clone())
            .split(*row_area);

        for (col_idx, col_area) in col_areas.iter().enumerate() {
            let metric_idx = row_idx * cols + col_idx;
            if let Some(metric) = state.metrics().get(metric_idx) {
                let is_selected = state.selected() == Some(metric_idx);
                render_metric_card(
                    metric,
                    is_selected,
                    state,
                    frame,
                    *col_area,
                    theme,
                    focused,
                    disabled,
                );
            }
        }
    }
}

/// Renders a single metric card within the grid.
#[allow(clippy::too_many_arguments)]
fn render_metric_card(
    metric: &super::AlertMetric,
    is_selected: bool,
    state: &AlertPanelState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    let border_style = if disabled {
        theme.disabled_style()
    } else if is_selected && focused {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .title(metric.name())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let state_style = if disabled {
        theme.disabled_style()
    } else {
        state_color(metric.state(), theme)
    };

    // Build the status line: "● OK  45.2%"
    let indicator = state_indicator(metric.state());
    let state_label = metric.state().to_string();
    let display_val = metric.display_value();

    let status_line = Line::from(vec![
        Span::styled(format!("{} ", indicator), state_style),
        Span::styled(state_label.to_string(), state_style),
        Span::raw("  "),
        Span::styled(display_val, state_style),
    ]);

    // Optionally show thresholds
    let show_sparkline = state.show_sparklines() && !metric.history().is_empty();
    let show_thresholds = state.show_thresholds();

    // Calculate layout: status line + optional threshold line + optional sparkline
    let needed_rows = 1 + if show_thresholds { 1 } else { 0 } + if show_sparkline { 1 } else { 0 };

    if inner.height < needed_rows as u16 {
        // Just render the status line if there's no space for extras
        let paragraph = Paragraph::new(status_line);
        frame.render_widget(paragraph, inner);
        return;
    }

    let mut constraints = vec![Constraint::Length(1)];
    if show_thresholds {
        constraints.push(Constraint::Length(1));
    }
    if show_sparkline {
        constraints.push(Constraint::Min(1));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    // Status line
    let paragraph = Paragraph::new(status_line);
    frame.render_widget(paragraph, chunks[0]);

    let mut chunk_idx = 1;

    // Threshold line
    if show_thresholds && chunk_idx < chunks.len() {
        let threshold = metric.threshold();
        let threshold_line = Line::from(vec![
            Span::styled(
                format!("W:{:.0}", threshold.warning),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" "),
            Span::styled(
                format!("C:{:.0}", threshold.critical),
                Style::default().fg(Color::Red),
            ),
        ]);
        let paragraph = Paragraph::new(threshold_line);
        frame.render_widget(paragraph, chunks[chunk_idx]);
        chunk_idx += 1;
    }

    // Sparkline
    if show_sparkline && chunk_idx < chunks.len() {
        let data = history_to_sparkline_data(metric.history());
        let sparkline_style = if disabled {
            theme.disabled_style()
        } else {
            state_color(metric.state(), theme)
        };
        let sparkline = Sparkline::default().data(&data).style(sparkline_style);
        frame.render_widget(sparkline, chunks[chunk_idx]);
    }
}

/// Returns the indicator symbol for an alert state.
fn state_indicator(state: &AlertState) -> &'static str {
    match state {
        AlertState::Ok => OK_INDICATOR,
        AlertState::Warning => WARN_INDICATOR,
        AlertState::Critical => CRIT_INDICATOR,
        AlertState::Unknown => UNKNOWN_INDICATOR,
    }
}

/// Returns the appropriate style for a given alert state.
fn state_color(state: &AlertState, theme: &Theme) -> Style {
    match state {
        AlertState::Ok => theme.success_style(),
        AlertState::Warning => theme.warning_style(),
        AlertState::Critical => theme.error_style(),
        AlertState::Unknown => Style::default().fg(Color::DarkGray),
    }
}

/// Converts f64 history values to u64 sparkline data.
fn history_to_sparkline_data(history: &[f64]) -> Vec<u64> {
    if history.is_empty() {
        return Vec::new();
    }

    let min = history.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let max = history.iter().copied().reduce(f64::max).unwrap_or(0.0);
    let range = max - min;

    if range == 0.0 {
        return history.iter().map(|_| 50).collect();
    }

    history
        .iter()
        .map(|v| ((v - min) / range * 100.0) as u64)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_indicator() {
        assert_eq!(state_indicator(&AlertState::Ok), OK_INDICATOR);
        assert_eq!(state_indicator(&AlertState::Warning), WARN_INDICATOR);
        assert_eq!(state_indicator(&AlertState::Critical), CRIT_INDICATOR);
        assert_eq!(state_indicator(&AlertState::Unknown), UNKNOWN_INDICATOR);
    }

    #[test]
    fn test_history_to_sparkline_data_empty() {
        assert!(history_to_sparkline_data(&[]).is_empty());
    }

    #[test]
    fn test_history_to_sparkline_data_constant() {
        let data = history_to_sparkline_data(&[5.0, 5.0, 5.0]);
        assert_eq!(data, vec![50, 50, 50]);
    }

    #[test]
    fn test_history_to_sparkline_data_varying() {
        let data = history_to_sparkline_data(&[0.0, 50.0, 100.0]);
        assert_eq!(data, vec![0, 50, 100]);
    }
}
