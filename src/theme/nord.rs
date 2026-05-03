//! Nord color palette constants.
//!
//! The Nord palette is a north-bluish, clean and elegant color scheme.
//! It consists of four groups: Polar Night (dark backgrounds), Snow Storm
//! (light text), Frost (cool blue accents), and Aurora (warm accent colors).
//!
//! See <https://www.nordtheme.com/docs/colors-and-palettes> for the full palette.

use ratatui::style::Color;

// =============================================================================
// Polar Night
// =============================================================================

/// Nord Polar Night - darkest background
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Base) or NamedColor::Surface* for theme-aware lookup"
)]
pub const NORD0: Color = Color::Rgb(46, 52, 64);
/// Nord Polar Night - dark background
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Surface1) or NamedColor::Surface0 for theme-aware lookup"
)]
pub const NORD1: Color = Color::Rgb(59, 66, 82);
/// Nord Polar Night - medium dark
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Surface2) for theme-aware lookup"
)]
pub const NORD2: Color = Color::Rgb(67, 76, 94);
/// Nord Polar Night - lighter dark (borders)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Overlay0) (or Overlay1/Overlay2) for theme-aware lookup"
)]
pub const NORD3: Color = Color::Rgb(76, 86, 106);

// =============================================================================
// Snow Storm
// =============================================================================

/// Nord Snow Storm - light text (dim)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Subtext0) for theme-aware lookup"
)]
pub const NORD4: Color = Color::Rgb(216, 222, 233);
/// Nord Snow Storm - light text (medium)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Subtext1) for theme-aware lookup"
)]
pub const NORD5: Color = Color::Rgb(229, 233, 240);
/// Nord Snow Storm - light text (bright)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Text) for theme-aware lookup"
)]
pub const NORD6: Color = Color::Rgb(236, 239, 244);

// =============================================================================
// Frost
// =============================================================================

/// Nord Frost - teal
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Teal) for theme-aware lookup"
)]
pub const NORD7: Color = Color::Rgb(143, 188, 187);
/// Nord Frost - light blue (primary focus color)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Sky) for theme-aware lookup"
)]
pub const NORD8: Color = Color::Rgb(136, 192, 208);
/// Nord Frost - blue
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Sapphire) for theme-aware lookup"
)]
pub const NORD9: Color = Color::Rgb(129, 161, 193);
/// Nord Frost - dark blue
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Blue) for theme-aware lookup"
)]
pub const NORD10: Color = Color::Rgb(94, 129, 172);

// =============================================================================
// Aurora
// =============================================================================

/// Nord Aurora - red (error)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Red) for theme-aware lookup"
)]
pub const NORD11: Color = Color::Rgb(191, 97, 106);
/// Nord Aurora - orange
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Peach) for theme-aware lookup"
)]
pub const NORD12: Color = Color::Rgb(208, 135, 112);
/// Nord Aurora - yellow (warning)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Yellow) for theme-aware lookup"
)]
pub const NORD13: Color = Color::Rgb(235, 203, 139);
/// Nord Aurora - green (success)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Green) for theme-aware lookup"
)]
pub const NORD14: Color = Color::Rgb(163, 190, 140);
/// Nord Aurora - purple
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Lavender) or NamedColor::Mauve for theme-aware lookup"
)]
pub const NORD15: Color = Color::Rgb(180, 142, 173);
