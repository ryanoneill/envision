//! A status bar component for displaying application state.
//!
//! [`StatusBar`] provides a horizontal bar typically displayed at the bottom of the
//! screen showing application status, mode indicators, and other information. State
//! is stored in [`StatusBarState`] and updated via [`StatusBarMessage`].
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

mod item;
pub use item::*;

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{Component, RenderContext};
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

/// State for a StatusBar component.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
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
    /// Whether the component is disabled.
    disabled: bool,
}

impl Default for StatusBarState {
    /// Creates a default empty status bar with `" | "` separator and
    /// `DarkGray` background.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let state = StatusBarState::default();
    /// assert!(state.left().is_empty());
    /// assert!(state.center().is_empty());
    /// assert!(state.right().is_empty());
    /// assert_eq!(state.separator(), " | ");
    /// ```
    fn default() -> Self {
        Self {
            left: Vec::new(),
            center: Vec::new(),
            right: Vec::new(),
            separator: " | ".to_string(),
            background: Color::DarkGray,
            disabled: false,
        }
    }
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
        Self::default()
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
            ..Self::default()
        }
    }

    /// Returns the left section items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem};
    ///
    /// let mut state = StatusBarState::new();
    /// state.push_left(StatusBarItem::new("Mode"));
    /// assert_eq!(state.left().len(), 1);
    /// assert_eq!(state.left()[0].text(), "Mode");
    /// ```
    pub fn left(&self) -> &[StatusBarItem] {
        &self.left
    }

    /// Returns the center section items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem};
    ///
    /// let mut state = StatusBarState::new();
    /// state.push_center(StatusBarItem::new("main.rs"));
    /// assert_eq!(state.center().len(), 1);
    /// assert_eq!(state.center()[0].text(), "main.rs");
    /// ```
    pub fn center(&self) -> &[StatusBarItem] {
        &self.center
    }

    /// Returns the right section items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem};
    ///
    /// let mut state = StatusBarState::new();
    /// state.push_right(StatusBarItem::new("Ln 1"));
    /// assert_eq!(state.right().len(), 1);
    /// assert_eq!(state.right()[0].text(), "Ln 1");
    /// ```
    pub fn right(&self) -> &[StatusBarItem] {
        &self.right
    }

    /// Sets the left section items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem};
    ///
    /// let mut state = StatusBarState::new();
    /// state.set_left(vec![StatusBarItem::new("INSERT"), StatusBarItem::new("Git")]);
    /// assert_eq!(state.left().len(), 2);
    /// ```
    pub fn set_left(&mut self, items: Vec<StatusBarItem>) {
        self.left = items;
    }

    /// Sets the center section items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem};
    ///
    /// let mut state = StatusBarState::new();
    /// state.set_center(vec![StatusBarItem::new("main.rs")]);
    /// assert_eq!(state.center().len(), 1);
    /// ```
    pub fn set_center(&mut self, items: Vec<StatusBarItem>) {
        self.center = items;
    }

    /// Sets the right section items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem};
    ///
    /// let mut state = StatusBarState::new();
    /// state.set_right(vec![StatusBarItem::new("UTF-8"), StatusBarItem::new("Ln 1")]);
    /// assert_eq!(state.right().len(), 2);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem};
    ///
    /// let mut state = StatusBarState::new();
    /// state.push_left(StatusBarItem::new("Mode"));
    /// state.push_right(StatusBarItem::new("Ln 1"));
    /// state.clear();
    /// assert!(state.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.left.clear();
        self.center.clear();
        self.right.clear();
    }

    /// Returns the separator string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let state = StatusBarState::new();
    /// assert_eq!(state.separator(), " | "); // default separator
    /// ```
    pub fn separator(&self) -> &str {
        &self.separator
    }

    /// Sets the separator string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let mut state = StatusBarState::new();
    /// state.set_separator(" :: ");
    /// assert_eq!(state.separator(), " :: ");
    /// ```
    pub fn set_separator(&mut self, separator: impl Into<String>) {
        self.separator = separator.into();
    }

    /// Returns the background color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    /// use ratatui::style::Color;
    ///
    /// let state = StatusBarState::new();
    /// assert_eq!(state.background(), Color::DarkGray);
    /// ```
    pub fn background(&self) -> Color {
        self.background
    }

    /// Sets the background color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    /// use ratatui::style::Color;
    ///
    /// let mut state = StatusBarState::new();
    /// state.set_background(Color::Black);
    /// assert_eq!(state.background(), Color::Black);
    /// ```
    pub fn set_background(&mut self, color: Color) {
        self.background = color;
    }

    /// Returns true if the status bar is disabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let state = StatusBarState::new();
    /// assert!(!state.is_disabled());
    /// ```
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let mut state = StatusBarState::new();
    /// state.set_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Sets the disabled state using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let state = StatusBarState::new()
    ///     .with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns true if all sections are empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let state = StatusBarState::new();
    /// assert!(state.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.left.is_empty() && self.center.is_empty() && self.right.is_empty()
    }

    /// Returns the total number of items across all sections.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem};
    ///
    /// let mut state = StatusBarState::new();
    /// state.push_left(StatusBarItem::new("A"));
    /// state.push_center(StatusBarItem::new("B"));
    /// state.push_right(StatusBarItem::new("C"));
    /// assert_eq!(state.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.left.len() + self.center.len() + self.right.len()
    }

    /// Returns the items in the specified section.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem, Section};
    ///
    /// let mut state = StatusBarState::new();
    /// state.push_left(StatusBarItem::new("Mode"));
    /// assert_eq!(state.section(Section::Left).len(), 1);
    /// assert_eq!(state.section(Section::Right).len(), 0);
    /// ```
    pub fn section(&self, section: Section) -> &[StatusBarItem] {
        match section {
            Section::Left => &self.left,
            Section::Center => &self.center,
            Section::Right => &self.right,
        }
    }

    /// Returns a mutable reference to the items in the specified section.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem, Section};
    ///
    /// let mut state = StatusBarState::new();
    /// state.push_left(StatusBarItem::new("Mode"));
    /// assert_eq!(state.section_mut(Section::Left).len(), 1);
    /// ```
    pub fn section_mut(&mut self, section: Section) -> &mut Vec<StatusBarItem> {
        match section {
            Section::Left => &mut self.left,
            Section::Center => &mut self.center,
            Section::Right => &mut self.right,
        }
    }

    /// Returns a mutable reference to an item by section and index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarState, StatusBarItem, Section};
    ///
    /// let mut state = StatusBarState::new();
    /// state.push_left(StatusBarItem::new("Mode"));
    /// assert!(state.get_item_mut(Section::Left, 0).is_some());
    /// assert!(state.get_item_mut(Section::Left, 99).is_none());
    /// ```
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

    /// Truncates a list of spans to fit within `max_width` characters.
    /// Appends an ellipsis if truncation occurs.
    fn truncate_spans(spans: Vec<Span<'static>>, max_width: usize) -> Vec<Span<'static>> {
        if max_width == 0 {
            return Vec::new();
        }

        let total: usize = spans.iter().map(|s| s.content.len()).sum();
        if total <= max_width {
            return spans;
        }

        let mut result = Vec::new();
        let mut remaining = max_width.saturating_sub(1); // Reserve 1 for ellipsis

        for span in spans {
            let len = span.content.len();
            if remaining == 0 {
                break;
            }
            if len <= remaining {
                remaining -= len;
                result.push(span);
            } else {
                // Truncate this span
                let truncated: String = span.content.chars().take(remaining).collect();
                result.push(Span::styled(truncated, span.style));
                remaining = 0;
            }
        }

        // Add ellipsis with the style of the last span
        let ellipsis_style = result.last().map(|s| s.style).unwrap_or_default();
        result.push(Span::styled("…", ellipsis_style));

        result
    }
}

impl Component for StatusBar {
    type State = StatusBarState;
    type Message = StatusBarMessage;
    type Output = ();

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

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        // Render background
        let bg_style = Style::default().bg(state.background);

        // Calculate section widths
        let left_spans = Self::render_section(&state.left, &state.separator, ctx.theme);
        let center_spans = Self::render_section(&state.center, &state.separator, ctx.theme);
        let right_spans = Self::render_section(&state.right, &state.separator, ctx.theme);

        // Calculate the width of each section
        let left_width: usize = left_spans.iter().map(|s| s.content.len()).sum();
        let center_width: usize = center_spans.iter().map(|s| s.content.len()).sum();
        let right_width: usize = right_spans.iter().map(|s| s.content.len()).sum();

        let total_width = ctx.area.width as usize;

        // Determine how much space is available for center after left and right.
        // Priority: left (full), right (full), center (gets remainder).
        let available_for_center = total_width
            .saturating_sub(left_width)
            .saturating_sub(right_width);
        let effective_center_width = center_width.min(available_for_center);

        // Truncate center spans if they exceed available space
        let center_spans = Self::truncate_spans(center_spans, effective_center_width);

        // Build the line with proper spacing
        let mut line_spans: Vec<Span> = Vec::new();

        // Add left section
        line_spans.extend(left_spans);

        // Calculate padding for center
        let left_padding = if effective_center_width > 0 {
            let center_start = (total_width.saturating_sub(effective_center_width)) / 2;
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
        let current_width = left_width + left_padding + effective_center_width;
        let right_padding = total_width.saturating_sub(current_width + right_width);

        if right_padding > 0 {
            line_spans.push(Span::raw(" ".repeat(right_padding)));
        }

        // Add right section
        line_spans.extend(right_spans);

        let line = Line::from(line_spans);
        let paragraph = Paragraph::new(line).style(bg_style);

        let item_count = state.left.len() + state.center.len() + state.right.len();
        let annotation =
            crate::annotation::Annotation::new(crate::annotation::WidgetType::StatusBar)
                .with_id("status_bar")
                .with_meta("item_count", item_count.to_string());
        let annotated = crate::annotation::Annotate::new(paragraph, annotation);
        ctx.frame.render_widget(annotated, ctx.area);
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
