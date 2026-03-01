use super::*;

#[test]
fn test_terminal_event_subscription_creation() {
    use crossterm::event::{Event, KeyCode, KeyEvent};

    // Test that we can create a TerminalEventSubscription
    let _sub = TerminalEventSubscription::new(|event| {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            ..
        }) = event
        {
            Some(TestMsg::Quit)
        } else {
            None
        }
    });

    // Test the convenience function
    let _sub2 = terminal_events(|event| {
        if let Event::Key(KeyEvent {
            code: KeyCode::Enter,
            ..
        }) = event
        {
            Some(TestMsg::Tick)
        } else {
            None
        }
    });
}

#[test]
fn test_terminal_event_handler_filters_events() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    // Create handler that only responds to 'q'
    let handler = |event: Event| -> Option<TestMsg> {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            ..
        }) = event
        {
            Some(TestMsg::Quit)
        } else {
            None
        }
    };

    // Test q key
    let q_event = Event::Key(KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    });
    assert_eq!(handler(q_event), Some(TestMsg::Quit));

    // Test other key (should be None)
    let a_event = Event::Key(KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    });
    assert_eq!(handler(a_event), None);

    // Test resize event (should be None)
    let resize_event = Event::Resize(80, 24);
    assert_eq!(handler(resize_event), None);
}

#[test]
fn test_terminal_event_handler_with_modifiers() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    // Create handler that responds to Ctrl+C
    let handler = |event: Event| -> Option<TestMsg> {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers,
            ..
        }) = event
        {
            if modifiers.contains(KeyModifiers::CONTROL) {
                Some(TestMsg::Quit)
            } else {
                None
            }
        } else {
            None
        }
    };

    // Test Ctrl+C
    let ctrl_c = Event::Key(KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    });
    assert_eq!(handler(ctrl_c), Some(TestMsg::Quit));

    // Test plain 'c' (should be None)
    let plain_c = Event::Key(KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    });
    assert_eq!(handler(plain_c), None);
}

#[test]
fn test_terminal_event_handler_resize() {
    use crossterm::event::Event;

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
    let key_event = Event::Key(crossterm::event::KeyEvent {
        code: crossterm::event::KeyCode::Enter,
        modifiers: crossterm::event::KeyModifiers::empty(),
        kind: crossterm::event::KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    });
    assert_eq!(handler(key_event), None);
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
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    let sub = terminal_events(|event: Event| -> Option<TestMsgWithQuit> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => Some(TestMsgWithQuit::Quit),
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => Some(TestMsgWithQuit::Key(c)),
            _ => None,
        }
    });

    // Verify the handler works correctly by testing it directly
    let q_event = Event::Key(KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    });
    assert_eq!((sub.event_handler)(q_event), Some(TestMsgWithQuit::Quit));
}
