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
mod tests {
    use super::*;

    // ==================== BreadcrumbSegment Tests ====================

    #[test]
    fn test_segment_new() {
        let segment = BreadcrumbSegment::new("Home");
        assert_eq!(segment.label(), "Home");
        assert_eq!(segment.data(), None);
    }

    #[test]
    fn test_segment_with_data() {
        let segment = BreadcrumbSegment::new("Products").with_data("/products");
        assert_eq!(segment.label(), "Products");
        assert_eq!(segment.data(), Some("/products"));
    }

    #[test]
    fn test_segment_accessors() {
        let segment = BreadcrumbSegment::new("Test").with_data("data");
        assert_eq!(segment.label(), "Test");
        assert_eq!(segment.data(), Some("data"));
    }

    #[test]
    fn test_segment_clone() {
        let segment = BreadcrumbSegment::new("Clone").with_data("data");
        let cloned = segment.clone();
        assert_eq!(segment, cloned);
    }

    #[test]
    fn test_segment_eq() {
        let seg1 = BreadcrumbSegment::new("Test").with_data("data");
        let seg2 = BreadcrumbSegment::new("Test").with_data("data");
        let seg3 = BreadcrumbSegment::new("Other");
        assert_eq!(seg1, seg2);
        assert_ne!(seg1, seg3);
    }

    // ==================== State Creation Tests ====================

    #[test]
    fn test_new() {
        let segments = vec![
            BreadcrumbSegment::new("Home"),
            BreadcrumbSegment::new("Products"),
        ];
        let state = BreadcrumbState::new(segments);
        assert_eq!(state.len(), 2);
        assert_eq!(state.focused_index(), 0);
    }

    #[test]
    fn test_from_labels() {
        let state = BreadcrumbState::from_labels(vec!["Home", "Products", "Item"]);
        assert_eq!(state.len(), 3);
        assert_eq!(state.segments()[0].label(), "Home");
        assert_eq!(state.segments()[1].label(), "Products");
        assert_eq!(state.segments()[2].label(), "Item");
    }

    #[test]
    fn test_from_path() {
        let state = BreadcrumbState::from_path("home/user/documents", "/");
        assert_eq!(state.len(), 3);
        assert_eq!(state.segments()[0].label(), "home");
        assert_eq!(state.segments()[1].label(), "user");
        assert_eq!(state.segments()[2].label(), "documents");
    }

    #[test]
    fn test_from_path_with_leading_separator() {
        let state = BreadcrumbState::from_path("/home/user", "/");
        assert_eq!(state.len(), 2);
        assert_eq!(state.segments()[0].label(), "home");
    }

    #[test]
    fn test_default() {
        let state = BreadcrumbState::default();
        assert!(state.is_empty());
        assert_eq!(state.separator(), " > ");
        assert_eq!(state.max_visible(), None);
    }

    #[test]
    fn test_new_empty() {
        let state = BreadcrumbState::new(vec![]);
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }

    // ==================== Accessor Tests ====================

    #[test]
    fn test_segments() {
        let state = BreadcrumbState::from_labels(vec!["A", "B"]);
        let segments = state.segments();
        assert_eq!(segments.len(), 2);
    }

    #[test]
    fn test_len() {
        let state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        assert_eq!(state.len(), 3);
    }

    #[test]
    fn test_is_empty() {
        let empty = BreadcrumbState::default();
        let non_empty = BreadcrumbState::from_labels(vec!["A"]);
        assert!(empty.is_empty());
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_focused_index() {
        let state = BreadcrumbState::from_labels(vec!["A", "B"]);
        assert_eq!(state.focused_index(), 0);
    }

    #[test]
    fn test_focused_segment() {
        let state = BreadcrumbState::from_labels(vec!["Home", "Products"]);
        assert_eq!(state.focused_segment().unwrap().label(), "Home");
    }

    #[test]
    fn test_focused_segment_empty() {
        let state = BreadcrumbState::default();
        assert!(state.focused_segment().is_none());
    }

    #[test]
    fn test_is_disabled() {
        let mut state = BreadcrumbState::default();
        assert!(!state.is_disabled());
        state.set_disabled(true);
        assert!(state.is_disabled());
    }

    #[test]
    fn test_separator() {
        let state = BreadcrumbState::default();
        assert_eq!(state.separator(), " > ");
    }

    #[test]
    fn test_max_visible() {
        let mut state = BreadcrumbState::default();
        assert_eq!(state.max_visible(), None);
        state.set_max_visible(Some(3));
        assert_eq!(state.max_visible(), Some(3));
    }

    #[test]
    fn test_current() {
        let state = BreadcrumbState::from_labels(vec!["Home", "Products", "Item"]);
        assert_eq!(state.current().unwrap().label(), "Item");
    }

    #[test]
    fn test_current_empty() {
        let state = BreadcrumbState::default();
        assert!(state.current().is_none());
    }

    // ==================== Mutator Tests ====================

    #[test]
    fn test_set_segments() {
        let mut state = BreadcrumbState::from_labels(vec!["A"]);
        state.set_segments(vec![
            BreadcrumbSegment::new("X"),
            BreadcrumbSegment::new("Y"),
        ]);
        assert_eq!(state.len(), 2);
        assert_eq!(state.segments()[0].label(), "X");
        assert_eq!(state.focused_index(), 0);
    }

    #[test]
    fn test_push() {
        let mut state = BreadcrumbState::from_labels(vec!["Home"]);
        state.push(BreadcrumbSegment::new("Products"));
        assert_eq!(state.len(), 2);
        assert_eq!(state.segments()[1].label(), "Products");
    }

    #[test]
    fn test_pop() {
        let mut state = BreadcrumbState::from_labels(vec!["Home", "Products", "Item"]);
        let popped = state.pop();
        assert_eq!(popped.unwrap().label(), "Item");
        assert_eq!(state.len(), 2);
    }

    #[test]
    fn test_pop_adjusts_focus() {
        let mut state = BreadcrumbState::from_labels(vec!["Home", "Products"]);
        state.focused_index = 1;
        state.pop();
        assert_eq!(state.focused_index(), 0);
    }

    #[test]
    fn test_pop_empty() {
        let mut state = BreadcrumbState::default();
        assert!(state.pop().is_none());
    }

    #[test]
    fn test_set_separator() {
        let mut state = BreadcrumbState::default();
        state.set_separator(" / ");
        assert_eq!(state.separator(), " / ");
    }

    #[test]
    fn test_set_max_visible() {
        let mut state = BreadcrumbState::default();
        state.set_max_visible(Some(5));
        assert_eq!(state.max_visible(), Some(5));
        state.set_max_visible(None);
        assert_eq!(state.max_visible(), None);
    }

    #[test]
    fn test_set_disabled() {
        let mut state = BreadcrumbState::default();
        state.set_disabled(true);
        assert!(state.is_disabled());
        state.set_disabled(false);
        assert!(!state.is_disabled());
    }

    // ==================== Truncation Tests ====================

    #[test]
    fn test_is_truncated_false() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        state.set_max_visible(Some(5));
        assert!(!state.is_truncated());
    }

    #[test]
    fn test_is_truncated_true() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C", "D", "E"]);
        state.set_max_visible(Some(3));
        assert!(state.is_truncated());
    }

    #[test]
    fn test_is_truncated_no_max() {
        let state = BreadcrumbState::from_labels(vec!["A", "B", "C", "D", "E"]);
        assert!(!state.is_truncated());
    }

    #[test]
    fn test_visible_segments() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C", "D", "E"]);
        state.set_max_visible(Some(3));
        let visible = state.visible_segments();
        assert_eq!(visible.len(), 3);
        assert_eq!(visible[0].label(), "C");
        assert_eq!(visible[1].label(), "D");
        assert_eq!(visible[2].label(), "E");
    }

    #[test]
    fn test_visible_segments_no_truncation() {
        let state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        let visible = state.visible_segments();
        assert_eq!(visible.len(), 3);
        assert_eq!(visible[0].label(), "A");
    }

    #[test]
    fn test_truncation_shows_last_n() {
        let mut state =
            BreadcrumbState::from_labels(vec!["Root", "Level1", "Level2", "Level3", "Current"]);
        state.set_max_visible(Some(3));
        let visible = state.visible_segments();
        assert_eq!(visible[0].label(), "Level2");
        assert_eq!(visible[1].label(), "Level3");
        assert_eq!(visible[2].label(), "Current");
    }

    // ==================== Navigation Tests ====================

    #[test]
    fn test_left() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        state.focused_index = 2;
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Left);
        assert_eq!(output, Some(BreadcrumbOutput::FocusChanged(1)));
        assert_eq!(state.focused_index(), 1);
    }

    #[test]
    fn test_right() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Right);
        assert_eq!(output, Some(BreadcrumbOutput::FocusChanged(1)));
        assert_eq!(state.focused_index(), 1);
    }

    #[test]
    fn test_left_at_start() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Left);
        assert_eq!(output, None);
        assert_eq!(state.focused_index(), 0);
    }

    #[test]
    fn test_right_at_end() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        state.focused_index = 2;
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Right);
        assert_eq!(output, None);
        assert_eq!(state.focused_index(), 2);
    }

    #[test]
    fn test_first() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        state.focused_index = 2;
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::First);
        assert_eq!(output, Some(BreadcrumbOutput::FocusChanged(0)));
        assert_eq!(state.focused_index(), 0);
    }

    #[test]
    fn test_first_already_at_first() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::First);
        assert_eq!(output, None);
    }

    #[test]
    fn test_last() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Last);
        assert_eq!(output, Some(BreadcrumbOutput::FocusChanged(2)));
        assert_eq!(state.focused_index(), 2);
    }

    #[test]
    fn test_last_already_at_last() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        state.focused_index = 2;
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Last);
        assert_eq!(output, None);
    }

    #[test]
    fn test_navigation_empty() {
        let mut state = BreadcrumbState::default();
        assert_eq!(
            Breadcrumb::update(&mut state, BreadcrumbMessage::Left),
            None
        );
        assert_eq!(
            Breadcrumb::update(&mut state, BreadcrumbMessage::Right),
            None
        );
        assert_eq!(
            Breadcrumb::update(&mut state, BreadcrumbMessage::First),
            None
        );
        assert_eq!(
            Breadcrumb::update(&mut state, BreadcrumbMessage::Last),
            None
        );
    }

    #[test]
    fn test_navigation_returns_focus_changed() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B"]);
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Right);
        assert!(matches!(output, Some(BreadcrumbOutput::FocusChanged(_))));
    }

    // ==================== Selection Tests ====================

    #[test]
    fn test_select() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        state.focused_index = 1;
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Select);
        assert_eq!(output, Some(BreadcrumbOutput::Selected(1)));
    }

    #[test]
    fn test_select_returns_selected() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B"]);
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Select);
        assert!(matches!(output, Some(BreadcrumbOutput::Selected(_))));
    }

    #[test]
    fn test_select_index() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::SelectIndex(2));
        assert_eq!(output, Some(BreadcrumbOutput::Selected(2)));
    }

    #[test]
    fn test_select_index_out_of_bounds() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B"]);
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::SelectIndex(5));
        assert_eq!(output, None);
    }

    #[test]
    fn test_select_empty() {
        let mut state = BreadcrumbState::default();
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Select);
        assert_eq!(output, None);
    }

    // ==================== Disabled State Tests ====================

    #[test]
    fn test_disabled_ignores_messages() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        state.set_disabled(true);

        assert_eq!(
            Breadcrumb::update(&mut state, BreadcrumbMessage::Right),
            None
        );
        assert_eq!(
            Breadcrumb::update(&mut state, BreadcrumbMessage::Left),
            None
        );
        assert_eq!(
            Breadcrumb::update(&mut state, BreadcrumbMessage::Select),
            None
        );
        assert_eq!(state.focused_index(), 0);
    }

    #[test]
    fn test_disabling_preserves_state() {
        let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
        state.focused_index = 1;
        state.set_disabled(true);
        assert_eq!(state.focused_index(), 1);
        assert_eq!(state.len(), 3);
    }

    // ==================== Focus Tests ====================

    #[test]
    fn test_focusable_is_focused() {
        let mut state = BreadcrumbState::default();
        assert!(!Breadcrumb::is_focused(&state));
        state.focused = true;
        assert!(Breadcrumb::is_focused(&state));
    }

    #[test]
    fn test_focusable_set_focused() {
        let mut state = BreadcrumbState::default();
        Breadcrumb::set_focused(&mut state, true);
        assert!(state.focused);
        Breadcrumb::set_focused(&mut state, false);
        assert!(!state.focused);
    }

    #[test]
    fn test_focus_blur() {
        let mut state = BreadcrumbState::default();
        Breadcrumb::focus(&mut state);
        assert!(Breadcrumb::is_focused(&state));
        Breadcrumb::blur(&mut state);
        assert!(!Breadcrumb::is_focused(&state));
    }

    // ==================== View Tests ====================

    #[test]
    fn test_view_empty() {
        use crate::backend::CaptureBackend;
        use crate::theme::Theme;
        use ratatui::Terminal;

        let state = BreadcrumbState::default();

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        // Empty state renders nothing
        assert!(output.trim().is_empty());
    }

    #[test]
    fn test_view_single() {
        use crate::backend::CaptureBackend;
        use crate::theme::Theme;
        use ratatui::Terminal;

        let state = BreadcrumbState::from_labels(vec!["Home"]);

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Home"));
    }

    #[test]
    fn test_view_multiple() {
        use crate::backend::CaptureBackend;
        use crate::theme::Theme;
        use ratatui::Terminal;

        let state = BreadcrumbState::from_labels(vec!["Home", "Products", "Item"]);

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Home"));
        assert!(output.contains(">"));
        assert!(output.contains("Products"));
        assert!(output.contains("Item"));
    }

    #[test]
    fn test_view_focused_highlight() {
        use crate::backend::CaptureBackend;
        use crate::theme::Theme;
        use ratatui::Terminal;

        let mut state = BreadcrumbState::from_labels(vec!["Home", "Products"]);
        state.focused = true;
        state.focused_index = 1;

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        // Just verify it renders - style testing would need ANSI output
        assert!(output.contains("Products"));
    }

    #[test]
    fn test_view_truncated() {
        use crate::backend::CaptureBackend;
        use crate::theme::Theme;
        use ratatui::Terminal;

        let mut state =
            BreadcrumbState::from_labels(vec!["Root", "Level1", "Level2", "Level3", "Current"]);
        state.set_max_visible(Some(3));

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("…")); // Ellipsis for truncation
        assert!(output.contains("Level2"));
        assert!(output.contains("Current"));
        assert!(!output.contains("Root")); // Truncated
    }

    #[test]
    fn test_view_custom_separator() {
        use crate::backend::CaptureBackend;
        use crate::theme::Theme;
        use ratatui::Terminal;

        let mut state = BreadcrumbState::from_labels(vec!["Home", "Docs"]);
        state.set_separator(" / ");

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains(" / "));
    }

    #[test]
    fn test_view_disabled() {
        use crate::backend::CaptureBackend;
        use crate::theme::Theme;
        use ratatui::Terminal;

        let mut state = BreadcrumbState::from_labels(vec!["Home", "Products"]);
        state.set_disabled(true);

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        // Just verify it renders - disabled style would need ANSI output
        assert!(output.contains("Home"));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_clone() {
        let state = BreadcrumbState::from_labels(vec!["A", "B"]);
        let cloned = state.clone();
        assert_eq!(state.len(), cloned.len());
        assert_eq!(state.separator(), cloned.separator());
    }

    #[test]
    fn test_init() {
        let state = Breadcrumb::init();
        assert!(state.is_empty());
        assert_eq!(state.separator(), " > ");
    }

    #[test]
    fn test_full_workflow() {
        let mut state = BreadcrumbState::from_labels(vec!["Home", "Products", "Electronics"]);
        Breadcrumb::set_focused(&mut state, true);

        // Navigate right twice
        Breadcrumb::update(&mut state, BreadcrumbMessage::Right);
        Breadcrumb::update(&mut state, BreadcrumbMessage::Right);
        assert_eq!(state.focused_index(), 2);

        // Select the current segment
        let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Select);
        assert_eq!(output, Some(BreadcrumbOutput::Selected(2)));

        // Navigate back
        Breadcrumb::update(&mut state, BreadcrumbMessage::First);
        assert_eq!(state.focused_index(), 0);

        // Push a new segment
        state.push(BreadcrumbSegment::new("Item"));
        assert_eq!(state.len(), 4);

        // Pop a segment
        state.pop();
        assert_eq!(state.len(), 3);
    }
}
