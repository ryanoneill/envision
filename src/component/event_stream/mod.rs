//! A real-time filterable event feed with severity coloring and structured fields.
//!
//! [`EventStream`] displays structured events with typed key-value fields,
//! severity levels, and real-time filtering. Unlike [`LogViewer`](super::LogViewer)
//! which works with unstructured text entries, EventStream treats each event as
//! a structured record with named fields that can be displayed as columns.
//!
//! State is stored in [`EventStreamState`], updated via [`EventStreamMessage`],
//! and produces [`EventStreamOutput`].
//!
//! Focus and disabled state are managed via [`ViewContext`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, EventStream, EventStreamState,
//!     EventLevel, StreamEvent,
//! };
//!
//! let mut state = EventStreamState::new();
//! state.push_event(EventLevel::Info, "Request received");
//! state.push_event_with_fields(
//!     EventLevel::Warning,
//!     "Slow query",
//!     vec![("ms".into(), "1200".into()), ("table".into(), "users".into())],
//! );
//!
//! assert_eq!(state.event_count(), 2);
//! assert_eq!(state.visible_events().len(), 2);
//! ```

mod render;
mod state;
mod types;

use std::marker::PhantomData;

use ratatui::prelude::*;

use super::{Component, InputFieldMessage, ViewContext};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;

pub use state::EventStreamState;
pub use types::{EventLevel, StreamEvent};

// =============================================================================
// Internal focus target
// =============================================================================

/// Internal focus target for the event stream.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
enum Focus {
    /// The event list is focused.
    #[default]
    List,
    /// The search/filter bar is focused.
    Search,
}

// =============================================================================
// Messages
// =============================================================================

/// Messages that can be sent to an EventStream.
#[derive(Clone, Debug, PartialEq)]
pub enum EventStreamMessage {
    /// Add a new event.
    PushEvent(StreamEvent),
    /// Replace all events.
    SetEvents(Vec<StreamEvent>),
    /// Clear all events.
    Clear,
    /// Set the text filter.
    SetFilter(String),
    /// Set the minimum level filter.
    SetLevelFilter(Option<EventLevel>),
    /// Set the source filter.
    SetSourceFilter(Option<String>),
    /// Set which field columns to show.
    SetVisibleColumns(Vec<String>),
    /// Scroll up by one line.
    ScrollUp,
    /// Scroll down by one line.
    ScrollDown,
    /// Scroll to the top.
    ScrollToTop,
    /// Scroll to the bottom.
    ScrollToBottom,
    /// Toggle auto-scroll (follow new events).
    ToggleAutoScroll,
    /// Focus the search/filter input.
    FocusSearch,
    /// Return focus to the event list.
    FocusList,
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
    /// Set level filter by number (1=Trace, 2=Debug, 3=Info, 4=Warning, 5=Error).
    QuickLevelFilter(u8),
}

/// Output messages from an EventStream.
#[derive(Clone, Debug, PartialEq)]
pub enum EventStreamOutput {
    /// An event was added (includes event ID).
    EventAdded(u64),
    /// A filter changed (text, level, or source).
    FilterChanged,
    /// All events were cleared.
    EventsCleared,
}

// =============================================================================
// Component
// =============================================================================

/// A real-time filterable event feed with severity coloring and structured fields.
///
/// Displays structured events with typed key-value fields, severity levels,
/// and real-time filtering. Each event has an ID, timestamp, level, message,
/// optional source, and structured fields displayed as columns.
///
/// # Key Bindings (List Mode)
///
/// - `Up` / `k` -- Scroll up
/// - `Down` / `j` -- Scroll down
/// - `Home` / `g` -- Scroll to top
/// - `End` / `G` -- Scroll to bottom
/// - `/` -- Focus filter input
/// - `1`-`5` -- Quick level filter (1=Trace, 2=Debug, 3=Info, 4=Warning, 5=Error)
/// - `f` -- Toggle auto-scroll
///
/// # Key Bindings (Search Mode)
///
/// - `Escape` -- Clear search and return to list
/// - `Enter` -- Return to list (keep search text)
/// - Standard text editing keys
pub struct EventStream(PhantomData<()>);

impl Component for EventStream {
    type State = EventStreamState;
    type Message = EventStreamMessage;
    type Output = EventStreamOutput;

    fn init() -> Self::State {
        EventStreamState::default()
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
            Focus::List => match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(EventStreamMessage::ScrollUp),
                KeyCode::Down | KeyCode::Char('j') => Some(EventStreamMessage::ScrollDown),
                KeyCode::Home | KeyCode::Char('g') => Some(EventStreamMessage::ScrollToTop),
                KeyCode::End => Some(EventStreamMessage::ScrollToBottom),
                KeyCode::Char('G') => Some(EventStreamMessage::ScrollToBottom),
                KeyCode::Char('/') => Some(EventStreamMessage::FocusSearch),
                KeyCode::Char('1') => Some(EventStreamMessage::QuickLevelFilter(1)),
                KeyCode::Char('2') => Some(EventStreamMessage::QuickLevelFilter(2)),
                KeyCode::Char('3') => Some(EventStreamMessage::QuickLevelFilter(3)),
                KeyCode::Char('4') => Some(EventStreamMessage::QuickLevelFilter(4)),
                KeyCode::Char('5') => Some(EventStreamMessage::QuickLevelFilter(5)),
                KeyCode::Char('f') => Some(EventStreamMessage::ToggleAutoScroll),
                _ => None,
            },
            Focus::Search => match key.code {
                KeyCode::Esc => Some(EventStreamMessage::ClearSearch),
                KeyCode::Enter => Some(EventStreamMessage::FocusList),
                KeyCode::Char(c) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        None
                    } else {
                        Some(EventStreamMessage::SearchInput(c))
                    }
                }
                KeyCode::Backspace => Some(EventStreamMessage::SearchBackspace),
                KeyCode::Delete => Some(EventStreamMessage::SearchDelete),
                KeyCode::Left => Some(EventStreamMessage::SearchLeft),
                KeyCode::Right => Some(EventStreamMessage::SearchRight),
                KeyCode::Home => Some(EventStreamMessage::SearchHome),
                KeyCode::End => Some(EventStreamMessage::SearchEnd),
                _ => None,
            },
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            EventStreamMessage::PushEvent(event) => {
                let id = if event.id == 0 {
                    // Auto-assign ID
                    let new_id = state.next_id;
                    state.next_id += 1;
                    let mut ev = event;
                    ev.id = new_id;
                    state.events.push(ev);
                    new_id
                } else {
                    let id = event.id;
                    if event.id >= state.next_id {
                        state.next_id = event.id + 1;
                    }
                    state.events.push(event);
                    id
                };
                state.evict_oldest();
                if state.auto_scroll {
                    let len = state.visible_events().len();
                    state.scroll.set_content_length(len);
                    state.scroll.scroll_to_end();
                }
                Some(EventStreamOutput::EventAdded(id))
            }
            EventStreamMessage::SetEvents(events) => {
                state.events = events;
                state.next_id = state.events.iter().map(|e| e.id + 1).max().unwrap_or(0);
                state.scroll.set_offset(0);
                None
            }
            EventStreamMessage::Clear => {
                state.clear();
                Some(EventStreamOutput::EventsCleared)
            }
            EventStreamMessage::SetFilter(filter) => {
                state.filter_text = filter;
                state.scroll.set_offset(0);
                Some(EventStreamOutput::FilterChanged)
            }
            EventStreamMessage::SetLevelFilter(level) => {
                state.level_filter = level;
                state.scroll.set_offset(0);
                Some(EventStreamOutput::FilterChanged)
            }
            EventStreamMessage::SetSourceFilter(source) => {
                state.source_filter = source;
                state.scroll.set_offset(0);
                Some(EventStreamOutput::FilterChanged)
            }
            EventStreamMessage::SetVisibleColumns(columns) => {
                state.visible_columns = columns;
                None
            }
            EventStreamMessage::ScrollUp => {
                let len = state.visible_events().len();
                state.scroll.set_content_length(len);
                state.scroll.set_viewport_height(1.min(len));
                state.scroll.scroll_up();
                state.auto_scroll = false;
                None
            }
            EventStreamMessage::ScrollDown => {
                let len = state.visible_events().len();
                state.scroll.set_content_length(len);
                state.scroll.set_viewport_height(1.min(len));
                state.scroll.scroll_down();
                None
            }
            EventStreamMessage::ScrollToTop => {
                let len = state.visible_events().len();
                state.scroll.set_content_length(len);
                state.scroll.set_viewport_height(1.min(len));
                state.scroll.scroll_to_start();
                state.auto_scroll = false;
                None
            }
            EventStreamMessage::ScrollToBottom => {
                let len = state.visible_events().len();
                state.scroll.set_content_length(len);
                state.scroll.set_viewport_height(1.min(len));
                state.scroll.scroll_to_end();
                None
            }
            EventStreamMessage::ToggleAutoScroll => {
                state.auto_scroll = !state.auto_scroll;
                if state.auto_scroll {
                    let len = state.visible_events().len();
                    state.scroll.set_content_length(len);
                    state.scroll.scroll_to_end();
                }
                None
            }
            EventStreamMessage::FocusSearch => {
                state.focus = Focus::Search;
                None
            }
            EventStreamMessage::FocusList => {
                state.focus = Focus::List;
                None
            }
            EventStreamMessage::SearchInput(c) => {
                state.search.update(InputFieldMessage::Insert(c));
                state.filter_text = state.search.value().to_string();
                state.scroll.set_offset(0);
                Some(EventStreamOutput::FilterChanged)
            }
            EventStreamMessage::SearchBackspace => {
                state.search.update(InputFieldMessage::Backspace);
                state.filter_text = state.search.value().to_string();
                state.scroll.set_offset(0);
                Some(EventStreamOutput::FilterChanged)
            }
            EventStreamMessage::SearchDelete => {
                state.search.update(InputFieldMessage::Delete);
                state.filter_text = state.search.value().to_string();
                state.scroll.set_offset(0);
                Some(EventStreamOutput::FilterChanged)
            }
            EventStreamMessage::SearchLeft => {
                state.search.update(InputFieldMessage::Left);
                None
            }
            EventStreamMessage::SearchRight => {
                state.search.update(InputFieldMessage::Right);
                None
            }
            EventStreamMessage::SearchHome => {
                state.search.update(InputFieldMessage::Home);
                None
            }
            EventStreamMessage::SearchEnd => {
                state.search.update(InputFieldMessage::End);
                None
            }
            EventStreamMessage::ClearSearch => {
                state.search.update(InputFieldMessage::Clear);
                state.filter_text.clear();
                state.scroll.set_offset(0);
                state.focus = Focus::List;
                Some(EventStreamOutput::FilterChanged)
            }
            EventStreamMessage::QuickLevelFilter(n) => {
                let level = match n {
                    1 => Some(EventLevel::Trace),
                    2 => Some(EventLevel::Debug),
                    3 => Some(EventLevel::Info),
                    4 => Some(EventLevel::Warning),
                    5 => Some(EventLevel::Error),
                    _ => None,
                };
                // Toggle: if already set to this level, clear it
                if state.level_filter == level {
                    state.level_filter = None;
                } else {
                    state.level_filter = level;
                }
                state.scroll.set_offset(0);
                Some(EventStreamOutput::FilterChanged)
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        render::render_event_stream(state, frame, area, theme, ctx.focused, ctx.disabled);
    }
}

#[cfg(test)]
mod event_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
