//! A horizontal timeline component for visualizing events and spans.
//!
//! [`Timeline`] renders point events and duration spans along a horizontal
//! time axis with zoom, pan, and selection. Useful for trace visualizations
//! (Jaeger/Zipkin), deployment timelines, and incident timelines.
//!
//! State is stored in [`TimelineState`], updated via [`TimelineMessage`],
//! and produces [`TimelineOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Timeline, TimelineState, TimelineEvent, TimelineSpan,
//! };
//! use ratatui::style::Color;
//!
//! let mut state = TimelineState::new()
//!     .with_events(vec![
//!         TimelineEvent::new("e1", 100.0, "Start"),
//!         TimelineEvent::new("e2", 500.0, "Deploy"),
//!     ])
//!     .with_spans(vec![
//!         TimelineSpan::new("s1", 200.0, 800.0, "request-1"),
//!     ]);
//! assert_eq!(state.events().len(), 2);
//! assert_eq!(state.spans().len(), 1);
//! ```

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use super::{Component, Disableable, Focusable};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

pub(crate) mod render;
mod types;

pub use types::{SelectedType, TimelineEvent, TimelineMessage, TimelineOutput, TimelineSpan};

/// State for a Timeline component.
///
/// Contains point events, duration spans, view window, and selection state.
///
/// # Example
///
/// ```rust
/// use envision::component::{TimelineState, TimelineEvent, TimelineSpan};
///
/// let state = TimelineState::new()
///     .with_events(vec![TimelineEvent::new("e1", 100.0, "Start")])
///     .with_spans(vec![TimelineSpan::new("s1", 0.0, 200.0, "Init")])
///     .with_title("Deployment Timeline");
/// assert_eq!(state.events().len(), 1);
/// assert_eq!(state.spans().len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct TimelineState {
    /// Point events.
    pub(crate) events: Vec<TimelineEvent>,
    /// Duration spans.
    pub(crate) spans: Vec<TimelineSpan>,
    /// Visible window start time.
    pub(crate) view_start: f64,
    /// Visible window end time.
    pub(crate) view_end: f64,
    /// Selected event or span index.
    pub(crate) selected_index: Option<usize>,
    /// Whether the selection refers to an event or a span.
    pub(crate) selected_type: SelectedType,
    /// Optional title.
    pub(crate) title: Option<String>,
    /// Show labels on events/spans.
    pub(crate) show_labels: bool,
    /// Number of horizontal lanes for spans (0 = auto from data).
    pub(crate) lane_count: usize,
    /// Whether the component is focused.
    pub(crate) focused: bool,
    /// Whether the component is disabled.
    pub(crate) disabled: bool,
}

impl Default for TimelineState {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            spans: Vec::new(),
            view_start: 0.0,
            view_end: 1000.0,
            selected_index: None,
            selected_type: SelectedType::default(),
            title: None,
            show_labels: true,
            lane_count: 0,
            focused: false,
            disabled: false,
        }
    }
}

impl TimelineState {
    /// Creates an empty timeline.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineState;
    ///
    /// let state = TimelineState::new();
    /// assert!(state.events().is_empty());
    /// assert!(state.spans().is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets initial events (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TimelineState, TimelineEvent};
    ///
    /// let state = TimelineState::new()
    ///     .with_events(vec![TimelineEvent::new("e1", 100.0, "Start")]);
    /// assert_eq!(state.events().len(), 1);
    /// ```
    pub fn with_events(mut self, events: Vec<TimelineEvent>) -> Self {
        self.events = events;
        self
    }

    /// Sets initial spans (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TimelineState, TimelineSpan};
    ///
    /// let state = TimelineState::new()
    ///     .with_spans(vec![TimelineSpan::new("s1", 0.0, 200.0, "Init")]);
    /// assert_eq!(state.spans().len(), 1);
    /// ```
    pub fn with_spans(mut self, spans: Vec<TimelineSpan>) -> Self {
        self.spans = spans;
        self
    }

    /// Sets the initial view range (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineState;
    ///
    /// let state = TimelineState::new()
    ///     .with_view_range(0.0, 500.0);
    /// assert_eq!(state.view_range(), (0.0, 500.0));
    /// ```
    pub fn with_view_range(mut self, start: f64, end: f64) -> Self {
        self.view_start = start;
        self.view_end = end;
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineState;
    ///
    /// let state = TimelineState::new()
    ///     .with_title("Trace Timeline");
    /// assert_eq!(state.title(), Some("Trace Timeline"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether to show labels (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineState;
    ///
    /// let state = TimelineState::new()
    ///     .with_show_labels(false);
    /// assert!(!state.show_labels());
    /// ```
    pub fn with_show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineState;
    ///
    /// let state = TimelineState::new()
    ///     .with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // ---- Accessors ----

    /// Returns the point events.
    pub fn events(&self) -> &[TimelineEvent] {
        &self.events
    }

    /// Returns the duration spans.
    pub fn spans(&self) -> &[TimelineSpan] {
        &self.spans
    }

    /// Adds a point event.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TimelineState, TimelineEvent};
    ///
    /// let mut state = TimelineState::new();
    /// state.add_event(TimelineEvent::new("e1", 100.0, "Start"));
    /// assert_eq!(state.events().len(), 1);
    /// ```
    pub fn add_event(&mut self, event: TimelineEvent) {
        self.events.push(event);
    }

    /// Adds a duration span.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TimelineState, TimelineSpan};
    ///
    /// let mut state = TimelineState::new();
    /// state.add_span(TimelineSpan::new("s1", 0.0, 200.0, "Init"));
    /// assert_eq!(state.spans().len(), 1);
    /// ```
    pub fn add_span(&mut self, span: TimelineSpan) {
        self.spans.push(span);
    }

    /// Clears all events and spans.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TimelineState, TimelineEvent, TimelineSpan};
    ///
    /// let mut state = TimelineState::new()
    ///     .with_events(vec![TimelineEvent::new("e1", 0.0, "x")])
    ///     .with_spans(vec![TimelineSpan::new("s1", 0.0, 1.0, "y")]);
    /// state.clear();
    /// assert!(state.events().is_empty());
    /// assert!(state.spans().is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.events.clear();
        self.spans.clear();
        self.selected_index = None;
    }

    /// Returns the current visible time range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineState;
    ///
    /// let state = TimelineState::new().with_view_range(100.0, 900.0);
    /// assert_eq!(state.view_range(), (100.0, 900.0));
    /// ```
    pub fn view_range(&self) -> (f64, f64) {
        (self.view_start, self.view_end)
    }

    /// Sets the visible time range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineState;
    ///
    /// let mut state = TimelineState::new();
    /// state.set_view_range(50.0, 750.0);
    /// assert_eq!(state.view_range(), (50.0, 750.0));
    /// ```
    pub fn set_view_range(&mut self, start: f64, end: f64) {
        self.view_start = start;
        self.view_end = end;
    }

    /// Auto-fits the view to encompass all events and spans.
    ///
    /// Adds a 5% padding on each side. If there is no data, resets to 0..1000.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TimelineState, TimelineEvent, TimelineSpan};
    ///
    /// let mut state = TimelineState::new()
    ///     .with_events(vec![TimelineEvent::new("e1", 100.0, "A")])
    ///     .with_spans(vec![TimelineSpan::new("s1", 200.0, 800.0, "B")]);
    /// state.fit_all();
    /// let (start, end) = state.view_range();
    /// assert!(start < 100.0);
    /// assert!(end > 800.0);
    /// ```
    pub fn fit_all(&mut self) {
        let (data_min, data_max) = self.data_bounds();
        if data_min >= data_max {
            self.view_start = 0.0;
            self.view_end = 1000.0;
            return;
        }
        let range = data_max - data_min;
        let padding = range * 0.05;
        self.view_start = data_min - padding;
        self.view_end = data_max + padding;
    }

    /// Narrows the visible window by 25% (zoom in).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineState;
    ///
    /// let mut state = TimelineState::new()
    ///     .with_view_range(0.0, 1000.0);
    /// state.zoom_in();
    /// let (start, end) = state.view_range();
    /// assert!(start > 0.0);
    /// assert!(end < 1000.0);
    /// ```
    pub fn zoom_in(&mut self) {
        let range = self.view_end - self.view_start;
        let center = self.view_start + range / 2.0;
        let new_range = range * 0.75;
        // Prevent zooming in too far
        if new_range < 1.0 {
            return;
        }
        self.view_start = center - new_range / 2.0;
        self.view_end = center + new_range / 2.0;
    }

    /// Widens the visible window by 25% (zoom out).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineState;
    ///
    /// let mut state = TimelineState::new()
    ///     .with_view_range(250.0, 750.0);
    /// state.zoom_out();
    /// let (start, end) = state.view_range();
    /// assert!(start < 250.0);
    /// assert!(end > 750.0);
    /// ```
    pub fn zoom_out(&mut self) {
        let range = self.view_end - self.view_start;
        let center = self.view_start + range / 2.0;
        let new_range = range / 0.75;
        self.view_start = center - new_range / 2.0;
        self.view_end = center + new_range / 2.0;
    }

    /// Returns the selected event, if one is selected.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TimelineState, TimelineEvent, TimelineMessage};
    ///
    /// let mut state = TimelineState::new()
    ///     .with_events(vec![TimelineEvent::new("e1", 100.0, "Start")]);
    /// state.set_focused(true);
    /// state.update(TimelineMessage::SelectNext);
    /// assert_eq!(state.selected_event().unwrap().id, "e1");
    /// ```
    pub fn selected_event(&self) -> Option<&TimelineEvent> {
        if self.selected_type == SelectedType::Event {
            self.selected_index.and_then(|idx| self.events.get(idx))
        } else {
            None
        }
    }

    /// Returns the selected span, if one is selected.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TimelineState, TimelineSpan, TimelineMessage};
    ///
    /// let mut state = TimelineState::new()
    ///     .with_spans(vec![TimelineSpan::new("s1", 0.0, 200.0, "Init")]);
    /// state.set_focused(true);
    /// state.update(TimelineMessage::SelectNext);
    /// assert_eq!(state.selected_span().unwrap().id, "s1");
    /// ```
    pub fn selected_span(&self) -> Option<&TimelineSpan> {
        if self.selected_type == SelectedType::Span {
            self.selected_index.and_then(|idx| self.spans.get(idx))
        } else {
            None
        }
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
    /// use envision::component::TimelineState;
    ///
    /// let mut state = TimelineState::new();
    /// state.set_title("Request Timeline");
    /// assert_eq!(state.title(), Some("Request Timeline"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Returns whether labels are shown.
    pub fn show_labels(&self) -> bool {
        self.show_labels
    }

    /// Sets whether labels are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TimelineState;
    ///
    /// let mut state = TimelineState::new();
    /// state.set_show_labels(false);
    /// assert!(!state.show_labels());
    /// ```
    pub fn set_show_labels(&mut self, show: bool) {
        self.show_labels = show;
    }

    /// Returns the effective lane count (auto-computed if not set).
    pub fn effective_lane_count(&self) -> usize {
        if self.lane_count > 0 {
            self.lane_count
        } else {
            self.spans
                .iter()
                .map(|s| s.lane)
                .max()
                .map(|m| m + 1)
                .unwrap_or(0)
        }
    }

    /// Computes the minimum and maximum timestamps across all data.
    fn data_bounds(&self) -> (f64, f64) {
        let event_min = self.events.iter().map(|e| e.timestamp).reduce(f64::min);
        let event_max = self.events.iter().map(|e| e.timestamp).reduce(f64::max);
        let span_min = self.spans.iter().map(|s| s.start).reduce(f64::min);
        let span_max = self.spans.iter().map(|s| s.end).reduce(f64::max);

        let min = [event_min, span_min]
            .into_iter()
            .flatten()
            .reduce(f64::min)
            .unwrap_or(0.0);
        let max = [event_max, span_max]
            .into_iter()
            .flatten()
            .reduce(f64::max)
            .unwrap_or(0.0);

        (min, max)
    }

    /// Selects the next item in the combined events+spans list.
    fn select_next(&mut self) -> Option<TimelineOutput> {
        let total = self.events.len() + self.spans.len();
        if total == 0 {
            return None;
        }

        let current_flat = self.flat_index();
        let next_flat = match current_flat {
            Some(idx) => (idx + 1) % total,
            None => 0,
        };

        self.set_flat_index(next_flat);
        self.selection_output()
    }

    /// Selects the previous item in the combined events+spans list.
    fn select_prev(&mut self) -> Option<TimelineOutput> {
        let total = self.events.len() + self.spans.len();
        if total == 0 {
            return None;
        }

        let current_flat = self.flat_index();
        let prev_flat = match current_flat {
            Some(idx) => {
                if idx == 0 {
                    total - 1
                } else {
                    idx - 1
                }
            }
            None => total - 1,
        };

        self.set_flat_index(prev_flat);
        self.selection_output()
    }

    /// Returns the flat index (events first, then spans) of the current selection.
    fn flat_index(&self) -> Option<usize> {
        self.selected_index.map(|idx| match self.selected_type {
            SelectedType::Event => idx,
            SelectedType::Span => self.events.len() + idx,
        })
    }

    /// Sets the selection from a flat index (events first, then spans).
    fn set_flat_index(&mut self, flat: usize) {
        if flat < self.events.len() {
            self.selected_type = SelectedType::Event;
            self.selected_index = Some(flat);
        } else {
            self.selected_type = SelectedType::Span;
            self.selected_index = Some(flat - self.events.len());
        }
    }

    /// Returns the output for the current selection.
    fn selection_output(&self) -> Option<TimelineOutput> {
        match self.selected_type {
            SelectedType::Event => self
                .selected_event()
                .map(|e| TimelineOutput::EventSelected(e.id.clone())),
            SelectedType::Span => self
                .selected_span()
                .map(|s| TimelineOutput::SpanSelected(s.id.clone())),
        }
    }

    // ---- Instance methods ----

    /// Returns true if the component is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Maps an input event to a timeline message.
    pub fn handle_event(&self, event: &Event) -> Option<TimelineMessage> {
        Timeline::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<TimelineOutput> {
        Timeline::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: TimelineMessage) -> Option<TimelineOutput> {
        Timeline::update(self, msg)
    }
}

/// A horizontal timeline component for visualizing events and spans.
///
/// Displays point events and duration spans along a time axis with zoom,
/// pan, and selection capabilities.
///
/// # Key Bindings
///
/// - `Left` / `h` -- Pan left
/// - `Right` / `l` -- Pan right
/// - `+` / `=` -- Zoom in
/// - `-` -- Zoom out
/// - `Up` / `k` -- Select previous
/// - `Down` / `j` -- Select next
/// - `Home` -- Fit all
/// - `Enter` -- Confirm selection (emit output)
pub struct Timeline(PhantomData<()>);

impl Component for Timeline {
    type State = TimelineState;
    type Message = TimelineMessage;
    type Output = TimelineOutput;

    fn init() -> Self::State {
        TimelineState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        let key = event.as_key()?;

        match key.code {
            KeyCode::Left | KeyCode::Char('h') => Some(TimelineMessage::PanLeft),
            KeyCode::Right | KeyCode::Char('l') => Some(TimelineMessage::PanRight),
            KeyCode::Char('+') | KeyCode::Char('=') => Some(TimelineMessage::ZoomIn),
            KeyCode::Char('-') => Some(TimelineMessage::ZoomOut),
            KeyCode::Up | KeyCode::Char('k') => Some(TimelineMessage::SelectPrev),
            KeyCode::Down | KeyCode::Char('j') => Some(TimelineMessage::SelectNext),
            KeyCode::Home => Some(TimelineMessage::FitAll),
            KeyCode::Enter => {
                // On Enter, re-emit the current selection if any
                if state.selected_index.is_some() {
                    // We handle this in update by returning the selection output
                    // without changing anything
                    match state.selected_type {
                        SelectedType::Event => Some(TimelineMessage::SelectNext),
                        SelectedType::Span => Some(TimelineMessage::SelectNext),
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            TimelineMessage::AddEvent(event) => {
                state.events.push(event);
                None
            }
            TimelineMessage::AddSpan(span) => {
                state.spans.push(span);
                None
            }
            TimelineMessage::SetEvents(events) => {
                state.events = events;
                state.selected_index = None;
                None
            }
            TimelineMessage::SetSpans(spans) => {
                state.spans = spans;
                state.selected_index = None;
                None
            }
            TimelineMessage::Clear => {
                state.clear();
                None
            }
            TimelineMessage::ZoomIn => {
                state.zoom_in();
                Some(TimelineOutput::ViewChanged {
                    start: state.view_start,
                    end: state.view_end,
                })
            }
            TimelineMessage::ZoomOut => {
                state.zoom_out();
                Some(TimelineOutput::ViewChanged {
                    start: state.view_start,
                    end: state.view_end,
                })
            }
            TimelineMessage::PanLeft => {
                let range = state.view_end - state.view_start;
                let shift = range * 0.1;
                state.view_start -= shift;
                state.view_end -= shift;
                Some(TimelineOutput::ViewChanged {
                    start: state.view_start,
                    end: state.view_end,
                })
            }
            TimelineMessage::PanRight => {
                let range = state.view_end - state.view_start;
                let shift = range * 0.1;
                state.view_start += shift;
                state.view_end += shift;
                Some(TimelineOutput::ViewChanged {
                    start: state.view_start,
                    end: state.view_end,
                })
            }
            TimelineMessage::FitAll => {
                state.fit_all();
                Some(TimelineOutput::ViewChanged {
                    start: state.view_start,
                    end: state.view_end,
                })
            }
            TimelineMessage::SelectNext => state.select_next(),
            TimelineMessage::SelectPrev => state.select_prev(),
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if area.height < 3 || area.width < 3 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::container("timeline")
                    .with_focus(state.focused)
                    .with_disabled(state.disabled),
            );
        });

        let border_style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        if let Some(ref title) = state.title {
            block = block.title(title.as_str());
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        render::render_timeline(state, frame, inner, theme);
    }
}

impl Focusable for Timeline {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for Timeline {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
