//! A status bar component for displaying application state.
//!
//! `StatusBar` provides a horizontal bar typically displayed at the bottom of the
//! screen showing application status, mode indicators, and other information.
//!
//! # Features
//!
//! - **Static text**: Simple text labels
//! - **Elapsed time**: Auto-updating time display (requires periodic Tick messages)
//! - **Counters**: Numeric counters that can be incremented/set
//! - **Heartbeat**: Animated activity indicator
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
//!
//! # Dynamic Content Example
//!
//! ```rust
//! use envision::component::{StatusBar, StatusBarMessage, StatusBarState, StatusBarItem, Section, Component};
//!
//! let mut state = StatusBarState::new();
//!
//! // Add an elapsed time display
//! state.push_left(StatusBarItem::elapsed_time());
//!
//! // Add a counter
//! state.push_right(StatusBarItem::counter().with_label("Items"));
//!
//! // Add a heartbeat
//! state.push_right(StatusBarItem::heartbeat());
//!
//! // Update with tick (call periodically, e.g., every 100ms)
//! StatusBar::update(&mut state, StatusBarMessage::Tick(100));
//!
//! // Increment the counter
//! StatusBar::update(&mut state, StatusBarMessage::IncrementCounter {
//!     section: Section::Right,
//!     index: 0,
//! });
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::Component;

/// Section of the status bar for addressing items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    /// Left section.
    Left,
    /// Center section.
    Center,
    /// Right section.
    Right,
}

/// Content type for status bar items.
///
/// Items can display static text or dynamic content that updates over time.
#[derive(Clone, Debug, PartialEq)]
pub enum StatusBarItemContent {
    /// Static text content.
    Static(String),
    /// Elapsed time display.
    ///
    /// Shows time elapsed since the timer was started. Format depends on
    /// `long_format`: short format is "MM:SS", long format is "HH:MM:SS".
    ElapsedTime {
        /// Accumulated elapsed time in milliseconds.
        elapsed_ms: u64,
        /// Whether the timer is currently running.
        running: bool,
        /// Whether to use long format (HH:MM:SS vs MM:SS).
        long_format: bool,
    },
    /// Numeric counter display.
    ///
    /// Shows a counter value with an optional label.
    Counter {
        /// Current counter value.
        value: u64,
        /// Optional label (displayed before value).
        label: Option<String>,
    },
    /// Animated heartbeat indicator.
    ///
    /// Shows an animated indicator to show activity.
    Heartbeat {
        /// Whether the heartbeat is active.
        active: bool,
        /// Current animation frame.
        frame: usize,
    },
}

impl StatusBarItemContent {
    /// Creates static text content.
    pub fn static_text(text: impl Into<String>) -> Self {
        Self::Static(text.into())
    }

    /// Creates an elapsed time display.
    pub fn elapsed_time() -> Self {
        Self::ElapsedTime {
            elapsed_ms: 0,
            running: false,
            long_format: false,
        }
    }

    /// Creates a counter display.
    pub fn counter() -> Self {
        Self::Counter {
            value: 0,
            label: None,
        }
    }

    /// Creates a heartbeat indicator.
    pub fn heartbeat() -> Self {
        Self::Heartbeat {
            active: false,
            frame: 0,
        }
    }

    /// Returns the display text for this content.
    fn display_text(&self) -> String {
        match self {
            Self::Static(text) => text.clone(),
            Self::ElapsedTime {
                elapsed_ms,
                long_format,
                ..
            } => {
                let total_seconds = elapsed_ms / 1000;
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;

                if *long_format || hours > 0 {
                    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                } else {
                    format!("{:02}:{:02}", minutes, seconds)
                }
            }
            Self::Counter { value, label } => {
                if let Some(label) = label {
                    format!("{}: {}", label, value)
                } else {
                    value.to_string()
                }
            }
            Self::Heartbeat { active, frame } => {
                const FRAMES: [&str; 4] = ["♡", "♥", "♥", "♡"];
                if *active {
                    FRAMES[*frame % FRAMES.len()].to_string()
                } else {
                    "♡".to_string()
                }
            }
        }
    }

    /// Returns true if this is dynamic content that needs ticking.
    fn is_dynamic(&self) -> bool {
        !matches!(self, Self::Static(_))
    }
}

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
#[derive(Clone, Debug, PartialEq)]
pub struct StatusBarItem {
    /// The content of the item.
    content: StatusBarItemContent,
    /// The style of the item.
    style: StatusBarStyle,
    /// Whether to show a separator after this item.
    separator: bool,
}

impl StatusBarItem {
    /// Creates a new status bar item with static text content.
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
            content: StatusBarItemContent::Static(text.into()),
            style: StatusBarStyle::Default,
            separator: true,
        }
    }

    /// Creates a new status bar item with an elapsed time display.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarItem;
    ///
    /// let item = StatusBarItem::elapsed_time();
    /// assert_eq!(item.text(), "00:00");
    /// ```
    pub fn elapsed_time() -> Self {
        Self {
            content: StatusBarItemContent::elapsed_time(),
            style: StatusBarStyle::Default,
            separator: true,
        }
    }

    /// Creates an elapsed time display with long format (HH:MM:SS).
    pub fn elapsed_time_long() -> Self {
        Self {
            content: StatusBarItemContent::ElapsedTime {
                elapsed_ms: 0,
                running: false,
                long_format: true,
            },
            style: StatusBarStyle::Default,
            separator: true,
        }
    }

    /// Creates a new status bar item with a counter display.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarItem;
    ///
    /// let item = StatusBarItem::counter().with_label("Items");
    /// ```
    pub fn counter() -> Self {
        Self {
            content: StatusBarItemContent::counter(),
            style: StatusBarStyle::Default,
            separator: true,
        }
    }

    /// Creates a new status bar item with a heartbeat indicator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarItem;
    ///
    /// let item = StatusBarItem::heartbeat();
    /// ```
    pub fn heartbeat() -> Self {
        Self {
            content: StatusBarItemContent::heartbeat(),
            style: StatusBarStyle::Default,
            separator: true,
        }
    }

    /// Sets the label for counter items.
    ///
    /// This only has an effect on Counter content types.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        if let StatusBarItemContent::Counter {
            value,
            label: ref mut lbl,
        } = self.content
        {
            *lbl = Some(label.into());
            self.content = StatusBarItemContent::Counter {
                value,
                label: lbl.clone(),
            };
        }
        self
    }

    /// Sets long format for elapsed time items.
    ///
    /// This only has an effect on ElapsedTime content types.
    pub fn with_long_format(mut self, long: bool) -> Self {
        if let StatusBarItemContent::ElapsedTime {
            elapsed_ms,
            running,
            ..
        } = self.content
        {
            self.content = StatusBarItemContent::ElapsedTime {
                elapsed_ms,
                running,
                long_format: long,
            };
        }
        self
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

    /// Returns the display text for this item.
    pub fn text(&self) -> String {
        self.content.display_text()
    }

    /// Returns the content.
    pub fn content(&self) -> &StatusBarItemContent {
        &self.content
    }

    /// Returns a mutable reference to the content.
    pub fn content_mut(&mut self) -> &mut StatusBarItemContent {
        &mut self.content
    }

    /// Sets the text content (converts to static content).
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.content = StatusBarItemContent::Static(text.into());
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

    /// Returns true if this item has dynamic content.
    pub fn is_dynamic(&self) -> bool {
        self.content.is_dynamic()
    }

    /// Processes a tick for dynamic content.
    ///
    /// Returns true if the content was updated.
    fn tick(&mut self, delta_ms: u64) -> bool {
        match &mut self.content {
            StatusBarItemContent::ElapsedTime {
                elapsed_ms,
                running,
                ..
            } => {
                if *running {
                    *elapsed_ms += delta_ms;
                    true
                } else {
                    false
                }
            }
            StatusBarItemContent::Heartbeat { active, frame } => {
                if *active {
                    *frame = (*frame + 1) % 4;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
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

    // === Dynamic content messages ===
    /// Update elapsed time for all running timers.
    ///
    /// The parameter is the time delta in milliseconds since the last tick.
    Tick(u64),

    /// Start an elapsed time timer.
    StartTimer {
        /// Which section contains the timer.
        section: Section,
        /// Index of the item in the section.
        index: usize,
    },

    /// Stop an elapsed time timer.
    StopTimer {
        /// Which section contains the timer.
        section: Section,
        /// Index of the item in the section.
        index: usize,
    },

    /// Reset an elapsed time timer to zero.
    ResetTimer {
        /// Which section contains the timer.
        section: Section,
        /// Index of the item in the section.
        index: usize,
    },

    /// Increment a counter by 1.
    IncrementCounter {
        /// Which section contains the counter.
        section: Section,
        /// Index of the item in the section.
        index: usize,
    },

    /// Decrement a counter by 1 (won't go below 0).
    DecrementCounter {
        /// Which section contains the counter.
        section: Section,
        /// Index of the item in the section.
        index: usize,
    },

    /// Set a counter to a specific value.
    SetCounter {
        /// Which section contains the counter.
        section: Section,
        /// Index of the item in the section.
        index: usize,
        /// The value to set.
        value: u64,
    },

    /// Activate a heartbeat indicator.
    ActivateHeartbeat {
        /// Which section contains the heartbeat.
        section: Section,
        /// Index of the item in the section.
        index: usize,
    },

    /// Deactivate a heartbeat indicator.
    DeactivateHeartbeat {
        /// Which section contains the heartbeat.
        section: Section,
        /// Index of the item in the section.
        index: usize,
    },

    /// Pulse a heartbeat once (activate, advance frame, then deactivate).
    PulseHeartbeat {
        /// Which section contains the heartbeat.
        section: Section,
        /// Index of the item in the section.
        index: usize,
    },
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

    /// Returns the items in the specified section.
    pub fn section(&self, section: Section) -> &[StatusBarItem] {
        match section {
            Section::Left => &self.left,
            Section::Center => &self.center,
            Section::Right => &self.right,
        }
    }

    /// Returns a mutable reference to the items in the specified section.
    pub fn section_mut(&mut self, section: Section) -> &mut Vec<StatusBarItem> {
        match section {
            Section::Left => &mut self.left,
            Section::Center => &mut self.center,
            Section::Right => &mut self.right,
        }
    }

    /// Returns a mutable reference to an item by section and index.
    pub fn get_item_mut(&mut self, section: Section, index: usize) -> Option<&mut StatusBarItem> {
        self.section_mut(section).get_mut(index)
    }

    /// Processes a tick for all dynamic items.
    fn tick_all(&mut self, delta_ms: u64) {
        for item in &mut self.left {
            item.tick(delta_ms);
        }
        for item in &mut self.center {
            item.tick(delta_ms);
        }
        for item in &mut self.right {
            item.tick(delta_ms);
        }
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
    fn render_section(items: &[StatusBarItem], separator: &str) -> Vec<Span<'static>> {
        let mut spans = Vec::new();

        for (idx, item) in items.iter().enumerate() {
            let style = Style::default().fg(item.style.fg_color());
            spans.push(Span::styled(item.text(), style));

            // Add separator if not last item and item has separator enabled
            if idx < items.len() - 1 && item.has_separator() {
                spans.push(Span::styled(
                    separator.to_string(),
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

            // Dynamic content messages
            StatusBarMessage::Tick(delta_ms) => {
                state.tick_all(delta_ms);
            }

            StatusBarMessage::StartTimer { section, index } => {
                if let Some(item) = state.get_item_mut(section, index) {
                    if let StatusBarItemContent::ElapsedTime { running, .. } = &mut item.content {
                        *running = true;
                    }
                }
            }

            StatusBarMessage::StopTimer { section, index } => {
                if let Some(item) = state.get_item_mut(section, index) {
                    if let StatusBarItemContent::ElapsedTime { running, .. } = &mut item.content {
                        *running = false;
                    }
                }
            }

            StatusBarMessage::ResetTimer { section, index } => {
                if let Some(item) = state.get_item_mut(section, index) {
                    if let StatusBarItemContent::ElapsedTime {
                        elapsed_ms,
                        running,
                        ..
                    } = &mut item.content
                    {
                        *elapsed_ms = 0;
                        *running = false;
                    }
                }
            }

            StatusBarMessage::IncrementCounter { section, index } => {
                if let Some(item) = state.get_item_mut(section, index) {
                    if let StatusBarItemContent::Counter { value, .. } = &mut item.content {
                        *value = value.saturating_add(1);
                    }
                }
            }

            StatusBarMessage::DecrementCounter { section, index } => {
                if let Some(item) = state.get_item_mut(section, index) {
                    if let StatusBarItemContent::Counter { value, .. } = &mut item.content {
                        *value = value.saturating_sub(1);
                    }
                }
            }

            StatusBarMessage::SetCounter {
                section,
                index,
                value: new_value,
            } => {
                if let Some(item) = state.get_item_mut(section, index) {
                    if let StatusBarItemContent::Counter { value, .. } = &mut item.content {
                        *value = new_value;
                    }
                }
            }

            StatusBarMessage::ActivateHeartbeat { section, index } => {
                if let Some(item) = state.get_item_mut(section, index) {
                    if let StatusBarItemContent::Heartbeat { active, .. } = &mut item.content {
                        *active = true;
                    }
                }
            }

            StatusBarMessage::DeactivateHeartbeat { section, index } => {
                if let Some(item) = state.get_item_mut(section, index) {
                    if let StatusBarItemContent::Heartbeat { active, .. } = &mut item.content {
                        *active = false;
                    }
                }
            }

            StatusBarMessage::PulseHeartbeat { section, index } => {
                if let Some(item) = state.get_item_mut(section, index) {
                    if let StatusBarItemContent::Heartbeat { active, frame } = &mut item.content {
                        *active = true;
                        *frame = (*frame + 1) % 4;
                    }
                }
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

    // ========================================
    // Dynamic Content Tests
    // ========================================

    // StatusBarItemContent tests

    #[test]
    fn test_content_static_text() {
        let content = StatusBarItemContent::static_text("Hello");
        assert_eq!(content.display_text(), "Hello");
        assert!(!content.is_dynamic());
    }

    #[test]
    fn test_content_elapsed_time_default() {
        let content = StatusBarItemContent::elapsed_time();
        assert_eq!(content.display_text(), "00:00");
        assert!(content.is_dynamic());
    }

    #[test]
    fn test_content_elapsed_time_formatting() {
        let content = StatusBarItemContent::ElapsedTime {
            elapsed_ms: 65_000, // 1 min 5 sec
            running: false,
            long_format: false,
        };
        assert_eq!(content.display_text(), "01:05");
    }

    #[test]
    fn test_content_elapsed_time_long_format() {
        let content = StatusBarItemContent::ElapsedTime {
            elapsed_ms: 3_665_000, // 1 hr 1 min 5 sec
            running: false,
            long_format: true,
        };
        assert_eq!(content.display_text(), "01:01:05");
    }

    #[test]
    fn test_content_elapsed_time_auto_long_format() {
        // When hours > 0, should auto-switch to long format
        let content = StatusBarItemContent::ElapsedTime {
            elapsed_ms: 3_665_000, // 1 hr 1 min 5 sec
            running: false,
            long_format: false, // Not explicit, but should show hours
        };
        assert_eq!(content.display_text(), "01:01:05");
    }

    #[test]
    fn test_content_counter_default() {
        let content = StatusBarItemContent::counter();
        assert_eq!(content.display_text(), "0");
    }

    #[test]
    fn test_content_counter_with_value() {
        let content = StatusBarItemContent::Counter {
            value: 42,
            label: None,
        };
        assert_eq!(content.display_text(), "42");
    }

    #[test]
    fn test_content_counter_with_label() {
        let content = StatusBarItemContent::Counter {
            value: 5,
            label: Some("Items".to_string()),
        };
        assert_eq!(content.display_text(), "Items: 5");
    }

    #[test]
    fn test_content_heartbeat_inactive() {
        let content = StatusBarItemContent::Heartbeat {
            active: false,
            frame: 0,
        };
        assert_eq!(content.display_text(), "♡");
    }

    #[test]
    fn test_content_heartbeat_active_frames() {
        // Frame 0
        let content0 = StatusBarItemContent::Heartbeat {
            active: true,
            frame: 0,
        };
        assert_eq!(content0.display_text(), "♡");

        // Frame 1
        let content1 = StatusBarItemContent::Heartbeat {
            active: true,
            frame: 1,
        };
        assert_eq!(content1.display_text(), "♥");

        // Frame 2
        let content2 = StatusBarItemContent::Heartbeat {
            active: true,
            frame: 2,
        };
        assert_eq!(content2.display_text(), "♥");

        // Frame 3
        let content3 = StatusBarItemContent::Heartbeat {
            active: true,
            frame: 3,
        };
        assert_eq!(content3.display_text(), "♡");
    }

    // StatusBarItem factory method tests

    #[test]
    fn test_item_elapsed_time() {
        let item = StatusBarItem::elapsed_time();
        assert_eq!(item.text(), "00:00");
        assert!(item.is_dynamic());
    }

    #[test]
    fn test_item_elapsed_time_long() {
        let item = StatusBarItem::elapsed_time_long();
        assert_eq!(item.text(), "00:00:00");
    }

    #[test]
    fn test_item_counter() {
        let item = StatusBarItem::counter();
        assert_eq!(item.text(), "0");
    }

    #[test]
    fn test_item_counter_with_label() {
        let item = StatusBarItem::counter().with_label("Count");
        assert_eq!(item.text(), "Count: 0");
    }

    #[test]
    fn test_item_heartbeat() {
        let item = StatusBarItem::heartbeat();
        assert_eq!(item.text(), "♡");
    }

    #[test]
    fn test_item_with_long_format() {
        let item = StatusBarItem::elapsed_time().with_long_format(true);
        assert_eq!(item.text(), "00:00:00");
    }

    // Section tests

    #[test]
    fn test_section_enum() {
        assert_ne!(Section::Left, Section::Center);
        assert_ne!(Section::Center, Section::Right);
        assert_ne!(Section::Left, Section::Right);
    }

    #[test]
    fn test_state_section() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("L"));
        state.push_center(StatusBarItem::new("C"));
        state.push_right(StatusBarItem::new("R"));

        assert_eq!(state.section(Section::Left).len(), 1);
        assert_eq!(state.section(Section::Center).len(), 1);
        assert_eq!(state.section(Section::Right).len(), 1);
    }

    #[test]
    fn test_state_section_mut() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("L"));

        state.section_mut(Section::Left).push(StatusBarItem::new("L2"));
        assert_eq!(state.section(Section::Left).len(), 2);
    }

    #[test]
    fn test_state_get_item_mut() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::new("Test"));

        let item = state.get_item_mut(Section::Left, 0);
        assert!(item.is_some());
        item.unwrap().set_text("Updated");
        assert_eq!(state.left()[0].text(), "Updated");
    }

    #[test]
    fn test_state_get_item_mut_invalid_index() {
        let mut state = StatusBarState::new();
        assert!(state.get_item_mut(Section::Left, 0).is_none());
    }

    // Timer message tests

    #[test]
    fn test_tick_message() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::elapsed_time());

        // Start the timer
        StatusBar::update(
            &mut state,
            StatusBarMessage::StartTimer {
                section: Section::Left,
                index: 0,
            },
        );

        // Tick 5 seconds
        StatusBar::update(&mut state, StatusBarMessage::Tick(5000));
        assert_eq!(state.left()[0].text(), "00:05");

        // Tick another 65 seconds
        StatusBar::update(&mut state, StatusBarMessage::Tick(65000));
        assert_eq!(state.left()[0].text(), "01:10");
    }

    #[test]
    fn test_start_timer() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::elapsed_time());

        StatusBar::update(
            &mut state,
            StatusBarMessage::StartTimer {
                section: Section::Left,
                index: 0,
            },
        );

        if let StatusBarItemContent::ElapsedTime { running, .. } = state.left()[0].content() {
            assert!(*running);
        } else {
            panic!("Expected ElapsedTime content");
        }
    }

    #[test]
    fn test_stop_timer() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::elapsed_time());

        // Start then stop
        StatusBar::update(
            &mut state,
            StatusBarMessage::StartTimer {
                section: Section::Left,
                index: 0,
            },
        );
        StatusBar::update(
            &mut state,
            StatusBarMessage::StopTimer {
                section: Section::Left,
                index: 0,
            },
        );

        if let StatusBarItemContent::ElapsedTime { running, .. } = state.left()[0].content() {
            assert!(!*running);
        } else {
            panic!("Expected ElapsedTime content");
        }
    }

    #[test]
    fn test_reset_timer() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::elapsed_time());

        // Start, tick, then reset
        StatusBar::update(
            &mut state,
            StatusBarMessage::StartTimer {
                section: Section::Left,
                index: 0,
            },
        );
        StatusBar::update(&mut state, StatusBarMessage::Tick(10000));
        assert_eq!(state.left()[0].text(), "00:10");

        StatusBar::update(
            &mut state,
            StatusBarMessage::ResetTimer {
                section: Section::Left,
                index: 0,
            },
        );
        assert_eq!(state.left()[0].text(), "00:00");
    }

    #[test]
    fn test_timer_stopped_no_tick() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::elapsed_time());

        // Timer not started, ticking should not change time
        StatusBar::update(&mut state, StatusBarMessage::Tick(5000));
        assert_eq!(state.left()[0].text(), "00:00");
    }

    // Counter message tests

    #[test]
    fn test_increment_counter() {
        let mut state = StatusBarState::new();
        state.push_right(StatusBarItem::counter());

        StatusBar::update(
            &mut state,
            StatusBarMessage::IncrementCounter {
                section: Section::Right,
                index: 0,
            },
        );
        assert_eq!(state.right()[0].text(), "1");

        StatusBar::update(
            &mut state,
            StatusBarMessage::IncrementCounter {
                section: Section::Right,
                index: 0,
            },
        );
        assert_eq!(state.right()[0].text(), "2");
    }

    #[test]
    fn test_decrement_counter() {
        let mut state = StatusBarState::new();
        state.push_right(StatusBarItem::counter());

        // Set to 5, then decrement
        StatusBar::update(
            &mut state,
            StatusBarMessage::SetCounter {
                section: Section::Right,
                index: 0,
                value: 5,
            },
        );
        StatusBar::update(
            &mut state,
            StatusBarMessage::DecrementCounter {
                section: Section::Right,
                index: 0,
            },
        );
        assert_eq!(state.right()[0].text(), "4");
    }

    #[test]
    fn test_decrement_counter_no_underflow() {
        let mut state = StatusBarState::new();
        state.push_right(StatusBarItem::counter());

        // Try to decrement below 0
        StatusBar::update(
            &mut state,
            StatusBarMessage::DecrementCounter {
                section: Section::Right,
                index: 0,
            },
        );
        assert_eq!(state.right()[0].text(), "0");
    }

    #[test]
    fn test_set_counter() {
        let mut state = StatusBarState::new();
        state.push_right(StatusBarItem::counter().with_label("Items"));

        StatusBar::update(
            &mut state,
            StatusBarMessage::SetCounter {
                section: Section::Right,
                index: 0,
                value: 42,
            },
        );
        assert_eq!(state.right()[0].text(), "Items: 42");
    }

    // Heartbeat message tests

    #[test]
    fn test_activate_heartbeat() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::heartbeat());

        StatusBar::update(
            &mut state,
            StatusBarMessage::ActivateHeartbeat {
                section: Section::Left,
                index: 0,
            },
        );

        if let StatusBarItemContent::Heartbeat { active, .. } = state.left()[0].content() {
            assert!(*active);
        } else {
            panic!("Expected Heartbeat content");
        }
    }

    #[test]
    fn test_deactivate_heartbeat() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::heartbeat());

        StatusBar::update(
            &mut state,
            StatusBarMessage::ActivateHeartbeat {
                section: Section::Left,
                index: 0,
            },
        );
        StatusBar::update(
            &mut state,
            StatusBarMessage::DeactivateHeartbeat {
                section: Section::Left,
                index: 0,
            },
        );

        if let StatusBarItemContent::Heartbeat { active, .. } = state.left()[0].content() {
            assert!(!*active);
        } else {
            panic!("Expected Heartbeat content");
        }
    }

    #[test]
    fn test_pulse_heartbeat() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::heartbeat());

        // First pulse
        StatusBar::update(
            &mut state,
            StatusBarMessage::PulseHeartbeat {
                section: Section::Left,
                index: 0,
            },
        );

        if let StatusBarItemContent::Heartbeat { active, frame } = state.left()[0].content() {
            assert!(*active);
            assert_eq!(*frame, 1);
        } else {
            panic!("Expected Heartbeat content");
        }
    }

    #[test]
    fn test_heartbeat_tick() {
        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::heartbeat());

        // Activate and tick
        StatusBar::update(
            &mut state,
            StatusBarMessage::ActivateHeartbeat {
                section: Section::Left,
                index: 0,
            },
        );

        StatusBar::update(&mut state, StatusBarMessage::Tick(100));

        if let StatusBarItemContent::Heartbeat { frame, .. } = state.left()[0].content() {
            assert_eq!(*frame, 1);
        } else {
            panic!("Expected Heartbeat content");
        }
    }

    // View tests for dynamic content

    #[test]
    fn test_view_elapsed_time() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::elapsed_time());

        let backend = CaptureBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                StatusBar::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("00:00"));
    }

    #[test]
    fn test_view_counter_with_label() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = StatusBarState::new();
        state.push_right(StatusBarItem::counter().with_label("Files"));

        StatusBar::update(
            &mut state,
            StatusBarMessage::SetCounter {
                section: Section::Right,
                index: 0,
                value: 15,
            },
        );

        let backend = CaptureBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                StatusBar::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Files: 15"));
    }

    #[test]
    fn test_view_heartbeat() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = StatusBarState::new();
        state.push_left(StatusBarItem::heartbeat());

        let backend = CaptureBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                StatusBar::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("♡"));
    }

    // Integration tests for dynamic content

    #[test]
    fn test_media_player_status_bar() {
        let mut state = StatusBarState::new();

        // Left: elapsed time
        state.push_left(StatusBarItem::elapsed_time().with_style(StatusBarStyle::Info));

        // Center: file name
        state.push_center(StatusBarItem::new("song.mp3"));

        // Right: heartbeat for activity
        state.push_right(StatusBarItem::heartbeat());

        assert_eq!(state.len(), 3);
        assert!(state.left()[0].is_dynamic());
    }

    #[test]
    fn test_file_processor_status_bar() {
        let mut state = StatusBarState::new();

        // Left: timer for processing
        state.push_left(StatusBarItem::elapsed_time_long());

        // Center: file count
        state.push_center(StatusBarItem::counter().with_label("Processed"));

        // Right: remaining count
        state.push_right(StatusBarItem::counter().with_label("Remaining"));

        // Simulate processing
        StatusBar::update(
            &mut state,
            StatusBarMessage::StartTimer {
                section: Section::Left,
                index: 0,
            },
        );
        StatusBar::update(
            &mut state,
            StatusBarMessage::SetCounter {
                section: Section::Right,
                index: 0,
                value: 100,
            },
        );

        // Process one file
        StatusBar::update(
            &mut state,
            StatusBarMessage::IncrementCounter {
                section: Section::Center,
                index: 0,
            },
        );
        StatusBar::update(
            &mut state,
            StatusBarMessage::DecrementCounter {
                section: Section::Right,
                index: 0,
            },
        );

        assert_eq!(state.center()[0].text(), "Processed: 1");
        assert_eq!(state.right()[0].text(), "Remaining: 99");
    }
}
