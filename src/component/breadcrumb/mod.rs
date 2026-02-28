//! A breadcrumb navigation component.
//!
//! `Breadcrumb` displays a hierarchical navigation path with clickable segments.
//! Users can navigate through the path using keyboard navigation, and selecting
//! a segment emits an output for navigation handling.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Breadcrumb, BreadcrumbMessage, BreadcrumbOutput, BreadcrumbSegment,
//!     BreadcrumbState, Component, Focusable,
//! };
//!
//! // Create breadcrumb from segments
//! let segments = vec![
//!     BreadcrumbSegment::new("Home"),
//!     BreadcrumbSegment::new("Products"),
//!     BreadcrumbSegment::new("Electronics"),
//! ];
//! let mut state = BreadcrumbState::new(segments);
//! Breadcrumb::set_focused(&mut state, true);
//!
//! // Navigate right
//! let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Right);
//! assert_eq!(output, Some(BreadcrumbOutput::FocusChanged(1)));
//! assert_eq!(state.focused_index(), 1);
//!
//! // Select the focused segment
//! let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Select);
//! assert_eq!(output, Some(BreadcrumbOutput::Selected(1)));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{Component, Focusable};
use crate::theme::Theme;

/// A single breadcrumb segment.
///
/// Each segment has a display label and optional associated data
/// (e.g., a path or ID for navigation).
///
/// # Example
///
/// ```rust
/// use envision::component::BreadcrumbSegment;
///
/// let segment = BreadcrumbSegment::new("Products").with_data("/products");
/// assert_eq!(segment.label(), "Products");
/// assert_eq!(segment.data(), Some("/products"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BreadcrumbSegment {
    /// The display label for this segment.
    label: String,
    /// Optional data associated with this segment.
    data: Option<String>,
}

impl BreadcrumbSegment {
    /// Creates a new breadcrumb segment with the given label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BreadcrumbSegment;
    ///
    /// let segment = BreadcrumbSegment::new("Home");
    /// assert_eq!(segment.label(), "Home");
    /// assert_eq!(segment.data(), None);
    /// ```
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            data: None,
        }
    }

    /// Sets the data associated with this segment.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BreadcrumbSegment;
    ///
    /// let segment = BreadcrumbSegment::new("Products").with_data("/products");
    /// assert_eq!(segment.data(), Some("/products"));
    /// ```
    pub fn with_data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Returns the display label for this segment.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the data associated with this segment, if any.
    pub fn data(&self) -> Option<&str> {
        self.data.as_deref()
    }
}

/// Messages that can be sent to a Breadcrumb component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BreadcrumbMessage {
    /// Move focus to the previous (left) segment.
    Left,
    /// Move focus to the next (right) segment.
    Right,
    /// Jump to the first segment.
    First,
    /// Jump to the last segment.
    Last,
    /// Select the currently focused segment.
    Select,
    /// Select a specific segment by index.
    SelectIndex(usize),
}

/// Output messages from a Breadcrumb component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BreadcrumbOutput {
    /// A segment was selected (for navigation).
    Selected(usize),
    /// Focus moved to a segment.
    FocusChanged(usize),
}

/// State for a Breadcrumb component.
///
/// The state tracks the breadcrumb segments, the currently focused segment,
/// separator configuration, and truncation settings.
///
/// # Example
///
/// ```rust
/// use envision::component::BreadcrumbState;
///
/// let state = BreadcrumbState::from_labels(vec!["Home", "Products", "Item"]);
/// assert_eq!(state.len(), 3);
/// assert_eq!(state.separator(), " > ");
/// ```
#[derive(Clone, Debug)]
pub struct BreadcrumbState {
    /// The breadcrumb segments.
    segments: Vec<BreadcrumbSegment>,
    /// Currently focused segment index.
    focused_index: usize,
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
    /// The separator between segments.
    separator: String,
    /// Maximum visible segments (None = show all).
    max_visible: Option<usize>,
}

impl Default for BreadcrumbState {
    fn default() -> Self {
        Self {
            segments: Vec::new(),
            focused_index: 0,
            focused: false,
            disabled: false,
            separator: " > ".to_string(),
            max_visible: None,
        }
    }
}

impl BreadcrumbState {
    /// Creates a new breadcrumb with the given segments.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BreadcrumbSegment, BreadcrumbState};
    ///
    /// let segments = vec![
    ///     BreadcrumbSegment::new("Home"),
    ///     BreadcrumbSegment::new("Products"),
    /// ];
    /// let state = BreadcrumbState::new(segments);
    /// assert_eq!(state.len(), 2);
    /// ```
    pub fn new(segments: Vec<BreadcrumbSegment>) -> Self {
        Self {
            segments,
            focused_index: 0,
            focused: false,
            disabled: false,
            separator: " > ".to_string(),
            max_visible: None,
        }
    }

    /// Creates a breadcrumb from string labels.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BreadcrumbState;
    ///
    /// let state = BreadcrumbState::from_labels(vec!["Home", "Settings", "Profile"]);
    /// assert_eq!(state.len(), 3);
    /// assert_eq!(state.segments()[0].label(), "Home");
    /// ```
    pub fn from_labels<S: Into<String>>(labels: Vec<S>) -> Self {
        let segments = labels
            .into_iter()
            .map(|label| BreadcrumbSegment::new(label))
            .collect();
        Self::new(segments)
    }

    /// Creates a breadcrumb from a path string by splitting on a separator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BreadcrumbState;
    ///
    /// let state = BreadcrumbState::from_path("home/user/documents", "/");
    /// assert_eq!(state.len(), 3);
    /// assert_eq!(state.segments()[0].label(), "home");
    /// assert_eq!(state.segments()[1].label(), "user");
    /// assert_eq!(state.segments()[2].label(), "documents");
    /// ```
    pub fn from_path(path: &str, separator: &str) -> Self {
        let segments = path
            .split(separator)
            .filter(|s| !s.is_empty())
            .map(BreadcrumbSegment::new)
            .collect();
        Self::new(segments)
    }

    /// Returns the breadcrumb segments.
    pub fn segments(&self) -> &[BreadcrumbSegment] {
        &self.segments
    }

    /// Returns the number of segments.
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Returns true if there are no segments.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Returns the currently focused segment index.
    pub fn focused_index(&self) -> usize {
        self.focused_index
    }

    /// Returns the currently focused segment, if any.
    pub fn focused_segment(&self) -> Option<&BreadcrumbSegment> {
        self.segments.get(self.focused_index)
    }

    /// Returns whether the component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Returns the separator used between segments.
    pub fn separator(&self) -> &str {
        &self.separator
    }

    /// Returns the maximum number of visible segments.
    pub fn max_visible(&self) -> Option<usize> {
        self.max_visible
    }

    /// Returns the last segment (current location).
    pub fn current(&self) -> Option<&BreadcrumbSegment> {
        self.segments.last()
    }

    /// Sets new segments, resetting the focused index.
    pub fn set_segments(&mut self, segments: Vec<BreadcrumbSegment>) {
        self.segments = segments;
        self.focused_index = 0;
    }

    /// Adds a segment to the end.
    pub fn push(&mut self, segment: BreadcrumbSegment) {
        self.segments.push(segment);
    }

    /// Removes and returns the last segment.
    ///
    /// Adjusts the focused index if necessary.
    pub fn pop(&mut self) -> Option<BreadcrumbSegment> {
        let result = self.segments.pop();
        if !self.segments.is_empty() && self.focused_index >= self.segments.len() {
            self.focused_index = self.segments.len() - 1;
        }
        result
    }

    /// Sets the separator between segments.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BreadcrumbState;
    ///
    /// let mut state = BreadcrumbState::from_labels(vec!["Home", "Docs"]);
    /// state.set_separator(" / ");
    /// assert_eq!(state.separator(), " / ");
    /// ```
    pub fn set_separator(&mut self, separator: impl Into<String>) {
        self.separator = separator.into();
    }

    /// Sets the maximum number of visible segments.
    ///
    /// When set, only the last `max` segments are shown, with an ellipsis
    /// indicating truncation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BreadcrumbState;
    ///
    /// let mut state = BreadcrumbState::from_labels(vec![
    ///     "Root", "Level1", "Level2", "Level3", "Current"
    /// ]);
    /// state.set_max_visible(Some(3));
    /// assert!(state.is_truncated());
    /// ```
    pub fn set_max_visible(&mut self, max: Option<usize>) {
        self.max_visible = max;
    }

    /// Sets whether the component is disabled.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Returns whether the breadcrumb is truncated.
    pub fn is_truncated(&self) -> bool {
        match self.max_visible {
            Some(max) if max > 0 => self.segments.len() > max,
            _ => false,
        }
    }

    /// Returns the range of visible segment indices.
    fn visible_range(&self) -> (usize, usize) {
        match self.max_visible {
            Some(max) if max > 0 && self.segments.len() > max => {
                let start = self.segments.len() - max;
                (start, self.segments.len())
            }
            _ => (0, self.segments.len()),
        }
    }

    /// Returns the visible segments.
    ///
    /// When truncation is enabled and the number of segments exceeds
    /// `max_visible`, only the last `max_visible` segments are returned.
    pub fn visible_segments(&self) -> &[BreadcrumbSegment] {
        let (start, end) = self.visible_range();
        &self.segments[start..end]
    }
}

/// A breadcrumb navigation component.
///
/// `Breadcrumb` displays a hierarchical path with navigable segments.
/// Users can move focus between segments with Left/Right keys and
/// select a segment with Enter to trigger navigation.
///
/// # Navigation
///
/// - `Left` - Move focus to the previous segment
/// - `Right` - Move focus to the next segment
/// - `First` - Jump to the first segment
/// - `Last` - Jump to the last segment
/// - `Select` - Select the focused segment
/// - `SelectIndex(index)` - Select a specific segment
///
/// # Output
///
/// - `FocusChanged(index)` - Emitted when focus moves to a different segment
/// - `Selected(index)` - Emitted when a segment is selected
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Breadcrumb, BreadcrumbMessage, BreadcrumbOutput, BreadcrumbState, Component, Focusable,
/// };
///
/// let mut state = BreadcrumbState::from_labels(vec!["Home", "Products", "Item"]);
/// Breadcrumb::set_focused(&mut state, true);
///
/// // Navigate to Products
/// let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Right);
/// assert_eq!(output, Some(BreadcrumbOutput::FocusChanged(1)));
///
/// // Select it for navigation
/// let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Select);
/// assert_eq!(output, Some(BreadcrumbOutput::Selected(1)));
/// ```
pub struct Breadcrumb;

impl Component for Breadcrumb {
    type State = BreadcrumbState;
    type Message = BreadcrumbMessage;
    type Output = BreadcrumbOutput;

    fn init() -> Self::State {
        BreadcrumbState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled || state.segments.is_empty() {
            return None;
        }

        match msg {
            BreadcrumbMessage::Left => {
                if state.focused_index > 0 {
                    state.focused_index -= 1;
                    Some(BreadcrumbOutput::FocusChanged(state.focused_index))
                } else {
                    None
                }
            }
            BreadcrumbMessage::Right => {
                if state.focused_index < state.segments.len().saturating_sub(1) {
                    state.focused_index += 1;
                    Some(BreadcrumbOutput::FocusChanged(state.focused_index))
                } else {
                    None
                }
            }
            BreadcrumbMessage::First => {
                if state.focused_index != 0 {
                    state.focused_index = 0;
                    Some(BreadcrumbOutput::FocusChanged(0))
                } else {
                    None
                }
            }
            BreadcrumbMessage::Last => {
                let last = state.segments.len().saturating_sub(1);
                if state.focused_index != last {
                    state.focused_index = last;
                    Some(BreadcrumbOutput::FocusChanged(last))
                } else {
                    None
                }
            }
            BreadcrumbMessage::Select => Some(BreadcrumbOutput::Selected(state.focused_index)),
            BreadcrumbMessage::SelectIndex(index) => {
                if index < state.segments.len() {
                    Some(BreadcrumbOutput::Selected(index))
                } else {
                    None
                }
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if state.segments.is_empty() {
            return;
        }

        let mut spans: Vec<Span> = Vec::new();
        let (start, end) = state.visible_range();

        // Add ellipsis if truncated
        if state.is_truncated() {
            spans.push(Span::styled("…", theme.disabled_style()));
            spans.push(Span::raw(&state.separator));
        }

        for seg_idx in start..end {
            let segment = &state.segments[seg_idx];
            let is_last = seg_idx == state.segments.len() - 1;
            let is_focused_segment = state.focused && seg_idx == state.focused_index;

            let style = if state.disabled {
                theme.disabled_style()
            } else if is_focused_segment {
                theme
                    .focused_style()
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else if is_last {
                // Current location - bold, not underlined
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                // Navigable segments
                theme.info_style()
            };

            spans.push(Span::styled(segment.label(), style));

            if !is_last {
                spans.push(Span::raw(&state.separator));
            }
        }

        let line = Line::from(spans);
        frame.render_widget(Paragraph::new(line), area);
    }
}

impl Focusable for Breadcrumb {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests;
