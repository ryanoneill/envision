//! Overlay/modal abstraction for TUI applications.
//!
//! The overlay system provides a runtime-owned modal stack that automatically
//! intercepts events (before `handle_event_with_state`) and renders overlays
//! (after `view`).
//!
//! # Overview
//!
//! - [`Overlay`]: Trait for overlay implementations (dialogs, search bars, etc.)
//! - [`OverlayAction`]: Result of overlay event handling (consume, dismiss, propagate)
//! - [`OverlayStack`]: Stack of active overlays managed by the runtime

mod action;
mod stack;
mod traits;

pub use action::OverlayAction;
pub use stack::OverlayStack;
pub use traits::Overlay;
