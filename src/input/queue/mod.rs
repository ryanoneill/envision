//! Event queue for managing simulated input events.

use std::collections::VecDeque;
use std::time::Duration;

use crossterm::event::KeyCode;

use super::events::Event;

/// A queue of simulated input events.
///
/// This provides a way to pre-load a sequence of events that can be
/// consumed by an application's event loop, enabling automated testing.
///
/// # Example
///
/// ```rust
/// use envision::input::{EventQueue, KeyCode};
///
/// let mut queue = EventQueue::new();
///
/// // Type "hello" then press Enter
/// queue.type_str("hello");
/// queue.key(KeyCode::Enter);
///
/// // Process events
/// assert_eq!(queue.len(), 6); // 5 chars + Enter
/// ```
#[derive(Clone, Debug, Default)]
pub struct EventQueue {
    events: VecDeque<Event>,
}

impl EventQueue {
    /// Creates a new empty event queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a queue with pre-loaded events.
    pub fn with_events(events: impl IntoIterator<Item = Event>) -> Self {
        Self {
            events: events.into_iter().collect(),
        }
    }

    /// Returns the number of events in the queue.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clears all events from the queue.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Adds an event to the end of the queue.
    pub fn push(&mut self, event: Event) {
        self.events.push_back(event);
    }

    /// Adds an event to the front of the queue (next to be consumed).
    pub fn push_front(&mut self, event: Event) {
        self.events.push_front(event);
    }

    /// Removes and returns the next event, or None if empty.
    pub fn pop(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    /// Returns a reference to the next event without removing it.
    pub fn peek(&self) -> Option<&Event> {
        self.events.front()
    }

    /// Adds a key event for a special key.
    pub fn key(&mut self, code: KeyCode) {
        self.push(Event::key(code));
    }

    /// Adds a character key event.
    pub fn char(&mut self, c: char) {
        self.push(Event::char(c));
    }

    /// Adds key events for each character in a string.
    pub fn type_str(&mut self, s: &str) {
        for c in s.chars() {
            self.push(Event::char(c));
        }
    }

    /// Adds a Ctrl+key event.
    pub fn ctrl(&mut self, c: char) {
        self.push(Event::ctrl(c));
    }

    /// Adds an Alt+key event.
    pub fn alt(&mut self, c: char) {
        self.push(Event::alt(c));
    }

    /// Adds an Enter key event.
    pub fn enter(&mut self) {
        self.key(KeyCode::Enter);
    }

    /// Adds an Escape key event.
    pub fn escape(&mut self) {
        self.key(KeyCode::Esc);
    }

    /// Adds a Tab key event.
    pub fn tab(&mut self) {
        self.key(KeyCode::Tab);
    }

    /// Adds a Backspace key event.
    pub fn backspace(&mut self) {
        self.key(KeyCode::Backspace);
    }

    /// Adds a Delete key event.
    pub fn delete(&mut self) {
        self.key(KeyCode::Delete);
    }

    /// Adds an Up arrow key event.
    pub fn up(&mut self) {
        self.key(KeyCode::Up);
    }

    /// Adds a Down arrow key event.
    pub fn down(&mut self) {
        self.key(KeyCode::Down);
    }

    /// Adds a Left arrow key event.
    pub fn left(&mut self) {
        self.key(KeyCode::Left);
    }

    /// Adds a Right arrow key event.
    pub fn right(&mut self) {
        self.key(KeyCode::Right);
    }

    /// Adds a Home key event.
    pub fn home(&mut self) {
        self.key(KeyCode::Home);
    }

    /// Adds an End key event.
    pub fn end(&mut self) {
        self.key(KeyCode::End);
    }

    /// Adds a Page Up key event.
    pub fn page_up(&mut self) {
        self.key(KeyCode::PageUp);
    }

    /// Adds a Page Down key event.
    pub fn page_down(&mut self) {
        self.key(KeyCode::PageDown);
    }

    /// Adds a function key event (F1-F12).
    pub fn function(&mut self, n: u8) {
        self.key(KeyCode::F(n));
    }

    /// Adds a mouse click event.
    pub fn click(&mut self, x: u16, y: u16) {
        self.push(Event::click(x, y));
    }

    /// Adds a mouse double-click (two clicks at same position).
    pub fn double_click(&mut self, x: u16, y: u16) {
        self.push(Event::click(x, y));
        self.push(Event::mouse_up(x, y));
        self.push(Event::click(x, y));
        self.push(Event::mouse_up(x, y));
    }

    /// Adds mouse events to simulate a drag from one position to another.
    pub fn drag(&mut self, from: (u16, u16), to: (u16, u16)) {
        self.push(Event::click(from.0, from.1));
        self.push(Event::mouse_drag(
            to.0,
            to.1,
            crossterm::event::MouseButton::Left,
        ));
        self.push(Event::mouse_up(to.0, to.1));
    }

    /// Adds a scroll up event.
    pub fn scroll_up(&mut self, x: u16, y: u16) {
        self.push(Event::scroll_up(x, y));
    }

    /// Adds a scroll down event.
    pub fn scroll_down(&mut self, x: u16, y: u16) {
        self.push(Event::scroll_down(x, y));
    }

    /// Adds a resize event.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.push(Event::Resize(width, height));
    }

    /// Adds a paste event.
    pub fn paste(&mut self, content: impl Into<String>) {
        self.push(Event::Paste(content.into()));
    }

    /// Returns an iterator over all events (without consuming them).
    pub fn iter(&self) -> impl Iterator<Item = &Event> {
        self.events.iter()
    }

    /// Drains all events from the queue.
    pub fn drain(&mut self) -> impl Iterator<Item = Event> + '_ {
        self.events.drain(..)
    }

    /// Polls for an event with a timeout.
    ///
    /// In simulation mode, this ignores the timeout and immediately
    /// returns the next event if available.
    pub fn poll(&mut self, _timeout: Duration) -> Option<Event> {
        self.pop()
    }

    /// Extends the queue with events from an iterator.
    pub fn extend(&mut self, events: impl IntoIterator<Item = Event>) {
        self.events.extend(events);
    }
}

impl FromIterator<Event> for EventQueue {
    fn from_iter<T: IntoIterator<Item = Event>>(iter: T) -> Self {
        Self {
            events: iter.into_iter().collect(),
        }
    }
}

impl Extend<Event> for EventQueue {
    fn extend<T: IntoIterator<Item = Event>>(&mut self, iter: T) {
        self.events.extend(iter);
    }
}

#[cfg(test)]
mod tests;
