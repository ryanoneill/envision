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
mod tests {
    use super::*;

    #[test]
    fn test_new_queue_is_empty() {
        let queue = EventQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_push_pop() {
        let mut queue = EventQueue::new();
        queue.push(Event::char('a'));
        queue.push(Event::char('b'));

        assert_eq!(queue.len(), 2);

        let e1 = queue.pop().unwrap();
        assert_eq!(e1, Event::char('a'));

        let e2 = queue.pop().unwrap();
        assert_eq!(e2, Event::char('b'));

        assert!(queue.is_empty());
    }

    #[test]
    fn test_type_str() {
        let mut queue = EventQueue::new();
        queue.type_str("hi");

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.pop(), Some(Event::char('h')));
        assert_eq!(queue.pop(), Some(Event::char('i')));
    }

    #[test]
    fn test_convenience_methods() {
        let mut queue = EventQueue::new();
        queue.enter();
        queue.escape();
        queue.tab();
        queue.backspace();

        assert_eq!(queue.len(), 4);
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::Enter)));
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::Esc)));
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::Tab)));
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::Backspace)));
    }

    #[test]
    fn test_arrow_keys() {
        let mut queue = EventQueue::new();
        queue.up();
        queue.down();
        queue.left();
        queue.right();

        assert_eq!(queue.len(), 4);
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::Up)));
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::Down)));
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::Left)));
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::Right)));
    }

    #[test]
    fn test_ctrl_alt() {
        let mut queue = EventQueue::new();
        queue.ctrl('c');
        queue.alt('x');

        let e1 = queue.pop().unwrap();
        assert_eq!(e1, Event::ctrl('c'));

        let e2 = queue.pop().unwrap();
        assert_eq!(e2, Event::alt('x'));
    }

    #[test]
    fn test_mouse_events() {
        let mut queue = EventQueue::new();
        queue.click(10, 20);
        queue.scroll_up(5, 5);

        assert_eq!(queue.len(), 2);

        let click = queue.pop().unwrap();
        assert!(click.is_mouse());
        let mouse = click.as_mouse().unwrap();
        assert_eq!(mouse.column, 10);
        assert_eq!(mouse.row, 20);
    }

    #[test]
    fn test_peek() {
        let mut queue = EventQueue::new();
        queue.char('x');

        assert_eq!(queue.peek(), Some(&Event::char('x')));
        assert_eq!(queue.len(), 1); // Not consumed

        queue.pop();
        assert_eq!(queue.peek(), None);
    }

    #[test]
    fn test_with_events() {
        let events = vec![
            Event::char('a'),
            Event::key(KeyCode::Enter),
        ];

        let queue = EventQueue::with_events(events);
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_from_iterator() {
        let queue: EventQueue = vec![Event::char('a'), Event::char('b')]
            .into_iter()
            .collect();

        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut queue = EventQueue::new();
        queue.type_str("hello");
        assert_eq!(queue.len(), 5);

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_push_front() {
        let mut queue = EventQueue::new();
        queue.char('b');
        queue.push_front(Event::char('a'));

        assert_eq!(queue.pop(), Some(Event::char('a')));
        assert_eq!(queue.pop(), Some(Event::char('b')));
    }

    #[test]
    fn test_double_click() {
        let mut queue = EventQueue::new();
        queue.double_click(10, 10);

        assert_eq!(queue.len(), 4); // down, up, down, up
    }

    #[test]
    fn test_drag() {
        let mut queue = EventQueue::new();
        queue.drag((0, 0), (10, 10));

        assert_eq!(queue.len(), 3); // down, drag, up
    }

    #[test]
    fn test_delete() {
        let mut queue = EventQueue::new();
        queue.delete();

        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::Delete)));
    }

    #[test]
    fn test_home_end() {
        let mut queue = EventQueue::new();
        queue.home();
        queue.end();

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::Home)));
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::End)));
    }

    #[test]
    fn test_page_up_down() {
        let mut queue = EventQueue::new();
        queue.page_up();
        queue.page_down();

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::PageUp)));
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::PageDown)));
    }

    #[test]
    fn test_function_keys() {
        let mut queue = EventQueue::new();
        queue.function(1);
        queue.function(12);

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::F(1))));
        assert_eq!(queue.pop(), Some(Event::key(KeyCode::F(12))));
    }

    #[test]
    fn test_scroll_down() {
        let mut queue = EventQueue::new();
        queue.scroll_down(5, 10);

        assert_eq!(queue.len(), 1);
        let event = queue.pop().unwrap();
        assert!(event.is_mouse());
    }

    #[test]
    fn test_resize() {
        let mut queue = EventQueue::new();
        queue.resize(120, 40);

        assert_eq!(queue.len(), 1);
        if let Some(Event::Resize(w, h)) = queue.pop() {
            assert_eq!(w, 120);
            assert_eq!(h, 40);
        } else {
            panic!("Expected Resize event");
        }
    }

    #[test]
    fn test_paste() {
        let mut queue = EventQueue::new();
        queue.paste("pasted text");

        assert_eq!(queue.len(), 1);
        if let Some(Event::Paste(content)) = queue.pop() {
            assert_eq!(content, "pasted text");
        } else {
            panic!("Expected Paste event");
        }
    }

    #[test]
    fn test_iter() {
        let mut queue = EventQueue::new();
        queue.char('a');
        queue.char('b');
        queue.char('c');

        let events: Vec<_> = queue.iter().collect();
        assert_eq!(events.len(), 3);

        // Queue should still have all events
        assert_eq!(queue.len(), 3);
    }

    #[test]
    fn test_drain() {
        let mut queue = EventQueue::new();
        queue.char('x');
        queue.char('y');

        let drained: Vec<_> = queue.drain().collect();
        assert_eq!(drained.len(), 2);

        // Queue should now be empty
        assert!(queue.is_empty());
    }

    #[test]
    fn test_poll() {
        let mut queue = EventQueue::new();
        queue.char('p');

        let event = queue.poll(Duration::from_millis(100));
        assert_eq!(event, Some(Event::char('p')));

        // Empty queue returns None
        let event = queue.poll(Duration::from_millis(100));
        assert!(event.is_none());
    }

    #[test]
    fn test_extend() {
        let mut queue = EventQueue::new();
        queue.char('a');

        let more_events = vec![Event::char('b'), Event::char('c')];
        queue.extend(more_events);

        assert_eq!(queue.len(), 3);
    }

    #[test]
    fn test_extend_trait() {
        let mut queue = EventQueue::new();

        let events = vec![Event::char('x'), Event::char('y')];

        // Using Extend trait
        <EventQueue as Extend<Event>>::extend(&mut queue, events);

        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_queue_clone() {
        let mut queue = EventQueue::new();
        queue.char('a');
        queue.char('b');

        let cloned = queue.clone();
        assert_eq!(queue.len(), cloned.len());
    }

    #[test]
    fn test_queue_debug() {
        let queue = EventQueue::new();
        let debug = format!("{:?}", queue);
        assert!(debug.contains("EventQueue"));
    }

    #[test]
    fn test_queue_default() {
        let queue = EventQueue::default();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_key_method() {
        let mut queue = EventQueue::new();
        queue.key(KeyCode::Insert);

        assert_eq!(queue.pop(), Some(Event::key(KeyCode::Insert)));
    }
}
