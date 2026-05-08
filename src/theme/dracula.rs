//! Dracula color palette constants.
//!
//! The Dracula color scheme is a dark theme with vivid accent colors built
//! around purples, pinks, and a high-contrast foreground. These are the
//! official Dracula palette values.
//!
//! See <https://draculatheme.com/contribute#color-palette> for the full palette.

use ratatui::style::Color;

/// Dracula - background (#282A36)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Base) for theme-aware lookup"
)]
pub const DRACULA_BG: Color = Color::Rgb(40, 42, 54);
/// Dracula - current line (#44475A)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Surface2) for theme-aware lookup"
)]
pub const DRACULA_CURRENT: Color = Color::Rgb(68, 71, 90);
/// Dracula - foreground (#F8F8F2)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Text) for theme-aware lookup"
)]
pub const DRACULA_FG: Color = Color::Rgb(248, 248, 242);
/// Dracula - comment (#6272A4)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Subtext0) or NamedColor::Overlay* for theme-aware lookup"
)]
pub const DRACULA_COMMENT: Color = Color::Rgb(98, 114, 164);
/// Dracula - cyan (#8BE9FD)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Sky) or NamedColor::Sapphire for theme-aware lookup"
)]
pub const DRACULA_CYAN: Color = Color::Rgb(139, 233, 253);
/// Dracula - green (#50FA7B)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Green) for theme-aware lookup"
)]
pub const DRACULA_GREEN: Color = Color::Rgb(80, 250, 123);
/// Dracula - orange (#FFB86C)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Peach) for theme-aware lookup"
)]
pub const DRACULA_ORANGE: Color = Color::Rgb(255, 184, 108);
/// Dracula - pink (#FF79C6)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Pink) for theme-aware lookup"
)]
pub const DRACULA_PINK: Color = Color::Rgb(255, 121, 198);
/// Dracula - purple (#BD93F9)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Mauve) or NamedColor::Lavender for theme-aware lookup"
)]
pub const DRACULA_PURPLE: Color = Color::Rgb(189, 147, 249);
/// Dracula - red (#FF5555)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Red) for theme-aware lookup"
)]
pub const DRACULA_RED: Color = Color::Rgb(255, 85, 85);
/// Dracula - yellow (#F1FA8C)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Yellow) for theme-aware lookup"
)]
pub const DRACULA_YELLOW: Color = Color::Rgb(241, 250, 140);
