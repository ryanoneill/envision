//! Status log entry types.
//!
//! Contains [`StatusLogLevel`] and [`StatusLogEntry`] used by the
//! [`StatusLog`](super::StatusLog) component.

use ratatui::prelude::*;

/// Severity level for status log entries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum StatusLogLevel {
    /// Informational message.
    #[default]
    Info,
    /// Success message.
    Success,
    /// Warning message.
    Warning,
    /// Error message.
    Error,
}

impl StatusLogLevel {
    /// Returns the color associated with this level.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogLevel;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(StatusLogLevel::Info.color(), Color::Cyan);
    /// assert_eq!(StatusLogLevel::Success.color(), Color::Green);
    /// assert_eq!(StatusLogLevel::Warning.color(), Color::Yellow);
    /// assert_eq!(StatusLogLevel::Error.color(), Color::Red);
    /// ```
    pub fn color(&self) -> Color {
        match self {
            StatusLogLevel::Info => Color::Cyan,
            StatusLogLevel::Success => Color::Green,
            StatusLogLevel::Warning => Color::Yellow,
            StatusLogLevel::Error => Color::Red,
        }
    }

    /// Returns the prefix symbol for this level.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogLevel;
    ///
    /// assert_eq!(StatusLogLevel::Error.prefix(), "\u{2717}");
    /// assert_eq!(StatusLogLevel::Success.prefix(), "\u{2713}");
    /// ```
    pub fn prefix(&self) -> &'static str {
        match self {
            StatusLogLevel::Info => "ℹ",
            StatusLogLevel::Success => "✓",
            StatusLogLevel::Warning => "⚠",
            StatusLogLevel::Error => "✗",
        }
    }
}

/// A single status log entry.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct StatusLogEntry {
    /// Unique identifier.
    pub(super) id: u64,
    /// The message content.
    pub(super) message: String,
    /// Severity level.
    pub(super) level: StatusLogLevel,
    /// Optional timestamp string.
    pub(super) timestamp: Option<String>,
}

impl StatusLogEntry {
    /// Creates a new status log entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusLogEntry, StatusLogLevel};
    ///
    /// let entry = StatusLogEntry::new(0, "Starting", StatusLogLevel::Info);
    /// assert_eq!(entry.id(), 0);
    /// assert_eq!(entry.message(), "Starting");
    /// assert_eq!(entry.level(), StatusLogLevel::Info);
    /// assert_eq!(entry.timestamp(), None);
    /// ```
    pub fn new(id: u64, message: impl Into<String>, level: StatusLogLevel) -> Self {
        Self {
            id,
            message: message.into(),
            level,
            timestamp: None,
        }
    }

    /// Creates a new entry with a timestamp.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusLogEntry, StatusLogLevel};
    ///
    /// let entry = StatusLogEntry::with_timestamp(1, "Done", StatusLogLevel::Success, "12:00:00");
    /// assert_eq!(entry.timestamp(), Some("12:00:00"));
    /// ```
    pub fn with_timestamp(
        id: u64,
        message: impl Into<String>,
        level: StatusLogLevel,
        timestamp: impl Into<String>,
    ) -> Self {
        Self {
            id,
            message: message.into(),
            level,
            timestamp: Some(timestamp.into()),
        }
    }

    /// Returns the entry ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusLogEntry, StatusLogLevel};
    ///
    /// let entry = StatusLogEntry::new(42, "Test", StatusLogLevel::Info);
    /// assert_eq!(entry.id(), 42);
    /// ```
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusLogEntry, StatusLogLevel};
    ///
    /// let entry = StatusLogEntry::new(0, "Hello", StatusLogLevel::Info);
    /// assert_eq!(entry.message(), "Hello");
    /// ```
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the level.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusLogEntry, StatusLogLevel};
    ///
    /// let entry = StatusLogEntry::new(0, "Warning!", StatusLogLevel::Warning);
    /// assert_eq!(entry.level(), StatusLogLevel::Warning);
    /// ```
    pub fn level(&self) -> StatusLogLevel {
        self.level
    }

    /// Returns the timestamp if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusLogEntry, StatusLogLevel};
    ///
    /// let entry = StatusLogEntry::new(0, "No time", StatusLogLevel::Info);
    /// assert_eq!(entry.timestamp(), None);
    /// ```
    pub fn timestamp(&self) -> Option<&str> {
        self.timestamp.as_deref()
    }
}
