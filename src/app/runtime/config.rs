//! Runtime configuration.

use std::time::Duration;

/// Configuration for the runtime.
#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    /// How often to poll for events (default: 50ms)
    pub tick_rate: Duration,

    /// How often to render (default: 16ms for ~60fps)
    pub frame_rate: Duration,

    /// Maximum number of messages to process per tick (prevents infinite loops)
    pub max_messages_per_tick: usize,

    /// Whether to capture frame history
    pub capture_history: bool,

    /// Number of frames to keep in history
    pub history_capacity: usize,

    /// Capacity of the async message channel
    pub message_channel_capacity: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(50),
            frame_rate: Duration::from_millis(16),
            max_messages_per_tick: 100,
            capture_history: false,
            history_capacity: 10,
            message_channel_capacity: 256,
        }
    }
}

impl RuntimeConfig {
    /// Creates a new runtime config with default settings.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::RuntimeConfig;
    ///
    /// let config = RuntimeConfig::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the tick rate.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::RuntimeConfig;
    /// use std::time::Duration;
    ///
    /// let config = RuntimeConfig::new().tick_rate(Duration::from_millis(100));
    /// ```
    pub fn tick_rate(mut self, rate: Duration) -> Self {
        self.tick_rate = rate;
        self
    }

    /// Sets the frame rate.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::RuntimeConfig;
    /// use std::time::Duration;
    ///
    /// // ~30fps
    /// let config = RuntimeConfig::new().frame_rate(Duration::from_millis(33));
    /// ```
    pub fn frame_rate(mut self, rate: Duration) -> Self {
        self.frame_rate = rate;
        self
    }

    /// Enables frame history capture.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::RuntimeConfig;
    ///
    /// let config = RuntimeConfig::new().with_history(5);
    /// ```
    pub fn with_history(mut self, capacity: usize) -> Self {
        self.capture_history = true;
        self.history_capacity = capacity;
        self
    }

    /// Sets the maximum messages per tick.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::RuntimeConfig;
    ///
    /// let config = RuntimeConfig::new().max_messages(50);
    /// ```
    pub fn max_messages(mut self, max: usize) -> Self {
        self.max_messages_per_tick = max;
        self
    }

    /// Sets the message channel capacity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::RuntimeConfig;
    ///
    /// let config = RuntimeConfig::new().channel_capacity(512);
    /// ```
    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.message_channel_capacity = capacity;
        self
    }
}
