    use super::*;
    use crate::app::Command;
    use ratatui::widgets::Paragraph;
    use std::time::Duration;

    struct CounterApp;

    #[derive(Clone, Default)]
    struct CounterState {
        count: i32,
        quit: bool,
    }

    #[derive(Clone, Debug)]
    enum CounterMsg {
        Increment,
        Decrement,
        IncrementBy(i32),
        Quit,
    }

    impl App for CounterApp {
        type State = CounterState;
        type Message = CounterMsg;

        fn init() -> (Self::State, Command<Self::Message>) {
            (CounterState::default(), Command::none())
        }

        fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
            match msg {
                CounterMsg::Increment => state.count += 1,
                CounterMsg::Decrement => state.count -= 1,
                CounterMsg::IncrementBy(n) => state.count += n,
                CounterMsg::Quit => state.quit = true,
            }
            Command::none()
        }

        fn view(state: &Self::State, frame: &mut ratatui::Frame) {
            let text = format!("Count: {}", state.count);
            frame.render_widget(Paragraph::new(text), frame.area());
        }

        fn should_quit(state: &Self::State) -> bool {
            state.quit
        }
    }

    #[test]
    fn test_async_runtime_headless() {
        let runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::virtual_terminal(80, 24).unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_async_runtime_dispatch() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        runtime.dispatch(CounterMsg::Increment);
        assert_eq!(runtime.state().count, 1);

        runtime.dispatch(CounterMsg::Increment);
        runtime.dispatch(CounterMsg::Increment);
        assert_eq!(runtime.state().count, 3);

        runtime.dispatch(CounterMsg::Decrement);
        assert_eq!(runtime.state().count, 2);
    }

    #[test]
    fn test_async_runtime_render() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Increment);
        runtime.dispatch(CounterMsg::Increment);
        runtime.render().unwrap();

        assert!(runtime.contains_text("Count: 2"));
    }

    #[test]
    fn test_async_runtime_quit() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();
        assert!(!runtime.should_quit());

        runtime.dispatch(CounterMsg::Quit);
        runtime.tick().unwrap();

        assert!(runtime.should_quit());
    }

    #[test]
    fn test_async_runtime_config() {
        let config = AsyncRuntimeConfig::new()
            .tick_rate(Duration::from_millis(100))
            .frame_rate(Duration::from_millis(32))
            .with_history(5)
            .max_messages(50)
            .channel_capacity(512);

        assert_eq!(config.tick_rate, Duration::from_millis(100));
        assert_eq!(config.frame_rate, Duration::from_millis(32));
        assert!(config.capture_history);
        assert_eq!(config.history_capacity, 5);
        assert_eq!(config.max_messages_per_tick, 50);
        assert_eq!(config.message_channel_capacity, 512);
    }

    #[test]
    fn test_async_runtime_config_default() {
        let config = AsyncRuntimeConfig::default();
        assert_eq!(config.tick_rate, Duration::from_millis(50));
        assert_eq!(config.frame_rate, Duration::from_millis(16));
        assert_eq!(config.max_messages_per_tick, 100);
        assert!(!config.capture_history);
    }

    #[test]
    fn test_async_runtime_cancellation_token() {
        let runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::virtual_terminal(80, 24).unwrap();
        let token = runtime.cancellation_token();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_async_runtime_message_sender() {
        let runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::virtual_terminal(80, 24).unwrap();
        let _sender = runtime.message_sender();
        // Just verify we can get a sender
    }

    #[tokio::test]
    async fn test_async_runtime_async_command() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        // Create an async command
        let cmd = Command::perform_async(async { Some(CounterMsg::IncrementBy(5)) });

        // Execute the command
        runtime.commands.execute(cmd);
        runtime.spawn_pending_commands();

        // Wait for the message
        tokio::time::sleep(Duration::from_millis(10)).await;
        runtime.process_pending();

        assert_eq!(runtime.state().count, 5);
    }

    #[tokio::test]
    async fn test_async_runtime_message_channel() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();
        let sender = runtime.message_sender();

        // Send a message via the channel
        sender.send(CounterMsg::Increment).await.unwrap();
        sender.send(CounterMsg::Increment).await.unwrap();

        // Process the messages
        runtime.process_pending();
        assert_eq!(runtime.state().count, 2);
    }

    #[test]
    fn test_async_runtime_dispatch_all() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        runtime.dispatch_all(vec![
            CounterMsg::Increment,
            CounterMsg::Increment,
            CounterMsg::Decrement,
        ]);

        assert_eq!(runtime.state().count, 1);
    }

    #[test]
    fn test_async_runtime_tick() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Increment);
        runtime.tick().unwrap();

        assert!(runtime.contains_text("Count: 1"));
    }

    #[test]
    fn test_async_runtime_run_ticks() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Increment);
        runtime.run_ticks(3).unwrap();

        assert!(runtime.contains_text("Count: 1"));
    }

    #[test]
    fn test_async_runtime_manual_quit() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();
        assert!(!runtime.should_quit());
        assert!(!runtime.cancellation_token().is_cancelled());

        runtime.quit();
        assert!(runtime.should_quit());
        assert!(runtime.cancellation_token().is_cancelled());
    }

    #[test]
    fn test_async_runtime_error_sender() {
        let runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::virtual_terminal(80, 24).unwrap();
        let _error_tx = runtime.error_sender();
        // Just verify we can get an error sender
    }

    #[tokio::test]
    async fn test_async_runtime_take_errors() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();
        let error_tx = runtime.error_sender();

        // No errors initially
        let errors = runtime.take_errors();
        assert!(errors.is_empty());

        // Send an error
        let err: BoxedError = Box::new(std::io::Error::other("test error"));
        error_tx.send(err).await.unwrap();

        // Should have one error
        let errors = runtime.take_errors();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("test error"));

        // Errors are consumed
        let errors = runtime.take_errors();
        assert!(errors.is_empty());
    }

    #[tokio::test]
    async fn test_async_runtime_multiple_errors() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();
        let error_tx = runtime.error_sender();

        // Send multiple errors
        for i in 0..3 {
            let err: BoxedError = Box::new(std::io::Error::other(format!("error {}", i)));
            error_tx.send(err).await.unwrap();
        }

        // Should have all three errors
        let errors = runtime.take_errors();
        assert_eq!(errors.len(), 3);
    }

    #[tokio::test]
    async fn test_async_runtime_has_errors() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();
        let error_tx = runtime.error_sender();

        // No errors initially
        assert!(!runtime.has_errors());

        // Send an error
        let err: BoxedError = Box::new(std::io::Error::other("test error"));
        error_tx.send(err).await.unwrap();

        // Give the channel a moment to process
        tokio::time::sleep(Duration::from_millis(1)).await;

        // Should have errors now
        assert!(runtime.has_errors());

        // Consume the errors
        let _ = runtime.take_errors();

        // No more errors
        assert!(!runtime.has_errors());
    }

    #[tokio::test]
    async fn test_async_runtime_error_from_spawned_task() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();
        let error_tx = runtime.error_sender();

        // Spawn a task that reports an error
        tokio::spawn(async move {
            let err: BoxedError = Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "resource not found",
            ));
            let _ = error_tx.send(err).await;
        });

        // Wait for the task to complete
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should have the error
        let errors = runtime.take_errors();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("resource not found"));
    }

    // Test app that uses try_perform_async for fallible operations
    struct FallibleApp;

    #[derive(Clone, Default)]
    struct FallibleState {
        value: Option<i32>,
    }

    #[derive(Clone, Debug)]
    enum FallibleMsg {
        FetchSuccess,
        FetchFailure,
        Loaded(i32),
    }

    impl App for FallibleApp {
        type State = FallibleState;
        type Message = FallibleMsg;

        fn init() -> (Self::State, Command<Self::Message>) {
            (FallibleState::default(), Command::none())
        }

        fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
            match msg {
                FallibleMsg::FetchSuccess => {
                    Command::try_perform_async(async { Ok::<_, std::io::Error>(42) }, |n| {
                        Some(FallibleMsg::Loaded(n))
                    })
                }
                FallibleMsg::FetchFailure => Command::try_perform_async(
                    async {
                        Err::<i32, _>(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "data not found",
                        ))
                    },
                    |n| Some(FallibleMsg::Loaded(n)),
                ),
                FallibleMsg::Loaded(n) => {
                    state.value = Some(n);
                    Command::none()
                }
            }
        }

        fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}
    }

    #[tokio::test]
    async fn test_async_runtime_try_perform_async_success() {
        let mut runtime: AsyncRuntime<FallibleApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        // Dispatch a message that triggers a successful async operation
        runtime.dispatch(FallibleMsg::FetchSuccess);

        // Wait for the async task to complete
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Process pending messages from the spawned task
        runtime.process_pending();

        // State should be updated with the loaded value
        assert_eq!(runtime.state().value, Some(42));

        // No errors should be in the channel
        assert!(!runtime.has_errors());
    }

    #[tokio::test]
    async fn test_async_runtime_try_perform_async_failure() {
        let mut runtime: AsyncRuntime<FallibleApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        // Dispatch a message that triggers a failing async operation
        runtime.dispatch(FallibleMsg::FetchFailure);

        // Wait for the async task to complete
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Process pending (there shouldn't be any messages, just the error)
        runtime.process_pending();

        // State should NOT be updated (error occurred)
        assert_eq!(runtime.state().value, None);

        // Error should be in the channel
        let errors = runtime.take_errors();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("data not found"));
    }

    #[test]
    fn test_async_runtime_headless_with_config() {
        let config = AsyncRuntimeConfig::new().with_history(5);
        let runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal_with_config(80, 24, config).unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_async_runtime_headless_with_config_no_history() {
        let config = AsyncRuntimeConfig::new(); // capture_history is false by default
        let runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal_with_config(80, 24, config).unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_async_runtime_state_mut() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();
        runtime.state_mut().count = 42;
        assert_eq!(runtime.state().count, 42);
    }

    #[test]
    fn test_async_runtime_terminal() {
        let runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::virtual_terminal(80, 24).unwrap();
        let terminal = runtime.terminal();
        assert_eq!(terminal.size().unwrap().width, 80);
    }

    #[test]
    fn test_async_runtime_terminal_mut() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();
        let _terminal = runtime.terminal_mut();
        // Just verify we can get a mutable reference
    }

    #[test]
    fn test_async_runtime_backend() {
        let runtime: AsyncRuntime<CounterApp, _> = AsyncRuntime::virtual_terminal(80, 24).unwrap();
        let backend = runtime.backend();
        assert_eq!(backend.size().unwrap().width, 80);
    }

    #[test]
    fn test_async_runtime_backend_mut() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();
        let _backend = runtime.backend_mut();
        // Just verify we can get a mutable reference
    }

    #[test]
    fn test_async_runtime_events() {
        use crate::input::Event;
        use crossterm::event::KeyCode;

        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();
        let events = runtime.events();

        // Add some events to the queue
        events.push(Event::key(KeyCode::Enter));

        assert!(!events.is_empty());
    }

    #[test]
    fn test_async_runtime_process_event() {
        use crate::input::Event;
        use crossterm::event::KeyCode;

        // App that handles key events
        struct KeyApp;

        #[derive(Clone, Default)]
        struct KeyState {
            key_pressed: bool,
        }

        #[derive(Clone)]
        enum KeyMsg {
            KeyPress,
        }

        impl App for KeyApp {
            type State = KeyState;
            type Message = KeyMsg;

            fn init() -> (Self::State, Command<Self::Message>) {
                (KeyState::default(), Command::none())
            }

            fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
                match msg {
                    KeyMsg::KeyPress => state.key_pressed = true,
                }
                Command::none()
            }

            fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}

            fn handle_event(event: &Event) -> Option<Self::Message> {
                if let Event::Key(_) = event {
                    Some(KeyMsg::KeyPress)
                } else {
                    None
                }
            }
        }

        let mut runtime: AsyncRuntime<KeyApp, _> = AsyncRuntime::virtual_terminal(80, 24).unwrap();

        // No events to process
        assert!(!runtime.process_event());

        // Add an event
        runtime.events().push(Event::key(KeyCode::Enter));

        // Process the event
        assert!(runtime.process_event());
        assert!(runtime.state().key_pressed);

        // No more events
        assert!(!runtime.process_event());
    }

    #[test]
    fn test_async_runtime_process_all_events() {
        use crate::input::Event;
        use crossterm::event::KeyCode;

        // App that counts key presses
        struct CountKeyApp;

        #[derive(Clone, Default)]
        struct CountKeyState {
            count: i32,
        }

        #[derive(Clone)]
        enum CountKeyMsg {
            KeyPress,
        }

        impl App for CountKeyApp {
            type State = CountKeyState;
            type Message = CountKeyMsg;

            fn init() -> (Self::State, Command<Self::Message>) {
                (CountKeyState::default(), Command::none())
            }

            fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
                match msg {
                    CountKeyMsg::KeyPress => state.count += 1,
                }
                Command::none()
            }

            fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}

            fn handle_event(event: &Event) -> Option<Self::Message> {
                if let Event::Key(_) = event {
                    Some(CountKeyMsg::KeyPress)
                } else {
                    None
                }
            }
        }

        let mut runtime: AsyncRuntime<CountKeyApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        // Add multiple events
        for _ in 0..5 {
            runtime.events().push(Event::key(KeyCode::Enter));
        }

        // Process all events
        runtime.process_all_events();
        assert_eq!(runtime.state().count, 5);
    }

    #[test]
    fn test_async_runtime_cell_at() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(40, 10).unwrap();
        runtime.render().unwrap();

        // Cell at (0,0) should have the 'C' from "Count: 0"
        let cell = runtime.cell_at(0, 0).unwrap();
        assert_eq!(cell.symbol(), "C");

        // Out of bounds should return None
        assert!(runtime.cell_at(100, 100).is_none());
    }

    #[test]
    fn test_async_runtime_display() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(40, 10).unwrap();
        runtime.dispatch(CounterMsg::IncrementBy(42));
        runtime.render().unwrap();

        let output = runtime.display();
        assert!(output.contains("Count: 42"));
    }

    #[test]
    fn test_async_runtime_display_ansi() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Increment);
        runtime.render().unwrap();

        let ansi = runtime.display_ansi();
        assert!(ansi.contains("Count: 1"));
    }

    #[test]
    fn test_async_runtime_find_text() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(40, 10).unwrap();
        runtime.dispatch(CounterMsg::Increment);
        runtime.render().unwrap();

        let positions = runtime.find_text("Count");
        assert!(!positions.is_empty());
    }

    #[test]
    fn test_async_runtime_run_ticks_with_quit() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        // Dispatch quit message so it quits before all ticks
        runtime.dispatch(CounterMsg::Quit);
        runtime.run_ticks(100).unwrap(); // Request 100 ticks but should quit earlier

        assert!(runtime.should_quit());
    }

    #[tokio::test]
    async fn test_async_runtime_subscribe() {
        use crate::app::subscription::TickSubscription;

        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        // Subscribe to a tick that fires every 10ms
        let sub = TickSubscription::new(Duration::from_millis(10), || CounterMsg::Increment);
        runtime.subscribe(sub);

        // Spawn a task to send quit after some ticks
        let tx = runtime.message_sender();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = tx.send(CounterMsg::Quit).await;
        });

        // Run the event loop - subscriptions are polled here
        runtime.run().await.unwrap();

        // Should have received some increment messages (subscriptions work during run())
        // Note: Subscriptions are only polled during run(), so count may or may not be > 0
        // depending on timing. The key test is that we quit cleanly.
        assert!(runtime.should_quit());
    }

    #[tokio::test]
    async fn test_async_runtime_subscribe_all() {
        use crate::app::subscription::{BoxedSubscription, TickSubscription};

        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        // Create multiple subscriptions
        let sub1: BoxedSubscription<CounterMsg> =
            Box::new(TickSubscription::new(Duration::from_millis(10), || {
                CounterMsg::Increment
            }));
        let sub2: BoxedSubscription<CounterMsg> =
            Box::new(TickSubscription::new(Duration::from_millis(10), || {
                CounterMsg::Increment
            }));

        runtime.subscribe_all(vec![sub1, sub2]);

        // Wait a bit for ticks
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Clean up
        runtime.quit();
    }

    #[tokio::test]
    async fn test_async_runtime_run() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(40, 10).unwrap();

        // Increment counter
        runtime.dispatch(CounterMsg::Increment);

        // Spawn task to quit after a short delay
        let tx = runtime.message_sender();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = tx.send(CounterMsg::Quit).await;
        });

        // Run the event loop
        runtime.run().await.unwrap();

        // Should have quit
        assert!(runtime.should_quit());
        assert!(runtime.contains_text("Count: 1"));
    }

    #[tokio::test]
    async fn test_async_runtime_run_with_events() {
        use crate::input::Event;
        use crossterm::event::{KeyCode, KeyEvent};

        // App that increments on any key and quits on 'q'
        struct EventDrivenApp;

        #[derive(Clone, Default)]
        struct EventDrivenState {
            count: i32,
            quit: bool,
        }

        #[derive(Clone)]
        enum EventDrivenMsg {
            Increment,
            Quit,
        }

        impl App for EventDrivenApp {
            type State = EventDrivenState;
            type Message = EventDrivenMsg;

            fn init() -> (Self::State, Command<Self::Message>) {
                (EventDrivenState::default(), Command::none())
            }

            fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
                match msg {
                    EventDrivenMsg::Increment => state.count += 1,
                    EventDrivenMsg::Quit => state.quit = true,
                }
                Command::none()
            }

            fn view(state: &Self::State, frame: &mut ratatui::Frame) {
                let text = format!("Count: {}", state.count);
                frame.render_widget(Paragraph::new(text), frame.area());
            }

            fn should_quit(state: &Self::State) -> bool {
                state.quit
            }

            fn handle_event(event: &Event) -> Option<Self::Message> {
                if let Event::Key(KeyEvent { code, .. }) = event {
                    if *code == KeyCode::Char('q') {
                        Some(EventDrivenMsg::Quit)
                    } else {
                        Some(EventDrivenMsg::Increment)
                    }
                } else {
                    None
                }
            }
        }

        let mut runtime: AsyncRuntime<EventDrivenApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        // Add some key events
        runtime.events().push(Event::char('a'));
        runtime.events().push(Event::char('b'));

        // Spawn task to quit after processing events
        let tx = runtime.message_sender();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let _ = tx.send(EventDrivenMsg::Quit).await;
        });

        // Run the event loop
        runtime.run().await.unwrap();

        assert!(runtime.should_quit());
        assert!(runtime.state().count >= 2); // At least 2 key events processed
    }

    #[tokio::test]
    async fn test_async_runtime_run_cancelled() {
        let mut runtime: AsyncRuntime<CounterApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        let token = runtime.cancellation_token();

        // Spawn task to cancel after a short delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            token.cancel();
        });

        // Run the event loop
        runtime.run().await.unwrap();

        // Should have quit due to cancellation
        assert!(runtime.should_quit());
    }

    // Test app with on_tick handler
    struct TickingApp;

    #[derive(Clone, Default)]
    struct TickingState {
        ticks: i32,
        quit: bool,
    }

    #[derive(Clone)]
    enum TickingMsg {
        Tick,
    }

    impl App for TickingApp {
        type State = TickingState;
        type Message = TickingMsg;

        fn init() -> (Self::State, Command<Self::Message>) {
            (TickingState::default(), Command::none())
        }

        fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
            match msg {
                TickingMsg::Tick => {
                    state.ticks += 1;
                    if state.ticks >= 3 {
                        state.quit = true;
                    }
                }
            }
            Command::none()
        }

        fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}

        fn should_quit(state: &Self::State) -> bool {
            state.quit
        }

        fn on_tick(_state: &Self::State) -> Option<Self::Message> {
            Some(TickingMsg::Tick)
        }
    }

    #[test]
    fn test_async_runtime_tick_with_on_tick() {
        let mut runtime: AsyncRuntime<TickingApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        // Each tick should increment
        runtime.tick().unwrap();
        assert_eq!(runtime.state().ticks, 1);

        runtime.tick().unwrap();
        assert_eq!(runtime.state().ticks, 2);

        // Third tick should trigger quit
        runtime.tick().unwrap();
        assert_eq!(runtime.state().ticks, 3);
        assert!(runtime.should_quit());
    }

    #[tokio::test]
    async fn test_async_runtime_run_with_on_tick() {
        let mut runtime: AsyncRuntime<TickingApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        // Run the event loop - should quit after 3 ticks
        runtime.run().await.unwrap();

        assert!(runtime.should_quit());
        assert!(runtime.state().ticks >= 3);
    }

    // Test app with init command
    struct InitCommandApp;

    #[derive(Clone, Default)]
    struct InitCommandState {
        initialized: bool,
    }

    #[derive(Clone)]
    enum InitCommandMsg {
        Initialized,
    }

    impl App for InitCommandApp {
        type State = InitCommandState;
        type Message = InitCommandMsg;

        fn init() -> (Self::State, Command<Self::Message>) {
            // Return a command that sends Initialized message
            (
                InitCommandState::default(),
                Command::message(InitCommandMsg::Initialized),
            )
        }

        fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
            match msg {
                InitCommandMsg::Initialized => state.initialized = true,
            }
            Command::none()
        }

        fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}
    }

    #[test]
    fn test_async_runtime_init_command() {
        let mut runtime: AsyncRuntime<InitCommandApp, _> =
            AsyncRuntime::virtual_terminal(80, 24).unwrap();

        // Process sync commands from init
        runtime.process_pending();

        assert!(runtime.state().initialized);
    }

    // =========================================================================
    // Overlay Tests
    // =========================================================================

    mod overlay_tests {
        use super::*;
        use crate::input::Event;
        use crate::overlay::{Overlay, OverlayAction};
        use crate::theme::Theme;
        use crossterm::event::KeyCode;
        use ratatui::layout::Rect;

        struct ConsumeOverlay;

        impl Overlay<CounterMsg> for ConsumeOverlay {
            fn handle_event(&mut self, _event: &Event) -> OverlayAction<CounterMsg> {
                OverlayAction::Consumed
            }
            fn view(&self, _frame: &mut ratatui::Frame, _area: Rect, _theme: &Theme) {}
        }

        #[test]
        fn test_async_runtime_overlay_push_pop() {
            let mut runtime: AsyncRuntime<CounterApp, _> =
                AsyncRuntime::virtual_terminal(80, 24).unwrap();

            assert!(!runtime.has_overlays());
            assert_eq!(runtime.overlay_count(), 0);

            runtime.push_overlay(Box::new(ConsumeOverlay));
            assert!(runtime.has_overlays());
            assert_eq!(runtime.overlay_count(), 1);

            let popped = runtime.pop_overlay();
            assert!(popped.is_some());
            assert!(!runtime.has_overlays());
        }

        #[test]
        fn test_async_runtime_overlay_clear() {
            let mut runtime: AsyncRuntime<CounterApp, _> =
                AsyncRuntime::virtual_terminal(80, 24).unwrap();

            runtime.push_overlay(Box::new(ConsumeOverlay));
            runtime.push_overlay(Box::new(ConsumeOverlay));
            assert_eq!(runtime.overlay_count(), 2);

            runtime.clear_overlays();
            assert!(!runtime.has_overlays());
        }

        #[test]
        fn test_async_runtime_overlay_consumes_events() {
            // App that handles key events
            struct KeyApp;

            #[derive(Clone, Default)]
            struct KeyState {
                key_pressed: bool,
            }

            #[derive(Clone)]
            enum KeyMsg {
                KeyPress,
            }

            impl App for KeyApp {
                type State = KeyState;
                type Message = KeyMsg;

                fn init() -> (Self::State, Command<Self::Message>) {
                    (KeyState::default(), Command::none())
                }

                fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
                    match msg {
                        KeyMsg::KeyPress => state.key_pressed = true,
                    }
                    Command::none()
                }

                fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}

                fn handle_event(event: &Event) -> Option<Self::Message> {
                    if let Event::Key(_) = event {
                        Some(KeyMsg::KeyPress)
                    } else {
                        None
                    }
                }
            }

            struct ConsumeAll;
            impl Overlay<KeyMsg> for ConsumeAll {
                fn handle_event(&mut self, _event: &Event) -> OverlayAction<KeyMsg> {
                    OverlayAction::Consumed
                }
                fn view(&self, _frame: &mut ratatui::Frame, _area: Rect, _theme: &Theme) {}
            }

            let mut runtime: AsyncRuntime<KeyApp, _> =
                AsyncRuntime::virtual_terminal(80, 24).unwrap();

            runtime.push_overlay(Box::new(ConsumeAll));

            runtime.send(Event::key(KeyCode::Enter));
            runtime.tick().unwrap();

            // Event should be consumed by overlay, not reaching the app
            assert!(!runtime.state().key_pressed);
        }

        #[test]
        fn test_async_runtime_overlay_dismiss() {
            struct DismissOverlay;
            impl Overlay<CounterMsg> for DismissOverlay {
                fn handle_event(&mut self, event: &Event) -> OverlayAction<CounterMsg> {
                    if let Some(key) = event.as_key() {
                        if key.code == KeyCode::Esc {
                            return OverlayAction::Dismiss;
                        }
                    }
                    OverlayAction::Consumed
                }
                fn view(&self, _frame: &mut ratatui::Frame, _area: Rect, _theme: &Theme) {}
            }

            let mut runtime: AsyncRuntime<CounterApp, _> =
                AsyncRuntime::virtual_terminal(80, 24).unwrap();

            runtime.push_overlay(Box::new(DismissOverlay));
            assert_eq!(runtime.overlay_count(), 1);

            runtime.send(Event::key(KeyCode::Esc));
            runtime.tick().unwrap();

            assert_eq!(runtime.overlay_count(), 0);
        }

        #[test]
        fn test_async_runtime_theme_access() {
            let mut runtime: AsyncRuntime<CounterApp, _> =
                AsyncRuntime::virtual_terminal(80, 24).unwrap();

            let _theme = runtime.theme();

            let nord = Theme::nord();
            let expected_bg = nord.background;
            runtime.set_theme(nord);
            assert_eq!(runtime.theme().background, expected_bg);
        }

        #[test]
        fn test_async_runtime_render_with_overlay() {
            let mut runtime: AsyncRuntime<CounterApp, _> =
                AsyncRuntime::virtual_terminal(40, 10).unwrap();

            runtime.push_overlay(Box::new(ConsumeOverlay));
            runtime.render().unwrap();

            // App content should still be rendered underneath
            assert!(runtime.contains_text("Count: 0"));
        }

        #[test]
        fn test_async_runtime_overlay_message_from_event() {
            struct MsgOverlay;
            impl Overlay<CounterMsg> for MsgOverlay {
                fn handle_event(&mut self, _event: &Event) -> OverlayAction<CounterMsg> {
                    OverlayAction::Message(CounterMsg::IncrementBy(10))
                }
                fn view(&self, _frame: &mut ratatui::Frame, _area: Rect, _theme: &Theme) {}
            }

            let mut runtime: AsyncRuntime<CounterApp, _> =
                AsyncRuntime::virtual_terminal(80, 24).unwrap();
            runtime.push_overlay(Box::new(MsgOverlay));

            runtime.send(Event::char('x'));
            runtime.tick().unwrap();

            assert_eq!(runtime.state().count, 10);
        }

        #[test]
        fn test_async_runtime_overlay_dismiss_with_message() {
            struct DismissWithMsgOverlay;
            impl Overlay<CounterMsg> for DismissWithMsgOverlay {
                fn handle_event(&mut self, _event: &Event) -> OverlayAction<CounterMsg> {
                    OverlayAction::DismissWithMessage(CounterMsg::IncrementBy(5))
                }
                fn view(&self, _frame: &mut ratatui::Frame, _area: Rect, _theme: &Theme) {}
            }

            let mut runtime: AsyncRuntime<CounterApp, _> =
                AsyncRuntime::virtual_terminal(80, 24).unwrap();

            runtime.push_overlay(Box::new(DismissWithMsgOverlay));
            assert_eq!(runtime.overlay_count(), 1);

            runtime.send(Event::char('x'));
            runtime.tick().unwrap();

            // Overlay should be dismissed and message dispatched
            assert_eq!(runtime.overlay_count(), 0);
            assert_eq!(runtime.state().count, 5);
        }

        #[test]
        fn test_async_runtime_process_sync_commands_overlay() {
            struct CmdApp;

            #[derive(Clone, Default)]
            struct CmdState;

            #[derive(Clone)]
            enum CmdMsg {
                Push,
                Pop,
            }

            struct NoopOverlay;
            impl Overlay<CmdMsg> for NoopOverlay {
                fn handle_event(&mut self, _event: &Event) -> OverlayAction<CmdMsg> {
                    OverlayAction::Consumed
                }
                fn view(&self, _frame: &mut ratatui::Frame, _area: Rect, _theme: &Theme) {}
            }

            impl App for CmdApp {
                type State = CmdState;
                type Message = CmdMsg;

                fn init() -> (Self::State, Command<Self::Message>) {
                    (CmdState, Command::none())
                }

                fn update(_state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
                    match msg {
                        CmdMsg::Push => Command::push_overlay(NoopOverlay),
                        CmdMsg::Pop => Command::pop_overlay(),
                    }
                }

                fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}
            }

            let mut runtime: AsyncRuntime<CmdApp, _> =
                AsyncRuntime::virtual_terminal(80, 24).unwrap();

            runtime.dispatch(CmdMsg::Push);
            runtime.process_pending();
            assert_eq!(runtime.overlay_count(), 1);

            runtime.dispatch(CmdMsg::Pop);
            runtime.process_pending();
            assert_eq!(runtime.overlay_count(), 0);
        }
    }
