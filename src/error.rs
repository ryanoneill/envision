//! Error types for the Envision framework.
//!
//! This module provides structured error types for different failure modes
//! in Envision applications. For user-defined async errors that don't fit
//! these categories, [`BoxedError`] remains available.
//!
//! # Example
//!
//! ```rust
//! use envision::error::EnvisionError;
//!
//! fn setup() -> Result<(), EnvisionError> {
//!     // IO errors convert automatically
//!     let _file = std::fs::read_to_string("config.toml")
//!         .map_err(EnvisionError::from)?;
//!     Ok(())
//! }
//! ```

use std::fmt;

/// A boxed error type for user-defined async errors.
///
/// This type alias is provided for ergonomic use in async command handlers
/// where the specific error type is not known at compile time.
pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Structured error type for the Envision framework.
///
/// Represents the different categories of errors that can occur when
/// using Envision. Each variant provides structured context about the
/// failure mode, enabling callers to match on specific fields.
///
/// # Example
///
/// ```rust
/// use envision::error::EnvisionError;
///
/// let err = EnvisionError::config("theme", "invalid theme name");
/// assert_eq!(
///     err.to_string(),
///     "configuration error: field `theme`: invalid theme name"
/// );
/// ```
///
/// # Matching
///
/// `EnvisionError` is `#[non_exhaustive]`: new variants may be added in minor
/// releases, so a `match` outside this crate needs a wildcard arm.
///
/// ```rust
/// use envision::EnvisionError;
///
/// fn describe(e: &EnvisionError) -> &'static str {
///     match e {
///         EnvisionError::Io(_) => "io",
///         EnvisionError::Render { .. } => "render",
///         EnvisionError::Config { .. } => "config",
///         EnvisionError::Subscription { .. } => "subscription",
///         EnvisionError::Other(_) => "other",
///         _ => "unknown",
///     }
/// }
/// # assert_eq!(describe(&EnvisionError::config("f", "r")), "config");
/// ```
///
/// Omitting the wildcard arm does not compile:
///
/// ```compile_fail,E0004
/// use envision::EnvisionError;
///
/// fn describe(e: &EnvisionError) -> &'static str {
///     match e {
///         EnvisionError::Io(_) => "io",
///         EnvisionError::Render { .. } => "render",
///         EnvisionError::Config { .. } => "config",
///         EnvisionError::Subscription { .. } => "subscription",
///         EnvisionError::Other(_) => "other",
///     }
/// }
/// ```
///
/// # Custom backends
///
/// [`Runtime`](crate::Runtime) methods that render (`render`, `tick`, `run`,
/// `run_ticks`, `build`) require `EnvisionError: From<B::Error>`. Backends
/// whose error is `std::io::Error` (crossterm, termion, termwiz, termina)
/// satisfy it already. A backend whose error type is defined in another crate
/// cannot add that `From` impl itself (orphan rule); wrap the error in a local
/// newtype:
///
/// ```rust
/// # use envision::backend::Backend;
/// # use envision::prelude::*;
/// # use envision::ratatui::backend::{ClearType, WindowSize};
/// # use envision::ratatui::buffer::Cell;
/// # use envision::ratatui::layout::{Position, Size};
/// # use envision::{CaptureBackend, EnvisionError, Runtime};
/// #[derive(Debug)]
/// struct MyError(std::fmt::Error);
/// # impl std::fmt::Display for MyError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) } }
/// # impl std::error::Error for MyError {}
/// impl From<MyError> for EnvisionError {
///     fn from(e: MyError) -> Self {
///         EnvisionError::other(e)
///     }
/// }
/// # struct MyBackend(CaptureBackend);
/// # impl Backend for MyBackend {
/// #     type Error = MyError;
/// #     fn draw<'a, I: Iterator<Item = (u16, u16, &'a Cell)>>(&mut self, c: I) -> Result<(), MyError> { let Ok(()) = self.0.draw(c); Ok(()) }
/// #     fn hide_cursor(&mut self) -> Result<(), MyError> { Ok(()) }
/// #     fn show_cursor(&mut self) -> Result<(), MyError> { Ok(()) }
/// #     fn get_cursor_position(&mut self) -> Result<Position, MyError> { Ok(Position::ORIGIN) }
/// #     fn set_cursor_position<P: Into<Position>>(&mut self, _p: P) -> Result<(), MyError> { Ok(()) }
/// #     fn clear(&mut self) -> Result<(), MyError> { Ok(()) }
/// #     fn clear_region(&mut self, _c: ClearType) -> Result<(), MyError> { Ok(()) }
/// #     fn size(&self) -> Result<Size, MyError> { Ok(Size::new(4, 2)) }
/// #     fn window_size(&mut self) -> Result<WindowSize, MyError> { Ok(WindowSize { columns_rows: Size::new(4, 2), pixels: Size::new(32, 32) }) }
/// #     fn flush(&mut self) -> Result<(), MyError> { Ok(()) }
/// # }
/// # struct NoopApp;
/// # impl App for NoopApp { type State = (); type Message = (); type Args = ();
/// #   fn init(_: ()) -> ((), Command<()>) { ((), Command::none()) }
/// #   fn update(_: &mut (), _: ()) -> Command<()> { Command::none() }
/// #   fn view(_: &(), _: &mut Frame) {} }
/// let mut runtime = Runtime::<NoopApp, _>::builder(MyBackend(CaptureBackend::new(4, 2))).build()?;
/// runtime.render()?;
/// # Ok::<(), EnvisionError>(())
/// ```
///
/// Without the `From` impl, rendering does not compile:
///
/// ```compile_fail,E0277
/// # use envision::backend::Backend;
/// # use envision::prelude::*;
/// # use envision::ratatui::backend::{ClearType, WindowSize};
/// # use envision::ratatui::buffer::Cell;
/// # use envision::ratatui::layout::{Position, Size};
/// # use envision::{CaptureBackend, EnvisionError, Runtime};
/// #[derive(Debug)]
/// struct MyError(std::fmt::Error);
/// # impl std::fmt::Display for MyError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) } }
/// # impl std::error::Error for MyError {}
/// // no `impl From<MyError> for EnvisionError`
/// # struct MyBackend(CaptureBackend);
/// # impl Backend for MyBackend {
/// #     type Error = MyError;
/// #     fn draw<'a, I: Iterator<Item = (u16, u16, &'a Cell)>>(&mut self, c: I) -> Result<(), MyError> { let Ok(()) = self.0.draw(c); Ok(()) }
/// #     fn hide_cursor(&mut self) -> Result<(), MyError> { Ok(()) }
/// #     fn show_cursor(&mut self) -> Result<(), MyError> { Ok(()) }
/// #     fn get_cursor_position(&mut self) -> Result<Position, MyError> { Ok(Position::ORIGIN) }
/// #     fn set_cursor_position<P: Into<Position>>(&mut self, _p: P) -> Result<(), MyError> { Ok(()) }
/// #     fn clear(&mut self) -> Result<(), MyError> { Ok(()) }
/// #     fn clear_region(&mut self, _c: ClearType) -> Result<(), MyError> { Ok(()) }
/// #     fn size(&self) -> Result<Size, MyError> { Ok(Size::new(4, 2)) }
/// #     fn window_size(&mut self) -> Result<WindowSize, MyError> { Ok(WindowSize { columns_rows: Size::new(4, 2), pixels: Size::new(32, 32) }) }
/// #     fn flush(&mut self) -> Result<(), MyError> { Ok(()) }
/// # }
/// # struct NoopApp;
/// # impl App for NoopApp { type State = (); type Message = (); type Args = ();
/// #   fn init(_: ()) -> ((), Command<()>) { ((), Command::none()) }
/// #   fn update(_: &mut (), _: ()) -> Command<()> { Command::none() }
/// #   fn view(_: &(), _: &mut Frame) {} }
/// let mut runtime = Runtime::<NoopApp, _>::builder(MyBackend(CaptureBackend::new(4, 2))).build()?;
/// runtime.render()?;
/// # Ok::<(), EnvisionError>(())
/// ```
///
/// A generic helper that renders must carry the same bound:
///
/// ```rust
/// # use envision::backend::Backend;
/// # use envision::prelude::*;
/// # use envision::{EnvisionError, Runtime};
/// # struct NoopApp;
/// # impl App for NoopApp { type State = (); type Message = (); type Args = ();
/// #   fn init(_: ()) -> ((), Command<()>) { ((), Command::none()) }
/// #   fn update(_: &mut (), _: ()) -> Command<()> { Command::none() }
/// #   fn view(_: &(), _: &mut Frame) {} }
/// fn redraw<B: Backend>(runtime: &mut Runtime<NoopApp, B>) -> envision::error::Result<()>
/// where
///     EnvisionError: From<B::Error>,
/// {
///     runtime.render()
/// }
/// ```
///
/// ```compile_fail,E0277
/// # use envision::backend::Backend;
/// # use envision::prelude::*;
/// # use envision::Runtime;
/// # struct NoopApp;
/// # impl App for NoopApp { type State = (); type Message = (); type Args = ();
/// #   fn init(_: ()) -> ((), Command<()>) { ((), Command::none()) }
/// #   fn update(_: &mut (), _: ()) -> Command<()> { Command::none() }
/// #   fn view(_: &(), _: &mut Frame) {} }
/// fn redraw<B: Backend>(runtime: &mut Runtime<NoopApp, B>) -> envision::error::Result<()> {
///     runtime.render()
/// }
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub enum EnvisionError {
    /// An I/O error occurred (terminal, file system, etc.).
    Io(std::io::Error),

    /// A rendering error occurred.
    Render {
        /// The component that failed to render.
        component: &'static str,
        /// Details about the rendering failure.
        detail: String,
    },

    /// A configuration error occurred.
    Config {
        /// The configuration field that caused the error.
        field: String,
        /// The reason the configuration is invalid.
        reason: String,
    },

    /// A subscription error occurred.
    Subscription {
        /// The type of subscription that failed.
        subscription_type: &'static str,
        /// Details about the subscription failure.
        detail: String,
    },

    /// A catch-all variant for wrapping arbitrary errors.
    ///
    /// Use this when an error does not fit into the other structured
    /// categories. It wraps any error that implements
    /// `std::error::Error + Send + Sync + 'static`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::error::EnvisionError;
    ///
    /// let err = EnvisionError::other("something went wrong");
    /// assert_eq!(err.to_string(), "other error: something went wrong");
    /// ```
    Other(BoxedError),
}

impl EnvisionError {
    /// Creates a rendering error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::error::EnvisionError;
    ///
    /// let err = EnvisionError::render("ProgressBar", "width must be positive");
    /// assert_eq!(
    ///     err.to_string(),
    ///     "render error: component `ProgressBar`: width must be positive"
    /// );
    /// ```
    pub fn render(component: &'static str, detail: impl Into<String>) -> Self {
        EnvisionError::Render {
            component,
            detail: detail.into(),
        }
    }

    /// Creates a configuration error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::error::EnvisionError;
    ///
    /// let err = EnvisionError::config("theme", "unknown theme name");
    /// assert_eq!(
    ///     err.to_string(),
    ///     "configuration error: field `theme`: unknown theme name"
    /// );
    /// ```
    pub fn config(field: impl Into<String>, reason: impl Into<String>) -> Self {
        EnvisionError::Config {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Creates a subscription error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::error::EnvisionError;
    ///
    /// let err = EnvisionError::subscription("tick", "interval too small");
    /// assert_eq!(
    ///     err.to_string(),
    ///     "subscription error: type `tick`: interval too small"
    /// );
    /// ```
    pub fn subscription(subscription_type: &'static str, detail: impl Into<String>) -> Self {
        EnvisionError::Subscription {
            subscription_type,
            detail: detail.into(),
        }
    }

    /// Creates an `Other` error from any error type.
    ///
    /// This is a convenience constructor that wraps an arbitrary error
    /// into the [`Other`](EnvisionError::Other) variant, avoiding the
    /// need for manual `.map_err()` calls.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::error::EnvisionError;
    ///
    /// let err = EnvisionError::other("something went wrong");
    /// assert_eq!(err.to_string(), "other error: something went wrong");
    ///
    /// // Works with any error type
    /// let io_err = std::io::Error::other("disk full");
    /// let err = EnvisionError::other(io_err);
    /// assert!(err.to_string().contains("disk full"));
    /// ```
    pub fn other(err: impl Into<BoxedError>) -> Self {
        EnvisionError::Other(err.into())
    }
}

impl fmt::Display for EnvisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvisionError::Io(err) => write!(f, "I/O error: {err}"),
            EnvisionError::Render { component, detail } => {
                write!(f, "render error: component `{component}`: {detail}")
            }
            EnvisionError::Config { field, reason } => {
                write!(f, "configuration error: field `{field}`: {reason}")
            }
            EnvisionError::Subscription {
                subscription_type,
                detail,
            } => {
                write!(
                    f,
                    "subscription error: type `{subscription_type}`: {detail}"
                )
            }
            EnvisionError::Other(err) => write!(f, "other error: {err}"),
        }
    }
}

impl std::error::Error for EnvisionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EnvisionError::Io(err) => Some(err),
            EnvisionError::Other(err) => Some(err.as_ref()),
            EnvisionError::Render { .. }
            | EnvisionError::Config { .. }
            | EnvisionError::Subscription { .. } => None,
        }
    }
}

impl From<std::io::Error> for EnvisionError {
    fn from(err: std::io::Error) -> Self {
        EnvisionError::Io(err)
    }
}

/// Lets `?` propagate results from infallible backends such as
/// [`CaptureBackend`](crate::backend::CaptureBackend) into
/// [`Result`]. The body is unreachable by construction.
impl From<std::convert::Infallible> for EnvisionError {
    fn from(never: std::convert::Infallible) -> Self {
        match never {}
    }
}

/// A [`Result`] type alias using [`EnvisionError`].
///
/// This is the standard result type returned by most Envision operations
/// (runtime construction, rendering, ticking, etc.).
///
/// # Example
///
/// ```rust
/// use envision::error::Result;
///
/// fn setup() -> Result<()> {
///     // io::Error converts automatically via From
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, EnvisionError>;

#[cfg(test)]
mod tests {
    #[test]
    fn infallible_converts_via_question_mark() {
        fn uses_question_mark() -> Result<u8> {
            let value: std::result::Result<u8, std::convert::Infallible> = Ok(7);
            Ok(value?)
        }
        assert_eq!(uses_question_mark().unwrap(), 7);
    }

    use super::*;

    #[test]
    fn io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = EnvisionError::from(io_err);
        assert_eq!(err.to_string(), "I/O error: file not found");
    }

    #[test]
    fn render_error_display() {
        let err = EnvisionError::render("ProgressBar", "failed to draw widget");
        assert_eq!(
            err.to_string(),
            "render error: component `ProgressBar`: failed to draw widget"
        );
    }

    #[test]
    fn config_error_display() {
        let err = EnvisionError::config("theme", "invalid theme name");
        assert_eq!(
            err.to_string(),
            "configuration error: field `theme`: invalid theme name"
        );
    }

    #[test]
    fn subscription_error_display() {
        let err = EnvisionError::subscription("tick", "interval too small");
        assert_eq!(
            err.to_string(),
            "subscription error: type `tick`: interval too small"
        );
    }

    #[test]
    fn io_error_from_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err: EnvisionError = io_err.into();
        assert!(matches!(err, EnvisionError::Io(_)));
    }

    #[test]
    fn io_error_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err = EnvisionError::from(io_err);
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn render_error_no_source() {
        let err = EnvisionError::render("Widget", "bad render");
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn config_error_no_source() {
        let err = EnvisionError::config("key", "bad config");
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn subscription_error_no_source() {
        let err = EnvisionError::subscription("tick", "bad sub");
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn debug_format() {
        let err = EnvisionError::config("key", "test");
        let debug = format!("{err:?}");
        assert!(debug.contains("Config"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn boxed_error_alias_works() {
        fn returns_boxed() -> std::result::Result<(), BoxedError> {
            Err("test error".into())
        }
        assert!(returns_boxed().is_err());
    }

    #[test]
    fn render_error_fields_accessible() {
        let err = EnvisionError::render("Table", "column overflow");
        match err {
            EnvisionError::Render { component, detail } => {
                assert_eq!(component, "Table");
                assert_eq!(detail, "column overflow");
            }
            _ => panic!("expected Render variant"),
        }
    }

    #[test]
    fn config_error_fields_accessible() {
        let err = EnvisionError::config("tick_rate", "must be positive");
        match err {
            EnvisionError::Config { field, reason } => {
                assert_eq!(field, "tick_rate");
                assert_eq!(reason, "must be positive");
            }
            _ => panic!("expected Config variant"),
        }
    }

    #[test]
    fn subscription_error_fields_accessible() {
        let err = EnvisionError::subscription("interval", "already running");
        match err {
            EnvisionError::Subscription {
                subscription_type,
                detail,
            } => {
                assert_eq!(subscription_type, "interval");
                assert_eq!(detail, "already running");
            }
            _ => panic!("expected Subscription variant"),
        }
    }

    #[test]
    fn other_from_string_error() {
        let err = EnvisionError::Other("something went wrong".into());
        assert!(matches!(err, EnvisionError::Other(_)));
    }

    #[test]
    fn other_from_custom_error_type() {
        #[derive(Debug)]
        struct CustomError;

        impl fmt::Display for CustomError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "custom error occurred")
            }
        }

        impl std::error::Error for CustomError {}

        let err = EnvisionError::Other(Box::new(CustomError));
        assert!(matches!(err, EnvisionError::Other(_)));
        assert_eq!(err.to_string(), "other error: custom error occurred");
    }

    #[test]
    fn other_error_display() {
        let err = EnvisionError::Other("unexpected failure".into());
        assert_eq!(err.to_string(), "other error: unexpected failure");
    }

    #[test]
    fn other_error_source() {
        let io_err = std::io::Error::other("inner error");
        let err = EnvisionError::Other(Box::new(io_err));
        let source = std::error::Error::source(&err);
        assert!(source.is_some());
        assert_eq!(source.unwrap().to_string(), "inner error");
    }

    #[test]
    fn other_convenience_constructor() {
        let err = EnvisionError::other("convenience test");
        assert!(matches!(err, EnvisionError::Other(_)));
        assert_eq!(err.to_string(), "other error: convenience test");
    }

    #[test]
    fn other_convenience_constructor_with_io_error() {
        let io_err = std::io::Error::other("disk full");
        let err = EnvisionError::other(io_err);
        assert!(matches!(err, EnvisionError::Other(_)));
        assert!(err.to_string().contains("disk full"));
    }
}
