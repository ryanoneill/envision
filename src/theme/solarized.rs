//! Solarized Dark color palette constants.
//!
//! The Solarized Dark color scheme by Ethan Schoonover uses a carefully
//! selected set of base tones and accent colors optimized for readability
//! and reduced eye strain.
//!
//! See <https://ethanschoonover.com/solarized/> for the full palette.

use ratatui::style::Color;

// =============================================================================
// Base Tones
// =============================================================================

/// Solarized Dark - base03 (darkest background, #002B36)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Base) for theme-aware lookup"
)]
pub const SOLARIZED_BASE03: Color = Color::Rgb(0, 43, 54);
/// Solarized Dark - base02 (background highlights, #073642)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Surface2) for theme-aware lookup"
)]
pub const SOLARIZED_BASE02: Color = Color::Rgb(7, 54, 66);
/// Solarized Dark - base01 (comments, #586E75)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Subtext0) or NamedColor::Overlay* for theme-aware lookup"
)]
pub const SOLARIZED_BASE01: Color = Color::Rgb(88, 110, 117);
/// Solarized Dark - base0 (primary text, #839496)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Subtext1) for theme-aware lookup"
)]
pub const SOLARIZED_BASE0: Color = Color::Rgb(131, 148, 150);
/// Solarized Dark - base1 (emphasized text, #93A1A1)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Text) for theme-aware lookup"
)]
pub const SOLARIZED_BASE1: Color = Color::Rgb(147, 161, 161);

// =============================================================================
// Accent Colors
// =============================================================================

/// Solarized Dark - blue (#268BD2)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Blue) or NamedColor::Sapphire for theme-aware lookup"
)]
pub const SOLARIZED_BLUE: Color = Color::Rgb(38, 139, 210);
/// Solarized Dark - cyan (#2AA198)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Sky) or NamedColor::Teal for theme-aware lookup"
)]
pub const SOLARIZED_CYAN: Color = Color::Rgb(42, 161, 152);
/// Solarized Dark - green (#859900)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Green) for theme-aware lookup"
)]
pub const SOLARIZED_GREEN: Color = Color::Rgb(133, 153, 0);
/// Solarized Dark - yellow (#B58900)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Yellow) for theme-aware lookup"
)]
pub const SOLARIZED_YELLOW: Color = Color::Rgb(181, 137, 0);
/// Solarized Dark - orange (#CB4B16)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Peach) for theme-aware lookup"
)]
pub const SOLARIZED_ORANGE: Color = Color::Rgb(203, 75, 22);
/// Solarized Dark - red (#DC322F)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Red) for theme-aware lookup"
)]
pub const SOLARIZED_RED: Color = Color::Rgb(220, 50, 47);
/// Solarized Dark - magenta (#D33682)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Pink) or NamedColor::Mauve for theme-aware lookup"
)]
pub const SOLARIZED_MAGENTA: Color = Color::Rgb(211, 54, 130);
