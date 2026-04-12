use super::*;
use crate::input::{Event, Key};

#[test]
fn test_terminal_event_subscription_creation() {
    // Test that we can create a TerminalEventSubscription
    let _sub = TerminalEventSubscription::new(|event: Event| {
        if let Some(key) = event.as_key() {
            if key.key == Key::Char('q') {
                return Some(TestMsg::Quit);
            }
        }
        None
    });

    // Test the convenience function
    let _sub2 = terminal_events(|event: Event| {
        if let Some(key) = event.as_key() {
            if key.key == Key::Enter {
                return Some(TestMsg::Tick);
            }
        }
        None
    });
}

#[test]
fn test_terminal_event_handler_filters_events() {
    // Create handler that only responds to 'q'
    let handler = |event: Event| -> Option<TestMsg> {
        if let Some(key) = event.as_key() {
            if key.key == Key::Char('q') {
                return Some(TestMsg::Quit);
            }
        }
        None
    };

    // Test q key
    assert_eq!(handler(Event::char('q')), Some(TestMsg::Quit));

    // Test other key (should be None)
    assert_eq!(handler(Event::char('a')), None);

    // Test resize event (should be None)
    assert_eq!(handler(Event::Resize(80, 24)), None);
}

#[test]
fn test_terminal_event_handler_with_modifiers() {
    // Create handler that responds to Ctrl+C
    let handler = |event: Event| -> Option<TestMsg> {
        if let Some(key) = event.as_key() {
            if key.key == Key::Char('c') && key.modifiers.ctrl() {
                return Some(TestMsg::Quit);
            }
        }
        None
    };

    // Test Ctrl+C
    assert_eq!(handler(Event::ctrl('c')), Some(TestMsg::Quit));

    // Test plain 'c' (should be None)
    assert_eq!(handler(Event::char('c')), None);
}

#[test]
fn test_terminal_event_handler_resize() {
    #[derive(Debug, Clone, PartialEq)]
    enum ResizeMsg {
        Resize(u16, u16),
    }

    let handler = |event: Event| -> Option<ResizeMsg> {
        if let Event::Resize(width, height) = event {
            Some(ResizeMsg::Resize(width, height))
        } else {
            None
        }
    };

    let resize_event = Event::Resize(120, 40);
    assert_eq!(handler(resize_event), Some(ResizeMsg::Resize(120, 40)));

    // Key event should be None
    assert_eq!(handler(Event::key(Key::Enter)), None);
}

// Note: We can't test TerminalEventSubscription::into_stream in unit tests
// because crossterm's EventStream requires a real terminal to be attached.
// The handler logic is tested through the test_terminal_event_* tests above
// which verify the event handling works correctly.

#[derive(Clone, Debug, PartialEq)]
enum TestMsgWithQuit {
    Quit,
    Key(char),
}

#[test]
fn test_terminal_events_convenience_function() {
    let sub = terminal_events(|event: Event| -> Option<TestMsgWithQuit> {
        if let Some(key) = event.as_key() {
            if key.key == Key::Char('q') {
                return Some(TestMsgWithQuit::Quit);
            }
            if let Some(c) = key.raw_char {
                return Some(TestMsgWithQuit::Key(c));
            }
        }
        None
    });

    // Verify the handler works correctly by testing it directly
    assert_eq!(
        (sub.event_handler)(Event::char('q')),
        Some(TestMsgWithQuit::Quit)
    );
    assert_eq!(
        (sub.event_handler)(Event::char('a')),
        Some(TestMsgWithQuit::Key('a'))
    );
}
