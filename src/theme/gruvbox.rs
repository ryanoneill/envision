//! Gruvbox Dark color palette constants.
//!
//! The Gruvbox Dark color scheme features warm, earthy tones with high
//! contrast. It uses a retro-groove aesthetic with a brown/orange base and
//! vibrant accent colors.
//!
//! See <https://github.com/morhetz/gruvbox> for the full palette.

use ratatui::style::Color;

/// Gruvbox Dark - bg (dark background, #282828)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Base) for theme-aware lookup"
)]
pub const GRUVBOX_BG: Color = Color::Rgb(40, 40, 40);
/// Gruvbox Dark - bg1 (lighter background, #3C3836)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Surface2) for theme-aware lookup"
)]
pub const GRUVBOX_BG1: Color = Color::Rgb(60, 56, 54);
/// Gruvbox Dark - fg (light foreground, #EBDBB2)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Text) for theme-aware lookup"
)]
pub const GRUVBOX_FG: Color = Color::Rgb(235, 219, 178);
/// Gruvbox Dark - gray (#928374)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Subtext0) or NamedColor::Overlay* for theme-aware lookup"
)]
pub const GRUVBOX_GRAY: Color = Color::Rgb(146, 131, 116);
/// Gruvbox Dark - red (#FB4934)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Red) for theme-aware lookup"
)]
pub const GRUVBOX_RED: Color = Color::Rgb(251, 73, 52);
/// Gruvbox Dark - green (#B8BB26)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Green) for theme-aware lookup"
)]
pub const GRUVBOX_GREEN: Color = Color::Rgb(184, 187, 38);
/// Gruvbox Dark - yellow (#FABD2F)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Yellow) for theme-aware lookup"
)]
pub const GRUVBOX_YELLOW: Color = Color::Rgb(250, 189, 47);
/// Gruvbox Dark - blue (#83A598)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Blue) or NamedColor::Sapphire for theme-aware lookup"
)]
pub const GRUVBOX_BLUE: Color = Color::Rgb(131, 165, 152);
/// Gruvbox Dark - purple (#D3869B)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Mauve) or NamedColor::Lavender for theme-aware lookup"
)]
pub const GRUVBOX_PURPLE: Color = Color::Rgb(211, 134, 155);
/// Gruvbox Dark - aqua (#8EC07C)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Teal) or NamedColor::Sky for theme-aware lookup"
)]
pub const GRUVBOX_AQUA: Color = Color::Rgb(142, 192, 124);
/// Gruvbox Dark - orange (#FE8019)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Peach) for theme-aware lookup"
)]
pub const GRUVBOX_ORANGE: Color = Color::Rgb(254, 128, 25);
