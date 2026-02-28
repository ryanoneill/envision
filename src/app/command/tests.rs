use super::*;

#[derive(Clone, Debug, PartialEq)]
enum TestMsg {
    A,
    B,
    C,
    Value(i32),
}

#[test]
fn test_command_none() {
    let cmd: Command<TestMsg> = Command::none();
    assert!(cmd.is_none());
}

#[test]
fn test_command_message() {
    let cmd = Command::message(TestMsg::A);
    assert!(!cmd.is_none());
}

#[test]
fn test_command_batch() {
    let cmd = Command::batch([TestMsg::A, TestMsg::B, TestMsg::C]);
    assert!(!cmd.is_none());
}

#[test]
fn test_command_handler_message() {
    let mut handler = CommandHandler::new();
    handler.execute(Command::message(TestMsg::A));

    let messages = handler.take_messages();
    assert_eq!(messages, vec![TestMsg::A]);
}

#[test]
fn test_command_handler_batch() {
    let mut handler = CommandHandler::new();
    handler.execute(Command::batch([TestMsg::A, TestMsg::B]));

    let messages = handler.take_messages();
    assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
}

#[test]
fn test_command_handler_quit() {
    let mut handler: CommandHandler<TestMsg> = CommandHandler::new();
    assert!(!handler.should_quit());

    handler.execute(Command::quit());
    assert!(handler.should_quit());
}

#[test]
fn test_command_combine() {
    let cmd = Command::combine([Command::message(TestMsg::A), Command::message(TestMsg::B)]);

    let mut handler = CommandHandler::new();
    handler.execute(cmd);

    let messages = handler.take_messages();
    assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
}

#[test]
fn test_command_and() {
    let cmd = Command::message(TestMsg::A).and(Command::message(TestMsg::B));

    let mut handler = CommandHandler::new();
    handler.execute(cmd);

    let messages = handler.take_messages();
    assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
}

#[test]
fn test_command_perform() {
    let cmd = Command::perform(|| Some(TestMsg::A));

    let mut handler = CommandHandler::new();
    handler.execute(cmd);

    let messages = handler.take_messages();
    assert_eq!(messages, vec![TestMsg::A]);
}

#[test]
fn test_command_perform_none() {
    let cmd: Command<TestMsg> = Command::perform(|| None);

    let mut handler = CommandHandler::new();
    handler.execute(cmd);

    let messages = handler.take_messages();
    assert!(messages.is_empty());
}

#[test]
fn test_command_map() {
    #[derive(Clone, Debug, PartialEq)]
    enum OuterMsg {
        Inner(TestMsg),
    }

    let cmd = Command::message(TestMsg::A);
    let mapped = cmd.map(OuterMsg::Inner);

    let mut handler = CommandHandler::new();
    handler.execute(mapped);

    let messages = handler.take_messages();
    assert_eq!(messages, vec![OuterMsg::Inner(TestMsg::A)]);
}

#[test]
fn test_command_batch_empty() {
    let cmd: Command<TestMsg> = Command::batch(Vec::new());
    assert!(cmd.is_none());
}

#[test]
fn test_command_clone_message() {
    let cmd = Command::message(TestMsg::A);
    let cloned = cmd.clone();

    let mut handler = CommandHandler::new();
    handler.execute(cloned);

    let messages = handler.take_messages();
    assert_eq!(messages, vec![TestMsg::A]);
}

#[test]
fn test_command_clone_batch() {
    let cmd = Command::batch([TestMsg::A, TestMsg::B]);
    let cloned = cmd.clone();

    let mut handler = CommandHandler::new();
    handler.execute(cloned);

    let messages = handler.take_messages();
    assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
}

#[test]
fn test_command_clone_quit() {
    let cmd: Command<TestMsg> = Command::quit();
    let cloned = cmd.clone();

    let mut handler = CommandHandler::new();
    handler.execute(cloned);

    assert!(handler.should_quit());
}

#[test]
fn test_command_clone_callback_skipped() {
    let cmd = Command::perform(|| Some(TestMsg::A));
    let cloned = cmd.clone();

    // Callbacks can't be cloned, so cloned should have no actions
    assert!(cloned.is_none());
}

#[test]
fn test_command_debug() {
    let cmd = Command::message(TestMsg::A);
    let debug_str = format!("{:?}", cmd);

    assert!(debug_str.contains("Command"));
    assert!(debug_str.contains("action_count"));
    assert!(debug_str.contains("1"));
}

#[test]
fn test_command_map_batch() {
    #[derive(Clone, Debug, PartialEq)]
    enum OuterMsg {
        Inner(TestMsg),
    }

    let cmd = Command::batch([TestMsg::A, TestMsg::B]);
    let mapped = cmd.map(OuterMsg::Inner);

    let mut handler = CommandHandler::new();
    handler.execute(mapped);

    let messages = handler.take_messages();
    assert_eq!(
        messages,
        vec![OuterMsg::Inner(TestMsg::A), OuterMsg::Inner(TestMsg::B)]
    );
}

#[test]
fn test_command_map_quit() {
    #[derive(Clone, Debug, PartialEq)]
    enum OuterMsg {
        Inner(TestMsg),
    }

    let cmd: Command<TestMsg> = Command::quit();
    let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

    let mut handler = CommandHandler::new();
    handler.execute(mapped);

    assert!(handler.should_quit());
}

#[test]
fn test_command_map_callback() {
    #[derive(Clone, Debug, PartialEq)]
    enum OuterMsg {
        Inner(TestMsg),
    }

    let cmd = Command::perform(|| Some(TestMsg::A));
    let mapped = cmd.map(OuterMsg::Inner);

    let mut handler = CommandHandler::new();
    handler.execute(mapped);

    let messages = handler.take_messages();
    assert_eq!(messages, vec![OuterMsg::Inner(TestMsg::A)]);
}

#[test]
fn test_command_map_callback_none() {
    #[derive(Clone, Debug, PartialEq)]
    enum OuterMsg {
        Inner(TestMsg),
    }

    let cmd: Command<TestMsg> = Command::perform(|| None);
    let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

    let mut handler = CommandHandler::new();
    handler.execute(mapped);

    let messages = handler.take_messages();
    assert!(messages.is_empty());
}

#[test]
fn test_command_handler_reset_quit() {
    let mut handler: CommandHandler<TestMsg> = CommandHandler::new();
    handler.execute(Command::quit());
    assert!(handler.should_quit());

    handler.reset_quit();
    assert!(!handler.should_quit());
}

#[test]
fn test_command_handler_default() {
    let mut handler: CommandHandler<TestMsg> = CommandHandler::default();
    assert!(!handler.should_quit());
    assert!(handler.take_messages().is_empty());
}

#[test]
fn test_command_perform_async() {
    let cmd: Command<TestMsg> = Command::perform_async(async { Some(TestMsg::A) });

    // Async commands are not empty
    assert!(!cmd.is_none());

    // Sync handler skips async actions
    let mut handler = CommandHandler::new();
    handler.execute(cmd);
    assert!(handler.take_messages().is_empty());
}

#[test]
fn test_command_future_alias() {
    let cmd: Command<TestMsg> = Command::future(async { Some(TestMsg::A) });

    // Should behave identically to perform_async
    assert!(!cmd.is_none());

    // Sync handler skips async actions
    let mut handler = CommandHandler::new();
    handler.execute(cmd);
    assert!(handler.take_messages().is_empty());
}

#[test]
fn test_command_perform_async_none() {
    let cmd: Command<TestMsg> = Command::perform_async(async { None });

    assert!(!cmd.is_none());
}

#[test]
fn test_command_perform_async_fallible_ok() {
    let cmd: Command<TestMsg> = Command::perform_async_fallible(
        async { Ok::<_, std::io::Error>(42) },
        |result| match result {
            Ok(n) => TestMsg::Value(n),
            Err(_) => TestMsg::A,
        },
    );

    assert!(!cmd.is_none());
}

#[test]
fn test_command_perform_async_fallible_err() {
    let cmd: Command<TestMsg> = Command::perform_async_fallible(
        async { Err::<i32, _>(std::io::Error::other("test")) },
        |result| match result {
            Ok(n) => TestMsg::Value(n),
            Err(_) => TestMsg::B,
        },
    );

    assert!(!cmd.is_none());
}

#[test]
fn test_command_clone_async_skipped() {
    let cmd: Command<TestMsg> = Command::perform_async(async { Some(TestMsg::A) });
    let cloned = cmd.clone();

    // Async actions can't be cloned, so cloned should be empty
    assert!(cloned.is_none());
}

#[test]
fn test_command_map_async() {
    #[derive(Clone, Debug, PartialEq)]
    enum OuterMsg {
        Inner(TestMsg),
    }

    let cmd: Command<TestMsg> = Command::perform_async(async { Some(TestMsg::A) });
    let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

    // Mapped async command should still exist
    assert!(!mapped.is_none());
}

#[test]
fn test_command_handler_skips_async() {
    let cmd: Command<TestMsg> = Command::perform_async(async { Some(TestMsg::A) });

    let mut handler = CommandHandler::new();
    handler.execute(cmd);

    // Sync handler ignores async actions
    assert!(handler.take_messages().is_empty());
    assert!(!handler.should_quit());
}

#[test]
fn test_command_combine_with_async() {
    let cmd = Command::combine([
        Command::message(TestMsg::A),
        Command::perform_async(async { Some(TestMsg::B) }),
        Command::message(TestMsg::C),
    ]);

    let mut handler = CommandHandler::new();
    handler.execute(cmd);

    // Only sync messages are processed
    let messages = handler.take_messages();
    assert_eq!(messages, vec![TestMsg::A, TestMsg::C]);
}

#[test]
fn test_command_and_with_async() {
    let cmd = Command::message(TestMsg::A)
        .and(Command::perform_async(async { Some(TestMsg::B) }))
        .and(Command::quit());

    let mut handler = CommandHandler::new();
    handler.execute(cmd);

    let messages = handler.take_messages();
    assert_eq!(messages, vec![TestMsg::A]);
    assert!(handler.should_quit());
}

#[test]
fn test_command_try_perform_async_ok() {
    let cmd: Command<TestMsg> =
        Command::try_perform_async(async { Ok::<_, std::io::Error>(42) }, |n| {
            Some(TestMsg::Value(n))
        });

    // Command should not be empty
    assert!(!cmd.is_none());

    // Sync handler skips async actions
    let mut handler = CommandHandler::new();
    handler.execute(cmd);
    assert!(handler.take_messages().is_empty());
}

#[test]
fn test_command_try_perform_async_err() {
    let cmd: Command<TestMsg> = Command::try_perform_async(
        async { Err::<i32, _>(std::io::Error::other("test error")) },
        |n| Some(TestMsg::Value(n)),
    );

    assert!(!cmd.is_none());
}

#[test]
fn test_command_try_perform_async_returns_none() {
    let cmd: Command<TestMsg> =
        Command::try_perform_async(async { Ok::<_, std::io::Error>(42) }, |_n| None);

    assert!(!cmd.is_none());
}

#[test]
fn test_command_map_async_fallible() {
    #[derive(Clone, Debug, PartialEq)]
    enum OuterMsg {
        Inner(TestMsg),
    }

    let cmd: Command<TestMsg> =
        Command::try_perform_async(async { Ok::<_, std::io::Error>(42) }, |n| {
            Some(TestMsg::Value(n))
        });

    let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

    // Mapped command should still exist
    assert!(!mapped.is_none());
}

#[test]
fn test_command_clone_async_fallible_skipped() {
    let cmd: Command<TestMsg> =
        Command::try_perform_async(async { Ok::<_, std::io::Error>(42) }, |n| {
            Some(TestMsg::Value(n))
        });

    let cloned = cmd.clone();

    // AsyncFallible can't be cloned, so cloned should be empty
    assert!(cloned.is_none());
}

#[test]
fn test_command_combine_with_async_fallible() {
    let cmd = Command::combine([
        Command::message(TestMsg::A),
        Command::try_perform_async(async { Ok::<_, std::io::Error>(42) }, |n| {
            Some(TestMsg::Value(n))
        }),
        Command::message(TestMsg::C),
    ]);

    let mut handler = CommandHandler::new();
    handler.execute(cmd);

    // Only sync messages are processed
    let messages = handler.take_messages();
    assert_eq!(messages, vec![TestMsg::A, TestMsg::C]);
}

mod overlay_tests {
    use super::*;
    use crate::input::Event;
    use crate::overlay::{Overlay, OverlayAction};
    use crate::theme::Theme;
    use ratatui::layout::Rect;
    use ratatui::Frame;

    struct TestOverlay;

    impl Overlay<TestMsg> for TestOverlay {
        fn handle_event(&mut self, _event: &Event) -> OverlayAction<TestMsg> {
            OverlayAction::Consumed
        }

        fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
    }

    #[test]
    fn test_command_push_overlay() {
        let cmd: Command<TestMsg> = Command::push_overlay(TestOverlay);
        assert!(!cmd.is_none());
    }

    #[test]
    fn test_command_pop_overlay() {
        let cmd: Command<TestMsg> = Command::pop_overlay();
        assert!(!cmd.is_none());
    }

    #[test]
    fn test_command_handler_push_overlay() {
        let mut handler = CommandHandler::new();
        handler.execute(Command::push_overlay(TestOverlay));

        // No messages produced
        assert!(handler.take_messages().is_empty());

        // But there should be a pending overlay push
        let pushes = handler.take_overlay_pushes();
        assert_eq!(pushes.len(), 1);
    }

    #[test]
    fn test_command_handler_pop_overlay() {
        let mut handler: CommandHandler<TestMsg> = CommandHandler::new();
        handler.execute(Command::pop_overlay());

        let pops = handler.take_overlay_pops();
        assert_eq!(pops, 1);
    }

    #[test]
    fn test_command_handler_multiple_overlay_ops() {
        let mut handler = CommandHandler::new();
        let cmd = Command::combine([
            Command::push_overlay(TestOverlay),
            Command::push_overlay(TestOverlay),
            Command::pop_overlay(),
            Command::message(TestMsg::A),
        ]);
        handler.execute(cmd);

        assert_eq!(handler.take_messages(), vec![TestMsg::A]);
        assert_eq!(handler.take_overlay_pushes().len(), 2);
        assert_eq!(handler.take_overlay_pops(), 1);
    }

    #[test]
    fn test_command_clone_push_overlay_skipped() {
        let cmd: Command<TestMsg> = Command::push_overlay(TestOverlay);
        let cloned = cmd.clone();

        // PushOverlay can't be cloned, so cloned should be empty
        assert!(cloned.is_none());
    }

    #[test]
    fn test_command_clone_pop_overlay_preserved() {
        let cmd: Command<TestMsg> = Command::pop_overlay();
        let cloned = cmd.clone();

        // PopOverlay can be cloned
        assert!(!cloned.is_none());
    }

    #[test]
    fn test_command_map_push_overlay_skipped() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd: Command<TestMsg> = Command::push_overlay(TestOverlay);
        let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

        // PushOverlay can't be mapped, so mapped should be empty
        assert!(mapped.is_none());
    }

    #[test]
    fn test_command_map_pop_overlay_preserved() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd: Command<TestMsg> = Command::pop_overlay();
        let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

        // PopOverlay passes through map
        assert!(!mapped.is_none());
    }
}
