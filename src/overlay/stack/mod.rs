//! Overlay stack implementation.

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::input::Event;
use crate::theme::Theme;

use super::action::OverlayAction;
use super::traits::Overlay;

/// A stack of overlays managed by the runtime.
///
/// The stack renders overlays bottom-up (so the topmost draws last) and
/// processes events top-down (so the topmost gets first chance to handle).
pub struct OverlayStack<M> {
    layers: Vec<Box<dyn Overlay<M>>>,
}

impl<M> OverlayStack<M> {
    /// Creates a new empty overlay stack.
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Pushes an overlay onto the top of the stack.
    pub fn push(&mut self, overlay: Box<dyn Overlay<M>>) {
        self.layers.push(overlay);
    }

    /// Pops the topmost overlay from the stack.
    pub fn pop(&mut self) -> Option<Box<dyn Overlay<M>>> {
        self.layers.pop()
    }

    /// Clears all overlays from the stack.
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Returns true if there are active overlays.
    pub fn is_active(&self) -> bool {
        !self.layers.is_empty()
    }

    /// Returns true if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Returns the number of overlays on the stack.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Processes an event through the stack (top-down).
    ///
    /// Returns the action from the first overlay that doesn't Propagate,
    /// or Propagate if all overlays propagate (or stack is empty).
    pub(crate) fn handle_event(&mut self, event: &Event) -> OverlayAction<M> {
        // Process top-down (last element is topmost)
        for overlay in self.layers.iter_mut().rev() {
            match overlay.handle_event(event) {
                OverlayAction::Propagate => continue,
                action => return action,
            }
        }
        OverlayAction::Propagate
    }

    /// Renders all overlays bottom-up (so topmost draws last).
    pub(crate) fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        for overlay in &self.layers {
            overlay.view(frame, area, theme);
        }
    }
}

impl<M> Default for OverlayStack<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
