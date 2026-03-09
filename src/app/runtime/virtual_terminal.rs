//! Virtual terminal mode implementation.
//!
//! Provides the `Runtime` methods for running applications with a virtual
//! capture backend, useful for programmatic control (AI agents, automation,
//! testing).

use crate::error;

use super::config::RuntimeConfig;
use super::Runtime;
use crate::app::command::Command;
use crate::app::model::App;
use crate::backend::CaptureBackend;
use crate::input::Event;

// =============================================================================
// Virtual Terminal Mode - for programmatic control (agents, testing)
// =============================================================================

impl<A: App> Runtime<A, CaptureBackend> {
    /// Creates a virtual terminal for programmatic control.
    ///
    /// A virtual terminal is not connected to a physical terminal. Instead:
    /// - Events are injected via `send()`
    /// - The application is advanced via `tick()`
    /// - The display can be inspected via `display()`
    ///
    /// This is useful for:
    /// - AI agents driving the application
    /// - Automation and scripting
    /// - Testing
    ///
    /// # Errors
    ///
    /// Returns an error if creating the ratatui `Terminal` with the
    /// capture backend fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState;
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let mut vt = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
    /// vt.send(Event::key(KeyCode::Char('j')));
    /// vt.tick()?;
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn virtual_terminal(width: u16, height: u16) -> error::Result<Self> {
        let backend = CaptureBackend::new(width, height);
        Self::with_backend(backend)
    }

    /// Creates a virtual terminal with custom configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the ratatui `Terminal` with the
    /// capture backend fails.
    pub fn virtual_terminal_with_config(
        width: u16,
        height: u16,
        config: RuntimeConfig,
    ) -> error::Result<Self> {
        let backend = if config.capture_history {
            CaptureBackend::with_history(width, height, config.history_capacity)
        } else {
            CaptureBackend::new(width, height)
        };
        Self::with_backend_and_config(backend, config)
    }

    /// Creates a virtual terminal with a pre-built state, bypassing [`App::init()`].
    ///
    /// [`App::init()`] is **not called** — the provided `state` and `init_cmd`
    /// are used instead. This is the primary way to test or automate an
    /// application starting from a specific state. The `init_cmd` is executed
    /// immediately; pass [`Command::none()`] if no startup command is needed.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the ratatui `Terminal` with the
    /// capture backend fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState { count: i32 }
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState::default(), Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// // Start from a specific state instead of App::init()
    /// let state = MyState { count: 10 };
    /// let mut vt = Runtime::<MyApp, _>::virtual_terminal_with_state(
    ///     80, 24, state, Command::none(),
    /// )?;
    /// assert_eq!(vt.state().count, 10);
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn virtual_terminal_with_state(
        width: u16,
        height: u16,
        state: A::State,
        init_cmd: Command<A::Message>,
    ) -> error::Result<Self> {
        let backend = CaptureBackend::new(width, height);
        Self::with_backend_and_state(backend, state, init_cmd)
    }

    /// Creates a virtual terminal with a pre-built state and custom configuration.
    ///
    /// [`App::init()`] is **not called** — the provided `state` and `init_cmd`
    /// are used instead. Combines
    /// [`virtual_terminal_with_state`](Self::virtual_terminal_with_state)
    /// with custom [`RuntimeConfig`] options.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the ratatui `Terminal` with the
    /// capture backend fails.
    pub fn virtual_terminal_with_state_and_config(
        width: u16,
        height: u16,
        state: A::State,
        init_cmd: Command<A::Message>,
        config: RuntimeConfig,
    ) -> error::Result<Self> {
        let backend = if config.capture_history {
            CaptureBackend::with_history(width, height, config.history_capacity)
        } else {
            CaptureBackend::new(width, height)
        };
        Self::with_backend_state_and_config(backend, state, init_cmd, config)
    }

    /// Sends an event to the virtual terminal.
    ///
    /// The event is queued and will be processed on the next `tick()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState;
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let mut vt = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
    /// vt.send(Event::key(KeyCode::Enter));
    /// vt.tick()?;
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn send(&mut self, event: Event) {
        self.core.events.push(event);
    }

    /// Returns the current display content as plain text.
    ///
    /// This is what would be shown on a terminal screen.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState;
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let mut vt = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
    /// vt.tick()?;
    /// let screen = vt.display();
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn display(&self) -> String {
        self.core.terminal.backend().to_string()
    }

    /// Returns the display content with ANSI color codes.
    pub fn display_ansi(&self) -> String {
        self.core.terminal.backend().to_ansi()
    }
}

// =============================================================================
// Additional convenience methods for CaptureBackend (virtual terminal)
// =============================================================================

impl<A: App> Runtime<A, CaptureBackend> {
    /// Returns the cell at the given position, or `None` if out of bounds.
    ///
    /// Use this to assert on cell styling:
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState;
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// # let vt = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
    /// let cell = vt.cell_at(5, 3);
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn cell_at(&self, x: u16, y: u16) -> Option<&crate::backend::EnhancedCell> {
        self.core.terminal.backend().cell(x, y)
    }

    /// Returns true if the display contains the given text.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState;
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> {
    /// #         Command::none()
    /// #     }
    /// #     fn view(state: &MyState, frame: &mut Frame) {
    /// #         frame.render_widget(ratatui::widgets::Paragraph::new("Hello"), frame.area());
    /// #     }
    /// # }
    /// let mut vt = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
    /// vt.tick()?;
    /// assert!(vt.contains_text("Hello"));
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn contains_text(&self, needle: &str) -> bool {
        self.core.terminal.backend().contains_text(needle)
    }

    /// Finds all positions of the given text in the display.
    pub fn find_text(&self, needle: &str) -> Vec<ratatui::layout::Position> {
        self.core.terminal.backend().find_text(needle)
    }
}
