//! [`ConfiguredRuntimeBuilder`] — the typestate `RuntimeBuilder` transitions
//! into after [`with_args`](super::RuntimeBuilder::with_args) is called.
//!
//! Extracted from `builder.rs` to keep that file under the project's
//! 1000-line ceiling. Public API surface unchanged — both
//! [`RuntimeBuilder`](super::RuntimeBuilder) and `ConfiguredRuntimeBuilder`
//! are re-exported from `app::runtime`.

use std::time::Duration;

use ratatui::backend::Backend;

use super::Runtime;
use super::config::RuntimeConfig;
use crate::app::model::App;
use crate::error;

/// A `RuntimeBuilder` after [`with_args`](super::RuntimeBuilder::with_args) has been called.
///
/// Carries the args that will be passed to [`App::init`] plus all
/// configuration set so far. Has its own fluent config-shaping methods
/// (`config`, `tick_rate`, `frame_rate`, `max_messages`, `channel_capacity`)
/// and an unconditionally-available `build()`.
///
/// Most users never name this type — the typestate transition happens
/// implicitly when chaining `.with_args(...).build()`.
pub struct ConfiguredRuntimeBuilder<A: App, B: Backend> {
    pub(super) backend: B,
    pub(super) config: Option<RuntimeConfig>,
    pub(super) args: A::Args,
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
