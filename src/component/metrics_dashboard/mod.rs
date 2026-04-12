//! A configurable dashboard of metric widgets.
//!
//! [`MetricsDashboard`] displays a grid of metric widgets, each showing a
//! labeled value with an optional sparkline history. Supports keyboard
//! navigation between widgets and tick-based value updates. State is stored in
//! [`MetricsDashboardState`], updated via [`MetricsDashboardMessage`], and
//! produces [`MetricsDashboardOutput`].
//!
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, MetricsDashboard, MetricsDashboardState,
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
//! assert_eq!(state.selected_index(), Some(0));
//! ```

pub mod widget;

pub use widget::{MetricKind, MetricWidget};

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};
use crate::theme::Theme;

/// Messages that can be sent to a MetricsDashboard.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
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
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
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
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct MetricsDashboardState {
    /// The metric widgets.
    widgets: Vec<MetricWidget>,
    /// Number of columns in the grid layout.
    columns: usize,
    /// Currently selected widget index.
    selected: Option<usize>,
    /// Optional title.
    title: Option<String>,
}

impl Default for MetricsDashboardState {
    fn default() -> Self {
        Self {
            widgets: Vec::new(),
            columns: 3,
            selected: None,
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
        let selected = if widgets.is_empty() { None } else { Some(0) };
        Self {
            widgets,
            columns: columns.max(1),
            selected,
            title: None,
        }
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("Ops", 0),
    /// ], 1).with_title("System Metrics");
    /// assert_eq!(state.title(), Some("System Metrics"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    // ---- Accessors ----

    /// Returns the widgets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("A", 0),
    ///     MetricWidget::counter("B", 0),
    /// ], 2);
    /// assert_eq!(state.widgets().len(), 2);
    /// ```
    pub fn widgets(&self) -> &[MetricWidget] {
        &self.widgets
    }

    /// Returns a mutable reference to the widgets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let mut state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("Ops", 0),
    /// ], 1);
    /// state.widgets_mut()[0].set_counter_value(42);
    /// assert_eq!(state.widgets()[0].display_value(), "42");
    /// ```
    pub fn widgets_mut(&mut self) -> &mut [MetricWidget] {
        &mut self.widgets
    }

    /// Returns a reference to the widget at the given index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("Ops", 42),
    /// ], 1);
    /// assert_eq!(state.widget(0).unwrap().display_value(), "42");
    /// assert!(state.widget(1).is_none());
    /// ```
    pub fn widget(&self, index: usize) -> Option<&MetricWidget> {
        self.widgets.get(index)
    }

    /// Returns a mutable reference to the widget at the given index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let mut state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("Ops", 0),
    /// ], 1);
    /// state.widget_mut(0).unwrap().set_counter_value(10);
    /// assert_eq!(state.widget(0).unwrap().display_value(), "10");
    /// ```
    pub fn widget_mut(&mut self, index: usize) -> Option<&mut MetricWidget> {
        self.widgets.get_mut(index)
    }

    /// Returns a reference to the first widget with the given label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("CPU", 42),
    ///     MetricWidget::gauge("Memory", 75, 100),
    /// ], 2);
    /// assert!(state.widget_by_label("CPU").is_some());
    /// assert!(state.widget_by_label("Disk").is_none());
    /// ```
    pub fn widget_by_label(&self, label: &str) -> Option<&MetricWidget> {
        self.widgets.iter().find(|w| w.label() == label)
    }

    /// Returns a mutable reference to the first widget with the given label.
    ///
    /// This is the primary way to update a metric by name rather than by index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let mut state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("Requests", 0),
    ///     MetricWidget::gauge("CPU", 50, 100),
    /// ], 2);
    /// state.widget_by_label_mut("Requests").unwrap().set_counter_value(42);
    /// assert_eq!(state.widget_by_label("Requests").unwrap().display_value(), "42");
    /// ```
    pub fn widget_by_label_mut(&mut self, label: &str) -> Option<&mut MetricWidget> {
        self.widgets.iter_mut().find(|w| w.label() == label)
    }

    /// Returns the number of widgets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("A", 0),
    ///     MetricWidget::counter("B", 0),
    /// ], 2);
    /// assert_eq!(state.widget_count(), 2);
    /// ```
    pub fn widget_count(&self) -> usize {
        self.widgets.len()
    }

    /// Returns the number of columns.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("A", 0),
    /// ], 3);
    /// assert_eq!(state.columns(), 3);
    /// ```
    pub fn columns(&self) -> usize {
        self.columns
    }

    /// Sets the number of columns.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let mut state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("A", 0),
    /// ], 3);
    /// state.set_columns(2);
    /// assert_eq!(state.columns(), 2);
    /// ```
    pub fn set_columns(&mut self, columns: usize) {
        self.columns = columns.max(1);
    }

    /// Returns the number of rows (based on widget count and columns).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("A", 0),
    ///     MetricWidget::counter("B", 0),
    ///     MetricWidget::counter("C", 0),
    ///     MetricWidget::counter("D", 0),
    /// ], 3);
    /// assert_eq!(state.rows(), 2); // 4 widgets in 3 columns = 2 rows
    /// ```
    pub fn rows(&self) -> usize {
        if self.widgets.is_empty() {
            0
        } else {
            self.widgets.len().div_ceil(self.columns)
        }
    }

    /// Returns the selected widget index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("A", 0),
    /// ], 1);
    /// assert_eq!(state.selected_index(), Some(0));
    /// ```
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Alias for [`selected_index()`](Self::selected_index).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("A", 0),
    /// ], 1);
    /// assert_eq!(state.selected(), state.selected_index());
    /// ```
    pub fn selected(&self) -> Option<usize> {
        self.selected_index()
    }

    /// Sets the selected widget index.
    ///
    /// The index is clamped to the valid range. Has no effect on empty dashboards.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let mut state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("A", 0),
    ///     MetricWidget::counter("B", 0),
    ///     MetricWidget::counter("C", 0),
    /// ], 3);
    /// state.set_selected(Some(2));
    /// assert_eq!(state.selected_index(), Some(2));
    /// ```
    pub fn set_selected(&mut self, index: Option<usize>) {
        match index {
            Some(i) => {
                if self.widgets.is_empty() {
                    return;
                }
                self.selected = Some(i.min(self.widgets.len() - 1));
            }
            None => self.selected = None,
        }
    }

    /// Returns a reference to the selected widget.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("Ops", 42),
    /// ], 1);
    /// assert_eq!(state.selected_widget().unwrap().label(), "Ops");
    /// ```
    pub fn selected_widget(&self) -> Option<&MetricWidget> {
        self.widgets.get(self.selected?)
    }

    /// Returns the (row, column) position of the selected widget.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let mut state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("A", 0),
    ///     MetricWidget::counter("B", 0),
    ///     MetricWidget::counter("C", 0),
    ///     MetricWidget::counter("D", 0),
    /// ], 3);
    /// state.set_selected(Some(3)); // 4th widget
    /// assert_eq!(state.selected_position(), Some((1, 0))); // row 1, col 0
    /// ```
    pub fn selected_position(&self) -> Option<(usize, usize)> {
        let selected = self.selected?;
        Some((selected / self.columns, selected % self.columns))
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let state = MetricsDashboardState::new(vec![], 1);
    /// assert_eq!(state.title(), None);
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let mut state = MetricsDashboardState::new(vec![], 1);
    /// state.set_title(Some("Dashboard".to_string()));
    /// assert_eq!(state.title(), Some("Dashboard"));
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns true if the dashboard has no widgets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricsDashboardState, MetricWidget};
    ///
    /// let state = MetricsDashboardState::new(vec![], 1);
    /// assert!(state.is_empty());
    ///
    /// let state2 = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("A", 0),
    /// ], 1);
    /// assert!(!state2.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{
    ///     MetricsDashboardState, MetricsDashboardMessage,
    ///     MetricsDashboardOutput, MetricWidget,
    /// };
    ///
    /// let mut state = MetricsDashboardState::new(vec![
    ///     MetricWidget::counter("A", 0),
    /// ], 1);
    /// let output = state.update(MetricsDashboardMessage::Select);
    /// assert_eq!(output, Some(MetricsDashboardOutput::Selected(0)));
    /// ```
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

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;

        match key.key {
            Key::Left | Key::Char('h') => Some(MetricsDashboardMessage::Left),
            Key::Right | Key::Char('l') => Some(MetricsDashboardMessage::Right),
            Key::Up | Key::Char('k') => Some(MetricsDashboardMessage::Up),
            Key::Down | Key::Char('j') => Some(MetricsDashboardMessage::Down),
            Key::Home => Some(MetricsDashboardMessage::First),
            Key::End => Some(MetricsDashboardMessage::Last),
            Key::Enter => Some(MetricsDashboardMessage::Select),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.widgets.is_empty() {
            return None;
        }

        let len = state.widgets.len();
        let cols = state.columns;
        let current = state.selected.unwrap_or(0);
        let current_row = current / cols;
        let current_col = current % cols;

        match msg {
            MetricsDashboardMessage::Left => {
                if current_col > 0 {
                    let new_index = current - 1;
                    state.selected = Some(new_index);
                    Some(MetricsDashboardOutput::SelectionChanged(new_index))
                } else {
                    None
                }
            }
            MetricsDashboardMessage::Right => {
                if current_col < cols - 1 && current + 1 < len {
                    let new_index = current + 1;
                    state.selected = Some(new_index);
                    Some(MetricsDashboardOutput::SelectionChanged(new_index))
                } else {
                    None
                }
            }
            MetricsDashboardMessage::Up => {
                if current_row > 0 {
                    let new_index = (current_row - 1) * cols + current_col;
                    if new_index < len {
                        state.selected = Some(new_index);
                        Some(MetricsDashboardOutput::SelectionChanged(new_index))
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
                    state.selected = Some(new_index);
                    Some(MetricsDashboardOutput::SelectionChanged(new_index))
                } else {
                    None
                }
            }
            MetricsDashboardMessage::First => {
                if current != 0 {
                    state.selected = Some(0);
                    Some(MetricsDashboardOutput::SelectionChanged(0))
                } else {
                    None
                }
            }
            MetricsDashboardMessage::Last => {
                let last = len - 1;
                if current != last {
                    state.selected = Some(last);
                    Some(MetricsDashboardOutput::SelectionChanged(last))
                } else {
                    None
                }
            }
            MetricsDashboardMessage::Select => Some(MetricsDashboardOutput::Selected(current)),
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        if state.widgets.is_empty() || ctx.area.height < 3 || ctx.area.width < 3 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::container("metrics_dashboard")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        let rows = state.rows();
        let cols = state.columns;

        // Compute row heights
        let row_constraints: Vec<Constraint> = (0..rows)
            .map(|_| Constraint::Ratio(1, rows as u32))
            .collect();

        let row_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(ctx.area);

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
                    let is_selected = state.selected == Some(widget_idx);
                    render_widget(widget, is_selected, &mut ctx.with_area(*col_area));
                }
            }
        }
    }
}

/// Renders a single metric widget.
fn render_widget(widget: &MetricWidget, is_selected: bool, ctx: &mut RenderContext<'_, '_>) {
    let border_style = if ctx.disabled {
        ctx.theme.disabled_style()
    } else if is_selected && ctx.focused {
        ctx.theme.focused_border_style()
    } else {
        ctx.theme.border_style()
    };

    let block = Block::default()
        .title(widget.label())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(ctx.area);
    ctx.frame.render_widget(block, ctx.area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let value_style = if ctx.disabled {
        ctx.theme.disabled_style()
    } else {
        value_color(widget, ctx.theme)
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
        ctx.frame.render_widget(paragraph, chunks[0]);

        // Sparkline
        let sparkline = Sparkline::default()
            .data(&widget.history)
            .style(value_style);
        ctx.frame.render_widget(sparkline, chunks[1]);
    } else {
        // Just value
        let value_text = widget.display_value();
        let paragraph = Paragraph::new(value_text)
            .style(value_style)
            .alignment(Alignment::Center);
        ctx.frame.render_widget(paragraph, inner);
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
mod snapshot_tests;
#[cfg(test)]
mod tests;
