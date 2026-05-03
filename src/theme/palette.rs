//! Named-color palette and severity helper for `Theme`.
//!
//! This module adds three new public types — [`NamedColor`], [`Palette`],
//! and [`Severity`] — plus three new methods on [`Theme`] (`color`,
//! `severity_color`, `severity_style`). Together they let consumers access
//! palette colors by name and bucket numeric values into a four-band
//! severity gradient without reaching for raw color constants.
//!
//! See the [theme module documentation](super) for an overview.

#[allow(unused_imports)]
// These imports are needed by subsequent tasks; removed when all types land.
use ratatui::style::{Color, Modifier, Style};

#[allow(unused_imports)]
// Needed by subsequent tasks when impl Theme methods are added.
use super::Theme;

// Subsequent tasks add NamedColor, Palette, Severity, and the impl Theme block
// for color / severity_color / severity_style.
