//! Named-color palette and severity helper for `Theme`.
//!
//! This module adds three new public types — [`NamedColor`], [`Palette`],
//! and [`Severity`] — plus three new methods on [`Theme`] (`color`,
//! `severity_color`, `severity_style`). Together they let consumers access
//! palette colors by name and bucket numeric values into a four-band
//! severity gradient without reaching for raw color constants.
//!
//! See the [theme module documentation](super) for an overview.

use ratatui::style::{Color, Modifier, Style};

use super::Theme;

// =============================================================================
// Severity
// =============================================================================

/// A four-band severity gradient for value-based coloring (good → mild → bad → critical).
///
/// `Severity` provides a unified vocabulary for "color this number by how bad it is" —
/// the most common visual primitive in monitoring, profiling, and status dashboards.
/// Pair with [`Severity::from_thresholds`] to bucket a numeric value, then pass to
/// [`Theme::severity_color`] or [`Theme::severity_style`] for theme-aware coloring.
///
/// `#[non_exhaustive]` so envision can add severity bands later without breaking
/// downstream `match` arms.
///
/// # Example
///
/// ```rust
/// use envision::theme::{Severity, Theme};
///
/// let theme = Theme::catppuccin_mocha();
/// let ratio = 5.2;
/// let sev = Severity::from_thresholds(ratio, &[
///     (1.0,  Severity::Good),
///     (3.0,  Severity::Mild),
///     (10.0, Severity::Bad),
/// ]);
/// let style = theme.severity_style(sev);
/// // Use `style` in a Cell or StyledInline for value-coloring.
/// # let _ = style;
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Healthy band — typically rendered green.
    Good,
    /// Slightly elevated band — typically rendered yellow.
    Mild,
    /// Concerning band — typically rendered peach/orange.
    Bad,
    /// Critical band — typically rendered red and bold.
    Critical,
}

impl Severity {
    /// Pick a `Severity` by linear thresholds.
    ///
    /// Thresholds are evaluated in slice order: the first `(cutoff, severity)` entry where
    /// `value < cutoff` wins. Values at or above all cutoffs return `Severity::Critical`.
    ///
    /// # Sorting
    ///
    /// Pass thresholds sorted ascending by cutoff for predictable bucketing. Unsorted
    /// input is well-defined (first-match-wins) but typically counter-intuitive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use envision::theme::Severity;
    ///
    /// let thresholds = [
    ///     (1.0,  Severity::Good),
    ///     (3.0,  Severity::Mild),
    ///     (10.0, Severity::Bad),
    /// ];
    ///
    /// assert_eq!(Severity::from_thresholds(0.5,  &thresholds), Severity::Good);
    /// assert_eq!(Severity::from_thresholds(2.0,  &thresholds), Severity::Mild);
    /// assert_eq!(Severity::from_thresholds(5.0,  &thresholds), Severity::Bad);
    /// assert_eq!(Severity::from_thresholds(20.0, &thresholds), Severity::Critical);
    /// ```
    pub fn from_thresholds(value: f64, thresholds: &[(f64, Severity)]) -> Severity {
        for (cutoff, sev) in thresholds {
            if value < *cutoff {
                return *sev;
            }
        }
        Severity::Critical
    }
}

// =============================================================================
// Named Palette Colors
// =============================================================================

/// A flat enum of 26 palette color names derived from Catppuccin Mocha — the most
/// complete shipped palette.
///
/// Use [`Theme::color`] to look up a named color in the active theme. Every theme
/// returns a sensible color for every variant via its [`Palette`] mapping; for
/// non-Catppuccin themes that lack a native equivalent, the mapping uses the
/// nearest-shade match (documented per theme).
///
/// `#[non_exhaustive]` so envision can add palette names later without breaking
/// downstream `match` arms.
///
/// # Example
///
/// ```rust
/// use envision::theme::{NamedColor, Theme};
///
/// let theme = Theme::catppuccin_mocha();
/// let lavender = theme.color(NamedColor::Lavender);
/// // Use `lavender` in a Style, Cell, or StyledInline span.
/// # let _ = lavender;
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NamedColor {
    // Accent colors (warm)
    /// Rosewater — pastel pink with rose undertone.
    Rosewater,
    /// Flamingo — pastel pink with peach undertone.
    Flamingo,
    /// Pink — saturated pink.
    Pink,
    /// Mauve — pastel purple.
    Mauve,
    /// Red — saturated red.
    Red,
    /// Maroon — darker red with brown undertone.
    Maroon,
    /// Peach — pastel orange.
    Peach,
    /// Yellow — pastel yellow.
    Yellow,
    /// Green — pastel green.
    Green,
    /// Teal — pastel teal (green-cyan).
    Teal,

    // Accent colors (cool)
    /// Sky — light cyan-blue.
    Sky,
    /// Sapphire — saturated cyan-blue.
    Sapphire,
    /// Blue — pastel blue.
    Blue,
    /// Lavender — pastel purple-blue.
    Lavender,

    // Text + overlay (light → dark)
    /// Text — primary foreground (lightest text tone).
    Text,
    /// Subtext1 — slightly muted foreground.
    Subtext1,
    /// Subtext0 — more muted foreground.
    Subtext0,
    /// Overlay2 — lightest overlay tone.
    Overlay2,
    /// Overlay1 — medium overlay tone.
    Overlay1,
    /// Overlay0 — darkest overlay tone.
    Overlay0,
    /// Surface2 — lightest surface tone (panels, popovers).
    Surface2,
    /// Surface1 — medium surface tone.
    Surface1,
    /// Surface0 — darkest surface tone (background-ish).
    Surface0,
    /// Base — primary background.
    Base,
    /// Mantle — secondary background (slightly darker than Base).
    Mantle,
    /// Crust — darkest background tone.
    Crust,
}

// =============================================================================
// Palette
// =============================================================================

/// A complete 26-color palette mapping every [`NamedColor`] variant to a `Color`.
///
/// Each shipped theme stores a `Palette` populated at construction time. Custom user
/// themes can construct a `Palette` directly; no envision modification required.
///
/// # Example
///
/// Construct a custom palette for a hand-crafted theme:
///
/// ```rust
/// use envision::theme::Palette;
/// use ratatui::style::Color;
///
/// let palette = Palette {
///     rosewater: Color::Rgb(0xF5, 0xE0, 0xDC),
///     flamingo:  Color::Rgb(0xF2, 0xCD, 0xCD),
///     pink:      Color::Rgb(0xF5, 0xC2, 0xE7),
///     mauve:     Color::Rgb(0xCB, 0xA6, 0xF7),
///     red:       Color::Rgb(0xF3, 0x8B, 0xA8),
///     maroon:    Color::Rgb(0xEB, 0xA0, 0xAC),
///     peach:     Color::Rgb(0xFA, 0xB3, 0x87),
///     yellow:    Color::Rgb(0xF9, 0xE2, 0xAF),
///     green:     Color::Rgb(0xA6, 0xE3, 0xA1),
///     teal:      Color::Rgb(0x94, 0xE2, 0xD5),
///     sky:       Color::Rgb(0x89, 0xDC, 0xEB),
///     sapphire:  Color::Rgb(0x74, 0xC7, 0xEC),
///     blue:      Color::Rgb(0x89, 0xB4, 0xFA),
///     lavender:  Color::Rgb(0xB4, 0xBE, 0xFE),
///     text:      Color::Rgb(0xCD, 0xD6, 0xF4),
///     subtext1:  Color::Rgb(0xBA, 0xC2, 0xDE),
///     subtext0:  Color::Rgb(0xA6, 0xAD, 0xC8),
///     overlay2:  Color::Rgb(0x93, 0x99, 0xB2),
///     overlay1:  Color::Rgb(0x7F, 0x84, 0x9C),
///     overlay0:  Color::Rgb(0x6C, 0x70, 0x86),
///     surface2:  Color::Rgb(0x58, 0x5B, 0x70),
///     surface1:  Color::Rgb(0x45, 0x47, 0x5A),
///     surface0:  Color::Rgb(0x31, 0x32, 0x44),
///     base:      Color::Rgb(0x1E, 0x1E, 0x2E),
///     mantle:    Color::Rgb(0x18, 0x18, 0x25),
///     crust:     Color::Rgb(0x11, 0x11, 0x1B),
/// };
/// # let _ = palette;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// Color for [`NamedColor::Rosewater`].
    pub rosewater: Color,
    /// Color for [`NamedColor::Flamingo`].
    pub flamingo: Color,
    /// Color for [`NamedColor::Pink`].
    pub pink: Color,
    /// Color for [`NamedColor::Mauve`].
    pub mauve: Color,
    /// Color for [`NamedColor::Red`].
    pub red: Color,
    /// Color for [`NamedColor::Maroon`].
    pub maroon: Color,
    /// Color for [`NamedColor::Peach`].
    pub peach: Color,
    /// Color for [`NamedColor::Yellow`].
    pub yellow: Color,
    /// Color for [`NamedColor::Green`].
    pub green: Color,
    /// Color for [`NamedColor::Teal`].
    pub teal: Color,
    /// Color for [`NamedColor::Sky`].
    pub sky: Color,
    /// Color for [`NamedColor::Sapphire`].
    pub sapphire: Color,
    /// Color for [`NamedColor::Blue`].
    pub blue: Color,
    /// Color for [`NamedColor::Lavender`].
    pub lavender: Color,
    /// Color for [`NamedColor::Text`].
    pub text: Color,
    /// Color for [`NamedColor::Subtext1`].
    pub subtext1: Color,
    /// Color for [`NamedColor::Subtext0`].
    pub subtext0: Color,
    /// Color for [`NamedColor::Overlay2`].
    pub overlay2: Color,
    /// Color for [`NamedColor::Overlay1`].
    pub overlay1: Color,
    /// Color for [`NamedColor::Overlay0`].
    pub overlay0: Color,
    /// Color for [`NamedColor::Surface2`].
    pub surface2: Color,
    /// Color for [`NamedColor::Surface1`].
    pub surface1: Color,
    /// Color for [`NamedColor::Surface0`].
    pub surface0: Color,
    /// Color for [`NamedColor::Base`].
    pub base: Color,
    /// Color for [`NamedColor::Mantle`].
    pub mantle: Color,
    /// Color for [`NamedColor::Crust`].
    pub crust: Color,
}

// =============================================================================
// Theme accessors (color, severity_color, severity_style)
// =============================================================================

impl Theme {
    /// Returns the theme's color for a [`NamedColor`] palette name.
    ///
    /// This is the recommended way to access named palette colors that aren't
    /// covered by a semantic slot (`focused`, `success`, etc.). Always returns
    /// a sensible color for every variant; non-Catppuccin themes use
    /// nearest-equivalent mappings documented per theme.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::{NamedColor, Theme};
    ///
    /// let theme = Theme::catppuccin_mocha();
    /// let lavender = theme.color(NamedColor::Lavender);
    /// // Use in a Cell, Span, or Style.
    /// # let _ = lavender;
    /// ```
    pub fn color(&self, named: NamedColor) -> Color {
        match named {
            NamedColor::Rosewater => self.palette.rosewater,
            NamedColor::Flamingo => self.palette.flamingo,
            NamedColor::Pink => self.palette.pink,
            NamedColor::Mauve => self.palette.mauve,
            NamedColor::Red => self.palette.red,
            NamedColor::Maroon => self.palette.maroon,
            NamedColor::Peach => self.palette.peach,
            NamedColor::Yellow => self.palette.yellow,
            NamedColor::Green => self.palette.green,
            NamedColor::Teal => self.palette.teal,
            NamedColor::Sky => self.palette.sky,
            NamedColor::Sapphire => self.palette.sapphire,
            NamedColor::Blue => self.palette.blue,
            NamedColor::Lavender => self.palette.lavender,
            NamedColor::Text => self.palette.text,
            NamedColor::Subtext1 => self.palette.subtext1,
            NamedColor::Subtext0 => self.palette.subtext0,
            NamedColor::Overlay2 => self.palette.overlay2,
            NamedColor::Overlay1 => self.palette.overlay1,
            NamedColor::Overlay0 => self.palette.overlay0,
            NamedColor::Surface2 => self.palette.surface2,
            NamedColor::Surface1 => self.palette.surface1,
            NamedColor::Surface0 => self.palette.surface0,
            NamedColor::Base => self.palette.base,
            NamedColor::Mantle => self.palette.mantle,
            NamedColor::Crust => self.palette.crust,
        }
    }

    /// Returns the theme's color for a [`Severity`] band.
    ///
    /// Maps `Good` → green, `Mild` → yellow, `Bad` → peach, `Critical` → red,
    /// routed through the theme's palette. Non-Catppuccin themes use their
    /// nearest-equivalent palette mappings.
    ///
    /// On the [`Default`](Theme::default) theme, `Mild` and `Bad` both collapse
    /// to `Color::Yellow` (basic-Color palette has no peach). Use
    /// [`severity_style`](Theme::severity_style) for distinguishability via the
    /// `BOLD` modifier on `Critical`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::{Severity, Theme};
    ///
    /// let theme = Theme::catppuccin_mocha();
    /// let color = theme.severity_color(Severity::Bad);
    /// // Use in a Cell, Span, or Style.
    /// # let _ = color;
    /// ```
    pub fn severity_color(&self, sev: Severity) -> Color {
        match sev {
            Severity::Good => self.color(NamedColor::Green),
            Severity::Mild => self.color(NamedColor::Yellow),
            Severity::Bad => self.color(NamedColor::Peach),
            Severity::Critical => self.color(NamedColor::Red),
        }
    }

    /// Returns a `Style` for a [`Severity`] band — color plus reasonable defaults.
    ///
    /// Equivalent to `Style::default().fg(theme.severity_color(sev))` plus a
    /// `BOLD` modifier when `sev == Severity::Critical`. The `BOLD` on Critical
    /// is intentional: critical events should stand out beyond color alone (for
    /// color-blind users, low-contrast terminals, partial color rendering).
    ///
    /// Drop-in for `Cell::with_style(CellStyle::Custom(...))` and
    /// `StyledInline::Styled { style, ... }` sites.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::{Severity, Theme};
    ///
    /// let theme = Theme::catppuccin_mocha();
    /// let style = theme.severity_style(Severity::Critical);
    /// // style.fg is Catppuccin red and style includes BOLD.
    /// # let _ = style;
    /// ```
    pub fn severity_style(&self, sev: Severity) -> Style {
        let style = Style::default().fg(self.severity_color(sev));
        if sev == Severity::Critical {
            style.add_modifier(Modifier::BOLD)
        } else {
            style
        }
    }
}
