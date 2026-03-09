//! Runtime configuration.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

/// A hook callback invoked during terminal lifecycle events.
///
/// Called after terminal setup and before terminal teardown respectively.
/// Return `Ok(())` to continue normally, or `Err` to propagate the error.
pub type TerminalHook = Arc<dyn Fn() -> std::io::Result<()> + Send + Sync>;

/// Configuration for the runtime.
#[derive(Clone)]
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

    /// Hook called after terminal setup (raw mode, alternate screen, mouse capture).
    ///
    /// Use this to redirect stderr, configure logging, or perform other
    /// initialization that depends on the terminal being in raw mode.
    pub on_setup: Option<TerminalHook>,

    /// Hook called before terminal teardown (restoring normal mode).
    ///
    /// Use this to flush logs, restore stderr, or perform other cleanup
    /// before the terminal is restored to normal mode.
    pub on_teardown: Option<TerminalHook>,
}

impl fmt::Debug for RuntimeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuntimeConfig")
            .field("tick_rate", &self.tick_rate)
            .field("frame_rate", &self.frame_rate)
            .field("max_messages_per_tick", &self.max_messages_per_tick)
            .field("capture_history", &self.capture_history)
            .field("history_capacity", &self.history_capacity)
            .field("message_channel_capacity", &self.message_channel_capacity)
            .field("on_setup", &self.on_setup.as_ref().map(|_| "<hook>"))
            .field("on_teardown", &self.on_teardown.as_ref().map(|_| "<hook>"))
            .finish()
    }
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
            on_setup: None,
            on_teardown: None,
        }
    }
}

impl RuntimeConfig {
    /// Creates a new runtime config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the tick rate.
    pub fn tick_rate(mut self, rate: Duration) -> Self {
        self.tick_rate = rate;
        self
    }

    /// Sets the frame rate.
    pub fn frame_rate(mut self, rate: Duration) -> Self {
        self.frame_rate = rate;
        self
    }

    /// Enables frame history capture.
    pub fn with_history(mut self, capacity: usize) -> Self {
        self.capture_history = true;
        self.history_capacity = capacity;
        self
    }

    /// Sets the maximum messages per tick.
    pub fn max_messages(mut self, max: usize) -> Self {
        self.max_messages_per_tick = max;
        self
    }

    /// Sets the message channel capacity.
    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.message_channel_capacity = capacity;
        self
    }

    /// Sets a hook to be called after terminal setup.
    ///
    /// The hook runs after raw mode, alternate screen, and mouse capture
    /// have been enabled. Use this for redirecting stderr, configuring
    /// logging, or other setup that depends on the terminal state.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use envision::RuntimeConfig;
    ///
    /// let config = RuntimeConfig::new()
    ///     .on_setup(Arc::new(|| {
    ///         // Redirect stderr to a file for logging
    ///         eprintln!("Terminal is set up");
    ///         Ok(())
    ///     }));
    /// ```
    pub fn on_setup(mut self, hook: TerminalHook) -> Self {
        self.on_setup = Some(hook);
        self
    }

    /// Sets a hook to be called before terminal teardown.
    ///
    /// The hook runs before raw mode is disabled, the alternate screen is
    /// left, and mouse capture is disabled. Use this for flushing logs,
    /// restoring stderr, or other cleanup.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use envision::RuntimeConfig;
    ///
    /// let config = RuntimeConfig::new()
    ///     .on_teardown(Arc::new(|| {
    ///         eprintln!("Terminal is being torn down");
    ///         Ok(())
    ///     }));
    /// ```
    pub fn on_teardown(mut self, hook: TerminalHook) -> Self {
        self.on_teardown = Some(hook);
        self
    }

    /// Sets a hook to be called after terminal setup, accepting a `FnOnce` closure.
    ///
    /// This is a convenience wrapper around [`on_setup`](Self::on_setup) for closures
    /// that consume captured state. The closure runs at most once; subsequent calls
    /// (e.g., on a cloned config) are no-ops.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use envision::RuntimeConfig;
    ///
    /// let config = RuntimeConfig::new()
    ///     .on_setup_once(|| {
    ///         // Move captured values into this closure
    ///         eprintln!("Terminal is set up");
    ///         Ok(())
    ///     });
    /// ```
    pub fn on_setup_once<F>(self, hook: F) -> Self
    where
        F: FnOnce() -> std::io::Result<()> + Send + Sync + 'static,
    {
        let hook = std::sync::Mutex::new(Some(hook));
        self.on_setup(Arc::new(move || {
            if let Some(f) = hook.lock().unwrap().take() {
                f()
            } else {
                Ok(())
            }
        }))
    }

    /// Sets a hook to be called before terminal teardown, accepting a `FnOnce` closure.
    ///
    /// This is a convenience wrapper around [`on_teardown`](Self::on_teardown) for closures
    /// that consume captured state. The closure runs at most once; subsequent calls
    /// (e.g., on a cloned config) are no-ops.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use envision::RuntimeConfig;
    ///
    /// let config = RuntimeConfig::new()
    ///     .on_teardown_once(|| {
    ///         // Move captured values into this closure
    ///         eprintln!("Terminal is being torn down");
    ///         Ok(())
    ///     });
    /// ```
    pub fn on_teardown_once<F>(self, hook: F) -> Self
    where
        F: FnOnce() -> std::io::Result<()> + Send + Sync + 'static,
    {
        let hook = std::sync::Mutex::new(Some(hook));
        self.on_teardown(Arc::new(move || {
            if let Some(f) = hook.lock().unwrap().take() {
                f()
            } else {
                Ok(())
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_no_hooks() {
        let config = RuntimeConfig::default();
        assert!(config.on_setup.is_none());
        assert!(config.on_teardown.is_none());
    }

    #[test]
    fn test_on_setup_hook_stored() {
        let config = RuntimeConfig::new().on_setup(Arc::new(|| Ok(())));
        assert!(config.on_setup.is_some());
        assert!(config.on_teardown.is_none());
    }

    #[test]
    fn test_on_teardown_hook_stored() {
        let config = RuntimeConfig::new().on_teardown(Arc::new(|| Ok(())));
        assert!(config.on_setup.is_none());
        assert!(config.on_teardown.is_some());
    }

    #[test]
    fn test_both_hooks_stored() {
        let config = RuntimeConfig::new()
            .on_setup(Arc::new(|| Ok(())))
            .on_teardown(Arc::new(|| Ok(())));
        assert!(config.on_setup.is_some());
        assert!(config.on_teardown.is_some());
    }

    #[test]
    fn test_hooks_are_callable() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let setup_called = Arc::new(AtomicBool::new(false));
        let teardown_called = Arc::new(AtomicBool::new(false));

        let setup_flag = setup_called.clone();
        let teardown_flag = teardown_called.clone();

        let config = RuntimeConfig::new()
            .on_setup(Arc::new(move || {
                setup_flag.store(true, Ordering::SeqCst);
                Ok(())
            }))
            .on_teardown(Arc::new(move || {
                teardown_flag.store(true, Ordering::SeqCst);
                Ok(())
            }));

        config.on_setup.as_ref().unwrap()().unwrap();
        assert!(setup_called.load(Ordering::SeqCst));

        config.on_teardown.as_ref().unwrap()().unwrap();
        assert!(teardown_called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_hook_error_propagation() {
        let config =
            RuntimeConfig::new().on_setup(Arc::new(|| Err(std::io::Error::other("setup failed"))));

        let result = config.on_setup.as_ref().unwrap()();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("setup failed"));
    }

    #[test]
    fn test_config_clone_with_hooks() {
        let config = RuntimeConfig::new()
            .on_setup(Arc::new(|| Ok(())))
            .on_teardown(Arc::new(|| Ok(())));

        let cloned = config.clone();
        assert!(cloned.on_setup.is_some());
        assert!(cloned.on_teardown.is_some());
    }

    #[test]
    fn test_config_debug_with_hooks() {
        let config = RuntimeConfig::new()
            .on_setup(Arc::new(|| Ok(())))
            .on_teardown(Arc::new(|| Ok(())));

        let debug = format!("{:?}", config);
        assert!(debug.contains("on_setup"));
        assert!(debug.contains("on_teardown"));
        assert!(debug.contains("<hook>"));
    }

    #[test]
    fn test_config_debug_without_hooks() {
        let config = RuntimeConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("on_setup: None"));
        assert!(debug.contains("on_teardown: None"));
    }

    #[test]
    fn test_on_setup_once_stored() {
        let config = RuntimeConfig::new().on_setup_once(|| Ok(()));
        assert!(config.on_setup.is_some());
        assert!(config.on_teardown.is_none());
    }

    #[test]
    fn test_on_teardown_once_stored() {
        let config = RuntimeConfig::new().on_teardown_once(|| Ok(()));
        assert!(config.on_setup.is_none());
        assert!(config.on_teardown.is_some());
    }

    #[test]
    fn test_on_setup_once_callable() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let called = Arc::new(AtomicBool::new(false));
        let flag = called.clone();

        let config = RuntimeConfig::new().on_setup_once(move || {
            flag.store(true, Ordering::SeqCst);
            Ok(())
        });

        config.on_setup.as_ref().unwrap()().unwrap();
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_on_teardown_once_callable() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let called = Arc::new(AtomicBool::new(false));
        let flag = called.clone();

        let config = RuntimeConfig::new().on_teardown_once(move || {
            flag.store(true, Ordering::SeqCst);
            Ok(())
        });

        config.on_teardown.as_ref().unwrap()().unwrap();
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_on_setup_once_runs_only_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let call_count = Arc::new(AtomicUsize::new(0));
        let counter = call_count.clone();

        let config = RuntimeConfig::new().on_setup_once(move || {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        let hook = config.on_setup.as_ref().unwrap();
        hook().unwrap();
        hook().unwrap();
        hook().unwrap();

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_on_setup_once_with_consuming_capture() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let dropped = Arc::new(AtomicBool::new(false));

        struct Guard {
            flag: Arc<AtomicBool>,
        }

        impl Drop for Guard {
            fn drop(&mut self) {
                self.flag.store(true, Ordering::SeqCst);
            }
        }

        let guard = Guard {
            flag: dropped.clone(),
        };

        let config = RuntimeConfig::new().on_setup_once(move || {
            drop(guard);
            Ok(())
        });

        assert!(!dropped.load(Ordering::SeqCst));
        config.on_setup.as_ref().unwrap()().unwrap();
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[test]
    fn test_cloned_config_once_hook_runs_on_first_only() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let call_count = Arc::new(AtomicUsize::new(0));
        let counter = call_count.clone();

        let config = RuntimeConfig::new().on_setup_once(move || {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        let cloned = config.clone();

        // Call on original - should run the FnOnce
        config.on_setup.as_ref().unwrap()().unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Call on clone - shares the same Arc, so FnOnce is already consumed
        cloned.on_setup.as_ref().unwrap()().unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }
}
