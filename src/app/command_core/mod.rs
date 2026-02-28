//! Core command handler logic.
//!
//! This module provides `CommandHandlerCore`, a struct containing the fields
//! and methods for managing sync command results used by `CommandHandler`.

use crate::overlay::Overlay;

use super::command::CommandAction;

/// Core command handler state.
///
/// Contains the fields and methods for managing sync command results
/// (messages, overlay operations, quit flag).
pub(crate) struct CommandHandlerCore<M> {
    pub(crate) pending_messages: Vec<M>,
    pub(crate) pending_overlay_pushes: Vec<Box<dyn Overlay<M> + Send>>,
    pub(crate) pending_overlay_pops: usize,
    pub(crate) should_quit: bool,
}

impl<M> CommandHandlerCore<M> {
    /// Creates a new core handler.
    pub(crate) fn new() -> Self {
        Self {
            pending_messages: Vec::new(),
            pending_overlay_pushes: Vec::new(),
            pending_overlay_pops: 0,
            should_quit: false,
        }
    }

    /// Processes a command action, collecting messages and overlay operations.
    ///
    /// Returns `None` if the action was handled (sync action), or `Some(action)` if
    /// the action is async and needs to be handled by the caller.
    pub(crate) fn execute_action(
        &mut self,
        action: CommandAction<M>,
    ) -> Option<CommandAction<M>> {
        match action {
            CommandAction::Message(m) => {
                self.pending_messages.push(m);
                None
            }
            CommandAction::Batch(msgs) => {
                self.pending_messages.extend(msgs);
                None
            }
            CommandAction::Quit => {
                self.should_quit = true;
                None
            }
            CommandAction::Callback(cb) => {
                if let Some(m) = cb() {
                    self.pending_messages.push(m);
                }
                None
            }
            CommandAction::PushOverlay(overlay) => {
                self.pending_overlay_pushes.push(overlay);
                None
            }
            CommandAction::PopOverlay => {
                self.pending_overlay_pops += 1;
                None
            }
            async_action @ (CommandAction::Async(_) | CommandAction::AsyncFallible(_)) => {
                Some(async_action)
            }
        }
    }

    /// Takes all pending messages.
    pub(crate) fn take_messages(&mut self) -> Vec<M> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Takes all pending overlay pushes.
    pub(crate) fn take_overlay_pushes(&mut self) -> Vec<Box<dyn Overlay<M> + Send>> {
        std::mem::take(&mut self.pending_overlay_pushes)
    }

    /// Takes the count of pending overlay pops and resets the counter.
    pub(crate) fn take_overlay_pops(&mut self) -> usize {
        std::mem::replace(&mut self.pending_overlay_pops, 0)
    }

    /// Returns true if a quit command was executed.
    pub(crate) fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Resets the quit flag.
    pub(crate) fn reset_quit(&mut self) {
        self.should_quit = false;
    }
}

#[cfg(test)]
mod tests;
