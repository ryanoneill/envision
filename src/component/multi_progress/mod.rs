//! A component for displaying multiple concurrent progress indicators.
//!
//! [`MultiProgress`] provides a scrollable list of progress bars for tracking
//! multiple concurrent operations, commonly used in file processing, downloads,
//! or batch operations. State is stored in [`MultiProgressState`] and updated
//! via [`MultiProgressMessage`].
//!
//! See also [`ProgressBar`](super::ProgressBar) for a single progress indicator.
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

mod types;

pub use types::*;

use ratatui::widgets::{Block, Borders, List, ListItem};

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};

/// State for the MultiProgress component.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct MultiProgressState {
    /// All progress items.
    items: Vec<ProgressItem>,
    /// Maximum number of visible items.
    max_visible: usize,
    /// Scroll offset.
    scroll_offset: usize,
    /// Currently selected item index.
    selected: Option<usize>,
    /// Whether to auto-remove completed items.
    auto_remove_completed: bool,
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
            selected: None,
            auto_remove_completed: false,
            title: None,
            show_percentages: true,
        }
    }
}

impl MultiProgressState {
    /// Creates a new empty MultiProgress state.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::MultiProgressState;
    ///
    /// let state = MultiProgressState::new();
    /// assert!(state.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of visible items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let state = MultiProgressState::new().with_max_visible(5);
    /// assert_eq!(state.max_visible(), 5);
    /// ```
    pub fn with_max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    /// Sets whether to auto-remove completed items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let state = MultiProgressState::new().with_auto_remove_completed(true);
    /// assert!(state.auto_remove_completed());
    /// ```
    pub fn with_auto_remove_completed(mut self, auto_remove: bool) -> Self {
        self.auto_remove_completed = auto_remove;
        self
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::MultiProgressState;
    ///
    /// let state = MultiProgressState::new().with_title("Downloads");
    /// assert_eq!(state.title(), Some("Downloads"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether to show percentages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let state = MultiProgressState::new().with_show_percentages(false);
    /// assert!(!state.show_percentages());
    /// ```
    pub fn with_show_percentages(mut self, show: bool) -> Self {
        self.show_percentages = show;
        self
    }

    /// Adds a new progress item.
    ///
    /// Returns true if the item was added (id was unique).
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// assert!(state.add("task1", "Download file"));
    /// assert!(!state.add("task1", "Duplicate")); // duplicate ID rejected
    /// assert_eq!(state.len(), 1);
    /// ```
    pub fn add(&mut self, id: impl Into<String>, label: impl Into<String>) -> bool {
        let id = id.into();
        if self.items.iter().any(|i| i.id == id) {
            return false;
        }
        self.items.push(ProgressItem::new(id, label));
        if self.selected.is_none() {
            self.selected = Some(0);
        }
        true
    }

    /// Returns all items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// state.add("t1", "Task 1");
    /// assert_eq!(state.items().len(), 1);
    /// assert_eq!(state.items()[0].label(), "Task 1");
    /// ```
    pub fn items(&self) -> &[ProgressItem] {
        &self.items
    }

    /// Returns the number of items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// assert_eq!(state.len(), 0);
    /// state.add("t1", "Task");
    /// assert_eq!(state.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if there are no items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let state = MultiProgressState::new();
    /// assert!(state.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of completed items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MultiProgress, MultiProgressState, MultiProgressMessage, Component};
    ///
    /// let mut state = MultiProgressState::new();
    /// state.add("t1", "Task 1");
    /// state.add("t2", "Task 2");
    /// MultiProgress::update(&mut state, MultiProgressMessage::Complete("t1".to_string()));
    /// assert_eq!(state.completed_count(), 1);
    /// ```
    pub fn completed_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.status == ProgressItemStatus::Completed)
            .count()
    }

    /// Returns the number of failed items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MultiProgress, MultiProgressState, MultiProgressMessage, Component};
    ///
    /// let mut state = MultiProgressState::new();
    /// state.add("t1", "Task 1");
    /// MultiProgress::update(&mut state, MultiProgressMessage::Fail {
    ///     id: "t1".to_string(),
    ///     message: Some("timeout".to_string()),
    /// });
    /// assert_eq!(state.failed_count(), 1);
    /// ```
    pub fn failed_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.status == ProgressItemStatus::Failed)
            .count()
    }

    /// Returns the number of active items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MultiProgress, MultiProgressState, MultiProgressMessage, Component};
    ///
    /// let mut state = MultiProgressState::new();
    /// state.add("t1", "Task 1");
    /// MultiProgress::update(&mut state, MultiProgressMessage::SetProgress {
    ///     id: "t1".to_string(),
    ///     progress: 0.5,
    /// });
    /// assert_eq!(state.active_count(), 1);
    /// ```
    pub fn active_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.status == ProgressItemStatus::Active)
            .count()
    }

    /// Returns the overall progress (average of all items).
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::{MultiProgress, MultiProgressState, MultiProgressMessage, Component};
    ///
    /// let mut state = MultiProgressState::new();
    /// state.add("a", "Task A");
    /// state.add("b", "Task B");
    ///
    /// MultiProgress::update(&mut state, MultiProgressMessage::SetProgress {
    ///     id: "a".to_string(),
    ///     progress: 1.0,
    /// });
    /// assert!((state.overall_progress() - 0.5).abs() < f32::EPSILON);
    /// ```
    pub fn overall_progress(&self) -> f32 {
        if self.items.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.items.iter().map(|i| i.progress).sum();
        sum / self.items.len() as f32
    }

    /// Finds an item by ID.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// state.add("dl", "Download");
    /// assert_eq!(state.find("dl").unwrap().label(), "Download");
    /// assert!(state.find("missing").is_none());
    /// ```
    pub fn find(&self, id: &str) -> Option<&ProgressItem> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Finds a mutable item by ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// state.add("dl", "Download");
    /// assert!(state.find_mut("dl").is_some());
    /// assert!(state.find_mut("missing").is_none());
    /// ```
    pub fn find_mut(&mut self, id: &str) -> Option<&mut ProgressItem> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    /// Removes an item by ID.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// state.add("task1", "Task");
    /// assert!(state.remove("task1"));
    /// assert!(state.is_empty());
    /// ```
    pub fn remove(&mut self, id: &str) -> bool {
        let len_before = self.items.len();
        self.items.retain(|i| i.id != id);
        self.items.len() < len_before
    }

    /// Clears all items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// state.add("t1", "Task");
    /// state.clear();
    /// assert!(state.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.items.clear();
        self.scroll_offset = 0;
    }

    /// Returns the maximum visible items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let state = MultiProgressState::new();
    /// assert_eq!(state.max_visible(), 8); // default
    /// ```
    pub fn max_visible(&self) -> usize {
        self.max_visible
    }

    /// Sets the maximum visible items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// state.set_max_visible(5);
    /// assert_eq!(state.max_visible(), 5);
    /// ```
    pub fn set_max_visible(&mut self, max: usize) {
        self.max_visible = max;
    }

    /// Returns the scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let state = MultiProgressState::new();
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Returns the currently selected item index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// state.add("t1", "Task 1");
    /// assert_eq!(state.selected(), Some(0));
    /// ```
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Returns a reference to the currently selected item.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// state.add("t1", "Task 1");
    /// assert_eq!(state.selected_item().unwrap().label(), "Task 1");
    /// ```
    pub fn selected_item(&self) -> Option<&ProgressItem> {
        self.selected.and_then(|i| self.items.get(i))
    }

    /// Sets the selected item index. Clamped to valid range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// state.add("t1", "Task 1");
    /// state.add("t2", "Task 2");
    /// state.set_selected(Some(1));
    /// assert_eq!(state.selected(), Some(1));
    /// ```
    pub fn set_selected(&mut self, index: Option<usize>) {
        self.selected = index.map(|i| i.min(self.items.len().saturating_sub(1)));
    }

    /// Sets the viewport scroll offset.
    ///
    /// This controls which items are visible in the viewport, independent
    /// of which item is selected. Use [`set_selected`](Self::set_selected)
    /// to change the highlighted item. Keyboard navigation (Up/Down)
    /// adjusts both selection and scroll together.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// state.add("t1", "Task 1");
    /// state.add("t2", "Task 2");
    /// state.set_scroll_offset(1);
    /// assert_eq!(state.scroll_offset(), 1);
    /// ```
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset.min(self.items.len().saturating_sub(1));
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let state = MultiProgressState::new().with_title("Downloads");
    /// assert_eq!(state.title(), Some("Downloads"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// state.set_title(Some("Tasks".to_string()));
    /// assert_eq!(state.title(), Some("Tasks"));
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns whether percentages are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let state = MultiProgressState::new();
    /// assert!(state.show_percentages()); // enabled by default
    /// ```
    pub fn show_percentages(&self) -> bool {
        self.show_percentages
    }

    /// Sets whether to show percentages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// state.set_show_percentages(false);
    /// assert!(!state.show_percentages());
    /// ```
    pub fn set_show_percentages(&mut self, show: bool) {
        self.show_percentages = show;
    }

    /// Returns whether auto-remove is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let state = MultiProgressState::new();
    /// assert!(!state.auto_remove_completed()); // disabled by default
    /// ```
    pub fn auto_remove_completed(&self) -> bool {
        self.auto_remove_completed
    }

    /// Sets whether to auto-remove completed items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MultiProgressState;
    ///
    /// let mut state = MultiProgressState::new();
    /// state.set_auto_remove_completed(true);
    /// assert!(state.auto_remove_completed());
    /// ```
    pub fn set_auto_remove_completed(&mut self, auto_remove: bool) {
        self.auto_remove_completed = auto_remove;
    }

    /// Updates the multi-progress state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MultiProgressMessage, MultiProgressOutput, MultiProgressState};
    ///
    /// let mut state = MultiProgressState::default();
    /// let output = state.update(MultiProgressMessage::Add {
    ///     id: "task1".to_string(),
    ///     label: "Task 1".to_string(),
    /// });
    /// assert_eq!(output, Some(MultiProgressOutput::Added("task1".to_string())));
    /// ```
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

            MultiProgressMessage::Select => {
                if let Some(index) = state.selected {
                    if index < state.items.len() {
                        return Some(MultiProgressOutput::Selected(index));
                    }
                }
                None
            }

            MultiProgressMessage::ScrollUp => {
                let current = state.selected.unwrap_or(0);
                if current > 0 {
                    state.selected = Some(current - 1);
                    // Adjust scroll to keep selection visible
                    if state.selected.unwrap_or(0) < state.scroll_offset {
                        state.scroll_offset = state.selected.unwrap_or(0);
                    }
                }
                None
            }

            MultiProgressMessage::ScrollDown => {
                let current = state.selected.unwrap_or(0);
                let max = state.items.len().saturating_sub(1);
                if current < max {
                    state.selected = Some(current + 1);
                    // Adjust scroll to keep selection visible
                    let sel = state.selected.unwrap_or(0);
                    if sel >= state.scroll_offset + state.max_visible {
                        state.scroll_offset =
                            sel.saturating_sub(state.max_visible.saturating_sub(1));
                    }
                }
                None
            }

            MultiProgressMessage::ScrollToTop => {
                state.selected = if state.items.is_empty() {
                    None
                } else {
                    Some(0)
                };
                state.scroll_offset = 0;
                None
            }

            MultiProgressMessage::ScrollToBottom => {
                let last = state.items.len().saturating_sub(1);
                state.selected = if state.items.is_empty() {
                    None
                } else {
                    Some(last)
                };
                state.scroll_offset = state.items.len().saturating_sub(state.max_visible);
                None
            }
        }
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Up | Key::Char('k') => Some(MultiProgressMessage::ScrollUp),
                Key::Down | Key::Char('j') => Some(MultiProgressMessage::ScrollDown),
                Key::Enter => Some(MultiProgressMessage::Select),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        if ctx.area.width == 0 || ctx.area.height == 0 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::MultiProgress)
                    .with_id("multi_progress")
                    .with_meta("item_count", state.items.len().to_string()),
            );
        });

        let block = if let Some(title) = &state.title {
            Block::default().borders(Borders::ALL).title(title.as_str())
        } else {
            Block::default().borders(Borders::ALL)
        };

        let inner = block.inner(ctx.area);
        ctx.frame.render_widget(block, ctx.area);

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
                let style = if ctx.disabled {
                    ctx.theme.disabled_style()
                } else {
                    item.status.style(ctx.theme)
                };

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
        ctx.frame.render_widget(list, inner);
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
