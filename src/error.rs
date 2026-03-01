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
/// using Envision. Each variant provides context about the failure mode.
///
/// # Example
///
/// ```rust
/// use envision::error::EnvisionError;
///
/// let err = EnvisionError::Config("invalid theme name".into());
/// assert_eq!(err.to_string(), "configuration error: invalid theme name");
/// ```
#[derive(Debug)]
pub enum EnvisionError {
    /// An I/O error occurred (terminal, file system, etc.).
    Io(std::io::Error),

    /// A rendering error occurred.
    Render(String),

    /// A configuration error occurred.
    Config(String),

    /// A subscription error occurred.
    Subscription(String),
}

impl fmt::Display for EnvisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvisionError::Io(err) => write!(f, "I/O error: {}", err),
            EnvisionError::Render(msg) => write!(f, "render error: {}", msg),
            EnvisionError::Config(msg) => write!(f, "configuration error: {}", msg),
            EnvisionError::Subscription(msg) => write!(f, "subscription error: {}", msg),
        }
    }
}

impl std::error::Error for EnvisionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EnvisionError::Io(err) => Some(err),
            EnvisionError::Render(_) | EnvisionError::Config(_) | EnvisionError::Subscription(_) => {
                None
            }
        }
    }
}

impl From<std::io::Error> for EnvisionError {
    fn from(err: std::io::Error) -> Self {
        EnvisionError::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = EnvisionError::from(io_err);
        assert_eq!(err.to_string(), "I/O error: file not found");
    }

    #[test]
    fn render_error_display() {
        let err = EnvisionError::Render("failed to draw widget".into());
        assert_eq!(err.to_string(), "render error: failed to draw widget");
    }

    #[test]
    fn config_error_display() {
        let err = EnvisionError::Config("invalid theme name".into());
        assert_eq!(err.to_string(), "configuration error: invalid theme name");
    }

    #[test]
    fn subscription_error_display() {
        let err = EnvisionError::Subscription("tick interval too small".into());
        assert_eq!(
            err.to_string(),
            "subscription error: tick interval too small"
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
        let err = EnvisionError::Render("bad render".into());
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn config_error_no_source() {
        let err = EnvisionError::Config("bad config".into());
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn subscription_error_no_source() {
        let err = EnvisionError::Subscription("bad sub".into());
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn debug_format() {
        let err = EnvisionError::Config("test".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Config"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn boxed_error_alias_works() {
        fn returns_boxed() -> Result<(), BoxedError> {
            Err("test error".into())
        }
        assert!(returns_boxed().is_err());
    }
}
