//! Virtual terminal mode implementation.
//!
//! Provides the `Runtime` methods for running applications with a virtual
//! capture backend, useful for programmatic control (AI agents, automation,
//! testing).

use super::Runtime;
use crate::app::model::App;
use crate::backend::CaptureBackend;
use crate::input::Event;

// =============================================================================
// Virtual Terminal Mode - for programmatic control (agents, testing)
// =============================================================================

impl<A: App> Runtime<A, CaptureBackend> {
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
    /// #     type Args = ();
    /// #     fn init(_args: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let mut vt = Runtime::<MyApp, _>::virtual_builder(80, 24).build()?;
    /// vt.send(Event::key(Key::Enter));
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
    /// #     type Args = ();
    /// #     fn init(_args: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let mut vt = Runtime::<MyApp, _>::virtual_builder(80, 24).build()?;
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
    /// #     type Args = ();
    /// #     fn init(_args: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// # let vt = Runtime::<MyApp, _>::virtual_builder(80, 24).build()?;
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
    /// #     type Args = ();
    /// #     fn init(_args: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> {
    /// #         Command::none()
    /// #     }
    /// #     fn view(state: &MyState, frame: &mut Frame) {
    /// #         frame.render_widget(ratatui::widgets::Paragraph::new("Hello"), frame.area());
    /// #     }
    /// # }
    /// let mut vt = Runtime::<MyApp, _>::virtual_builder(80, 24).build()?;
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
