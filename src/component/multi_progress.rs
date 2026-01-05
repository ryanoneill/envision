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
mod tests {
    use super::*;
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    // ========================================
    // ProgressItemStatus Tests
    // ========================================

    #[test]
    fn test_status_default() {
        let status = ProgressItemStatus::default();
        assert_eq!(status, ProgressItemStatus::Pending);
    }

    #[test]
    fn test_status_styles() {
        let theme = Theme::default();
        assert_eq!(
            ProgressItemStatus::Pending.style(&theme),
            theme.disabled_style()
        );
        assert_eq!(ProgressItemStatus::Active.style(&theme), theme.info_style());
        assert_eq!(
            ProgressItemStatus::Completed.style(&theme),
            theme.success_style()
        );
        assert_eq!(
            ProgressItemStatus::Failed.style(&theme),
            theme.error_style()
        );
    }

    #[test]
    fn test_status_symbols() {
        assert_eq!(ProgressItemStatus::Pending.symbol(), "○");
        assert_eq!(ProgressItemStatus::Active.symbol(), "●");
        assert_eq!(ProgressItemStatus::Completed.symbol(), "✓");
        assert_eq!(ProgressItemStatus::Failed.symbol(), "✗");
    }

    // ========================================
    // ProgressItem Tests
    // ========================================

    #[test]
    fn test_item_new() {
        let item = ProgressItem::new("id1", "Label");
        assert_eq!(item.id(), "id1");
        assert_eq!(item.label(), "Label");
        assert_eq!(item.progress(), 0.0);
        assert_eq!(item.status(), ProgressItemStatus::Pending);
        assert!(item.message().is_none());
    }

    #[test]
    fn test_item_percentage() {
        let mut item = ProgressItem::new("id1", "Test");
        item.progress = 0.5;
        assert_eq!(item.percentage(), 50);

        item.progress = 0.0;
        assert_eq!(item.percentage(), 0);

        item.progress = 1.0;
        assert_eq!(item.percentage(), 100);
    }

    // ========================================
    // State Creation Tests
    // ========================================

    #[test]
    fn test_state_new() {
        let state = MultiProgressState::new();
        assert!(state.is_empty());
        assert_eq!(state.max_visible(), 8);
        assert!(!state.auto_remove_completed());
        assert!(state.show_percentages());
    }

    #[test]
    fn test_state_with_max_visible() {
        let state = MultiProgressState::new().with_max_visible(5);
        assert_eq!(state.max_visible(), 5);
    }

    #[test]
    fn test_state_with_auto_remove() {
        let state = MultiProgressState::new().with_auto_remove(true);
        assert!(state.auto_remove_completed());
    }

    #[test]
    fn test_state_with_title() {
        let state = MultiProgressState::new().with_title("Progress");
        assert_eq!(state.title(), Some("Progress"));
    }

    #[test]
    fn test_state_with_percentages() {
        let state = MultiProgressState::new().with_percentages(false);
        assert!(!state.show_percentages());
    }

    // ========================================
    // State Manipulation Tests
    // ========================================

    #[test]
    fn test_add_item() {
        let mut state = MultiProgressState::new();
        assert!(state.add("id1", "Item 1"));
        assert_eq!(state.len(), 1);
        assert!(!state.is_empty());
    }

    #[test]
    fn test_add_duplicate_id() {
        let mut state = MultiProgressState::new();
        assert!(state.add("id1", "Item 1"));
        assert!(!state.add("id1", "Item 1 again")); // Duplicate
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn test_find() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        assert!(state.find("id1").is_some());
        assert!(state.find("nonexistent").is_none());
    }

    #[test]
    fn test_find_mut() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        if let Some(item) = state.find_mut("id1") {
            item.progress = 0.5;
        }

        assert_eq!(state.find("id1").unwrap().progress(), 0.5);
    }

    #[test]
    fn test_remove() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");
        state.add("id2", "Item 2");

        assert!(state.remove("id1"));
        assert_eq!(state.len(), 1);
        assert!(!state.remove("id1")); // Already removed
    }

    #[test]
    fn test_clear() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");
        state.add("id2", "Item 2");

        state.clear();
        assert!(state.is_empty());
    }

    // ========================================
    // Progress Counting Tests
    // ========================================

    #[test]
    fn test_completed_count() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");
        state.add("id2", "Item 2");
        state.add("id3", "Item 3");

        MultiProgress::update(
            &mut state,
            MultiProgressMessage::Complete("id1".to_string()),
        );
        MultiProgress::update(
            &mut state,
            MultiProgressMessage::Complete("id2".to_string()),
        );

        assert_eq!(state.completed_count(), 2);
    }

    #[test]
    fn test_failed_count() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");
        state.add("id2", "Item 2");

        MultiProgress::update(
            &mut state,
            MultiProgressMessage::Fail {
                id: "id1".to_string(),
                message: Some("Error".to_string()),
            },
        );

        assert_eq!(state.failed_count(), 1);
    }

    #[test]
    fn test_active_count() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");
        state.add("id2", "Item 2");

        MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetStatus {
                id: "id1".to_string(),
                status: ProgressItemStatus::Active,
            },
        );

        assert_eq!(state.active_count(), 1);
    }

    #[test]
    fn test_overall_progress() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");
        state.add("id2", "Item 2");

        MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetProgress {
                id: "id1".to_string(),
                progress: 0.5,
            },
        );
        MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetProgress {
                id: "id2".to_string(),
                progress: 1.0,
            },
        );

        assert_eq!(state.overall_progress(), 0.75);
    }

    #[test]
    fn test_overall_progress_empty() {
        let state = MultiProgressState::new();
        assert_eq!(state.overall_progress(), 0.0);
    }

    // ========================================
    // Component Tests
    // ========================================

    #[test]
    fn test_init() {
        let state = MultiProgress::init();
        assert!(state.is_empty());
    }

    #[test]
    fn test_update_add() {
        let mut state = MultiProgress::init();
        let output = MultiProgress::update(
            &mut state,
            MultiProgressMessage::Add {
                id: "id1".to_string(),
                label: "Item 1".to_string(),
            },
        );
        assert_eq!(output, Some(MultiProgressOutput::Added("id1".to_string())));
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn test_update_set_progress() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetProgress {
                id: "id1".to_string(),
                progress: 0.5,
            },
        );

        assert_eq!(state.find("id1").unwrap().progress(), 0.5);
        // Should auto-activate
        assert_eq!(
            state.find("id1").unwrap().status(),
            ProgressItemStatus::Active
        );
    }

    #[test]
    fn test_update_set_progress_clamped() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetProgress {
                id: "id1".to_string(),
                progress: 1.5, // Should be clamped to 1.0
            },
        );

        assert_eq!(state.find("id1").unwrap().progress(), 1.0);

        MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetProgress {
                id: "id1".to_string(),
                progress: -0.5, // Should be clamped to 0.0
            },
        );

        assert_eq!(state.find("id1").unwrap().progress(), 0.0);
    }

    #[test]
    fn test_update_complete() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        let output = MultiProgress::update(
            &mut state,
            MultiProgressMessage::Complete("id1".to_string()),
        );

        assert_eq!(
            output,
            Some(MultiProgressOutput::Completed("id1".to_string()))
        );
        let item = state.find("id1").unwrap();
        assert_eq!(item.progress(), 1.0);
        assert_eq!(item.status(), ProgressItemStatus::Completed);
    }

    #[test]
    fn test_update_complete_auto_remove() {
        let mut state = MultiProgressState::new().with_auto_remove(true);
        state.add("id1", "Item 1");

        let output = MultiProgress::update(
            &mut state,
            MultiProgressMessage::Complete("id1".to_string()),
        );

        assert_eq!(
            output,
            Some(MultiProgressOutput::Removed("id1".to_string()))
        );
        assert!(state.is_empty());
    }

    #[test]
    fn test_update_fail() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        let output = MultiProgress::update(
            &mut state,
            MultiProgressMessage::Fail {
                id: "id1".to_string(),
                message: Some("Timeout".to_string()),
            },
        );

        assert_eq!(output, Some(MultiProgressOutput::Failed("id1".to_string())));
        let item = state.find("id1").unwrap();
        assert_eq!(item.status(), ProgressItemStatus::Failed);
        assert_eq!(item.message(), Some("Timeout"));
    }

    #[test]
    fn test_update_remove() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        let output =
            MultiProgress::update(&mut state, MultiProgressMessage::Remove("id1".to_string()));

        assert_eq!(
            output,
            Some(MultiProgressOutput::Removed("id1".to_string()))
        );
        assert!(state.is_empty());
    }

    #[test]
    fn test_update_clear() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");
        state.add("id2", "Item 2");

        let output = MultiProgress::update(&mut state, MultiProgressMessage::Clear);

        assert_eq!(output, Some(MultiProgressOutput::Cleared));
        assert!(state.is_empty());
    }

    #[test]
    fn test_update_clear_empty() {
        let mut state = MultiProgress::init();
        let output = MultiProgress::update(&mut state, MultiProgressMessage::Clear);
        assert!(output.is_none());
    }

    // ========================================
    // Scroll Tests
    // ========================================

    #[test]
    fn test_scroll_down() {
        let mut state = MultiProgressState::new();
        for i in 0..10 {
            state.add(format!("id{}", i), format!("Item {}", i));
        }

        MultiProgress::update(&mut state, MultiProgressMessage::ScrollDown);
        assert_eq!(state.scroll_offset(), 1);
    }

    #[test]
    fn test_scroll_up() {
        let mut state = MultiProgressState::new();
        for i in 0..10 {
            state.add(format!("id{}", i), format!("Item {}", i));
        }
        state.set_scroll_offset(5);

        MultiProgress::update(&mut state, MultiProgressMessage::ScrollUp);
        assert_eq!(state.scroll_offset(), 4);
    }

    #[test]
    fn test_scroll_to_top() {
        let mut state = MultiProgressState::new();
        for i in 0..10 {
            state.add(format!("id{}", i), format!("Item {}", i));
        }
        state.set_scroll_offset(5);

        MultiProgress::update(&mut state, MultiProgressMessage::ScrollToTop);
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_scroll_to_bottom() {
        let mut state = MultiProgressState::new();
        for i in 0..10 {
            state.add(format!("id{}", i), format!("Item {}", i));
        }

        MultiProgress::update(&mut state, MultiProgressMessage::ScrollToBottom);
        assert_eq!(state.scroll_offset(), 9);
    }

    // ========================================
    // Focusable Tests
    // ========================================

    #[test]
    fn test_focusable() {
        let mut state = MultiProgressState::new();
        assert!(!MultiProgress::is_focused(&state));

        MultiProgress::focus(&mut state);
        assert!(MultiProgress::is_focused(&state));

        MultiProgress::blur(&mut state);
        assert!(!MultiProgress::is_focused(&state));
    }

    // ========================================
    // View Tests
    // ========================================

    #[test]
    fn test_view_empty() {
        let state = MultiProgressState::new();
        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        // Should render border only
        let output = terminal.backend().to_string();
        assert!(output.contains("─") || output.contains("│"));
    }

    #[test]
    fn test_view_with_items() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");
        state.add("id2", "Item 2");

        MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetProgress {
                id: "id1".to_string(),
                progress: 0.5,
            },
        );

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Item 1"));
        assert!(output.contains("Item 2"));
        assert!(output.contains("50%"));
    }

    #[test]
    fn test_view_with_title() {
        let state = MultiProgressState::new().with_title("Downloads");
        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Downloads"));
    }

    #[test]
    fn test_view_failed_item() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        MultiProgress::update(
            &mut state,
            MultiProgressMessage::Fail {
                id: "id1".to_string(),
                message: Some("Connection lost".to_string()),
            },
        );

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Error"));
        assert!(output.contains("Connection lost"));
    }

    #[test]
    fn test_view_completed_item() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        MultiProgress::update(
            &mut state,
            MultiProgressMessage::Complete("id1".to_string()),
        );

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("100%"));
        assert!(output.contains("✓"));
    }

    #[test]
    fn test_clone() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        let cloned = state.clone();
        assert_eq!(cloned.len(), 1);
    }

    // ========================================
    // Additional Coverage Tests
    // ========================================

    #[test]
    fn test_view_zero_size_area() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");
        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        // Test with zero width
        terminal
            .draw(|frame| {
                MultiProgress::view(&state, frame, Rect::new(0, 0, 0, 10), &Theme::default());
            })
            .unwrap();

        // Test with zero height
        terminal
            .draw(|frame| {
                MultiProgress::view(&state, frame, Rect::new(0, 0, 60, 0), &Theme::default());
            })
            .unwrap();
    }

    #[test]
    fn test_view_without_percentages() {
        let mut state = MultiProgressState::new().with_percentages(false);
        state.add("id1", "Item 1");

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Item 1"));
        assert!(!output.contains("%")); // No percentage shown
    }

    #[test]
    fn test_view_failed_without_message() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        MultiProgress::update(
            &mut state,
            MultiProgressMessage::Fail {
                id: "id1".to_string(),
                message: None, // No message
            },
        );

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| MultiProgress::view(&state, frame, frame.area(), &Theme::default()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Error"));
    }

    #[test]
    fn test_update_add_duplicate() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        let output = MultiProgress::update(
            &mut state,
            MultiProgressMessage::Add {
                id: "id1".to_string(),
                label: "Duplicate".to_string(),
            },
        );

        assert!(output.is_none());
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn test_update_set_progress_nonexistent() {
        let mut state = MultiProgressState::new();

        let output = MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetProgress {
                id: "nonexistent".to_string(),
                progress: 0.5,
            },
        );

        assert!(output.is_none());
    }

    #[test]
    fn test_update_set_status_nonexistent() {
        let mut state = MultiProgressState::new();

        let output = MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetStatus {
                id: "nonexistent".to_string(),
                status: ProgressItemStatus::Active,
            },
        );

        assert!(output.is_none());
    }

    #[test]
    fn test_update_set_message() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetMessage {
                id: "id1".to_string(),
                message: Some("Processing...".to_string()),
            },
        );

        assert_eq!(state.find("id1").unwrap().message(), Some("Processing..."));
    }

    #[test]
    fn test_update_set_message_nonexistent() {
        let mut state = MultiProgressState::new();

        let output = MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetMessage {
                id: "nonexistent".to_string(),
                message: Some("Message".to_string()),
            },
        );

        assert!(output.is_none());
    }

    #[test]
    fn test_update_complete_nonexistent() {
        let mut state = MultiProgressState::new();

        let output = MultiProgress::update(
            &mut state,
            MultiProgressMessage::Complete("nonexistent".to_string()),
        );

        assert!(output.is_none());
    }

    #[test]
    fn test_update_fail_nonexistent() {
        let mut state = MultiProgressState::new();

        let output = MultiProgress::update(
            &mut state,
            MultiProgressMessage::Fail {
                id: "nonexistent".to_string(),
                message: None,
            },
        );

        assert!(output.is_none());
    }

    #[test]
    fn test_update_remove_nonexistent() {
        let mut state = MultiProgressState::new();

        let output = MultiProgress::update(
            &mut state,
            MultiProgressMessage::Remove("nonexistent".to_string()),
        );

        assert!(output.is_none());
    }

    #[test]
    fn test_scroll_up_at_top() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");
        assert_eq!(state.scroll_offset(), 0);

        MultiProgress::update(&mut state, MultiProgressMessage::ScrollUp);
        assert_eq!(state.scroll_offset(), 0); // Should stay at 0
    }

    #[test]
    fn test_scroll_down_at_bottom() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");
        state.add("id2", "Item 2");
        state.set_scroll_offset(1); // At the last item

        MultiProgress::update(&mut state, MultiProgressMessage::ScrollDown);
        assert_eq!(state.scroll_offset(), 1); // Should stay at 1
    }

    #[test]
    fn test_set_scroll_offset_clamped() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");
        state.add("id2", "Item 2");

        state.set_scroll_offset(100); // Too large
        assert_eq!(state.scroll_offset(), 1); // Clamped to last valid
    }

    #[test]
    fn test_set_title() {
        let mut state = MultiProgressState::new();
        assert!(state.title().is_none());

        state.set_title(Some("New Title".to_string()));
        assert_eq!(state.title(), Some("New Title"));

        state.set_title(None);
        assert!(state.title().is_none());
    }

    #[test]
    fn test_set_show_percentages() {
        let mut state = MultiProgressState::new();
        assert!(state.show_percentages());

        state.set_show_percentages(false);
        assert!(!state.show_percentages());
    }

    #[test]
    fn test_set_auto_remove_completed() {
        let mut state = MultiProgressState::new();
        assert!(!state.auto_remove_completed());

        state.set_auto_remove_completed(true);
        assert!(state.auto_remove_completed());
    }

    #[test]
    fn test_set_max_visible() {
        let mut state = MultiProgressState::new();
        assert_eq!(state.max_visible(), 8);

        state.set_max_visible(5);
        assert_eq!(state.max_visible(), 5);
    }

    #[test]
    fn test_set_progress_no_auto_activate_if_already_active() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        // First set to Active
        MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetStatus {
                id: "id1".to_string(),
                status: ProgressItemStatus::Active,
            },
        );

        // Now set progress
        MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetProgress {
                id: "id1".to_string(),
                progress: 0.5,
            },
        );

        assert_eq!(
            state.find("id1").unwrap().status(),
            ProgressItemStatus::Active
        );
    }

    #[test]
    fn test_set_progress_no_auto_activate_if_zero() {
        let mut state = MultiProgressState::new();
        state.add("id1", "Item 1");

        // Set progress to 0 (should not auto-activate)
        MultiProgress::update(
            &mut state,
            MultiProgressMessage::SetProgress {
                id: "id1".to_string(),
                progress: 0.0,
            },
        );

        assert_eq!(
            state.find("id1").unwrap().status(),
            ProgressItemStatus::Pending
        );
    }
}
