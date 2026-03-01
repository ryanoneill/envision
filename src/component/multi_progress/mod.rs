//! A component for displaying multiple concurrent progress indicators.
//!
//! `MultiProgress` provides a scrollable list of progress bars for tracking
//! multiple concurrent operations, commonly used in file processing, downloads,
//! or batch operations.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{MultiProgress, MultiProgressState, MultiProgressMessage, Component};
//!
//! let mut state = MultiProgressState::new();
//!
//! // Add items
//! state.add("ch1", "Chapter 1");
//! state.add("ch2", "Chapter 2");
//!
//! // Update progress
//! MultiProgress::update(&mut state, MultiProgressMessage::SetProgress {
//!     id: "ch1".to_string(),
//!     progress: 0.5,
//! });
//!
//! // Complete an item
//! MultiProgress::update(&mut state, MultiProgressMessage::Complete("ch1".to_string()));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

use super::{Component, Focusable};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// Status of a progress item.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProgressItemStatus {
    /// Item is pending (not yet started).
    #[default]
    Pending,
    /// Item is actively being processed.
    Active,
    /// Item has completed successfully.
    Completed,
    /// Item has failed.
    Failed,
}

impl ProgressItemStatus {
    /// Returns the style for this status using the theme.
    pub fn style(&self, theme: &Theme) -> Style {
        match self {
            Self::Pending => theme.disabled_style(),
            Self::Active => theme.info_style(),
            Self::Completed => theme.success_style(),
            Self::Failed => theme.error_style(),
        }
    }

    /// Returns the symbol for this status.
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::Active => "●",
            Self::Completed => "✓",
            Self::Failed => "✗",
        }
    }
}

/// A single progress item.
#[derive(Clone, Debug)]
pub struct ProgressItem {
    /// Unique identifier.
    id: String,
    /// Display label.
    label: String,
    /// Progress from 0.0 to 1.0.
    progress: f32,
    /// Current status.
    status: ProgressItemStatus,
    /// Optional status message.
    message: Option<String>,
}

impl ProgressItem {
    /// Creates a new progress item.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            progress: 0.0,
            status: ProgressItemStatus::Pending,
            message: None,
        }
    }

    /// Returns the item ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the progress (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Returns the status.
    pub fn status(&self) -> ProgressItemStatus {
        self.status
    }

    /// Returns the optional message.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Returns the progress as a percentage (0-100).
    pub fn percentage(&self) -> u16 {
        (self.progress * 100.0).round() as u16
    }
}

/// Messages for the MultiProgress component.
#[derive(Clone, Debug, PartialEq)]
pub enum MultiProgressMessage {
    /// Add a new progress item.
    Add {
        /// Unique identifier.
        id: String,
        /// Display label.
        label: String,
    },
    /// Set progress for an item.
    SetProgress {
        /// Item identifier.
        id: String,
        /// Progress from 0.0 to 1.0.
        progress: f32,
    },
    /// Set status for an item.
    SetStatus {
        /// Item identifier.
        id: String,
        /// New status.
        status: ProgressItemStatus,
    },
    /// Set an optional message for an item.
    SetMessage {
        /// Item identifier.
        id: String,
        /// Optional message.
        message: Option<String>,
    },
    /// Mark an item as completed (sets progress to 1.0 and status to Completed).
    Complete(String),
    /// Mark an item as failed.
    Fail {
        /// Item identifier.
        id: String,
        /// Optional error message.
        message: Option<String>,
    },
    /// Remove an item.
    Remove(String),
    /// Clear all items.
    Clear,
    /// Scroll up.
    ScrollUp,
    /// Scroll down.
    ScrollDown,
    /// Scroll to top.
    ScrollToTop,
    /// Scroll to bottom.
    ScrollToBottom,
}

/// Output messages from MultiProgress.
#[derive(Clone, Debug, PartialEq)]
pub enum MultiProgressOutput {
    /// An item was added.
    Added(String),
    /// An item was completed.
    Completed(String),
    /// An item failed.
    Failed(String),
    /// An item was removed.
    Removed(String),
    /// All items were cleared.
    Cleared,
}

/// State for the MultiProgress component.
#[derive(Clone, Debug)]
pub struct MultiProgressState {
    /// All progress items.
    items: Vec<ProgressItem>,
    /// Maximum number of visible items.
    max_visible: usize,
    /// Scroll offset.
    scroll_offset: usize,
    /// Whether to auto-remove completed items.
    auto_remove_completed: bool,
    /// Whether the component is focused.
    focused: bool,
    /// Optional title.
    title: Option<String>,
    /// Whether to show percentages.
    show_percentages: bool,
}

impl Default for MultiProgressState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            max_visible: 8,
            scroll_offset: 0,
            auto_remove_completed: false,
            focused: false,
            title: None,
            show_percentages: true,
        }
    }
}

impl MultiProgressState {
    /// Creates a new empty MultiProgress state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of visible items.
    pub fn with_max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    /// Sets whether to auto-remove completed items.
    pub fn with_auto_remove(mut self, auto_remove: bool) -> Self {
        self.auto_remove_completed = auto_remove;
        self
    }

    /// Sets the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether to show percentages.
    pub fn with_percentages(mut self, show: bool) -> Self {
        self.show_percentages = show;
        self
    }

    /// Adds a new progress item.
    ///
    /// Returns true if the item was added (id was unique).
    pub fn add(&mut self, id: impl Into<String>, label: impl Into<String>) -> bool {
        let id = id.into();
        if self.items.iter().any(|i| i.id == id) {
            return false;
        }
        self.items.push(ProgressItem::new(id, label));
        true
    }

    /// Returns all items.
    pub fn items(&self) -> &[ProgressItem] {
        &self.items
    }

    /// Returns the number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if there are no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of completed items.
    pub fn completed_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.status == ProgressItemStatus::Completed)
            .count()
    }

    /// Returns the number of failed items.
    pub fn failed_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.status == ProgressItemStatus::Failed)
            .count()
    }

    /// Returns the number of active items.
    pub fn active_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.status == ProgressItemStatus::Active)
            .count()
    }

    /// Returns the overall progress (average of all items).
    pub fn overall_progress(&self) -> f32 {
        if self.items.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.items.iter().map(|i| i.progress).sum();
        sum / self.items.len() as f32
    }

    /// Finds an item by ID.
    pub fn find(&self, id: &str) -> Option<&ProgressItem> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Finds a mutable item by ID.
    pub fn find_mut(&mut self, id: &str) -> Option<&mut ProgressItem> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    /// Removes an item by ID.
    pub fn remove(&mut self, id: &str) -> bool {
        let len_before = self.items.len();
        self.items.retain(|i| i.id != id);
        self.items.len() < len_before
    }

    /// Clears all items.
    pub fn clear(&mut self) {
        self.items.clear();
        self.scroll_offset = 0;
    }

    /// Returns the maximum visible items.
    pub fn max_visible(&self) -> usize {
        self.max_visible
    }

    /// Sets the maximum visible items.
    pub fn set_max_visible(&mut self, max: usize) {
        self.max_visible = max;
    }

    /// Returns the scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Sets the scroll offset.
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset.min(self.items.len().saturating_sub(1));
    }

    /// Returns the title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns whether percentages are shown.
    pub fn show_percentages(&self) -> bool {
        self.show_percentages
    }

    /// Sets whether to show percentages.
    pub fn set_show_percentages(&mut self, show: bool) {
        self.show_percentages = show;
    }

    /// Returns whether auto-remove is enabled.
    pub fn auto_remove_completed(&self) -> bool {
        self.auto_remove_completed
    }

    /// Sets whether to auto-remove completed items.
    pub fn set_auto_remove_completed(&mut self, auto_remove: bool) {
        self.auto_remove_completed = auto_remove;
    }

    /// Returns true if the multi-progress is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Maps an input event to a multi-progress message.
    pub fn handle_event(&self, event: &Event) -> Option<MultiProgressMessage> {
        MultiProgress::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<MultiProgressOutput> {
        MultiProgress::dispatch_event(self, event)
    }

    /// Updates the multi-progress state with a message, returning any output.
    pub fn update(&mut self, msg: MultiProgressMessage) -> Option<MultiProgressOutput> {
        MultiProgress::update(self, msg)
    }
}

/// A component for displaying multiple concurrent progress indicators.
///
/// # Visual Format
///
/// ```text
/// ┌─Progress─────────────────────────┐
/// │ ● Chapter 1      50% ████████░░░ │
/// │ ○ Chapter 2       0% ░░░░░░░░░░░ │
/// │ ✓ Chapter 3     100% ███████████ │
/// │ ✗ Chapter 4     Error: Timeout   │
/// └──────────────────────────────────┘
/// ```
pub struct MultiProgress;

impl Component for MultiProgress {
    type State = MultiProgressState;
    type Message = MultiProgressMessage;
    type Output = MultiProgressOutput;

    fn init() -> Self::State {
        MultiProgressState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            MultiProgressMessage::Add { id, label } => {
                if state.add(&id, label) {
                    Some(MultiProgressOutput::Added(id))
                } else {
                    None
                }
            }

            MultiProgressMessage::SetProgress { id, progress } => {
                if let Some(item) = state.find_mut(&id) {
                    item.progress = progress.clamp(0.0, 1.0);
                    // Auto-activate if pending and progress > 0
                    if item.status == ProgressItemStatus::Pending && progress > 0.0 {
                        item.status = ProgressItemStatus::Active;
                    }
                }
                None
            }

            MultiProgressMessage::SetStatus { id, status } => {
                if let Some(item) = state.find_mut(&id) {
                    item.status = status;
                }
                None
            }

            MultiProgressMessage::SetMessage { id, message } => {
                if let Some(item) = state.find_mut(&id) {
                    item.message = message;
                }
                None
            }

            MultiProgressMessage::Complete(id) => {
                if let Some(item) = state.find_mut(&id) {
                    item.progress = 1.0;
                    item.status = ProgressItemStatus::Completed;

                    if state.auto_remove_completed {
                        state.remove(&id);
                        return Some(MultiProgressOutput::Removed(id));
                    }
                    return Some(MultiProgressOutput::Completed(id));
                }
                None
            }

            MultiProgressMessage::Fail { id, message } => {
                if let Some(item) = state.find_mut(&id) {
                    item.status = ProgressItemStatus::Failed;
                    item.message = message;
                    return Some(MultiProgressOutput::Failed(id));
                }
                None
            }

            MultiProgressMessage::Remove(id) => {
                if state.remove(&id) {
                    Some(MultiProgressOutput::Removed(id))
                } else {
                    None
                }
            }

            MultiProgressMessage::Clear => {
                if state.items.is_empty() {
                    None
                } else {
                    state.clear();
                    Some(MultiProgressOutput::Cleared)
                }
            }

            MultiProgressMessage::ScrollUp => {
                if state.scroll_offset > 0 {
                    state.scroll_offset -= 1;
                }
                None
            }

            MultiProgressMessage::ScrollDown => {
                if state.scroll_offset < state.items.len().saturating_sub(1) {
                    state.scroll_offset += 1;
                }
                None
            }

            MultiProgressMessage::ScrollToTop => {
                state.scroll_offset = 0;
                None
            }

            MultiProgressMessage::ScrollToBottom => {
                state.scroll_offset = state.items.len().saturating_sub(1);
                None
            }
        }
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(MultiProgressMessage::ScrollUp),
                KeyCode::Down | KeyCode::Char('j') => Some(MultiProgressMessage::ScrollDown),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let block = if let Some(title) = &state.title {
            Block::default().borders(Borders::ALL).title(title.as_str())
        } else {
            Block::default().borders(Borders::ALL)
        };

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if state.items.is_empty() || inner.height == 0 {
            return;
        }

        // Calculate how many items we can show
        let visible_count = (inner.height as usize).min(state.items.len() - state.scroll_offset);

        // Build list items
        let items: Vec<ListItem> = state
            .items
            .iter()
            .skip(state.scroll_offset)
            .take(visible_count)
            .map(|item| {
                let symbol = item.status.symbol();
                let style = item.status.style(theme);

                // Build the content string
                let content = if item.status == ProgressItemStatus::Failed {
                    if let Some(msg) = &item.message {
                        format!("{} {}  Error: {}", symbol, item.label, msg)
                    } else {
                        format!("{} {}  Error", symbol, item.label)
                    }
                } else if state.show_percentages {
                    let bar_width = inner.width as usize - item.label.len() - 12; // symbol + spaces + percentage
                    let filled = ((item.progress * bar_width as f32) as usize).min(bar_width);
                    let empty = bar_width.saturating_sub(filled);
                    format!(
                        "{} {} {:>3}% {}{}",
                        symbol,
                        item.label,
                        item.percentage(),
                        "█".repeat(filled),
                        "░".repeat(empty)
                    )
                } else {
                    format!("{} {}", symbol, item.label)
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner);
    }
}

impl Focusable for MultiProgress {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests;
