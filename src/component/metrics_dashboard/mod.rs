//! A configurable dashboard of metric widgets.
//!
//! `MetricsDashboard` displays a grid of metric widgets, each showing a
//! labeled value with an optional sparkline history. Supports keyboard
//! navigation between widgets and tick-based value updates.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Focusable, MetricsDashboard, MetricsDashboardState,
//!     MetricsDashboardMessage, MetricWidget, MetricKind,
//! };
//!
//! let mut state = MetricsDashboardState::new(vec![
//!     MetricWidget::counter("Requests", 0),
//!     MetricWidget::gauge("CPU %", 0, 100),
//!     MetricWidget::status("API", true),
//! ], 3);
//!
//! assert_eq!(state.widget_count(), 3);
//! assert_eq!(state.columns(), 3);
//! assert_eq!(state.selected_index(), 0);
//! ```

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};

use super::{Component, Focusable};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// The kind of metric a widget displays.
#[derive(Clone, Debug, PartialEq)]
pub enum MetricKind {
    /// A numeric counter value.
    Counter {
        /// The current value.
        value: i64,
    },
    /// A gauge value with a known range.
    Gauge {
        /// The current value.
        value: u64,
        /// The maximum value.
        max: u64,
    },
    /// A status indicator (up/down).
    Status {
        /// Whether the status is "up" (healthy).
        up: bool,
    },
    /// A text-based metric.
    Text {
        /// The display text.
        text: String,
    },
}

/// A single metric widget in the dashboard.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricWidget {
    /// The display label.
    label: String,
    /// The metric kind and value.
    kind: MetricKind,
    /// Sparkline history (recent values for trend display).
    history: Vec<u64>,
    /// Maximum history length.
    max_history: usize,
}

impl MetricWidget {
    /// Creates a counter widget.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::counter("Requests", 42);
    /// assert_eq!(widget.label(), "Requests");
    /// assert_eq!(widget.display_value(), "42");
    /// ```
    pub fn counter(label: impl Into<String>, value: i64) -> Self {
        Self {
            label: label.into(),
            kind: MetricKind::Counter { value },
            history: Vec::new(),
            max_history: 20,
        }
    }

    /// Creates a gauge widget with a maximum value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::gauge("CPU %", 75, 100);
    /// assert_eq!(widget.display_value(), "75/100");
    /// ```
    pub fn gauge(label: impl Into<String>, value: u64, max: u64) -> Self {
        Self {
            label: label.into(),
            kind: MetricKind::Gauge { value, max },
            history: Vec::new(),
            max_history: 20,
        }
    }

    /// Creates a status indicator widget.
    pub fn status(label: impl Into<String>, up: bool) -> Self {
        Self {
            label: label.into(),
            kind: MetricKind::Status { up },
            history: Vec::new(),
            max_history: 0,
        }
    }

    /// Creates a text metric widget.
    pub fn text(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: MetricKind::Text { text: text.into() },
            history: Vec::new(),
            max_history: 0,
        }
    }

    /// Sets the maximum history length for sparkline display (builder pattern).
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Returns the label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the metric kind.
    pub fn kind(&self) -> &MetricKind {
        &self.kind
    }

    /// Returns the sparkline history.
    pub fn history(&self) -> &[u64] {
        &self.history
    }

    /// Returns the display value as a string.
    pub fn display_value(&self) -> String {
        match &self.kind {
            MetricKind::Counter { value } => value.to_string(),
            MetricKind::Gauge { value, max } => format!("{}/{}", value, max),
            MetricKind::Status { up } => {
                if *up {
                    "UP".to_string()
                } else {
                    "DOWN".to_string()
                }
            }
            MetricKind::Text { text } => text.clone(),
        }
    }

    /// Sets the counter value.
    pub fn set_counter_value(&mut self, value: i64) {
        if let MetricKind::Counter { value: ref mut v } = self.kind {
            *v = value;
            if self.max_history > 0 {
                self.history.push(value.unsigned_abs());
                while self.history.len() > self.max_history {
                    self.history.remove(0);
                }
            }
        }
    }

    /// Sets the gauge value.
    pub fn set_gauge_value(&mut self, value: u64) {
        if let MetricKind::Gauge {
            value: ref mut v,
            max,
        } = self.kind
        {
            *v = value.min(max);
            if self.max_history > 0 {
                self.history.push(value);
                while self.history.len() > self.max_history {
                    self.history.remove(0);
                }
            }
        }
    }

    /// Sets the status.
    pub fn set_status(&mut self, up: bool) {
        if let MetricKind::Status { up: ref mut u } = self.kind {
            *u = up;
        }
    }

    /// Sets the text value.
    pub fn set_text(&mut self, text: impl Into<String>) {
        if let MetricKind::Text { text: ref mut t } = self.kind {
            *t = text.into();
        }
    }

    /// Increments a counter by the given amount.
    pub fn increment(&mut self, amount: i64) {
        if let MetricKind::Counter { ref mut value } = self.kind {
            *value += amount;
            if self.max_history > 0 {
                self.history.push(value.unsigned_abs());
                while self.history.len() > self.max_history {
                    self.history.remove(0);
                }
            }
        }
    }

    /// Returns the gauge fill percentage (0.0 to 1.0).
    pub fn gauge_percentage(&self) -> Option<f64> {
        match &self.kind {
            MetricKind::Gauge { value, max } if *max > 0 => Some(*value as f64 / *max as f64),
            _ => None,
        }
    }
}

/// Messages that can be sent to a MetricsDashboard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetricsDashboardMessage {
    /// Move selection left.
    Left,
    /// Move selection right.
    Right,
    /// Move selection up.
    Up,
    /// Move selection down.
    Down,
    /// Move to the first widget.
    First,
    /// Move to the last widget.
    Last,
    /// Select the current widget (emit output).
    Select,
}

/// Output messages from a MetricsDashboard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetricsDashboardOutput {
    /// The selected widget changed.
    SelectionChanged(usize),
    /// A widget was activated (Enter pressed).
    Selected(usize),
}

/// State for a MetricsDashboard component.
///
/// Contains the grid of widgets and navigation state.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricsDashboardState {
    /// The metric widgets.
    widgets: Vec<MetricWidget>,
    /// Number of columns in the grid layout.
    columns: usize,
    /// Currently selected widget index.
    selected: usize,
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
    /// Optional title.
    title: Option<String>,
}

impl Default for MetricsDashboardState {
    fn default() -> Self {
        Self {
            widgets: Vec::new(),
            columns: 3,
            selected: 0,
            focused: false,
            disabled: false,
            title: None,
        }
    }
}

impl MetricsDashboardState {
    /// Creates a new dashboard with the given widgets and column count.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("Items", 0),
    ///     MetricWidget::gauge("Memory", 512, 1024),
    /// ], 2);
    /// assert_eq!(state.widget_count(), 2);
    /// assert_eq!(state.columns(), 2);
    /// ```
    pub fn new(widgets: Vec<MetricWidget>, columns: usize) -> Self {
        Self {
            widgets,
            columns: columns.max(1),
            selected: 0,
            focused: false,
            disabled: false,
            title: None,
        }
    }

    /// Sets the title (builder pattern).
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the disabled state (builder pattern).
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // ---- Accessors ----

    /// Returns the widgets.
    pub fn widgets(&self) -> &[MetricWidget] {
        &self.widgets
    }

    /// Returns a mutable reference to the widgets.
    pub fn widgets_mut(&mut self) -> &mut [MetricWidget] {
        &mut self.widgets
    }

    /// Returns a reference to the widget at the given index.
    pub fn widget(&self, index: usize) -> Option<&MetricWidget> {
        self.widgets.get(index)
    }

    /// Returns a mutable reference to the widget at the given index.
    pub fn widget_mut(&mut self, index: usize) -> Option<&mut MetricWidget> {
        self.widgets.get_mut(index)
    }

    /// Returns the number of widgets.
    pub fn widget_count(&self) -> usize {
        self.widgets.len()
    }

    /// Returns the number of columns.
    pub fn columns(&self) -> usize {
        self.columns
    }

    /// Sets the number of columns.
    pub fn set_columns(&mut self, columns: usize) {
        self.columns = columns.max(1);
    }

    /// Returns the number of rows (based on widget count and columns).
    pub fn rows(&self) -> usize {
        if self.widgets.is_empty() {
            0
        } else {
            self.widgets.len().div_ceil(self.columns)
        }
    }

    /// Returns the selected widget index.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Returns a reference to the selected widget.
    pub fn selected_widget(&self) -> Option<&MetricWidget> {
        self.widgets.get(self.selected)
    }

    /// Returns the (row, column) position of the selected widget.
    pub fn selected_position(&self) -> (usize, usize) {
        (self.selected / self.columns, self.selected % self.columns)
    }

    /// Returns the title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns true if the dashboard has no widgets.
    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
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

    /// Maps an input event to a dashboard message.
    pub fn handle_event(&self, event: &Event) -> Option<MetricsDashboardMessage> {
        MetricsDashboard::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<MetricsDashboardOutput> {
        MetricsDashboard::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: MetricsDashboardMessage) -> Option<MetricsDashboardOutput> {
        MetricsDashboard::update(self, msg)
    }
}

/// A configurable dashboard of metric widgets.
///
/// Displays widgets in a grid layout with keyboard navigation.
///
/// # Key Bindings
///
/// - `Left` / `h` — Move selection left
/// - `Right` / `l` — Move selection right
/// - `Up` / `k` — Move selection up
/// - `Down` / `j` — Move selection down
/// - `Home` — Select first widget
/// - `End` — Select last widget
/// - `Enter` — Select current widget
pub struct MetricsDashboard(PhantomData<()>);

impl Component for MetricsDashboard {
    type State = MetricsDashboardState;
    type Message = MetricsDashboardMessage;
    type Output = MetricsDashboardOutput;

    fn init() -> Self::State {
        MetricsDashboardState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        let key = event.as_key()?;

        match key.code {
            KeyCode::Left | KeyCode::Char('h') => Some(MetricsDashboardMessage::Left),
            KeyCode::Right | KeyCode::Char('l') => Some(MetricsDashboardMessage::Right),
            KeyCode::Up | KeyCode::Char('k') => Some(MetricsDashboardMessage::Up),
            KeyCode::Down | KeyCode::Char('j') => Some(MetricsDashboardMessage::Down),
            KeyCode::Home => Some(MetricsDashboardMessage::First),
            KeyCode::End => Some(MetricsDashboardMessage::Last),
            KeyCode::Enter => Some(MetricsDashboardMessage::Select),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled || state.widgets.is_empty() {
            return None;
        }

        let len = state.widgets.len();
        let cols = state.columns;
        let current = state.selected;
        let current_row = current / cols;
        let current_col = current % cols;

        match msg {
            MetricsDashboardMessage::Left => {
                if current_col > 0 {
                    state.selected = current - 1;
                    Some(MetricsDashboardOutput::SelectionChanged(state.selected))
                } else {
                    None
                }
            }
            MetricsDashboardMessage::Right => {
                if current_col < cols - 1 && current + 1 < len {
                    state.selected = current + 1;
                    Some(MetricsDashboardOutput::SelectionChanged(state.selected))
                } else {
                    None
                }
            }
            MetricsDashboardMessage::Up => {
                if current_row > 0 {
                    let new_index = (current_row - 1) * cols + current_col;
                    if new_index < len {
                        state.selected = new_index;
                        Some(MetricsDashboardOutput::SelectionChanged(state.selected))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            MetricsDashboardMessage::Down => {
                let new_index = (current_row + 1) * cols + current_col;
                if new_index < len {
                    state.selected = new_index;
                    Some(MetricsDashboardOutput::SelectionChanged(state.selected))
                } else {
                    None
                }
            }
            MetricsDashboardMessage::First => {
                if current != 0 {
                    state.selected = 0;
                    Some(MetricsDashboardOutput::SelectionChanged(0))
                } else {
                    None
                }
            }
            MetricsDashboardMessage::Last => {
                let last = len - 1;
                if current != last {
                    state.selected = last;
                    Some(MetricsDashboardOutput::SelectionChanged(last))
                } else {
                    None
                }
            }
            MetricsDashboardMessage::Select => Some(MetricsDashboardOutput::Selected(current)),
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if state.widgets.is_empty() || area.height < 3 || area.width < 3 {
            return;
        }

        let rows = state.rows();
        let cols = state.columns;

        // Compute row heights
        let row_constraints: Vec<Constraint> = (0..rows)
            .map(|_| Constraint::Ratio(1, rows as u32))
            .collect();

        let row_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(area);

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
                let widget_idx = row_idx * cols + col_idx;
                if let Some(widget) = state.widgets.get(widget_idx) {
                    let is_selected = widget_idx == state.selected;
                    render_widget(widget, is_selected, state, frame, *col_area, theme);
                }
            }
        }
    }
}

impl Focusable for MetricsDashboard {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

/// Renders a single metric widget.
fn render_widget(
    widget: &MetricWidget,
    is_selected: bool,
    state: &MetricsDashboardState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let border_style = if state.disabled {
        theme.disabled_style()
    } else if is_selected && state.focused {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .title(widget.label())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let value_style = if state.disabled {
        theme.disabled_style()
    } else {
        value_color(widget, theme)
    };

    // Show sparkline if there's history and enough space
    if !widget.history.is_empty() && inner.height >= 3 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(inner);

        // Value line
        let value_text = widget.display_value();
        let paragraph = Paragraph::new(value_text).style(value_style);
        frame.render_widget(paragraph, chunks[0]);

        // Sparkline
        let sparkline = Sparkline::default()
            .data(&widget.history)
            .style(value_style);
        frame.render_widget(sparkline, chunks[1]);
    } else {
        // Just value
        let value_text = widget.display_value();
        let paragraph = Paragraph::new(value_text)
            .style(value_style)
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, inner);
    }
}

/// Returns the appropriate style for a widget's value.
fn value_color(widget: &MetricWidget, theme: &Theme) -> Style {
    match &widget.kind {
        MetricKind::Counter { .. } => theme.info_style(),
        MetricKind::Gauge { value, max } => {
            let pct = if *max > 0 {
                *value as f64 / *max as f64
            } else {
                0.0
            };
            if pct >= 0.9 {
                theme.error_style()
            } else if pct >= 0.7 {
                theme.warning_style()
            } else {
                theme.success_style()
            }
        }
        MetricKind::Status { up } => {
            if *up {
                theme.success_style()
            } else {
                theme.error_style()
            }
        }
        MetricKind::Text { .. } => theme.normal_style(),
    }
}

#[cfg(test)]
mod tests;
