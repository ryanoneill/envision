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
use crate::theme::Theme;

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
    /// Returns the style for this status bar style variant.
    fn style(self, theme: &Theme) -> Style {
        match self {
            Self::Default => theme.normal_style(),
            Self::Info => theme.info_style(),
            Self::Success => theme.success_style(),
            Self::Warning => theme.warning_style(),
            Self::Error => theme.error_style(),
            Self::Muted => theme.disabled_style(),
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
    fn render_section(
        items: &[StatusBarItem],
        separator: &str,
        theme: &Theme,
    ) -> Vec<Span<'static>> {
        let mut spans = Vec::new();

        for (idx, item) in items.iter().enumerate() {
            let style = item.style.style(theme);
            spans.push(Span::styled(item.text(), style));

            // Add separator if not last item and item has separator enabled
            if idx < items.len() - 1 && item.has_separator() {
                spans.push(Span::styled(separator.to_string(), theme.disabled_style()));
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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Render background
        let bg_style = Style::default().bg(state.background);

        // Calculate section widths
        let left_spans = Self::render_section(&state.left, &state.separator, theme);
        let center_spans = Self::render_section(&state.center, &state.separator, theme);
        let right_spans = Self::render_section(&state.right, &state.separator, theme);

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
mod tests;
