//! Backend module providing the `CaptureBackend` implementation.
//!
//! This module contains our custom ratatui backend that captures rendered frames
//! for inspection, testing, and headless operation.

mod capture;
mod cell;
pub mod output;

pub use capture::{CaptureBackend, FrameSnapshot};
pub use cell::EnhancedCell;
pub use output::OutputFormat;
