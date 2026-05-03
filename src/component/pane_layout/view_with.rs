//! Closure-based per-pane child rendering.
//!
//! The `view_with` method on [`PaneLayout`] draws pane chrome (borders,
//! titles, focus rings) and invokes the consumer's render closure once
//! per pane with a chrome-inset child [`RenderContext`] whose
//! `chrome_owned` flag is set to `true` so embedded components suppress
//! their own chrome.
//!
//! See [`super::PaneLayout::view_with`] for the public API.

// Imports are added in Task 5 when `view_with` is implemented. This file
// is created in Task 4 as a placeholder so adding the implementation in
// Task 5 doesn't push pane_layout/mod.rs over the 1000-line project
// limit.
