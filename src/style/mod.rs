//! Style types and re-exports.
//!
//! This module re-exports ratatui's style types so applications can write
//! `use envision::style::*` instead of `use ratatui::style::*`.
//!
//! For semantic styles (focused, selected, disabled, etc.), see the
//! [`Theme`](crate::theme::Theme) type which provides context-aware styling.
//!
//! # Example
//!
//! ```rust
//! use envision::style::{Color, Style, Modifier, Stylize};
//!
//! let style = Style::default()
//!     .fg(Color::White)
//!     .bg(Color::Black)
//!     .add_modifier(Modifier::BOLD);
//! ```

// Re-export ratatui style types
pub use ratatui::style::{Color, Modifier, Style, Stylize};
