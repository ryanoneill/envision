//! Types for the Timeline component.
//!
//! Contains [`TimelineEvent`], [`TimelineSpan`], [`SelectedType`],
//! [`TimelineMessage`], and [`TimelineOutput`].

use ratatui::prelude::*;

/// A point event on the timeline.
///
/// Represents a single moment in time, rendered as a marker on the timeline.
///
/// # Example
///
/// ```rust
/// use envision::component::TimelineEvent;
/// use ratatui::style::Color;
///
/// let event = TimelineEvent::new("e1", 100.0, "Deploy")
///     .with_color(Color::Green);
/// assert_eq!(event.id, "e1");
/// assert_eq!(event.timestamp, 100.0);
/// assert_eq!(event.label, "Deploy");
/// assert_eq!(event.color, Color::Green);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct TimelineEvent {
    /// Unique identifier.
    pub id: String,
    /// Timestamp in milliseconds from epoch (or any consistent unit).
    pub timestamp: f64,
    /// Display label.
    pub label: String,
    /// Color for this event marker.
    pub color: Color,
}

impl TimelineEvent {
    /// Creates a new timeline event with default color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineEvent;
    /// use ratatui::style::Color;
    ///
    /// let event = TimelineEvent::new("deploy-1", 500.0, "Deployed v2.0");
    /// assert_eq!(event.id, "deploy-1");
    /// assert_eq!(event.timestamp, 500.0);
    /// assert_eq!(event.label, "Deployed v2.0");
    /// assert_eq!(event.color, Color::Yellow);
    /// ```
    pub fn new(id: impl Into<String>, timestamp: f64, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            timestamp,
            label: label.into(),
            color: Color::Yellow,
        }
    }

    /// Sets the color (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineEvent;
    /// use ratatui::style::Color;
    ///
    /// let event = TimelineEvent::new("e1", 0.0, "Start")
    ///     .with_color(Color::Red);
    /// assert_eq!(event.color, Color::Red);
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineEvent;
    /// use ratatui::style::Color;
    ///
    /// let mut event = TimelineEvent::new("e1", 0.0, "Start");
    /// event.set_color(Color::Red);
    /// assert_eq!(event.color, Color::Red);
    /// ```
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

/// A span (duration) on the timeline.
///
/// Represents a range of time, rendered as a horizontal bar.
///
/// # Example
///
/// ```rust
/// use envision::component::TimelineSpan;
/// use ratatui::style::Color;
///
/// let span = TimelineSpan::new("s1", 100.0, 500.0, "HTTP Request")
///     .with_color(Color::Cyan)
///     .with_lane(1);
/// assert_eq!(span.id, "s1");
/// assert_eq!(span.start, 100.0);
/// assert_eq!(span.end, 500.0);
/// assert_eq!(span.duration(), 400.0);
/// assert_eq!(span.lane, 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct TimelineSpan {
    /// Unique identifier.
    pub id: String,
    /// Start timestamp.
    pub start: f64,
    /// End timestamp.
    pub end: f64,
    /// Display label.
    pub label: String,
    /// Color for this span bar.
    pub color: Color,
    /// Optional row/lane index for vertical positioning.
    pub lane: usize,
}

impl TimelineSpan {
    /// Creates a new timeline span with default color and lane 0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineSpan;
    /// use ratatui::style::Color;
    ///
    /// let span = TimelineSpan::new("s1", 200.0, 800.0, "request-1");
    /// assert_eq!(span.id, "s1");
    /// assert_eq!(span.start, 200.0);
    /// assert_eq!(span.end, 800.0);
    /// assert_eq!(span.label, "request-1");
    /// assert_eq!(span.color, Color::Cyan);
    /// assert_eq!(span.lane, 0);
    /// ```
    pub fn new(id: impl Into<String>, start: f64, end: f64, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            start,
            end,
            label: label.into(),
            color: Color::Cyan,
            lane: 0,
        }
    }

    /// Sets the color (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineSpan;
    /// use ratatui::style::Color;
    ///
    /// let span = TimelineSpan::new("s1", 0.0, 100.0, "task")
    ///     .with_color(Color::Red);
    /// assert_eq!(span.color, Color::Red);
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineSpan;
    /// use ratatui::style::Color;
    ///
    /// let mut span = TimelineSpan::new("s1", 0.0, 100.0, "task");
    /// span.set_color(Color::Red);
    /// assert_eq!(span.color, Color::Red);
    /// ```
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Sets the lane (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineSpan;
    ///
    /// let span = TimelineSpan::new("s1", 0.0, 100.0, "task")
    ///     .with_lane(2);
    /// assert_eq!(span.lane, 2);
    /// ```
    pub fn with_lane(mut self, lane: usize) -> Self {
        self.lane = lane;
        self
    }

    /// Returns the duration of this span.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineSpan;
    ///
    /// let span = TimelineSpan::new("s1", 100.0, 400.0, "task");
    /// assert_eq!(span.duration(), 300.0);
    /// ```
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }
}

/// Distinguishes whether the selected item is an event or a span.
///
/// # Example
///
/// ```rust
/// use envision::component::SelectedType;
///
/// let default = SelectedType::default();
/// assert_eq!(default, SelectedType::Event);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum SelectedType {
    /// A point event is selected.
    #[default]
    Event,
    /// A span is selected.
    Span,
}

/// Messages that can be sent to a Timeline.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, Timeline, TimelineState, TimelineMessage, TimelineEvent,
/// };
///
/// let mut state = TimelineState::new();
/// let event = TimelineEvent::new("e1", 100.0, "Start");
/// state.update(TimelineMessage::AddEvent(event));
/// assert_eq!(state.events().len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum TimelineMessage {
    /// Add a point event.
    AddEvent(TimelineEvent),
    /// Add a duration span.
    AddSpan(TimelineSpan),
    /// Replace all events.
    SetEvents(Vec<TimelineEvent>),
    /// Replace all spans.
    SetSpans(Vec<TimelineSpan>),
    /// Clear everything.
    Clear,
    /// Narrow the visible window (zoom in).
    ZoomIn,
    /// Widen the visible window (zoom out).
    ZoomOut,
    /// Shift visible window left.
    PanLeft,
    /// Shift visible window right.
    PanRight,
    /// Adjust view to show all events/spans.
    FitAll,
    /// Select next event/span.
    SelectNext,
    /// Select previous event/span.
    SelectPrev,
}

/// Output messages from a Timeline.
///
/// # Example
///
/// ```rust
/// use envision::component::TimelineOutput;
///
/// let output = TimelineOutput::EventSelected("e1".into());
/// assert_eq!(output, TimelineOutput::EventSelected("e1".into()));
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum TimelineOutput {
    /// An event was selected (carries event id).
    EventSelected(String),
    /// A span was selected (carries span id).
    SpanSelected(String),
    /// The visible time window changed.
    ViewChanged {
        /// New view start.
        start: f64,
        /// New view end.
        end: f64,
    },
}
