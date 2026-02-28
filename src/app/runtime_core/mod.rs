//! Shared runtime logic between sync and async runtimes.
//!
//! This module provides `RuntimeCore`, a struct containing the fields and methods
//! that are identical between `Runtime` and `AsyncRuntime`. Both runtimes embed
//! this struct and delegate shared operations to it.

use std::io;

use ratatui::backend::Backend;
use ratatui::Terminal;

use super::model::App;
use crate::input::EventQueue;
use crate::overlay::{Overlay, OverlayAction, OverlayStack};
use crate::theme::Theme;

/// Core runtime state shared between sync and async runtimes.
///
/// Contains the fields and methods that are identical across both runtime
/// implementations. Each runtime embeds this struct and delegates shared
/// operations to it.
pub(crate) struct RuntimeCore<A: App, B: Backend> {
    pub(crate) state: A::State,
    pub(crate) terminal: Terminal<B>,
    pub(crate) events: EventQueue,
    pub(crate) overlay_stack: OverlayStack<A::Message>,
    pub(crate) theme: Theme,
    pub(crate) should_quit: bool,
    pub(crate) max_messages_per_tick: usize,
}

impl<A: App, B: Backend> RuntimeCore<A, B> {
    /// Renders the current state to the terminal.
    ///
    /// Renders the main app view first, then any active overlays on top.
    pub(crate) fn render(&mut self) -> io::Result<()> {
        let theme = &self.theme;
        let overlay_stack = &self.overlay_stack;
        self.terminal.draw(|frame| {
            A::view(&self.state, frame);
            overlay_stack.render(frame, frame.area(), theme);
        })?;
        Ok(())
    }

    /// Processes the next event from the queue.
    ///
    /// If the overlay stack is active, events are routed through it first.
    /// Only if the overlay propagates the event will it reach the app's
    /// `handle_event_with_state`.
    ///
    /// Returns `Some(msg)` if a message should be dispatched, `None` if no event
    /// was available. The `bool` in the tuple indicates whether an event was processed.
    ///
    /// The caller must dispatch any returned message through its own `dispatch()` method,
    /// since dispatch logic differs between sync and async runtimes.
    pub(crate) fn process_event(&mut self) -> ProcessEventResult<A::Message> {
        if let Some(event) = self.events.pop() {
            match self.overlay_stack.handle_event(&event) {
                OverlayAction::Consumed => ProcessEventResult::Consumed,
                OverlayAction::Message(msg) => ProcessEventResult::Dispatch(msg),
                OverlayAction::Dismiss => {
                    self.overlay_stack.pop();
                    ProcessEventResult::Consumed
                }
                OverlayAction::DismissWithMessage(msg) => {
                    self.overlay_stack.pop();
                    ProcessEventResult::Dispatch(msg)
                }
                OverlayAction::Propagate => {
                    if let Some(msg) = A::handle_event_with_state(&self.state, &event) {
                        ProcessEventResult::Dispatch(msg)
                    } else {
                        ProcessEventResult::Consumed
                    }
                }
            }
        } else {
            ProcessEventResult::NoEvent
        }
    }

    /// Pushes an overlay onto the stack.
    pub(crate) fn push_overlay(&mut self, overlay: Box<dyn Overlay<A::Message>>) {
        self.overlay_stack.push(overlay);
    }

    /// Pops the topmost overlay from the stack.
    pub(crate) fn pop_overlay(&mut self) -> Option<Box<dyn Overlay<A::Message>>> {
        self.overlay_stack.pop()
    }

    /// Clears all overlays from the stack.
    pub(crate) fn clear_overlays(&mut self) {
        self.overlay_stack.clear();
    }

    /// Returns true if there are active overlays.
    pub(crate) fn has_overlays(&self) -> bool {
        self.overlay_stack.is_active()
    }

    /// Returns the number of overlays on the stack.
    pub(crate) fn overlay_count(&self) -> usize {
        self.overlay_stack.len()
    }
}

/// Result of processing a single event.
pub(crate) enum ProcessEventResult<M> {
    /// No event was available in the queue.
    NoEvent,
    /// An event was processed and consumed (no dispatch needed).
    Consumed,
    /// An event was processed and produced a message to dispatch.
    Dispatch(M),
}

#[cfg(test)]
mod tests;
