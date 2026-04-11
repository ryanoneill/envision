//! Chart annotations: data types and rendering.
//!
//! Provides [`ChartAnnotation`], a text label placed at a specific (x, y) data
//! coordinate on the chart surface, and the rendering logic that converts data
//! coordinates to screen positions.

use ratatui::prelude::*;

use super::ChartState;
use super::render::compute_graph_area;

/// A text annotation at a specific data coordinate on the chart.
///
/// Annotations are rendered as text labels near specified (x, y) data points,
/// useful for calling out notable values, events, or outliers directly on the
/// chart surface.
///
/// # Example
///
/// ```rust
/// use envision::component::ChartAnnotation;
/// use ratatui::style::Color;
///
/// let ann = ChartAnnotation::new(3.0, 95.0, "Peak", Color::Yellow);
/// assert_eq!(ann.x, 3.0);
/// assert_eq!(ann.y, 95.0);
/// assert_eq!(ann.label, "Peak");
/// assert_eq!(ann.color, Color::Yellow);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ChartAnnotation {
    /// The x-coordinate in data space.
    pub x: f64,
    /// The y-coordinate in data space.
    pub y: f64,
    /// The text label to display.
    pub label: String,
    /// The color for the annotation text.
    pub color: Color,
}

impl ChartAnnotation {
    /// Creates a new chart annotation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartAnnotation;
    /// use ratatui::style::Color;
    ///
    /// let ann = ChartAnnotation::new(5.0, 42.0, "Answer", Color::Cyan);
    /// assert_eq!(ann.x, 5.0);
    /// assert_eq!(ann.y, 42.0);
    /// assert_eq!(ann.label, "Answer");
    /// ```
    pub fn new(x: f64, y: f64, label: impl Into<String>, color: Color) -> Self {
        Self {
            x,
            y,
            label: label.into(),
            color,
        }
    }
}

/// Axis bounds and scale information used for coordinate mapping.
pub(super) struct AxisBounds {
    pub(super) x_min: f64,
    pub(super) x_max: f64,
    pub(super) y_min: f64,
    pub(super) y_max: f64,
    pub(super) is_log: bool,
}

/// Renders text annotations at data coordinates on the chart surface.
///
/// Each annotation's (x, y) data coordinates are converted to screen positions
/// using the same axis bounds as the chart. The label text is written starting
/// one column to the right and one row above the data point to avoid overlapping
/// the plotted data. Only empty or space cells are overwritten, preserving
/// existing chart content.
pub(super) fn render_annotations(
    state: &ChartState,
    frame: &mut Frame,
    area: Rect,
    y_labels: &[String],
    x_labels: &[String],
    bounds: AxisBounds,
) {
    if state.annotations.is_empty() {
        return;
    }

    let graph_area = compute_graph_area(area, y_labels, x_labels);
    if graph_area.width == 0 || graph_area.height == 0 {
        return;
    }

    let x_range = bounds.x_max - bounds.x_min;
    let y_range = bounds.y_max - bounds.y_min;
    if x_range <= 0.0 || y_range <= 0.0 {
        return;
    }

    let buf = frame.buffer_mut();

    for ann in &state.annotations {
        let ann_y = if bounds.is_log {
            state.y_scale.transform(ann.y.max(f64::MIN_POSITIVE))
        } else {
            ann.y
        };

        let x_frac = (ann.x - bounds.x_min) / x_range;
        let y_frac = (ann_y - bounds.y_min) / y_range;

        // Skip annotations outside the visible graph area
        if !(0.0..=1.0).contains(&x_frac) || !(0.0..=1.0).contains(&y_frac) {
            continue;
        }

        let screen_x = graph_area.x + (x_frac * (graph_area.width as f64 - 1.0)).round() as u16;
        let screen_y = graph_area
            .bottom()
            .saturating_sub(1)
            .saturating_sub((y_frac * (graph_area.height as f64 - 1.0)).round() as u16);

        // Offset: 1 col right, 1 row up to avoid overlapping the data point
        let label_x = screen_x.saturating_add(1);
        let label_y = screen_y.saturating_sub(1);

        // Ensure the label row is within the graph area
        if label_y < graph_area.y || label_y >= graph_area.bottom() {
            continue;
        }

        for (i, ch) in ann.label.chars().enumerate() {
            let cx = label_x + i as u16;
            if cx >= graph_area.right() {
                break;
            }
            if let Some(cell) = buf.cell_mut(Position::new(cx, label_y)) {
                if cell.symbol() == " " || cell.symbol().trim().is_empty() {
                    cell.set_char(ch);
                    cell.set_fg(ann.color);
                }
            }
        }
    }
}
