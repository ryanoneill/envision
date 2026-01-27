//! Input module for terminal events.
//!
//! This module provides types and utilities for working with terminal input
//! events. The same types are used whether events come from a real terminal
//! or are injected programmatically (for virtual terminals or testing).
//!
//! # Example
//!
//! ```rust
//! use envision::input::{EventQueue, Event};
//! use crossterm::event::{KeyCode, KeyModifiers};
//!
//! let mut queue = EventQueue::new();
//!
//! // Type "hello" then press Enter
//! queue.type_str("hello");
//! queue.key(KeyCode::Enter);
//!
//! // Process events
//! while let Some(event) = queue.pop() {
//!     // Handle event...
//! }
//! ```

mod events;
mod queue;

pub use events::{Event, KeyEventBuilder, MouseEventBuilder};
pub use queue::EventQueue;

// Re-export crossterm event types for convenience
pub use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
