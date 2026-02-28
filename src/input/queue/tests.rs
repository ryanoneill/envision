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
    let events = vec![Event::char('a'), Event::key(KeyCode::Enter)];

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
