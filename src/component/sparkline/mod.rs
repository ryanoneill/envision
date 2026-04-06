//! A compact inline data trend display component.
//!
//! [`Sparkline`] provides a visual data trend indicator that renders a series
//! of values as small bars. This is a **display-only** component that does not
//! receive keyboard focus. State is stored in [`SparklineState`] and updated
//! via [`SparklineMessage`].
//!
//! See also [`ProgressBar`](super::ProgressBar) for determinate progress,
//! and [`Chart`](super::Chart) for full-featured charting.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Sparkline, SparklineMessage, SparklineState, Component};
//!
//! // Create a sparkline with data
//! let mut state = SparklineState::with_data(vec![1.0, 3.0, 7.0, 2.0, 5.0, 8.0, 4.0]);
//! assert_eq!(state.len(), 7);
//!
//! // Push a new data point
//! Sparkline::update(&mut state, SparklineMessage::Push(6.0));
//! assert_eq!(state.len(), 8);
//! assert_eq!(state.last(), Some(6.0));
//!
//! // Push with bounded capacity
//! Sparkline::update(&mut state, SparklineMessage::PushBounded(9.0, 5));
//! assert_eq!(state.len(), 5);
//!
//! // Clear all data
//! Sparkline::update(&mut state, SparklineMessage::Clear);
//! assert!(state.is_empty());
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, RenderDirection, Sparkline as RatatuiSparkline};

use super::{Component, ViewContext};
use crate::theme::Theme;

/// The direction in which sparkline data is rendered.
///
/// # Example
///
/// ```rust
/// use envision::component::SparklineDirection;
///
/// let dir = SparklineDirection::default();
/// assert_eq!(dir, SparklineDirection::LeftToRight);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum SparklineDirection {
    /// Data is rendered from left to right (default).
    #[default]
    LeftToRight,
    /// Data is rendered from right to left.
    RightToLeft,
}

impl From<SparklineDirection> for RenderDirection {
    fn from(dir: SparklineDirection) -> Self {
        match dir {
            SparklineDirection::LeftToRight => RenderDirection::LeftToRight,
            SparklineDirection::RightToLeft => RenderDirection::RightToLeft,
        }
    }
}

/// Messages that can be sent to a Sparkline.
#[derive(Clone, Debug, PartialEq)]
pub enum SparklineMessage {
    /// Append a data point.
    Push(f64),
    /// Append a data point with a maximum capacity, evicting the oldest if exceeded.
    PushBounded(f64, usize),
    /// Replace all data.
    SetData(Vec<f64>),
    /// Clear all data.
    Clear,
    /// Set the maximum number of displayed data points.
    SetMaxDisplayPoints(Option<usize>),
}

/// Output messages from a Sparkline.
///
/// Sparkline is display-only and does not produce output.
pub type SparklineOutput = ();

/// State for a Sparkline component.
///
/// # Example
///
/// ```rust
/// use envision::component::SparklineState;
///
/// let state = SparklineState::with_data(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
/// assert_eq!(state.len(), 5);
/// assert_eq!(state.min(), Some(1.0));
/// assert_eq!(state.max(), Some(5.0));
/// assert_eq!(state.last(), Some(5.0));
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct SparklineState {
    /// The data points.
    data: Vec<f64>,
    /// Limit the number of displayed data points (shows last N).
    max_display_points: Option<usize>,
    /// Optional title/label.
    title: Option<String>,
    /// Render direction.
    direction: SparklineDirection,
    /// Optional color override.
    color: Option<Color>,
}

impl SparklineState {
    /// Creates a new empty sparkline.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    ///
    /// let state = SparklineState::new();
    /// assert!(state.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a sparkline with initial data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    ///
    /// let state = SparklineState::with_data(vec![1.0, 2.0, 3.0]);
    /// assert_eq!(state.len(), 3);
    /// assert_eq!(state.data(), &[1.0, 2.0, 3.0]);
    /// ```
    pub fn with_data(data: Vec<f64>) -> Self {
        Self {
            data,
            ..Self::default()
        }
    }

    /// Sets the title using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    ///
    /// let state = SparklineState::new().with_title("CPU Usage");
    /// assert_eq!(state.title(), Some("CPU Usage"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the render direction using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SparklineState, SparklineDirection};
    ///
    /// let state = SparklineState::new()
    ///     .with_direction(SparklineDirection::RightToLeft);
    /// assert_eq!(state.direction(), &SparklineDirection::RightToLeft);
    /// ```
    pub fn with_direction(mut self, direction: SparklineDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the maximum number of displayed data points using builder pattern.
    ///
    /// When the data has more points than this limit, only the last N are displayed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    ///
    /// let state = SparklineState::with_data(vec![1.0, 2.0, 3.0, 4.0, 5.0])
    ///     .with_max_display_points(3);
    /// assert_eq!(state.max_display_points(), Some(3));
    /// ```
    pub fn with_max_display_points(mut self, max: usize) -> Self {
        self.max_display_points = Some(max);
        self
    }

    /// Sets the color override using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    /// use ratatui::style::Color;
    ///
    /// let state = SparklineState::new().with_color(Color::Green);
    /// assert_eq!(state.color(), Some(Color::Green));
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Returns the data points.
    pub fn data(&self) -> &[f64] {
        &self.data
    }

    /// Appends a data point.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    ///
    /// let mut state = SparklineState::new();
    /// state.push(42.0);
    /// assert_eq!(state.data(), &[42.0]);
    /// ```
    pub fn push(&mut self, value: f64) {
        self.data.push(value);
    }

    /// Appends a data point, evicting the oldest if the length exceeds `max_len`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    ///
    /// let mut state = SparklineState::with_data(vec![1.0, 2.0, 3.0]);
    /// state.push_bounded(4.0, 3);
    /// assert_eq!(state.data(), &[2.0, 3.0, 4.0]);
    /// ```
    pub fn push_bounded(&mut self, value: f64, max_len: usize) {
        self.data.push(value);
        if self.data.len() > max_len {
            let excess = self.data.len() - max_len;
            self.data.drain(..excess);
        }
    }

    /// Clears all data points.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    ///
    /// let mut state = SparklineState::with_data(vec![1.0, 2.0, 3.0]);
    /// state.clear();
    /// assert!(state.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Returns the number of data points.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if there are no data points.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the last data point, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    ///
    /// let state = SparklineState::with_data(vec![1.0, 2.0, 3.0]);
    /// assert_eq!(state.last(), Some(3.0));
    ///
    /// let empty = SparklineState::new();
    /// assert_eq!(empty.last(), None);
    /// ```
    pub fn last(&self) -> Option<f64> {
        self.data.last().copied()
    }

    /// Returns the minimum data point, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    ///
    /// let state = SparklineState::with_data(vec![3.0, 1.0, 2.0]);
    /// assert_eq!(state.min(), Some(1.0));
    /// ```
    pub fn min(&self) -> Option<f64> {
        self.data.iter().copied().reduce(f64::min)
    }

    /// Returns the maximum data point, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    ///
    /// let state = SparklineState::with_data(vec![3.0, 1.0, 2.0]);
    /// assert_eq!(state.max(), Some(3.0));
    /// ```
    pub fn max(&self) -> Option<f64> {
        self.data.iter().copied().reduce(f64::max)
    }

    /// Returns the title, if set.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    ///
    /// let mut state = SparklineState::new();
    /// state.set_title("CPU Usage");
    /// assert_eq!(state.title(), Some("CPU Usage"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Returns the render direction.
    pub fn direction(&self) -> &SparklineDirection {
        &self.direction
    }

    /// Returns the maximum display points setting.
    pub fn max_display_points(&self) -> Option<usize> {
        self.max_display_points
    }

    /// Sets the maximum number of display points.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    ///
    /// let mut state = SparklineState::new();
    /// state.set_max_display_points(Some(100));
    /// assert_eq!(state.max_display_points(), Some(100));
    /// ```
    pub fn set_max_display_points(&mut self, max: Option<usize>) {
        self.max_display_points = max;
    }

    /// Returns the color override, if set.
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Sets the color override.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SparklineState;
    /// use ratatui::style::Color;
    ///
    /// let mut state = SparklineState::new();
    /// state.set_color(Some(Color::Green));
    /// assert_eq!(state.color(), Some(Color::Green));
    /// ```
    pub fn set_color(&mut self, color: Option<Color>) {
        self.color = color;
    }

    /// Updates the state with a message, returning any output.
    ///
    /// This is an instance method equivalent to [`Sparkline::update`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SparklineMessage, SparklineState};
    ///
    /// let mut state = SparklineState::new();
    /// state.update(SparklineMessage::Push(42.0));
    /// assert_eq!(state.last(), Some(42.0));
    /// ```
    pub fn update(&mut self, msg: SparklineMessage) -> Option<SparklineOutput> {
        Sparkline::update(self, msg)
    }
}

/// A compact inline data trend display component.
///
/// `Sparkline` renders a series of data points as small bars using ratatui's
/// `Sparkline` widget. This is a display-only component that does not
/// receive keyboard focus.
///
/// # Visual Format
///
/// The sparkline renders data as a compact bar chart:
/// ```text
/// ▁▃▅▇█▅▃▁▂▄▆█▇▅▃
/// ```
///
/// With an optional title:
/// ```text
/// ┌CPU Usage──────┐
/// │▁▃▅▇█▅▃▁▂▄▆█▇▅│
/// └───────────────┘
/// ```
///
/// # Messages
///
/// - `Push(f64)` - Append a data point
/// - `PushBounded(f64, usize)` - Append with max capacity
/// - `SetData(Vec<f64>)` - Replace all data
/// - `Clear` - Clear all data
/// - `SetMaxDisplayPoints(Option<usize>)` - Set display limit
///
/// # Example
///
/// ```rust
/// use envision::component::{Sparkline, SparklineMessage, SparklineState, Component};
///
/// let mut state = SparklineState::with_data(vec![1.0, 3.0, 7.0, 2.0, 5.0, 8.0, 4.0]);
///
/// // Append new data
/// Sparkline::update(&mut state, SparklineMessage::Push(6.0));
/// assert_eq!(state.len(), 8);
///
/// // Replace data
/// Sparkline::update(&mut state, SparklineMessage::SetData(vec![10.0, 20.0, 30.0]));
/// assert_eq!(state.len(), 3);
/// ```
pub struct Sparkline;

impl Component for Sparkline {
    type State = SparklineState;
    type Message = SparklineMessage;
    type Output = SparklineOutput;

    fn init() -> Self::State {
        SparklineState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            SparklineMessage::Push(value) => {
                state.push(value);
            }
            SparklineMessage::PushBounded(value, max_len) => {
                state.push_bounded(value, max_len);
            }
            SparklineMessage::SetData(data) => {
                state.data = data;
            }
            SparklineMessage::Clear => {
                state.clear();
            }
            SparklineMessage::SetMaxDisplayPoints(max) => {
                state.max_display_points = max;
            }
        }
        None
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        let display_data = match state.max_display_points {
            Some(n) if state.data.len() > n => &state.data[state.data.len() - n..],
            _ => &state.data,
        };

        // Normalize f64 data to u64 for ratatui's Sparkline widget.
        let normalized: Vec<u64> = if display_data.is_empty() {
            Vec::new()
        } else {
            let min_val = display_data.iter().copied().reduce(f64::min).unwrap_or(0.0);
            let max_val = display_data.iter().copied().reduce(f64::max).unwrap_or(1.0);
            let range = (max_val - min_val).max(f64::EPSILON);
            display_data
                .iter()
                .map(|&v| ((v - min_val) / range * 100.0) as u64)
                .collect()
        };

        let style = if ctx.disabled {
            theme.disabled_style()
        } else if let Some(color) = state.color {
            Style::default().fg(color)
        } else {
            theme.normal_style()
        };

        let direction: RenderDirection = state.direction.clone().into();

        let mut sparkline = RatatuiSparkline::default()
            .data(&normalized)
            .direction(direction)
            .style(style);

        if let Some(title) = &state.title {
            sparkline =
                sparkline.block(Block::default().title(title.as_str()).borders(Borders::ALL));
        }

        let annotation =
            crate::annotation::Annotation::new(crate::annotation::WidgetType::Sparkline)
                .with_id("sparkline")
                .with_label(state.title.as_deref().unwrap_or(""));
        let annotated = crate::annotation::Annotate::new(sparkline, annotation);
        frame.render_widget(annotated, area);
    }
}

#[cfg(test)]
mod tests;
