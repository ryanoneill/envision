//! A searchable log viewer with severity filtering.
//!
//! [`LogViewer`] composes a [`StatusLog`](super::StatusLog) with an
//! [`InputField`](super::InputField) search bar and severity-level toggle
//! filters. Press `/` to focus the search bar, `Escape` to clear and return
//! to the log, and `1`-`4` to toggle Info/Success/Warning/Error filters.
//!
//! Features include follow mode for auto-scrolling to new entries, regex
//! search (when the `regex` feature is enabled), and search history
//! navigation with Up/Down arrow keys.
//!
//! State is stored in [`LogViewerState`], updated via [`LogViewerMessage`],
//! and produces [`LogViewerOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Focusable, LogViewer, LogViewerState,
//!     LogViewerMessage, LogViewerOutput,
//! };
//!
//! let mut state = LogViewerState::new();
//! state.push_info("Application started");
//! state.push_warning("Disk space low");
//! state.push_error("Connection failed");
//!
//! assert_eq!(state.visible_entries().len(), 3);
//!
//! // Filter to errors only (toggle off Info, Success, Warning)
//! LogViewer::update(&mut state, LogViewerMessage::ToggleInfo);
//! LogViewer::update(&mut state, LogViewerMessage::ToggleSuccess);
//! LogViewer::update(&mut state, LogViewerMessage::ToggleWarning);
//! assert_eq!(state.visible_entries().len(), 1);
//! ```

mod state;
mod view;

use std::marker::PhantomData;

use ratatui::prelude::*;

use super::{
    Component, Disableable, Focusable, InputFieldMessage, InputFieldState, StatusLogEntry,
    StatusLogLevel, ViewContext,
};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;

pub use state::LogViewerState;

/// Internal focus target for the log viewer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
enum Focus {
    /// The log list is focused.
    #[default]
    Log,
    /// The search bar is focused.
    Search,
}

/// Messages that can be sent to a LogViewer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogViewerMessage {
    /// Scroll the log up by one line.
    ScrollUp,
    /// Scroll the log down by one line.
    ScrollDown,
    /// Scroll to the top of the log.
    ScrollToTop,
    /// Scroll to the bottom of the log.
    ScrollToBottom,
    /// Focus the search bar.
    FocusSearch,
    /// Return focus to the log (and optionally clear search).
    FocusLog,
    /// Type a character in the search bar.
    SearchInput(char),
    /// Delete character before cursor in search bar.
    SearchBackspace,
    /// Delete character at cursor in search bar.
    SearchDelete,
    /// Move search cursor left.
    SearchLeft,
    /// Move search cursor right.
    SearchRight,
    /// Move search cursor to start.
    SearchHome,
    /// Move search cursor to end.
    SearchEnd,
    /// Clear the search text.
    ClearSearch,
    /// Toggle the Info level filter.
    ToggleInfo,
    /// Toggle the Success level filter.
    ToggleSuccess,
    /// Toggle the Warning level filter.
    ToggleWarning,
    /// Toggle the Error level filter.
    ToggleError,
    /// Add an entry to the log.
    Push {
        /// The message text.
        message: String,
        /// The severity level.
        level: StatusLogLevel,
        /// Optional timestamp.
        timestamp: Option<String>,
    },
    /// Clear all log entries.
    Clear,
    /// Remove a specific entry by ID.
    Remove(u64),
    /// Toggle follow mode (auto-scroll to bottom on new entries).
    ToggleFollow,
    /// Toggle regex search mode.
    ToggleRegex,
    /// Confirm search and save to history.
    ConfirmSearch,
    /// Navigate to previous search history entry.
    SearchHistoryUp,
    /// Navigate to next search history entry.
    SearchHistoryDown,
}

/// Output messages from a LogViewer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogViewerOutput {
    /// A log entry was added.
    Added(u64),
    /// A log entry was removed.
    Removed(u64),
    /// All entries were cleared.
    Cleared,
    /// An old entry was evicted due to max_entries limit.
    Evicted(u64),
    /// The search text changed.
    SearchChanged(String),
    /// A filter toggle changed.
    FilterChanged,
    /// Follow mode was toggled.
    FollowToggled(bool),
    /// Regex mode was toggled.
    RegexToggled(bool),
}

/// A searchable log viewer with severity filtering.
///
/// Composes a log display with a search input field and severity-level
/// toggle filters. The search is case-insensitive and matches against
/// entry messages. Supports follow mode for auto-scrolling, regex search
/// (with the `regex` feature), and search history.
///
/// # Key Bindings (Log Mode)
///
/// - `Up` / `k` — Scroll up (disables follow mode)
/// - `Down` / `j` — Scroll down (disables follow mode)
/// - `Home` — Scroll to top (newest)
/// - `End` — Scroll to bottom (oldest)
/// - `/` — Focus search bar
/// - `f` — Toggle follow mode
/// - `1` — Toggle Info filter
/// - `2` — Toggle Success filter
/// - `3` — Toggle Warning filter
/// - `4` — Toggle Error filter
///
/// # Key Bindings (Search Mode)
///
/// - `Escape` — Clear search and return to log
/// - `Enter` — Confirm search (save to history) and return to log
/// - `Up` — Previous search history entry
/// - `Down` — Next search history entry
/// - `Ctrl+r` — Toggle regex search mode
/// - Standard text editing keys
pub struct LogViewer(PhantomData<()>);

impl Component for LogViewer {
    type State = LogViewerState;
    type Message = LogViewerMessage;
    type Output = LogViewerOutput;

    fn init() -> Self::State {
        LogViewerState::default()
    }

    fn handle_event(
        state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;

        match state.focus {
            Focus::Log => match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(LogViewerMessage::ScrollUp),
                KeyCode::Down | KeyCode::Char('j') => Some(LogViewerMessage::ScrollDown),
                KeyCode::Home => Some(LogViewerMessage::ScrollToTop),
                KeyCode::End => Some(LogViewerMessage::ScrollToBottom),
                KeyCode::Char('/') => Some(LogViewerMessage::FocusSearch),
                KeyCode::Char('f') => Some(LogViewerMessage::ToggleFollow),
                KeyCode::Char('1') => Some(LogViewerMessage::ToggleInfo),
                KeyCode::Char('2') => Some(LogViewerMessage::ToggleSuccess),
                KeyCode::Char('3') => Some(LogViewerMessage::ToggleWarning),
                KeyCode::Char('4') => Some(LogViewerMessage::ToggleError),
                _ => None,
            },
            Focus::Search => match key.code {
                KeyCode::Esc => Some(LogViewerMessage::ClearSearch),
                KeyCode::Enter => Some(LogViewerMessage::ConfirmSearch),
                KeyCode::Up => Some(LogViewerMessage::SearchHistoryUp),
                KeyCode::Down => Some(LogViewerMessage::SearchHistoryDown),
                KeyCode::Char(c) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match c {
                            'r' => Some(LogViewerMessage::ToggleRegex),
                            _ => None,
                        }
                    } else {
                        Some(LogViewerMessage::SearchInput(c))
                    }
                }
                KeyCode::Backspace => Some(LogViewerMessage::SearchBackspace),
                KeyCode::Delete => Some(LogViewerMessage::SearchDelete),
                KeyCode::Left => Some(LogViewerMessage::SearchLeft),
                KeyCode::Right => Some(LogViewerMessage::SearchRight),
                KeyCode::Home => Some(LogViewerMessage::SearchHome),
                KeyCode::End => Some(LogViewerMessage::SearchEnd),
                _ => None,
            },
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        match msg {
            LogViewerMessage::ScrollUp => {
                state.follow = false;
                let len = state.visible_entries().len();
                state.scroll.set_content_length(len);
                state.scroll.set_viewport_height(1.min(len));
                state.scroll.scroll_up();
                None
            }
            LogViewerMessage::ScrollDown => {
                state.follow = false;
                let len = state.visible_entries().len();
                state.scroll.set_content_length(len);
                state.scroll.set_viewport_height(1.min(len));
                state.scroll.scroll_down();
                None
            }
            LogViewerMessage::ScrollToTop => {
                let len = state.visible_entries().len();
                state.scroll.set_content_length(len);
                state.scroll.set_viewport_height(1.min(len));
                state.scroll.scroll_to_start();
                None
            }
            LogViewerMessage::ScrollToBottom => {
                let len = state.visible_entries().len();
                state.scroll.set_content_length(len);
                state.scroll.set_viewport_height(1.min(len));
                state.scroll.scroll_to_end();
                None
            }
            LogViewerMessage::FocusSearch => {
                state.focus = Focus::Search;
                state.search.set_focused(true);
                None
            }
            LogViewerMessage::FocusLog => {
                state.focus = Focus::Log;
                state.search.set_focused(false);
                state.history_index = None;
                None
            }
            LogViewerMessage::SearchInput(c) => {
                state.search.update(InputFieldMessage::Insert(c));
                state.search_text = state.search.value().to_string();
                state.scroll.set_offset(0);
                Some(LogViewerOutput::SearchChanged(state.search_text.clone()))
            }
            LogViewerMessage::SearchBackspace => {
                state.search.update(InputFieldMessage::Backspace);
                state.search_text = state.search.value().to_string();
                state.scroll.set_offset(0);
                Some(LogViewerOutput::SearchChanged(state.search_text.clone()))
            }
            LogViewerMessage::SearchDelete => {
                state.search.update(InputFieldMessage::Delete);
                state.search_text = state.search.value().to_string();
                state.scroll.set_offset(0);
                Some(LogViewerOutput::SearchChanged(state.search_text.clone()))
            }
            LogViewerMessage::SearchLeft => {
                state.search.update(InputFieldMessage::Left);
                None
            }
            LogViewerMessage::SearchRight => {
                state.search.update(InputFieldMessage::Right);
                None
            }
            LogViewerMessage::SearchHome => {
                state.search.update(InputFieldMessage::Home);
                None
            }
            LogViewerMessage::SearchEnd => {
                state.search.update(InputFieldMessage::End);
                None
            }
            LogViewerMessage::ClearSearch => {
                state.search.update(InputFieldMessage::Clear);
                state.search_text.clear();
                state.scroll.set_offset(0);
                state.focus = Focus::Log;
                state.search.set_focused(false);
                state.history_index = None;
                Some(LogViewerOutput::SearchChanged(String::new()))
            }
            LogViewerMessage::ToggleInfo => {
                state.show_info = !state.show_info;
                state.scroll.set_offset(0);
                Some(LogViewerOutput::FilterChanged)
            }
            LogViewerMessage::ToggleSuccess => {
                state.show_success = !state.show_success;
                state.scroll.set_offset(0);
                Some(LogViewerOutput::FilterChanged)
            }
            LogViewerMessage::ToggleWarning => {
                state.show_warning = !state.show_warning;
                state.scroll.set_offset(0);
                Some(LogViewerOutput::FilterChanged)
            }
            LogViewerMessage::ToggleError => {
                state.show_error = !state.show_error;
                state.scroll.set_offset(0);
                Some(LogViewerOutput::FilterChanged)
            }
            LogViewerMessage::Push {
                message,
                level,
                timestamp,
            } => {
                let id = state.push_entry(message, level, timestamp);
                // Auto-scroll to top (newest) when follow mode is enabled
                if state.follow {
                    state.scroll.set_offset(0);
                }
                Some(LogViewerOutput::Added(id))
            }
            LogViewerMessage::Clear => {
                state.clear();
                Some(LogViewerOutput::Cleared)
            }
            LogViewerMessage::Remove(id) => {
                if state.remove(id) {
                    Some(LogViewerOutput::Removed(id))
                } else {
                    None
                }
            }
            LogViewerMessage::ToggleFollow => {
                state.follow = !state.follow;
                if state.follow {
                    // When enabling follow, scroll to top (newest)
                    state.scroll.set_offset(0);
                }
                Some(LogViewerOutput::FollowToggled(state.follow))
            }
            LogViewerMessage::ToggleRegex => {
                state.use_regex = !state.use_regex;
                state.scroll.set_offset(0);
                Some(LogViewerOutput::RegexToggled(state.use_regex))
            }
            LogViewerMessage::ConfirmSearch => {
                // Save non-empty search text to history
                if !state.search_text.is_empty() {
                    // Remove duplicate if it already exists in history
                    state.search_history.retain(|h| h != &state.search_text);
                    state.search_history.push(state.search_text.clone());
                    // Enforce max history size
                    while state.search_history.len() > state.max_history {
                        state.search_history.remove(0);
                    }
                }
                state.history_index = None;
                state.focus = Focus::Log;
                state.search.set_focused(false);
                None
            }
            LogViewerMessage::SearchHistoryUp => {
                if state.search_history.is_empty() {
                    return None;
                }
                let new_index = match state.history_index {
                    None => state.search_history.len().saturating_sub(1),
                    Some(idx) => idx.saturating_sub(1),
                };
                state.history_index = Some(new_index);
                let text = state.search_history[new_index].clone();
                // Update the search field with the history entry
                state.search.update(InputFieldMessage::Clear);
                for c in text.chars() {
                    state.search.update(InputFieldMessage::Insert(c));
                }
                state.search_text = text;
                state.scroll.set_offset(0);
                Some(LogViewerOutput::SearchChanged(state.search_text.clone()))
            }
            LogViewerMessage::SearchHistoryDown => {
                if state.search_history.is_empty() {
                    return None;
                }
                match state.history_index {
                    None => None,
                    Some(idx) => {
                        if idx + 1 >= state.search_history.len() {
                            // Past the end of history, clear to empty
                            state.history_index = None;
                            state.search.update(InputFieldMessage::Clear);
                            state.search_text.clear();
                            state.scroll.set_offset(0);
                            Some(LogViewerOutput::SearchChanged(String::new()))
                        } else {
                            let new_index = idx + 1;
                            state.history_index = Some(new_index);
                            let text = state.search_history[new_index].clone();
                            state.search.update(InputFieldMessage::Clear);
                            for c in text.chars() {
                                state.search.update(InputFieldMessage::Insert(c));
                            }
                            state.search_text = text;
                            state.scroll.set_offset(0);
                            Some(LogViewerOutput::SearchChanged(state.search_text.clone()))
                        }
                    }
                }
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        if area.height < 3 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::container("log_viewer")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        // Layout: search bar (1 line) + filter bar (1 line) + log area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(area);

        let search_area = chunks[0];
        let filter_area = chunks[1];
        let log_area = chunks[2];

        // Render search bar
        view::render_search_bar(state, frame, search_area, theme);

        // Render filter bar
        view::render_filter_bar(state, frame, filter_area, theme);

        // Render log entries
        view::render_log(state, frame, log_area, theme);
    }
}

impl Focusable for LogViewer {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for LogViewer {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod enhancement_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
