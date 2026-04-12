//! Input module for terminal events.
//!
//! This module provides types and utilities for working with terminal input
//! events. The same types are used whether events come from a real terminal
//! or are injected programmatically (for virtual terminals or testing).
//!
//! # Example
//!
//! ```rust
//! use envision::input::{EventQueue, Event, Key};
//!
//! let mut queue = EventQueue::new();
//!
//! // Type "hello" then press Enter
//! queue.type_str("hello");
//! queue.key(Key::Enter);
//!
//! // Process events
//! while let Some(event) = queue.pop() {
//!     // Handle event...
//! }
//! ```

pub(crate) mod convert;
mod events;
pub mod key;
pub mod mouse;
mod queue;

pub use events::{Event, KeyEventBuilder, MouseEventBuilder};
pub use key::{Key, KeyEvent, KeyEventKind, Modifiers};
pub use mouse::{MouseButton, MouseEvent, MouseEventKind};
pub use queue::EventQueue;
