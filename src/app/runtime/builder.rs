//! Builder pattern for constructing [`Runtime`] instances.
//!
//! [`RuntimeBuilder`] provides a fluent API for configuring and creating
//! a runtime. It replaces the combinatorial explosion of 12 constructor
//! methods on [`Runtime`] with a single builder chain.
//!
//! # Entry Points
//!
//! There are three entry points, one for each backend type:
//!
//! - [`Runtime::terminal_builder()`] — real terminal (crossterm)
//! - [`Runtime::virtual_builder()`] — virtual capture backend
//! - [`Runtime::builder()`] — any backend implementing [`Backend`]
//!
//! # Examples
//!
//! ## Virtual terminal (testing / automation)
//!
//! ```rust
//! # use envision::prelude::*;
//! # struct MyApp;
//! # #[derive(Default, Clone)]
//! # struct MyState;
//! # #[derive(Clone)]
//! # enum MyMsg {}
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     type Args = ();
//! #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
//! let vt = Runtime::<MyApp, _>::virtual_builder(80, 24).build()?;
//! # Ok::<(), envision::EnvisionError>(())
//! ```
//!
//! ## With custom config
//!
//! ```rust
//! # use envision::prelude::*;
//! # use std::time::Duration;
//! # struct MyApp;
//! # #[derive(Default, Clone)]
//! # struct MyState;
//! # #[derive(Clone)]
//! # enum MyMsg {}
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     type Args = ();
//! #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
//! let vt = Runtime::<MyApp, _>::virtual_builder(80, 24)
//!     .tick_rate(Duration::from_millis(100))
//!     .build()?;
//! # Ok::<(), envision::EnvisionError>(())
//! ```
//!
//! ## With injected args
//!
//! ```rust
//! # use envision::prelude::*;
//! # struct MyApp;
//! # struct MyArgs { count: i32 }
//! # #[derive(Default, Clone)]
//! # struct MyState { count: i32 }
//! # #[derive(Clone)]
//! # enum MyMsg {}
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     type Args = MyArgs;
//! #     fn init(args: MyArgs) -> (MyState, Command<MyMsg>) {
//! #         (MyState { count: args.count }, Command::none())
//! #     }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
//! let vt = Runtime::<MyApp, _>::virtual_builder(80, 24)
//!     .with_args(MyArgs { count: 42 })
//!     .build()?;
//! assert_eq!(vt.state().count, 42);
//! # Ok::<(), envision::EnvisionError>(())
//! ```
//!
//! ## Real terminal (production)
//!
//! ```rust,no_run
//! # use envision::prelude::*;
//! # struct MyApp;
//! # #[derive(Default, Clone)]
//! # struct MyState;
//! # #[derive(Clone)]
//! # enum MyMsg {}
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     type Args = ();
//! #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
//! # #[tokio::main]
//! # async fn main() -> envision::Result<()> {
//! let _final_state = Runtime::<MyApp, _>::terminal_builder()?
//!     .build()?
//!     .run_terminal()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use std::io::Stdout;
use std::marker::PhantomData;
use std::time::Duration;

use ratatui::backend::{Backend, CrosstermBackend};

use super::Runtime;
use super::config::RuntimeConfig;
use crate::app::model::App;
use crate::backend::CaptureBackend;
use crate::error;

/// A builder for constructing [`Runtime`] instances.
///
/// Created via [`Runtime::builder()`], [`Runtime::terminal_builder()`],
/// or [`Runtime::virtual_builder()`].
///
/// The builder provides fluent methods to configure:
/// - **Args**: `.with_args(args)` to provide the args passed to `App::init`
/// - **Config**: `.config(config)` to supply a full [`RuntimeConfig`]
/// - **Individual settings**: `.tick_rate()`, `.frame_rate()`, etc.
///
/// `with_args(args)` returns a [`ConfiguredRuntimeBuilder`] — see that type
/// for the post-args build path. Calling `.build()` directly on
/// `RuntimeBuilder` is only available when `A::Args: OptionalArgs`
/// (i.e. `A::Args = ()`).
///
/// # Example
///
/// ```rust
/// # use envision::prelude::*;
/// # use std::time::Duration;
/// # struct MyApp;
/// # #[derive(Default, Clone)]
/// # struct MyState;
/// # #[derive(Clone)]
/// # enum MyMsg {}
/// # impl App for MyApp {
/// #     type State = MyState;
/// #     type Message = MyMsg;
/// #     type Args = ();
/// #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
/// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
/// #     fn view(state: &MyState, frame: &mut Frame) {}
/// # }
/// let vt = Runtime::<MyApp, _>::virtual_builder(80, 24)
///     .tick_rate(Duration::from_millis(100))
///     .frame_rate(Duration::from_millis(32))
///     .build()?;
/// # Ok::<(), envision::EnvisionError>(())
/// ```
pub struct RuntimeBuilder<A: App, B: Backend> {
    backend: B,
    config: Option<RuntimeConfig>,
    _phantom: PhantomData<A>,
}

impl<A: App, B: Backend> RuntimeBuilder<A, B> {
    /// Creates a new builder with the given backend.
    ///
    /// Prefer the convenience entry points [`Runtime::terminal_builder()`]
    /// and [`Runtime::virtual_builder()`] for common backends. Use this
    /// method when providing a custom [`Backend`] implementation.
    pub(crate) fn new(backend: B) -> Self {
        Self {
            backend,
            config: None,
            _phantom: PhantomData,
        }
    }

    /// Provides the args for [`App::init`].
    ///
    /// Consumes the `RuntimeBuilder` and produces a [`ConfiguredRuntimeBuilder`]
    /// whose `build()` is unconditionally available. Any prior config-shaping
    /// calls (`tick_rate`, `frame_rate`, etc.) are preserved.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # use std::path::PathBuf;
    /// # struct MyApp;
    /// # #[derive(Clone)]
    /// # struct MyArgs { dir: PathBuf }
    /// # #[derive(Default, Clone)]
    /// # struct MyState { dir: PathBuf }
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     type Args = MyArgs;
    /// #     fn init(args: MyArgs) -> (MyState, Command<MyMsg>) {
    /// #         (MyState { dir: args.dir }, Command::none())
    /// #     }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let args = MyArgs { dir: PathBuf::from("/tmp/example") };
    /// let runtime = Runtime::<MyApp, _>::virtual_builder(80, 24)
    ///     .with_args(args)
    ///     .build()?;
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    ///
    /// For apps whose `Args = ()` you typically don't need this method — the
    /// `OptionalArgs` shortcut on [`build`](Self::build) handles the unit
    /// case implicitly.
    pub fn with_args(self, args: A::Args) -> ConfiguredRuntimeBuilder<A, B> {
        ConfiguredRuntimeBuilder {
            backend: self.backend,
            config: self.config,
            args,
        }
    }

    /// Sets the full runtime configuration.
    ///
    /// This replaces any previously set configuration (including individual
    /// settings like [`tick_rate`](Self::tick_rate) or
    /// [`frame_rate`](Self::frame_rate)). If you want to set only specific
    /// fields, use the individual builder methods instead.
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
    /// #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let config = RuntimeConfig::new()
    ///     .tick_rate(std::time::Duration::from_millis(100))
    ///     .max_messages(50);
    /// let vt = Runtime::<MyApp, _>::virtual_builder(80, 24)
    ///     .config(config)
    ///     .build()?;
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn config(mut self, config: RuntimeConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Sets the tick rate (how often to poll for events).
    ///
    /// Default: 50ms.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # use std::time::Duration;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState;
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     type Args = ();
    /// #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let vt = Runtime::<MyApp, _>::virtual_builder(80, 24)
    ///     .tick_rate(Duration::from_millis(100))
    ///     .build()?;
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn tick_rate(mut self, rate: Duration) -> Self {
        self.config_mut().tick_rate = rate;
        self
    }

    /// Sets the frame rate (how often to render).
    ///
    /// Default: 16ms (~60fps).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use envision::prelude::*;
    /// # use std::time::Duration;
    /// # struct MyApp;
    /// # #[derive(Default, Clone)]
    /// # struct MyState;
    /// # #[derive(Clone)]
    /// # enum MyMsg {}
    /// # impl App for MyApp {
    /// #     type State = MyState;
    /// #     type Message = MyMsg;
    /// #     type Args = ();
    /// #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let vt = Runtime::<MyApp, _>::virtual_builder(80, 24)
    ///     .frame_rate(Duration::from_millis(32))
    ///     .build()?;
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn frame_rate(mut self, rate: Duration) -> Self {
        self.config_mut().frame_rate = rate;
        self
    }

    /// Sets the maximum number of messages to process per tick.
    ///
    /// This prevents infinite loops when messages trigger other messages.
    /// Default: 100.
    pub fn max_messages(mut self, max: usize) -> Self {
        self.config_mut().max_messages_per_tick = max;
        self
    }

    /// Sets the capacity of the async message channel.
    ///
    /// Default: 256.
    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.config_mut().message_channel_capacity = capacity;
        self
    }

    /// Returns a mutable reference to the config, creating a default if needed.
    fn config_mut(&mut self) -> &mut RuntimeConfig {
        self.config.get_or_insert_with(RuntimeConfig::default)
    }
}

// `build()` for the no-args path — only available when A::Args: OptionalArgs.
//
// On stable Rust this means A::Args == (). Calling `.build()` for an App
// whose Args is anything other than () fails to compile here, which is the
// compile-time enforcement the redesign promises.
impl<A: App, B: Backend> RuntimeBuilder<A, B>
where
    A::Args: crate::app::OptionalArgs,
{
    /// Builds the [`Runtime`].
    ///
    /// Available only when `A::Args = ()`. For apps with non-`()` args,
    /// call [`with_args`](Self::with_args) first and then `.build()` on
    /// the resulting [`ConfiguredRuntimeBuilder`].
    ///
    /// # Errors
    ///
    /// Returns an error if creating the ratatui `Terminal` with the
    /// provided backend fails.
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
    /// #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let runtime = Runtime::<MyApp, _>::virtual_builder(80, 24).build()?;
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn build(self) -> error::Result<Runtime<A, B>> {
        use crate::app::model::optional_args::sealed::Sealed;
        let args = <A::Args as Sealed>::default_optional_args();
        self.with_args(args).build()
    }
}

// =============================================================================
// ConfiguredRuntimeBuilder — RuntimeBuilder after with_args has been called
// =============================================================================

/// A `RuntimeBuilder` after [`with_args`](RuntimeBuilder::with_args) has been called.
///
/// Carries the args that will be passed to [`App::init`] plus all
/// configuration set so far. Has its own fluent config-shaping methods
/// (`config`, `tick_rate`, `frame_rate`, `max_messages`, `channel_capacity`)
/// and an unconditionally-available `build()`.
///
/// Most users never name this type — the typestate transition happens
/// implicitly when chaining `.with_args(...).build()`.
pub struct ConfiguredRuntimeBuilder<A: App, B: Backend> {
    backend: B,
    config: Option<RuntimeConfig>,
    args: A::Args,
}

impl<A: App, B: Backend> ConfiguredRuntimeBuilder<A, B> {
    /// Sets the full runtime configuration. Replaces any previously set config.
    pub fn config(mut self, config: RuntimeConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Sets the tick rate (how often to poll for events). Default: 50ms.
    pub fn tick_rate(mut self, rate: Duration) -> Self {
        self.config_mut().tick_rate = rate;
        self
    }

    /// Sets the frame rate (how often to render). Default: 16ms (~60fps).
    pub fn frame_rate(mut self, rate: Duration) -> Self {
        self.config_mut().frame_rate = rate;
        self
    }

    /// Sets the max messages per tick. Default: 100.
    pub fn max_messages(mut self, max: usize) -> Self {
        self.config_mut().max_messages_per_tick = max;
        self
    }

    /// Sets the async message channel capacity. Default: 256.
    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.config_mut().message_channel_capacity = capacity;
        self
    }

    fn config_mut(&mut self) -> &mut RuntimeConfig {
        self.config.get_or_insert_with(RuntimeConfig::default)
    }

    /// Builds the [`Runtime`].
    ///
    /// Calls [`App::init`] with the previously-supplied args to obtain the
    /// initial state and startup command, then constructs the `Runtime`
    /// with the configured backend and runtime config.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the ratatui `Terminal` with the
    /// provided backend fails.
    pub fn build(self) -> error::Result<Runtime<A, B>> {
        let (state, init_cmd) = A::init(self.args);
        let config = self.config.unwrap_or_default();
        Runtime::with_backend_state_and_config(self.backend, state, init_cmd, config)
    }
}

// =============================================================================
// Entry points on Runtime
// =============================================================================

impl<A: App, B: Backend> Runtime<A, B> {
    /// Creates a [`RuntimeBuilder`] with the given backend.
    ///
    /// This is the most flexible entry point — it accepts any backend
    /// implementing [`Backend`]. For common backends, prefer the
    /// convenience methods:
    /// - [`terminal_builder()`](Runtime::terminal_builder) for real terminals
    /// - [`virtual_builder()`](Runtime::virtual_builder) for virtual terminals
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
    /// #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let backend = CaptureBackend::new(80, 24);
    /// let runtime = Runtime::<MyApp, _>::builder(backend).build()?;
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn builder(backend: B) -> RuntimeBuilder<A, B> {
        RuntimeBuilder::new(backend)
    }
}

// =============================================================================
// Terminal builder entry point
// =============================================================================

impl<A: App> Runtime<A, CrosstermBackend<Stdout>> {
    /// Creates a [`RuntimeBuilder`] for a real terminal.
    ///
    /// This performs terminal setup (raw mode, alternate screen, mouse
    /// capture) immediately and returns a builder for further configuration.
    /// The terminal remains in raw mode even if `build()` is never called,
    /// so callers should build promptly or handle cleanup.
    ///
    /// # Errors
    ///
    /// Returns an error if enabling raw mode, entering alternate screen,
    /// enabling mouse capture, or running the `on_setup` hook fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
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
    /// #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// # #[tokio::main]
    /// # async fn main() -> envision::Result<()> {
    /// let _final_state = Runtime::<MyApp, _>::terminal_builder()?
    ///     .build()?
    ///     .run_terminal()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn terminal_builder() -> error::Result<RuntimeBuilder<A, CrosstermBackend<Stdout>>> {
        let config = RuntimeConfig::default();
        let backend = Self::setup_terminal(&config)?;
        Ok(RuntimeBuilder::new(backend))
    }
}

// =============================================================================
// Virtual terminal builder entry point
// =============================================================================

impl<A: App> Runtime<A, CaptureBackend> {
    /// Creates a [`RuntimeBuilder`] for a virtual terminal.
    ///
    /// A virtual terminal is not connected to a physical terminal. Events
    /// are injected via `send()`, the application is advanced via `tick()`,
    /// and the display can be inspected via `display()`.
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
    /// #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
    /// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
    /// #     fn view(state: &MyState, frame: &mut Frame) {}
    /// # }
    /// let vt = Runtime::<MyApp, _>::virtual_builder(80, 24).build()?;
    /// # Ok::<(), envision::EnvisionError>(())
    /// ```
    pub fn virtual_builder(width: u16, height: u16) -> RuntimeBuilder<A, CaptureBackend> {
        let backend = CaptureBackend::new(width, height);
        RuntimeBuilder::new(backend)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::app::command::Command;
    use crate::app::model::App;
    use ratatui::widgets::Paragraph;

    // =========================================================================
    // Test App — Args = ()
    // =========================================================================

    struct TestApp;

    #[derive(Clone, Default)]
    struct TestState {
        count: i32,
        quit: bool,
    }

    #[derive(Clone, Debug)]
    enum TestMsg {
        Increment,
        Quit,
    }

    impl App for TestApp {
        type State = TestState;
        type Message = TestMsg;
        type Args = ();

        fn init(_args: ()) -> (Self::State, Command<Self::Message>) {
            (TestState::default(), Command::none())
        }

        fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
            match msg {
                TestMsg::Increment => state.count += 1,
                TestMsg::Quit => state.quit = true,
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

    // =========================================================================
    // ArgsApp — Args = TestState (used to exercise the with_args path)
    // =========================================================================

    struct ArgsApp;

    impl App for ArgsApp {
        type State = TestState;
        type Message = TestMsg;
        type Args = TestState;

        fn init(args: TestState) -> (Self::State, Command<Self::Message>) {
            (args, Command::none())
        }

        fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
            match msg {
                TestMsg::Increment => state.count += 1,
                TestMsg::Quit => state.quit = true,
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

    // =========================================================================
    // builder() — generic backend entry point
    // =========================================================================

    #[test]
    fn test_builder_with_capture_backend() {
        let backend = CaptureBackend::new(80, 24);
        let runtime = Runtime::<TestApp, _>::builder(backend).build().unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_builder_with_args() {
        let backend = CaptureBackend::new(80, 24);
        let state = TestState {
            count: 42,
            quit: false,
        };
        let runtime = Runtime::<ArgsApp, _>::builder(backend)
            .with_args(state)
            .build()
            .unwrap();
        assert_eq!(runtime.state().count, 42);
    }

    #[test]
    fn test_builder_with_config() {
        let backend = CaptureBackend::new(80, 24);
        let config = RuntimeConfig::new()
            .tick_rate(Duration::from_millis(100))
            .max_messages(50);
        let runtime = Runtime::<TestApp, _>::builder(backend)
            .config(config)
            .build()
            .unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_builder_with_args_and_config() {
        let backend = CaptureBackend::new(80, 24);
        let state = TestState {
            count: 7,
            quit: false,
        };
        let config = RuntimeConfig::new().tick_rate(Duration::from_millis(200));
        let runtime = Runtime::<ArgsApp, _>::builder(backend)
            .with_args(state)
            .config(config)
            .build()
            .unwrap();
        assert_eq!(runtime.state().count, 7);
    }

    // =========================================================================
    // virtual_builder() — CaptureBackend entry point
    // =========================================================================

    #[test]
    fn test_virtual_builder_default() {
        let runtime = Runtime::<TestApp, _>::virtual_builder(80, 24)
            .build()
            .unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_virtual_builder_with_args() {
        let state = TestState {
            count: 99,
            quit: false,
        };
        let runtime = Runtime::<ArgsApp, _>::virtual_builder(80, 24)
            .with_args(state)
            .build()
            .unwrap();
        assert_eq!(runtime.state().count, 99);
    }

    #[test]
    fn test_virtual_builder_with_tick_rate() {
        let mut runtime = Runtime::<TestApp, _>::virtual_builder(80, 24)
            .tick_rate(Duration::from_millis(200))
            .build()
            .unwrap();
        // Verify the runtime works (tick_rate is internal, but the runtime
        // should function correctly)
        runtime.dispatch(TestMsg::Increment);
        assert_eq!(runtime.state().count, 1);
    }

    #[test]
    fn test_virtual_builder_with_frame_rate() {
        let mut runtime = Runtime::<TestApp, _>::virtual_builder(80, 24)
            .frame_rate(Duration::from_millis(32))
            .build()
            .unwrap();
        runtime.tick().unwrap();
        assert!(runtime.contains_text("Count: 0"));
    }

    #[test]
    fn test_virtual_builder_with_max_messages() {
        let runtime = Runtime::<TestApp, _>::virtual_builder(80, 24)
            .max_messages(50)
            .build()
            .unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_virtual_builder_with_channel_capacity() {
        let runtime = Runtime::<TestApp, _>::virtual_builder(80, 24)
            .channel_capacity(512)
            .build()
            .unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_virtual_builder_chained_config() {
        let mut runtime = Runtime::<TestApp, _>::virtual_builder(80, 24)
            .tick_rate(Duration::from_millis(100))
            .frame_rate(Duration::from_millis(32))
            .max_messages(50)
            .channel_capacity(512)
            .build()
            .unwrap();

        runtime.dispatch(TestMsg::Increment);
        runtime.dispatch(TestMsg::Increment);
        runtime.tick().unwrap();
        assert_eq!(runtime.state().count, 2);
        assert!(runtime.contains_text("Count: 2"));
    }

    #[test]
    fn test_virtual_builder_args_and_config() {
        let state = TestState {
            count: 10,
            quit: false,
        };
        let mut runtime = Runtime::<ArgsApp, _>::virtual_builder(80, 24)
            .with_args(state)
            .tick_rate(Duration::from_millis(100))
            .build()
            .unwrap();

        assert_eq!(runtime.state().count, 10);
        runtime.dispatch(TestMsg::Increment);
        assert_eq!(runtime.state().count, 11);
    }

    #[test]
    fn test_virtual_builder_config_overrides_individual_settings() {
        // When .config() is called after individual settings, it replaces them
        let config = RuntimeConfig::new().tick_rate(Duration::from_millis(200));
        let runtime = Runtime::<TestApp, _>::virtual_builder(80, 24)
            .tick_rate(Duration::from_millis(50)) // this gets overridden
            .config(config)
            .build()
            .unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    #[test]
    fn test_virtual_builder_individual_settings_override_config() {
        // When individual settings are called after .config(), they modify it
        let config = RuntimeConfig::new().tick_rate(Duration::from_millis(200));
        let runtime = Runtime::<TestApp, _>::virtual_builder(80, 24)
            .config(config)
            .tick_rate(Duration::from_millis(50)) // this overrides the config's value
            .build()
            .unwrap();
        assert_eq!(runtime.state().count, 0);
    }

    // =========================================================================
    // Functional tests — verify built runtime works correctly
    // =========================================================================

    #[test]
    fn test_built_runtime_dispatch_and_render() {
        let mut runtime = Runtime::<TestApp, _>::virtual_builder(40, 10)
            .build()
            .unwrap();

        runtime.dispatch(TestMsg::Increment);
        runtime.dispatch(TestMsg::Increment);
        runtime.render().unwrap();

        assert!(runtime.contains_text("Count: 2"));
    }

    #[test]
    fn test_built_runtime_quit() {
        let mut runtime = Runtime::<TestApp, _>::virtual_builder(80, 24)
            .build()
            .unwrap();

        assert!(!runtime.should_quit());
        runtime.dispatch(TestMsg::Quit);
        runtime.tick().unwrap();
        assert!(runtime.should_quit());
    }

    #[test]
    fn test_built_runtime_send_and_tick() {
        use crate::input::Event;

        struct EventApp;

        #[derive(Clone, Default)]
        struct EventState {
            events: u32,
        }

        #[derive(Clone)]
        enum EventMsg {
            KeyPressed,
        }

        impl App for EventApp {
            type State = EventState;
            type Message = EventMsg;
            type Args = ();

            fn init(_args: ()) -> (Self::State, Command<Self::Message>) {
                (EventState::default(), Command::none())
            }
            fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
                match msg {
                    EventMsg::KeyPressed => state.events += 1,
                }
                Command::none()
            }
            fn view(state: &Self::State, frame: &mut ratatui::Frame) {
                let text = format!("Events: {}", state.events);
                frame.render_widget(Paragraph::new(text), frame.area());
            }
            fn handle_event(event: &crate::input::Event) -> Option<Self::Message> {
                if event.as_key().is_some() {
                    Some(EventMsg::KeyPressed)
                } else {
                    None
                }
            }
        }

        let mut runtime = Runtime::<EventApp, _>::virtual_builder(80, 24)
            .build()
            .unwrap();

        runtime.send(Event::char('a'));
        runtime.send(Event::char('b'));
        runtime.tick().unwrap();

        assert_eq!(runtime.state().events, 2);
    }

    #[test]
    fn test_built_runtime_with_args_uses_provided_state() {
        struct InitApp;

        #[derive(Clone, Default)]
        struct InitState {
            source: String,
        }

        #[derive(Clone)]
        enum InitMsg {}

        impl App for InitApp {
            type State = InitState;
            type Message = InitMsg;
            type Args = String;

            fn init(args: String) -> (Self::State, Command<Self::Message>) {
                (InitState { source: args }, Command::none())
            }
            fn update(_state: &mut Self::State, _msg: Self::Message) -> Command<Self::Message> {
                Command::none()
            }
            fn view(_state: &Self::State, _frame: &mut ratatui::Frame) {}
        }

        // Args carry the source string
        let runtime = Runtime::<InitApp, _>::virtual_builder(80, 24)
            .with_args("from args".into())
            .build()
            .unwrap();
        assert_eq!(runtime.state().source, "from args");
    }
}
