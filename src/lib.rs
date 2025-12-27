//! # Envision
//!
//! A ratatui framework for collaborative TUI development with headless testing support.
//!
//! Envision provides a custom `CaptureBackend` that implements ratatui's `Backend` trait,
//! enabling you to:
//!
//! - Capture rendered frames as inspectable text or structured data
//! - Track frame history and compute diffs between renders
//! - Annotate widgets with semantic information
//! - Simulate user input for testing
//! - Run applications in headless mode for CI/testing
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use envision::backend::CaptureBackend;
//! use ratatui::Terminal;
//! use ratatui::widgets::Paragraph;
//!
//! // Create a headless terminal
//! let backend = CaptureBackend::new(80, 24);
//! let mut terminal = Terminal::new(backend).unwrap();
//!
//! // Render something
//! terminal.draw(|frame| {
//!     frame.render_widget(Paragraph::new("Hello, Envision!"), frame.area());
//! }).unwrap();
//!
//! // Capture the output
//! let output = terminal.backend().to_string();
//! println!("{}", output);
//! ```
//!
//! ## Input Simulation
//!
//! ```rust
//! use envision::input::{EventQueue, KeyCode};
//!
//! let mut queue = EventQueue::new();
//! queue.type_str("hello");
//! queue.enter();
//!
//! // Events can be consumed by your app's event loop
//! while let Some(event) = queue.pop() {
//!     // handle event...
//! }
//! ```

pub mod backend;
pub mod input;

// Re-export commonly used types
pub use backend::{CaptureBackend, EnhancedCell, FrameSnapshot};
pub use input::{EventQueue, SimulatedEvent};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::backend::{CaptureBackend, EnhancedCell, FrameSnapshot, OutputFormat};
    pub use crate::input::{EventQueue, KeyCode, KeyModifiers, SimulatedEvent};
    pub use ratatui::prelude::*;
}
