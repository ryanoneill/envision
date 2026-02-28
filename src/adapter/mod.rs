//! Dual adapter for simultaneous real and capture backends.
//!
//! The adapter module provides backends that can write to multiple
//! destinations, enabling debugging and inspection during normal operation.
//!
//! # Example
//!
//! ```rust,no_run
//! use envision::adapter::DualBackend;
//! use envision::backend::CaptureBackend;
//! use ratatui::backend::CrosstermBackend;
//! use ratatui::Terminal;
//! use std::io::stdout;
//!
//! // Create a dual backend that writes to both terminal and capture
//! let capture = CaptureBackend::new(80, 24);
//! let crossterm = CrosstermBackend::new(stdout());
//! let dual = DualBackend::new(crossterm, capture);
//!
//! let mut terminal = Terminal::new(dual).unwrap();
//!
//! terminal.draw(|frame| {
//!     // Rendering goes to both backends
//! }).unwrap();
//!
//! // Access the capture backend for inspection
//! let text = terminal.backend().capture().to_string();
//! ```

mod dual;

pub use dual::{DualBackend, DualBackendBuilder};
