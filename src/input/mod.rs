//! Input simulation module for programmatic event injection.
//!
//! This module provides types and utilities for simulating user input
//! in headless mode, enabling automated testing of TUI applications.
//!
//! # Example
//!
//! ```rust
//! use envision::input::{EventQueue, SimulatedEvent};
//! use crossterm::event::{KeyCode, KeyModifiers};
//!
//! let mut queue = EventQueue::new();
//!
//! // Simulate typing "hello"
//! queue.type_str("hello");
//!
//! // Simulate pressing Enter
//! queue.key(KeyCode::Enter);
//!
//! // Process events
//! while let Some(event) = queue.pop() {
//!     // Handle event...
//! }
//! ```

mod events;
mod queue;

pub use events::{SimulatedEvent, KeyEventBuilder, MouseEventBuilder};
pub use queue::EventQueue;

// Re-export crossterm event types for convenience
pub use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
    MouseButton, MouseEvent, MouseEventKind,
};
