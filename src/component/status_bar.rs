//! A status bar component for displaying application state.
//!
//! `StatusBar` provides a horizontal bar typically displayed at the bottom of the
//! screen showing application status, mode indicators, and other information.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{StatusBar, StatusBarMessage, StatusBarState, StatusBarItem, StatusBarStyle, Component};
//!
//! // Create a status bar with items
//! let mut state = StatusBarState::new();
//! state.push_left(StatusBarItem::new("INSERT"));
//! state.push_center(StatusBarItem::new("main.rs"));
//! state.push_right(StatusBarItem::new("Ln 42, Col 8"));
//!
//! // Update an item
//! StatusBar::update(&mut state, StatusBarMessage::SetLeftItems(vec![
//!     StatusBarItem::new("NORMAL").with_style(StatusBarStyle::Info),
//! ]));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::Component;

/// Style variants for status bar items.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatusBarStyle {
    /// Default style (no special coloring).
    #[default]
    Default,
    /// Informational style (typically blue).
    Info,
    /// Success style (typically green).
    Success,
    /// Warning style (typically yellow).
    Warning,
    /// Error style (typically red).
    Error,
    /// Muted/secondary style (typically gray).
    Muted,
}

impl StatusBarStyle {
    /// Returns the foreground color for this style.
    fn fg_color(&self) -> Color {
        match self {
            Self::Default => Color::White,
            Self::Info => Color::Blue,
            Self::Success => Color::Green,
            Self::Warning => Color::Yellow,
            Self::Error => Color::Red,
            Self::Muted => Color::DarkGray,
        }
    }
}

/// A single item in the status bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusBarItem {
    /// The text content of the item.
    text: String,
    /// The style of the item.
    style: StatusBarStyle,
    /// Whether to show a separator after this item.
    separator: bool,
}

impl StatusBarItem {
    /// Creates a new status bar item with the given text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarItem;
    ///
    /// let item = StatusBarItem::new("Ready");
    /// assert_eq!(item.text(), "Ready");
    /// ```
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: StatusBarStyle::Default,
            separator: true,
        }
    }

    /// Sets the style for this item.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarItem, StatusBarStyle};
    ///
    /// let item = StatusBarItem::new("Error").with_style(StatusBarStyle::Error);
    /// assert_eq!(item.style(), StatusBarStyle::Error);
    /// ```
    pub fn with_style(mut self, style: StatusBarStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets whether to show a separator after this item.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarItem;
    ///
    /// let item = StatusBarItem::new("Last").with_separator(false);
    /// assert!(!item.has_separator());
    /// ```
    pub fn with_separator(mut self, separator: bool) -> Self {
        self.separator = separator;
        self
    }

    /// Returns the text content.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Sets the text content.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Returns the style.
    pub fn style(&self) -> StatusBarStyle {
        self.style
    }

    /// Sets the style.
    pub fn set_style(&mut self, style: StatusBarStyle) {
        self.style = style;
    }

    /// Returns whether this item has a separator.
    pub fn has_separator(&self) -> bool {
        self.separator
    }

    /// Sets whether to show a separator.
    pub fn set_separator(&mut self, separator: bool) {
        self.separator = separator;
    }
}

/// Messages that can be sent to a StatusBar.
#[derive(Clone, Debug, PartialEq)]
pub enum StatusBarMessage {
    /// Set the items in the left section.
    SetLeftItems(Vec<StatusBarItem>),
    /// Set the items in the center section.
    SetCenterItems(Vec<StatusBarItem>),
    /// Set the items in the right section.
    SetRightItems(Vec<StatusBarItem>),
    /// Clear all items from all sections.
    Clear,
    /// Clear items from the left section.
    ClearLeft,
    /// Clear items from the center section.
    ClearCenter,
    /// Clear items from the right section.
    ClearRight,
}

/// Output messages from a StatusBar.
///
/// StatusBar is display-only, so there are no output messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatusBarOutput {}

/// State for a StatusBar component.
#[derive(Clone, Debug, Default)]
pub struct StatusBarState {
    /// Items aligned to the left.
    left: Vec<StatusBarItem>,
    /// Items aligned to the center.
    center: Vec<StatusBarItem>,
    /// Items aligned to the right.
    right: Vec<StatusBarItem>,
    /// The separator character to use between items.
    separator: String,
    /// Background style for the entire bar.
    background: Color,
}

impl StatusBarState {
    /// Creates a new empty status bar.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let state = StatusBarState::new();
    /// assert!(state.left().is_empty());
    /// assert!(state.center().is_empty());
    /// assert!(state.right().is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            left: Vec::new(),
            center: Vec::new(),
            right: Vec::new(),
            separator: " | ".to_string(),
            background: Color::DarkGray,
        }
    }

    /// Creates a status bar with a custom separator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let state = StatusBarState::with_separator(" :: ");
    /// assert_eq!(state.separator(), " :: ");
    /// ```
    pub fn with_separator(separator: impl Into<String>) -> Self {
        Self {
            separator: separator.into(),
            ..Self::new()
        }
    }

    /// Returns the left section items.
    pub fn left(&self) -> &[StatusBarItem] {
        &self.left
    }

    /// Returns the center section items.
    pub fn center(&self) -> &[StatusBarItem] {
        &self.center
    }

    /// Returns the right section items.
    pub fn right(&self) -> &[StatusBarItem] {
        &self.right
    }

    /// Sets the left section items.
    pub fn set_left(&mut self, items: Vec<StatusBarItem>) {
        self.left = items;
    }

    /// Sets the center section items.
    pub fn set_center(&mut self, items: Vec<StatusBarItem>) {
        self.center = items;
    }

    /// Sets the right section items.
    pub fn set_right(&mut self, items: Vec<StatusBarItem>) {
        self.right = items;
    }

    /// Adds an item to the left section.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem};
    ///
    /// let mut state = StatusBarState::new();
    /// state.push_left(StatusBarItem::new("Mode"));
    /// assert_eq!(state.left().len(), 1);
    /// ```
    pub fn push_left(&mut self, item: StatusBarItem) {
        self.left.push(item);
    }

    /// Adds an item to the center section.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem};
    ///
    /// let mut state = StatusBarState::new();
    /// state.push_center(StatusBarItem::new("filename.rs"));
    /// assert_eq!(state.center().len(), 1);
    /// ```
    pub fn push_center(&mut self, item: StatusBarItem) {
        self.center.push(item);
    }

    /// Adds an item to the right section.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem};
    ///
    /// let mut state = StatusBarState::new();
    /// state.push_right(StatusBarItem::new("Ln 1"));
    /// assert_eq!(state.right().len(), 1);
    /// ```
    pub fn push_right(&mut self, item: StatusBarItem) {
        self.right.push(item);
    }

    /// Clears all sections.
    pub fn clear(&mut self) {
        self.left.clear();
        self.center.clear();
        self.right.clear();
    }

    /// Returns the separator string.
    pub fn separator(&self) -> &str {
        &self.separator
    }

    /// Sets the separator string.
    pub fn set_separator(&mut self, separator: impl Into<String>) {
        self.separator = separator.into();
    }

    /// Returns the background color.
    pub fn background(&self) -> Color {
        self.background
    }

    /// Sets the background color.
    pub fn set_background(&mut self, color: Color) {
        self.background = color;
    }

    /// Returns true if all sections are empty.
    pub fn is_empty(&self) -> bool {
        self.left.is_empty() && self.center.is_empty() && self.right.is_empty()
    }

    /// Returns the total number of items across all sections.
    pub fn len(&self) -> usize {
        self.left.len() + self.center.len() + self.right.len()
    }
}

/// A status bar component for displaying application state.
///
/// This component provides a horizontal bar typically shown at the bottom of
/// the screen. It supports three sections (left, center, right) with styled
/// items and customizable separators.
///
/// # Layout
///
/// ```text
/// ┌────────────────────────────────────────────────────────────┐
/// │ LEFT | ITEMS       CENTER ITEMS         RIGHT | ITEMS      │
/// └────────────────────────────────────────────────────────────┘
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{StatusBar, StatusBarMessage, StatusBarState, StatusBarItem, StatusBarStyle, Component};
///
/// let mut state = StatusBarState::new();
///
/// // Add items to different sections
/// state.push_left(StatusBarItem::new("NORMAL").with_style(StatusBarStyle::Info));
/// state.push_center(StatusBarItem::new("main.rs"));
/// state.push_right(StatusBarItem::new("UTF-8"));
/// state.push_right(StatusBarItem::new("Ln 42, Col 8"));
///
/// // Update via message
/// StatusBar::update(&mut state, StatusBarMessage::SetLeftItems(vec![
///     StatusBarItem::new("INSERT").with_style(StatusBarStyle::Success),
/// ]));
/// ```
pub struct StatusBar;

impl StatusBar {
    /// Renders a section of items to a span list.
    fn render_section<'a>(items: &'a [StatusBarItem], separator: &'a str) -> Vec<Span<'a>> {
        let mut spans = Vec::new();

        for (idx, item) in items.iter().enumerate() {
            let style = Style::default().fg(item.style.fg_color());
            spans.push(Span::styled(item.text.as_str(), style));

            // Add separator if not last item and item has separator enabled
            if idx < items.len() - 1 && item.has_separator() {
                spans.push(Span::styled(
                    separator,
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }

        spans
    }
}

impl Component for StatusBar {
    type State = StatusBarState;
    type Message = StatusBarMessage;
    type Output = StatusBarOutput;

    fn init() -> Self::State {
        StatusBarState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            StatusBarMessage::SetLeftItems(items) => {
                state.left = items;
            }
            StatusBarMessage::SetCenterItems(items) => {
                state.center = items;
            }
            StatusBarMessage::SetRightItems(items) => {
                state.right = items;
            }
            StatusBarMessage::Clear => {
                state.clear();
            }
            StatusBarMessage::ClearLeft => {
                state.left.clear();
            }
            StatusBarMessage::ClearCenter => {
                state.center.clear();
            }
            StatusBarMessage::ClearRight => {
                state.right.clear();
            }
        }
        None
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
        // Render background
        let bg_style = Style::default().bg(state.background);

        // Calculate section widths
        let left_spans = Self::render_section(&state.left, &state.separator);
        let center_spans = Self::render_section(&state.center, &state.separator);
        let right_spans = Self::render_section(&state.right, &state.separator);

        // Calculate the width of each section
        let left_width: usize = left_spans.iter().map(|s| s.content.len()).sum();
        let center_width: usize = center_spans.iter().map(|s| s.content.len()).sum();
        let right_width: usize = right_spans.iter().map(|s| s.content.len()).sum();

        let total_width = area.width as usize;

        // Build the line with proper spacing
        let mut line_spans: Vec<Span> = Vec::new();

        // Add left section
        line_spans.extend(left_spans);

        // Calculate padding for center
        let left_padding = if !state.center.is_empty() {
            let center_start = (total_width.saturating_sub(center_width)) / 2;
            center_start.saturating_sub(left_width)
        } else {
            0
        };

        if left_padding > 0 {
            line_spans.push(Span::raw(" ".repeat(left_padding)));
        }

        // Add center section
        line_spans.extend(center_spans);

        // Calculate padding for right
        let current_width = left_width + left_padding + center_width;
        let right_padding = total_width.saturating_sub(current_width + right_width);

        if right_padding > 0 {
            line_spans.push(Span::raw(" ".repeat(right_padding)));
        }

        // Add right section
        line_spans.extend(right_spans);

        let line = Line::from(line_spans);
        let paragraph = Paragraph::new(line).style(bg_style);

        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // StatusBarStyle tests

    #[test]
    fn test_style_default() {
        assert_eq!(StatusBarStyle::default(), StatusBarStyle::Default);
    }

    #[test]
    fn test_style_clone() {
        let style = StatusBarStyle::Error;
        let cloned = style;
        assert_eq!(cloned, StatusBarStyle::Error);
    }

    #[test]
    fn test_style_fg_color() {
        assert_eq!(StatusBarStyle::Default.fg_color(), Color::White);
        assert_eq!(StatusBarStyle::Info.fg_color(), Color::Blue);
        assert_eq!(StatusBarStyle::Success.fg_color(), Color::Green);
        assert_eq!(StatusBarStyle::Warning.fg_color(), Color::Yellow);
        assert_eq!(StatusBarStyle::Error.fg_color(), Color::Red);
        assert_eq!(StatusBarStyle::Muted.fg_color(), Color::DarkGray);
    }

    // StatusBarItem tests

    #[test]
    fn test_item_new() {
        let item = StatusBarItem::new("Test");
        assert_eq!(item.text(), "Test");
        assert_eq!(item.style(), StatusBarStyle::Default);
        assert!(item.has_separator());
    }

    #[test]
    fn test_item_with_style() {
        let item = StatusBarItem::new("Error").with_style(StatusBarStyle::Error);
        assert_eq!(item.style(), StatusBarStyle::Error);
    }

    #[test]
    fn test_item_with_separator() {
        let item = StatusBarItem::new("Last").with_separator(false);
        assert!(!item.has_separator());
    }

    #[test]
    fn test_item_set_text() {
        let mut item = StatusBarItem::new("Original");
        item.set_text("Updated");
        assert_eq!(item.text(), "Updated");
    }

    #[test]
    fn test_item_set_style() {
        let mut item = StatusBarItem::new("Test");
        item.set_style(StatusBarStyle::Success);
        assert_eq!(item.style(), StatusBarStyle::Success);
    }

    #[test]
    fn test_item_set_separator() {
        let mut item = StatusBarItem::new("Test");
        item.set_separator(false);
        assert!(!item.has_separator());
        item.set_separator(true);
        assert!(item.has_separator());
    }

    #[test]
    fn test_item_clone() {
        let item = StatusBarItem::new("Test").with_style(StatusBarStyle::Info);
        let cloned = item.clone();
        assert_eq!(cloned.text(), "Test");
        assert_eq!(cloned.style(), StatusBarStyle::Info);
    }

    #[test]
    fn test_item_eq() {
        let item1 = StatusBarItem::new("Test").with_style(StatusBarStyle::Info);
        let item2 = StatusBarItem::new("Test").with_style(StatusBarStyle::Info);
        let item3 = StatusBarItem::new("Different");
        assert_eq!(item1, item2);
        assert_ne!(item1, item3);
    }

    // StatusBarState tests

    #[test]
    fn test_state_new() {
        let state = StatusBarState::new();
        assert!(state.left().is_empty());
        assert!(state.center().is_empty());
        assert!(state.right().is_empty());
        assert_eq!(state.separator(), " | ");
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_state_default() {
        let state = StatusBarState::default();
        assert!(state.is_empty());
    }

    #[test]
    fn test_state_with_separator() {
        let state = StatusBarState::with_separator(" :: ");
        assert_eq!(state.separator(), " :: ");
    }

    #[test]
    fn test_state_set_left() {
        let mut state = StatusBarState::new();
        state.set_left(vec![StatusBarItem::new("A"), StatusBarItem::new("B")]);
        assert_eq!(state.left().len(), 2);
    }

    #[test]
    fn test_state_set_center() {
        let mut state = StatusBarState::new();
        state.set_center(vec![StatusBarItem::new("Center")]);
        assert_eq!(state.center().len(), 1);
    }

    #[test]
    fn test_state_set_right() {
        let mut state = StatusBarState::new();
        state.set_right(vec![StatusBarItem::new("Right")]);
        assert_eq!(state.right().len(), 1);
    }

    #[test]
    fn test_state_push_left() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("A"));
        state.push_left(StatusBarItem::new("B"));
        assert_eq!(state.left().len(), 2);
        assert_eq!(state.left()[0].text(), "A");
        assert_eq!(state.left()[1].text(), "B");
    }

    #[test]
    fn test_state_push_center() {
        let mut state = StatusBarState::new();
        state.push_center(StatusBarItem::new("Center"));
        assert_eq!(state.center().len(), 1);
    }

    #[test]
    fn test_state_push_right() {
        let mut state = StatusBarState::new();
        state.push_right(StatusBarItem::new("Right"));
        assert_eq!(state.right().len(), 1);
    }

    #[test]
    fn test_state_clear() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("L"));
        state.push_center(StatusBarItem::new("C"));
        state.push_right(StatusBarItem::new("R"));
        assert_eq!(state.len(), 3);

        state.clear();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_state_set_separator() {
        let mut state = StatusBarState::new();
        state.set_separator(" - ");
        assert_eq!(state.separator(), " - ");
    }

    #[test]
    fn test_state_background() {
        let mut state = StatusBarState::new();
        assert_eq!(state.background(), Color::DarkGray);

        state.set_background(Color::Blue);
        assert_eq!(state.background(), Color::Blue);
    }

    #[test]
    fn test_state_is_empty() {
        let mut state = StatusBarState::new();
        assert!(state.is_empty());

        state.push_left(StatusBarItem::new("Test"));
        assert!(!state.is_empty());
    }

    #[test]
    fn test_state_len() {
        let mut state = StatusBarState::new();
        assert_eq!(state.len(), 0);

        state.push_left(StatusBarItem::new("L1"));
        state.push_left(StatusBarItem::new("L2"));
        state.push_center(StatusBarItem::new("C1"));
        state.push_right(StatusBarItem::new("R1"));
        assert_eq!(state.len(), 4);
    }

    #[test]
    fn test_state_clone() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("Test"));
        state.set_separator(" :: ");

        let cloned = state.clone();
        assert_eq!(cloned.left().len(), 1);
        assert_eq!(cloned.separator(), " :: ");
    }

    // StatusBar component tests

    #[test]
    fn test_init() {
        let state = StatusBar::init();
        assert!(state.is_empty());
    }

    #[test]
    fn test_set_left_items() {
        let mut state = StatusBarState::new();
        let items = vec![StatusBarItem::new("A"), StatusBarItem::new("B")];

        StatusBar::update(&mut state, StatusBarMessage::SetLeftItems(items));
        assert_eq!(state.left().len(), 2);
    }

    #[test]
    fn test_set_center_items() {
        let mut state = StatusBarState::new();
        let items = vec![StatusBarItem::new("Center")];

        StatusBar::update(&mut state, StatusBarMessage::SetCenterItems(items));
        assert_eq!(state.center().len(), 1);
    }

    #[test]
    fn test_set_right_items() {
        let mut state = StatusBarState::new();
        let items = vec![StatusBarItem::new("Right")];

        StatusBar::update(&mut state, StatusBarMessage::SetRightItems(items));
        assert_eq!(state.right().len(), 1);
    }

    #[test]
    fn test_clear_message() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("L"));
        state.push_center(StatusBarItem::new("C"));
        state.push_right(StatusBarItem::new("R"));

        StatusBar::update(&mut state, StatusBarMessage::Clear);
        assert!(state.is_empty());
    }

    #[test]
    fn test_clear_left_message() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("L"));
        state.push_center(StatusBarItem::new("C"));

        StatusBar::update(&mut state, StatusBarMessage::ClearLeft);
        assert!(state.left().is_empty());
        assert_eq!(state.center().len(), 1);
    }

    #[test]
    fn test_clear_center_message() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("L"));
        state.push_center(StatusBarItem::new("C"));

        StatusBar::update(&mut state, StatusBarMessage::ClearCenter);
        assert_eq!(state.left().len(), 1);
        assert!(state.center().is_empty());
    }

    #[test]
    fn test_clear_right_message() {
        let mut state = StatusBarState::new();
        state.push_right(StatusBarItem::new("R"));
        state.push_center(StatusBarItem::new("C"));

        StatusBar::update(&mut state, StatusBarMessage::ClearRight);
        assert!(state.right().is_empty());
        assert_eq!(state.center().len(), 1);
    }

    #[test]
    fn test_update_returns_none() {
        let mut state = StatusBarState::new();
        let output = StatusBar::update(&mut state, StatusBarMessage::Clear);
        assert!(output.is_none());
    }

    // View tests

    #[test]
    fn test_view_empty() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = StatusBarState::new();

        let backend = CaptureBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                StatusBar::view(&state, frame, frame.area());
            })
            .unwrap();

        // Should render without panic
        let output = terminal.backend().to_string();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_view_left_only() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("LEFT"));

        let backend = CaptureBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                StatusBar::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("LEFT"));
    }

    #[test]
    fn test_view_right_only() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = StatusBarState::new();
        state.push_right(StatusBarItem::new("RIGHT"));

        let backend = CaptureBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                StatusBar::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("RIGHT"));
    }

    #[test]
    fn test_view_center_only() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = StatusBarState::new();
        state.push_center(StatusBarItem::new("CENTER"));

        let backend = CaptureBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                StatusBar::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("CENTER"));
    }

    #[test]
    fn test_view_all_sections() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("LEFT"));
        state.push_center(StatusBarItem::new("CENTER"));
        state.push_right(StatusBarItem::new("RIGHT"));

        let backend = CaptureBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                StatusBar::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("LEFT"));
        assert!(output.contains("CENTER"));
        assert!(output.contains("RIGHT"));
    }

    #[test]
    fn test_view_with_separator() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("A"));
        state.push_left(StatusBarItem::new("B"));

        let backend = CaptureBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                StatusBar::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("A"));
        assert!(output.contains("|"));
        assert!(output.contains("B"));
    }

    #[test]
    fn test_view_custom_separator() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = StatusBarState::with_separator(" :: ");
        state.push_left(StatusBarItem::new("A"));
        state.push_left(StatusBarItem::new("B"));

        let backend = CaptureBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                StatusBar::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("::"));
    }

    #[test]
    fn test_view_no_separator_on_last_item() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("A").with_separator(false));
        state.push_left(StatusBarItem::new("B"));

        let backend = CaptureBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                StatusBar::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("AB")); // No separator between A and B
    }

    #[test]
    fn test_view_styled_items() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("INFO").with_style(StatusBarStyle::Info));
        state.push_left(StatusBarItem::new("ERROR").with_style(StatusBarStyle::Error));

        let backend = CaptureBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                StatusBar::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("INFO"));
        assert!(output.contains("ERROR"));
    }

    // Integration tests

    #[test]
    fn test_typical_editor_status_bar() {
        let mut state = StatusBarState::new();

        // Left: mode indicator
        state.push_left(StatusBarItem::new("NORMAL").with_style(StatusBarStyle::Info));

        // Center: filename
        state.push_center(StatusBarItem::new("main.rs"));
        state.push_center(StatusBarItem::new("[+]").with_style(StatusBarStyle::Warning));

        // Right: position info
        state.push_right(StatusBarItem::new("UTF-8").with_style(StatusBarStyle::Muted));
        state.push_right(StatusBarItem::new("Ln 42, Col 8"));

        assert_eq!(state.left().len(), 1);
        assert_eq!(state.center().len(), 2);
        assert_eq!(state.right().len(), 2);
        assert_eq!(state.len(), 5);
    }

    #[test]
    fn test_update_mode_indicator() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("NORMAL").with_style(StatusBarStyle::Info));

        // Simulate mode change
        StatusBar::update(
            &mut state,
            StatusBarMessage::SetLeftItems(vec![
                StatusBarItem::new("INSERT").with_style(StatusBarStyle::Success)
            ]),
        );

        assert_eq!(state.left().len(), 1);
        assert_eq!(state.left()[0].text(), "INSERT");
        assert_eq!(state.left()[0].style(), StatusBarStyle::Success);
    }

    #[test]
    fn test_render_section_empty() {
        let spans = StatusBar::render_section(&[], " | ");
        assert!(spans.is_empty());
    }

    #[test]
    fn test_render_section_single_item() {
        let items = vec![StatusBarItem::new("Test")];
        let spans = StatusBar::render_section(&items, " | ");
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn test_render_section_multiple_items() {
        let items = vec![StatusBarItem::new("A"), StatusBarItem::new("B")];
        let spans = StatusBar::render_section(&items, " | ");
        // A + separator + B = 3 spans
        assert_eq!(spans.len(), 3);
    }
}
