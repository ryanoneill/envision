//! A conversation display for AI/LLM interactions with structured message blocks.
//!
//! [`ConversationView`] provides a scrollable, read-only view of a conversation
//! composed of structured messages. Each message has a [`ConversationRole`] (User,
//! Assistant, System, Tool) and contains one or more [`MessageBlock`]s (Text, Code,
//! ToolUse, Thinking, Error).
//!
//! Unlike [`ChatView`](super::ChatView) which includes an input field, this component
//! is purely a display widget for viewing conversation history, making it suitable
//! for read-only conversation logs, streaming AI responses, and tool-use displays.
//!
//! State is stored in [`ConversationViewState`], updated via [`ConversationViewMessage`],
//! and produces [`ConversationViewOutput`].
//!
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, ConversationView, ConversationViewState,
//!     ConversationMessage, ConversationRole, MessageBlock,
//! };
//!
//! let mut state = ConversationViewState::new();
//! state.push_message(ConversationMessage::new(ConversationRole::User, "Hello!"));
//! state.push_message(ConversationMessage::with_blocks(
//!     ConversationRole::Assistant,
//!     vec![
//!         MessageBlock::text("Here is some code:"),
//!         MessageBlock::code("fn main() {}", Some("rust")),
//!     ],
//! ));
//!
//! assert_eq!(state.message_count(), 2);
//! ```

mod render;
mod state;
pub mod types;

use types::MessageSource;
pub use types::{ConversationMessage, ConversationRole, MessageBlock, MessageHandle};

use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use ratatui::prelude::*;

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};
use crate::scroll::ScrollState;

impl MessageSource for ConversationViewState {
    fn source_messages(&self) -> &[ConversationMessage] {
        &self.messages
    }
}

/// Messages that can be sent to a ConversationView.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ConversationViewMessage {
    /// Scroll up by one line.
    ScrollUp,
    /// Scroll down by one line.
    ScrollDown,
    /// Scroll to the top of the conversation.
    ScrollToTop,
    /// Scroll to the bottom of the conversation.
    ScrollToBottom,
    /// Page up (scroll by viewport height).
    PageUp,
    /// Page down (scroll by viewport height).
    PageDown,
    /// Toggle collapse state of a named block.
    ToggleCollapse(String),
}

/// Output messages from a ConversationView.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ConversationViewOutput {
    /// The scroll position changed.
    ScrollChanged {
        /// The current scroll offset.
        offset: usize,
    },
}

/// State for a ConversationView component.
///
/// Contains the message history, scroll state, and display configuration.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ConversationViewState {
    /// Message history.
    pub(super) messages: Vec<ConversationMessage>,
    /// Scroll state for the conversation.
    pub(super) scroll: ScrollState,
    /// Whether to auto-scroll to bottom on new messages.
    pub(super) auto_scroll: bool,
    /// Maximum number of messages to keep.
    pub(super) max_messages: usize,
    /// Whether to show timestamps.
    pub(super) show_timestamps: bool,
    /// Whether to show role labels/headers.
    pub(super) show_role_labels: bool,
    /// Whether to render text blocks as markdown (requires `markdown` feature).
    pub(super) markdown_enabled: bool,
    /// Last known render width for scroll content length estimation.
    pub(super) last_known_width: usize,
    /// Optional title for the conversation panel.
    pub(super) title: Option<String>,
    /// Set of collapsed block keys (e.g., "tool:search", "thinking").
    pub(super) collapsed_blocks: HashSet<String>,
    /// Optional status text rendered at the bottom of the viewport, above the border.
    pub(super) status: Option<String>,
    /// Next unique ID for message handles.
    #[cfg_attr(feature = "serialization", serde(skip, default))]
    pub(super) next_id: u64,
    /// Per-role style overrides. When set, these take precedence over
    /// the default colors from [`ConversationRole::color()`].
    #[cfg_attr(feature = "serialization", serde(skip, default))]
    pub(super) role_style_overrides: HashMap<ConversationRole, Style>,
}

impl Default for ConversationViewState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            scroll: ScrollState::default(),
            auto_scroll: true,
            max_messages: 1000,
            show_timestamps: false,
            show_role_labels: true,
            markdown_enabled: false,
            last_known_width: 80,
            title: None,
            collapsed_blocks: HashSet::new(),
            status: None,
            next_id: 1,
            role_style_overrides: HashMap::new(),
        }
    }
}

impl PartialEq for ConversationViewState {
    fn eq(&self, other: &Self) -> bool {
        self.messages == other.messages
            && self.scroll == other.scroll
            && self.auto_scroll == other.auto_scroll
            && self.max_messages == other.max_messages
            && self.show_timestamps == other.show_timestamps
            && self.show_role_labels == other.show_role_labels
            && self.title == other.title
            && self.collapsed_blocks == other.collapsed_blocks
            && self.status == other.status
            && self.role_style_overrides == other.role_style_overrides
        // next_id is intentionally excluded from equality
    }
}

/// A read-only conversation display for AI/LLM interactions.
///
/// This component renders a scrollable list of conversation messages with
/// structured content blocks. It supports:
///
/// - **Role headers** with colored indicators (User, Assistant, System, Tool)
/// - **Code blocks** with left border styling
/// - **Collapsible** tool use and thinking blocks
/// - **Streaming cursor** for in-progress messages
/// - **Scrollbar** for long conversations
///
/// # Key Bindings
///
/// - `Up` / `k` -- Scroll up
/// - `Down` / `j` -- Scroll down
/// - `Home` / `g` -- Scroll to top
/// - `End` / `G` -- Scroll to bottom
/// - `PageUp` -- Page up
/// - `PageDown` -- Page down
pub struct ConversationView(PhantomData<()>);

impl Component for ConversationView {
    type State = ConversationViewState;
    type Message = ConversationViewMessage;
    type Output = ConversationViewOutput;

    fn init() -> Self::State {
        ConversationViewState::default()
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;

        match key.code {
            Key::Up | Key::Char('k') => Some(ConversationViewMessage::ScrollUp),
            Key::Down | Key::Char('j') => Some(ConversationViewMessage::ScrollDown),
            Key::Char('g') if key.modifiers.shift() => {
                Some(ConversationViewMessage::ScrollToBottom)
            }
            Key::Home | Key::Char('g') => Some(ConversationViewMessage::ScrollToTop),
            Key::End => Some(ConversationViewMessage::ScrollToBottom),
            Key::PageUp => Some(ConversationViewMessage::PageUp),
            Key::PageDown => Some(ConversationViewMessage::PageDown),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            ConversationViewMessage::ScrollUp => {
                state.update_scroll_content_length();
                state.scroll.scroll_up();
                state.auto_scroll = false;
                Some(ConversationViewOutput::ScrollChanged {
                    offset: state.scroll.offset(),
                })
            }
            ConversationViewMessage::ScrollDown => {
                state.update_scroll_content_length();
                state.scroll.scroll_down();
                if state.scroll.at_end() {
                    state.auto_scroll = true;
                }
                Some(ConversationViewOutput::ScrollChanged {
                    offset: state.scroll.offset(),
                })
            }
            ConversationViewMessage::ScrollToTop => {
                state.update_scroll_content_length();
                state.scroll.scroll_to_start();
                state.auto_scroll = false;
                Some(ConversationViewOutput::ScrollChanged {
                    offset: state.scroll.offset(),
                })
            }
            ConversationViewMessage::ScrollToBottom => {
                state.update_scroll_content_length();
                state.scroll.scroll_to_end();
                state.auto_scroll = true;
                Some(ConversationViewOutput::ScrollChanged {
                    offset: state.scroll.offset(),
                })
            }
            ConversationViewMessage::PageUp => {
                state.update_scroll_content_length();
                let page_size = state.scroll.viewport_height().max(1);
                state.scroll.page_up(page_size);
                state.auto_scroll = false;
                Some(ConversationViewOutput::ScrollChanged {
                    offset: state.scroll.offset(),
                })
            }
            ConversationViewMessage::PageDown => {
                state.update_scroll_content_length();
                let page_size = state.scroll.viewport_height().max(1);
                state.scroll.page_down(page_size);
                if state.scroll.at_end() {
                    state.auto_scroll = true;
                }
                Some(ConversationViewOutput::ScrollChanged {
                    offset: state.scroll.offset(),
                })
            }
            ConversationViewMessage::ToggleCollapse(key) => {
                state.toggle_collapse(&key);
                None
            }
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        if ctx.area.height < 3 || ctx.area.width < 5 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.open(
                ctx.area,
                crate::annotation::Annotation::container("conversation_view")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        render::render(
            state,
            ctx.frame,
            ctx.area,
            ctx.theme,
            ctx.focused,
            ctx.disabled,
        );

        crate::annotation::with_registry(|reg| {
            reg.close();
        });
    }
}

#[cfg(test)]
mod render_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
