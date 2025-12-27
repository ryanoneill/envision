//! Test harness for headless TUI testing.
//!
//! The harness module provides a unified testing interface that combines:
//!
//! - `CaptureBackend` for headless rendering
//! - Input simulation for programmatic interaction
//! - Widget annotations for semantic queries
//!
//! # Example
//!
//! ```rust,no_run
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
//! });
//!
//! harness.assert_contains("Hello, World!");
//! ```

mod harness;
mod assertions;
mod snapshot;

pub use harness::TestHarness;
pub use assertions::{Assertion, AssertionError, AssertionResult};
pub use snapshot::{Snapshot, SnapshotFormat};
