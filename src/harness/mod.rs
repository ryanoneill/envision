//! Test harness for headless TUI testing.
//!
//! Envision provides three testing entry points. Pick the one that
//! matches the scope of what you're testing:
//!
//! | Harness | Use when… | Closure or App? | Time control |
//! |---|---|---|---|
//! | [`TestHarness`] | Testing a render closure or a widget in isolation | Closure | None (synchronous) |
//! | [`AppHarness`] | Testing a full `App` with async commands/subscriptions | App | Via `tokio::test(start_paused)` + `advance_time()` |
//! | [`Runtime::virtual_builder`][vb] | Programmatic control (agents, scripted demos, integration tests) | App | None |
//!
//! [vb]: crate::app::Runtime::virtual_builder
//!
//! ## `TestHarness` — widget-level testing
//!
//! Wraps a `CaptureBackend` with input simulation and assertion helpers.
//! Renders closures, not full `App` implementations. Synchronous — no
//! async runtime.
//!
//! ## `AppHarness` — App-level async testing
//!
//! Wraps a `Runtime<A, CaptureBackend>` and exposes time-control
//! primitives that pair with `#[tokio::test(start_paused = true)]`. Use
//! when your `App` has subscriptions, commands, or any time-dependent
//! logic.
//!
//! ## `Runtime::virtual_builder` — programmatic App control
//!
//! Constructed via `Runtime::<A, _>::virtual_builder(w, h).build()`.
//! Returns a `Runtime<A, CaptureBackend>` with `send()`, `dispatch()`,
//! `tick()`, and `display()` methods. Useful for AI agents, scripted
//! demos, and integration tests that need full App semantics without
//! the time-control ceremony of `AppHarness`.
//!
//! See `examples/test_harness.rs` for runnable examples of all three.
//!
//! # Example: TestHarness
//!
//! ```rust
//! use envision::harness::{TestHarness, Assertion};
//! use envision::annotation::Annotation;
//! use ratatui::widgets::Paragraph;
//!
//! let mut harness = TestHarness::new(80, 24);
//!
//! harness.render(|frame| {
//!     frame.render_widget(
//!         Paragraph::new("Hello, World!"),
//!         frame.area(),
//!     );
//! }).unwrap();
//!
//! harness.assert_contains("Hello, World!");
//! ```

mod app_harness;
mod assertions;
mod snapshot;
mod test_harness;

pub use app_harness::AppHarness;
pub use assertions::{Assertion, AssertionError, AssertionResult};
pub use snapshot::{
    Snapshot, SnapshotFormat, SnapshotTest, assert_snapshot_eq, assert_snapshot_text,
};
pub use test_harness::TestHarness;
