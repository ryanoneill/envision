use ratatui::prelude::*;

/// Severity level for a stream event.
///
/// Levels are ordered from least to most severe, allowing minimum-level
/// filtering (e.g., show only Warning and above).
///
/// # Example
///
/// ```rust
/// use envision::component::EventLevel;
///
/// assert!(EventLevel::Error > EventLevel::Info);
/// assert!(EventLevel::Trace < EventLevel::Debug);
/// assert_eq!(EventLevel::default(), EventLevel::Info);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum EventLevel {
    /// Finest-grained informational events.
    Trace,
    /// Fine-grained informational events useful for debugging.
    Debug,
    /// Informational messages highlighting normal operation.
    #[default]
    Info,
    /// Potentially harmful situations.
    Warning,
    /// Error events that might still allow the application to continue.
    Error,
    /// Very severe error events that will likely cause the application to abort.
    Fatal,
}

impl EventLevel {
    /// Returns a short abbreviation for display (3 chars).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventLevel;
    ///
    /// assert_eq!(EventLevel::Info.abbreviation(), "INF");
    /// assert_eq!(EventLevel::Error.abbreviation(), "ERR");
    /// ```
    pub fn abbreviation(&self) -> &'static str {
        match self {
            EventLevel::Trace => "TRC",
            EventLevel::Debug => "DBG",
            EventLevel::Info => "INF",
            EventLevel::Warning => "WRN",
            EventLevel::Error => "ERR",
            EventLevel::Fatal => "FTL",
        }
    }

    /// Returns the display color for this severity level.
    ///
    /// - Trace: DarkGray (dim)
    /// - Debug: Gray
    /// - Info: Blue
    /// - Warning: Yellow
    /// - Error: Red
    /// - Fatal: LightRed (bright red)
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventLevel;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(EventLevel::Info.color(), Color::Blue);
    /// assert_eq!(EventLevel::Error.color(), Color::Red);
    /// ```
    pub fn color(&self) -> Color {
        match self {
            EventLevel::Trace => Color::DarkGray,
            EventLevel::Debug => Color::Gray,
            EventLevel::Info => Color::Blue,
            EventLevel::Warning => Color::Yellow,
            EventLevel::Error => Color::Red,
            EventLevel::Fatal => Color::LightRed,
        }
    }
}

impl std::fmt::Display for EventLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// A structured event with key-value fields.
///
/// Each event has an ID, timestamp, severity level, message, and optional
/// structured fields and source name. Fields are key-value pairs that
/// can be displayed as columns in the event stream.
///
/// # Example
///
/// ```rust
/// use envision::component::{EventLevel, StreamEvent};
///
/// let event = StreamEvent::new(1, 1000.0, EventLevel::Info, "Request received")
///     .with_field("path", "/api/users")
///     .with_field("method", "GET")
///     .with_source("api");
///
/// assert_eq!(event.message, "Request received");
/// assert_eq!(event.fields.len(), 2);
/// assert_eq!(event.source, Some("api".to_string()));
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct StreamEvent {
    /// Unique identifier.
    pub id: u64,
    /// Timestamp (milliseconds from epoch or any unit).
    pub timestamp: f64,
    /// Severity level.
    pub level: EventLevel,
    /// Short message/summary.
    pub message: String,
    /// Structured key-value fields.
    pub fields: Vec<(String, String)>,
    /// Optional source/service name.
    pub source: Option<String>,
}

impl StreamEvent {
    /// Creates a new event with the given ID, timestamp, level, and message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{EventLevel, StreamEvent};
    ///
    /// let event = StreamEvent::new(1, 1000.0, EventLevel::Info, "Hello");
    /// assert_eq!(event.id, 1);
    /// assert_eq!(event.message, "Hello");
    /// assert!(event.fields.is_empty());
    /// assert!(event.source.is_none());
    /// ```
    pub fn new(id: u64, timestamp: f64, level: EventLevel, message: impl Into<String>) -> Self {
        Self {
            id,
            timestamp,
            level,
            message: message.into(),
            fields: Vec::new(),
            source: None,
        }
    }

    /// Adds a key-value field to the event (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{EventLevel, StreamEvent};
    ///
    /// let event = StreamEvent::new(1, 0.0, EventLevel::Info, "Query")
    ///     .with_field("ms", "45")
    ///     .with_field("table", "users");
    /// assert_eq!(event.fields.len(), 2);
    /// ```
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((key.into(), value.into()));
        self
    }

    /// Sets the source/service name (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{EventLevel, StreamEvent};
    ///
    /// let event = StreamEvent::new(1, 0.0, EventLevel::Info, "msg")
    ///     .with_source("api");
    /// assert_eq!(event.source, Some("api".to_string()));
    /// ```
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}
