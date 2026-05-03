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
/// ```rust,ignore
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
/// ```rust,ignore
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
