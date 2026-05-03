//! Theming support for Envision components.
//!
//! The theme module provides customizable color schemes for all UI components.
//! Six themes are included by default: a `Default` theme matching ratatui's
//! standard colors, and five popular dark themes (Nord, Dracula, Solarized Dark,
//! Gruvbox Dark, Catppuccin Mocha).
//!
//! # Example
//!
//! ```rust
//! use envision::theme::Theme;
//!
//! // Use default theme (yellow focused, gray disabled)
//! let default_theme = Theme::default();
//!
//! // Use Nord theme (blue focused, muted disabled)
//! let nord_theme = Theme::nord();
//!
//! // Use Dracula theme (purple focused, dark disabled)
//! let dracula_theme = Theme::dracula();
//!
//! // Components use theme via RenderContext in their view() method:
//! // let mut ctx = RenderContext::new(frame, area, &nord_theme);
//! // Component::view(&state, &mut ctx);
//! ```
//!
//! # Creating Custom Themes
//!
//! You can create custom themes by constructing a `Theme` directly:
//!
//! ```rust
//! use envision::theme::Theme;
//! use ratatui::style::Color;
//!
//! let my_theme = Theme {
//!     focused: Color::Magenta,
//!     selected: Color::Cyan,
//!     ..Theme::default()
//! };
//! ```

pub mod catppuccin;
#[allow(deprecated)]
pub use catppuccin::*;

pub mod dracula;
#[allow(deprecated)]
pub use dracula::*;

pub mod gruvbox;
#[allow(deprecated)]
pub use gruvbox::*;

pub mod nord;
#[allow(deprecated)]
pub use nord::*;

pub mod palette;
pub use palette::{NamedColor, Palette, Severity};

pub mod solarized;
#[allow(deprecated)]
pub use solarized::*;

use ratatui::style::{Color, Modifier, Style};

// =============================================================================
// Theme Struct
// =============================================================================

/// A theme defines the color scheme for all Envision components.
///
/// Each color in the theme corresponds to a semantic UI state or element type.
/// Components use these colors through the theme's style helper methods.
///
/// # Fields
///
/// - **Base colors**: `background`, `foreground`, `border` - general UI colors
/// - **Interactive states**: `focused`, `selected`, `disabled`, `placeholder`
/// - **Semantic colors**: `primary`, `success`, `warning`, `error`, `info`
/// - **Progress bar**: `progress_filled`, `progress_empty`
///
/// # Example
///
/// ```rust
/// use envision::theme::Theme;
///
/// let theme = Theme::nord();
/// // Use with components via RenderContext.
/// # let _ = theme;
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theme {
    // Base colors
    /// Background color for UI elements.
    pub background: Color,
    /// Foreground (text) color.
    pub foreground: Color,
    /// Border color for boxes and frames.
    pub border: Color,

    // Interactive states
    /// Color for focused elements (borders, text).
    pub focused: Color,
    /// Color for selected items in lists/tables.
    pub selected: Color,
    /// Color for disabled elements.
    pub disabled: Color,
    /// Color for placeholder text.
    pub placeholder: Color,

    // Semantic colors
    /// Primary accent color.
    pub primary: Color,
    /// Success state color (green).
    pub success: Color,
    /// Warning state color (yellow/orange).
    pub warning: Color,
    /// Error state color (red).
    pub error: Color,
    /// Informational state color (blue/cyan).
    pub info: Color,

    // Progress bar specific
    /// Filled portion of progress bars.
    pub progress_filled: Color,
    /// Empty portion of progress bars.
    pub progress_empty: Color,

    // Named-color palette (26 entries; populated per-theme)
    /// Theme-specific palette of named colors. Use [`Theme::color`] for theme-aware
    /// lookup; this field is exposed primarily for users constructing custom themes.
    pub palette: Palette,
}

impl Default for Theme {
    /// Returns the default theme matching ratatui's standard colors.
    ///
    /// This theme uses:
    /// - Yellow for focused elements
    /// - DarkGray for disabled/placeholder elements
    /// - Cyan for primary/info
    /// - Standard Green/Yellow/Red for success/warning/error
    ///
    /// # Palette collapse note
    ///
    /// The `Default` theme uses ratatui's basic `Color` enum (Reset, Yellow, Red,
    /// Cyan, etc.) for maximum terminal compatibility. Many [`NamedColor`] variants
    /// collapse to the same basic `Color`:
    ///
    /// - `Peach` / `Yellow` → `Color::Yellow`
    /// - `Pink` / `Mauve` / `Lavender` → `Color::Magenta`
    /// - `Red` / `Maroon` / `Flamingo` / `Rosewater` → `Color::Red`
    /// - `Green` / `Teal` → `Color::Green`
    /// - `Sky` / `Sapphire` → `Color::Cyan`; `Blue` → `Color::Blue`
    /// - Text tones → `Color::White` / `Color::Gray`; overlay/surface → `Color::DarkGray` / `Color::Black`
    ///
    /// Notably this affects `Theme::severity_color`: on `Default`, `Mild`
    /// (`Yellow`) and `Bad` (`Peach`) both render as `Color::Yellow`. The
    /// four-band gradient effectively becomes three-band on the `Default` theme.
    /// `Theme::severity_style` still adds `BOLD` for `Critical`, so the
    /// strongest band stays distinguishable. Consumers wanting full palette
    /// fidelity should use [`Theme::catppuccin_mocha`] or another full-palette
    /// theme.
    fn default() -> Self {
        Self {
            background: Color::Reset,
            foreground: Color::Reset,
            border: Color::Reset,

            focused: Color::Yellow,
            selected: Color::Reset,
            disabled: Color::DarkGray,
            placeholder: Color::DarkGray,

            primary: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Cyan,

            progress_filled: Color::Cyan,
            progress_empty: Color::Black,

            palette: Palette {
                rosewater: Color::Red,
                flamingo: Color::Red,
                pink: Color::Magenta,
                mauve: Color::Magenta,
                red: Color::Red,
                maroon: Color::Red,
                peach: Color::Yellow,
                yellow: Color::Yellow,
                green: Color::Green,
                teal: Color::Green,
                sky: Color::Cyan,
                sapphire: Color::Cyan,
                blue: Color::Blue,
                lavender: Color::Magenta,
                text: Color::White,
                subtext1: Color::Gray,
                subtext0: Color::Gray,
                overlay2: Color::DarkGray,
                overlay1: Color::DarkGray,
                overlay0: Color::DarkGray,
                surface2: Color::DarkGray,
                surface1: Color::Black,
                surface0: Color::Black,
                base: Color::Reset,
                mantle: Color::Reset,
                crust: Color::Black,
            },
        }
    }
}

impl Theme {
    /// Creates a new Nord-themed color scheme.
    ///
    /// The Nord theme uses the popular Nord color palette with its
    /// characteristic frost blues and aurora accent colors.
    ///
    /// # Colors
    ///
    /// - Focused: Nord8 (light blue #88C0D0)
    /// - Selected: Nord9 (blue #81A1C1)
    /// - Disabled: Nord3 (muted gray #4C566A)
    /// - Success: Nord14 (green #A3BE8C)
    /// - Warning: Nord13 (yellow #EBCB8B)
    /// - Error: Nord11 (red #BF616A)
    ///
    /// # Palette mapping
    ///
    /// Nord's 16-color palette doesn't include every Catppuccin name. The palette is
    /// populated with nearest-equivalent mappings:
    ///
    /// - `Rosewater` / `Flamingo` → Nord4 (Snow Storm light)
    /// - `Pink` / `Mauve` / `Lavender` → Nord15 (Aurora purple — closest hue)
    /// - `Red` / `Maroon` → Nord11 (Aurora red)
    /// - `Peach` → Nord12 (Aurora orange)
    /// - `Yellow` → Nord13 (Aurora yellow)
    /// - `Green` → Nord14 (Aurora green)
    /// - `Teal` → Nord7 (Frost teal)
    /// - `Sky` / `Sapphire` / `Blue` → Nord8 / Nord9 / Nord10 (Frost blues, light → deep)
    /// - Text tones → Nord4–Nord6 (Snow Storm)
    /// - Overlay tones → Nord3 (Polar Night borders)
    /// - Surface / base tones → Nord0–Nord2 (Polar Night)
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::Theme;
    ///
    /// let theme = Theme::nord();
    /// // Use with components via RenderContext.
    /// # let _ = theme;
    /// ```
    #[allow(deprecated)]
    pub fn nord() -> Self {
        Self {
            background: NORD0,
            foreground: NORD6,
            border: NORD3,

            focused: NORD8,
            selected: NORD9,
            disabled: NORD3,
            placeholder: NORD3,

            primary: NORD10,
            success: NORD14,
            warning: NORD13,
            error: NORD11,
            info: NORD8,

            progress_filled: NORD8,
            progress_empty: NORD1,

            palette: Palette {
                rosewater: NORD4,
                flamingo: NORD4,
                pink: NORD15,
                mauve: NORD15,
                red: NORD11,
                maroon: NORD11,
                peach: NORD12,
                yellow: NORD13,
                green: NORD14,
                teal: NORD7,
                sky: NORD8,
                sapphire: NORD9,
                blue: NORD10,
                lavender: NORD15,
                text: NORD6,
                subtext1: NORD5,
                subtext0: NORD4,
                overlay2: NORD3,
                overlay1: NORD3,
                overlay0: NORD3,
                surface2: NORD2,
                surface1: NORD1,
                surface0: NORD1,
                base: NORD0,
                mantle: NORD0,
                crust: NORD0,
            },
        }
    }

    /// Creates a new Dracula-themed color scheme.
    ///
    /// The Dracula theme uses the popular Dracula color palette with its
    /// characteristic purples, pinks, and vibrant accent colors.
    ///
    /// # Colors
    ///
    /// - Focused: Purple (#BD93F9)
    /// - Selected: Pink (#FF79C6)
    /// - Disabled: Comment (#6272A4)
    /// - Success: Green (#50FA7B)
    /// - Warning: Yellow (#F1FA8C)
    /// - Error: Red (#FF5555)
    ///
    /// # Palette mapping
    ///
    /// Dracula's 9-color accent palette (Cyan/Green/Orange/Pink/Purple/Red/Yellow plus
    /// FG/BG/Comment/CurrentLine) maps as follows:
    ///
    /// - `Pink` / `Rosewater` / `Flamingo` → Dracula Pink (the only native pink)
    /// - `Mauve` / `Lavender` → Dracula Purple
    /// - `Red` / `Maroon` → Dracula Red
    /// - `Peach` → Dracula Orange
    /// - `Sky` / `Sapphire` / `Blue` / `Teal` → Dracula Cyan (the only native cool color)
    /// - Text/Overlay → FG / Comment
    /// - Surface / Base → CurrentLine / BG
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::Theme;
    ///
    /// let theme = Theme::dracula();
    /// // Use with components via RenderContext.
    /// # let _ = theme;
    /// ```
    #[allow(deprecated)]
    pub fn dracula() -> Self {
        Self {
            background: DRACULA_BG,
            foreground: DRACULA_FG,
            border: DRACULA_COMMENT,

            focused: DRACULA_PURPLE,
            selected: DRACULA_PINK,
            disabled: DRACULA_COMMENT,
            placeholder: DRACULA_COMMENT,

            primary: DRACULA_CYAN,
            success: DRACULA_GREEN,
            warning: DRACULA_YELLOW,
            error: DRACULA_RED,
            info: DRACULA_CYAN,

            progress_filled: DRACULA_PURPLE,
            progress_empty: DRACULA_CURRENT,

            palette: Palette {
                rosewater: DRACULA_PINK,
                flamingo: DRACULA_PINK,
                pink: DRACULA_PINK,
                mauve: DRACULA_PURPLE,
                red: DRACULA_RED,
                maroon: DRACULA_RED,
                peach: DRACULA_ORANGE,
                yellow: DRACULA_YELLOW,
                green: DRACULA_GREEN,
                teal: DRACULA_CYAN,
                sky: DRACULA_CYAN,
                sapphire: DRACULA_CYAN,
                blue: DRACULA_CYAN,
                lavender: DRACULA_PURPLE,
                text: DRACULA_FG,
                subtext1: DRACULA_FG,
                subtext0: DRACULA_COMMENT,
                overlay2: DRACULA_COMMENT,
                overlay1: DRACULA_COMMENT,
                overlay0: DRACULA_COMMENT,
                surface2: DRACULA_CURRENT,
                surface1: DRACULA_CURRENT,
                surface0: DRACULA_CURRENT,
                base: DRACULA_BG,
                mantle: DRACULA_BG,
                crust: DRACULA_BG,
            },
        }
    }

    /// Creates a new Solarized Dark-themed color scheme.
    ///
    /// The Solarized Dark theme uses Ethan Schoonover's carefully designed
    /// color palette optimized for readability and reduced eye strain.
    ///
    /// # Colors
    ///
    /// - Focused: Blue (#268BD2)
    /// - Selected: Cyan (#2AA198)
    /// - Disabled: Base01 (#586E75)
    /// - Success: Green (#859900)
    /// - Warning: Yellow (#B58900)
    /// - Error: Red (#DC322F)
    ///
    /// # Palette mapping
    ///
    /// Solarized's accent palette (yellow/orange/red/magenta/blue/cyan/green) maps
    /// to Catppuccin names as follows. Solarized has no native pink — magenta is
    /// the closest pinkish hue.
    ///
    /// - `Pink` / `Rosewater` / `Flamingo` / `Mauve` / `Lavender` → Magenta
    /// - `Red` / `Maroon` → Red
    /// - `Peach` → Orange
    /// - `Sky` / `Teal` → Cyan
    /// - `Sapphire` / `Blue` → Blue
    /// - Text/Overlay → Base1 / Base0 / Base01
    /// - Surface / Base → Base02 / Base03
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::Theme;
    ///
    /// let theme = Theme::solarized_dark();
    /// // Use with components via RenderContext.
    /// # let _ = theme;
    /// ```
    #[allow(deprecated)]
    pub fn solarized_dark() -> Self {
        Self {
            background: SOLARIZED_BASE03,
            foreground: SOLARIZED_BASE0,
            border: SOLARIZED_BASE01,

            focused: SOLARIZED_BLUE,
            selected: SOLARIZED_CYAN,
            disabled: SOLARIZED_BASE01,
            placeholder: SOLARIZED_BASE01,

            primary: SOLARIZED_BLUE,
            success: SOLARIZED_GREEN,
            warning: SOLARIZED_YELLOW,
            error: SOLARIZED_RED,
            info: SOLARIZED_CYAN,

            progress_filled: SOLARIZED_BLUE,
            progress_empty: SOLARIZED_BASE02,

            palette: Palette {
                rosewater: SOLARIZED_MAGENTA,
                flamingo: SOLARIZED_MAGENTA,
                pink: SOLARIZED_MAGENTA,
                mauve: SOLARIZED_MAGENTA,
                red: SOLARIZED_RED,
                maroon: SOLARIZED_RED,
                peach: SOLARIZED_ORANGE,
                yellow: SOLARIZED_YELLOW,
                green: SOLARIZED_GREEN,
                teal: SOLARIZED_CYAN,
                sky: SOLARIZED_CYAN,
                sapphire: SOLARIZED_BLUE,
                blue: SOLARIZED_BLUE,
                lavender: SOLARIZED_MAGENTA,
                text: SOLARIZED_BASE1,
                subtext1: SOLARIZED_BASE0,
                subtext0: SOLARIZED_BASE01,
                overlay2: SOLARIZED_BASE01,
                overlay1: SOLARIZED_BASE01,
                overlay0: SOLARIZED_BASE01,
                surface2: SOLARIZED_BASE02,
                surface1: SOLARIZED_BASE02,
                surface0: SOLARIZED_BASE02,
                base: SOLARIZED_BASE03,
                mantle: SOLARIZED_BASE03,
                crust: SOLARIZED_BASE03,
            },
        }
    }

    /// Creates a new Gruvbox Dark-themed color scheme.
    ///
    /// The Gruvbox Dark theme uses the retro-groove Gruvbox color palette
    /// with its warm, earthy tones and high contrast.
    ///
    /// # Colors
    ///
    /// - Focused: Yellow (#FABD2F)
    /// - Selected: Blue (#83A598)
    /// - Disabled: Gray (#928374)
    /// - Success: Green (#B8BB26)
    /// - Warning: Orange (#FE8019)
    /// - Error: Red (#FB4934)
    ///
    /// # Palette mapping
    ///
    /// Gruvbox's 7-color accent palette (red/green/yellow/blue/purple/aqua/orange)
    /// maps as follows:
    ///
    /// - `Pink` / `Rosewater` / `Flamingo` / `Mauve` / `Lavender` → Purple
    /// - `Red` / `Maroon` → Red
    /// - `Peach` → Orange
    /// - `Teal` / `Sky` → Aqua
    /// - `Sapphire` / `Blue` → Blue
    /// - Text/Overlay → FG / Gray
    /// - Surface/Base → BG1 / BG
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::Theme;
    ///
    /// let theme = Theme::gruvbox_dark();
    /// // Use with components via RenderContext.
    /// # let _ = theme;
    /// ```
    #[allow(deprecated)]
    pub fn gruvbox_dark() -> Self {
        Self {
            background: GRUVBOX_BG,
            foreground: GRUVBOX_FG,
            border: GRUVBOX_GRAY,

            focused: GRUVBOX_YELLOW,
            selected: GRUVBOX_BLUE,
            disabled: GRUVBOX_GRAY,
            placeholder: GRUVBOX_GRAY,

            primary: GRUVBOX_AQUA,
            success: GRUVBOX_GREEN,
            warning: GRUVBOX_ORANGE,
            error: GRUVBOX_RED,
            info: GRUVBOX_BLUE,

            progress_filled: GRUVBOX_YELLOW,
            progress_empty: GRUVBOX_BG1,

            palette: Palette {
                rosewater: GRUVBOX_PURPLE,
                flamingo: GRUVBOX_PURPLE,
                pink: GRUVBOX_PURPLE,
                mauve: GRUVBOX_PURPLE,
                red: GRUVBOX_RED,
                maroon: GRUVBOX_RED,
                peach: GRUVBOX_ORANGE,
                yellow: GRUVBOX_YELLOW,
                green: GRUVBOX_GREEN,
                teal: GRUVBOX_AQUA,
                sky: GRUVBOX_AQUA,
                sapphire: GRUVBOX_BLUE,
                blue: GRUVBOX_BLUE,
                lavender: GRUVBOX_PURPLE,
                text: GRUVBOX_FG,
                subtext1: GRUVBOX_FG,
                subtext0: GRUVBOX_GRAY,
                overlay2: GRUVBOX_GRAY,
                overlay1: GRUVBOX_GRAY,
                overlay0: GRUVBOX_GRAY,
                surface2: GRUVBOX_BG1,
                surface1: GRUVBOX_BG1,
                surface0: GRUVBOX_BG1,
                base: GRUVBOX_BG,
                mantle: GRUVBOX_BG,
                crust: GRUVBOX_BG,
            },
        }
    }

    /// Creates a new Catppuccin Mocha-themed color scheme.
    ///
    /// The Catppuccin Mocha theme is one of the most popular modern terminal
    /// palettes, featuring soothing pastel colors on a warm dark background.
    ///
    /// # Colors
    ///
    /// - Focused: Lavender (#B4BEFE)
    /// - Selected: Mauve (#CBA6F7)
    /// - Disabled: Surface2 (#585B70)
    /// - Success: Green (#A6E3A1)
    /// - Warning: Yellow (#F9E2AF)
    /// - Error: Red (#F38BA8)
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::Theme;
    ///
    /// let theme = Theme::catppuccin_mocha();
    /// // Use with components via RenderContext.
    /// # let _ = theme;
    /// ```
    #[allow(deprecated)]
    pub fn catppuccin_mocha() -> Self {
        Self {
            background: CATPPUCCIN_BASE,
            foreground: CATPPUCCIN_TEXT,
            border: CATPPUCCIN_SURFACE2,

            focused: CATPPUCCIN_LAVENDER,
            selected: CATPPUCCIN_MAUVE,
            disabled: CATPPUCCIN_SURFACE2,
            placeholder: CATPPUCCIN_OVERLAY0,

            primary: CATPPUCCIN_BLUE,
            success: CATPPUCCIN_GREEN,
            warning: CATPPUCCIN_YELLOW,
            error: CATPPUCCIN_RED,
            info: CATPPUCCIN_SAPPHIRE,

            progress_filled: CATPPUCCIN_LAVENDER,
            progress_empty: CATPPUCCIN_SURFACE0,

            palette: Palette {
                rosewater: CATPPUCCIN_ROSEWATER,
                flamingo: CATPPUCCIN_FLAMINGO,
                pink: CATPPUCCIN_PINK,
                mauve: CATPPUCCIN_MAUVE,
                red: CATPPUCCIN_RED,
                maroon: CATPPUCCIN_MAROON,
                peach: CATPPUCCIN_PEACH,
                yellow: CATPPUCCIN_YELLOW,
                green: CATPPUCCIN_GREEN,
                teal: CATPPUCCIN_TEAL,
                sky: CATPPUCCIN_SKY,
                sapphire: CATPPUCCIN_SAPPHIRE,
                blue: CATPPUCCIN_BLUE,
                lavender: CATPPUCCIN_LAVENDER,
                text: CATPPUCCIN_TEXT,
                subtext1: CATPPUCCIN_SUBTEXT1,
                subtext0: CATPPUCCIN_SUBTEXT0,
                overlay2: CATPPUCCIN_OVERLAY2,
                overlay1: CATPPUCCIN_OVERLAY1,
                overlay0: CATPPUCCIN_OVERLAY0,
                surface2: CATPPUCCIN_SURFACE2,
                surface1: CATPPUCCIN_SURFACE1,
                surface0: CATPPUCCIN_SURFACE0,
                base: CATPPUCCIN_BASE,
                mantle: CATPPUCCIN_MANTLE,
                crust: CATPPUCCIN_CRUST,
            },
        }
    }

    // =========================================================================
    // Style Helper Methods
    // =========================================================================

    /// Returns a style for focused elements.
    ///
    /// Uses the theme's focused color for foreground.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::Theme;
    ///
    /// let theme = Theme::nord();
    /// let style = theme.focused_style();
    /// ```
    pub fn focused_style(&self) -> Style {
        Style::default().fg(self.focused)
    }

    /// Returns a style for focused elements with bold modifier.
    pub fn focused_bold_style(&self) -> Style {
        Style::default()
            .fg(self.focused)
            .add_modifier(Modifier::BOLD)
    }

    /// Returns a style for focused borders.
    ///
    /// Unlike [`focused_style`](Theme::focused_style), this includes
    /// the background color so borders render correctly on themed backgrounds.
    pub fn focused_border_style(&self) -> Style {
        Style::default().fg(self.focused).bg(self.background)
    }

    /// Returns a style for selected items.
    ///
    /// Uses bold modifier. In focused context, also uses focused color.
    pub fn selected_style(&self, focused: bool) -> Style {
        if focused {
            Style::default()
                .fg(self.focused)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::BOLD)
        }
    }

    /// Returns a style for selected items with background highlight.
    pub fn selected_highlight_style(&self, focused: bool) -> Style {
        if focused {
            Style::default()
                .bg(self.selected)
                .fg(self.foreground)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(self.disabled).fg(self.foreground)
        }
    }

    /// Returns a style for text selection (highlighted text in input fields).
    pub fn selection_style(&self) -> Style {
        Style::default().bg(self.selected).fg(self.foreground)
    }

    /// Returns a style for disabled elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::Theme;
    ///
    /// let theme = Theme::default();
    /// let style = theme.disabled_style();
    /// ```
    pub fn disabled_style(&self) -> Style {
        Style::default().fg(self.disabled)
    }

    /// Returns a style for placeholder text.
    pub fn placeholder_style(&self) -> Style {
        Style::default().fg(self.placeholder)
    }

    /// Returns a style for default/normal elements.
    ///
    /// Uses the theme's foreground and background colors so components
    /// render correctly with non-default themes (e.g., Nord).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::Theme;
    /// use ratatui::widgets::Paragraph;
    /// use ratatui::prelude::Stylize;
    ///
    /// let theme = Theme::default();
    /// let paragraph = Paragraph::new("text").style(theme.normal_style());
    /// ```
    pub fn normal_style(&self) -> Style {
        Style::default().fg(self.foreground).bg(self.background)
    }

    /// Returns a style for primary accent elements.
    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.primary)
    }

    /// Returns a style for borders (non-focused).
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Returns a style for success messages/indicators.
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Returns a style for warning messages/indicators.
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Returns a style for error messages/indicators.
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Returns a style for informational messages/indicators.
    pub fn info_style(&self) -> Style {
        Style::default().fg(self.info)
    }

    /// Returns a style for the filled portion of progress bars.
    pub fn progress_filled_style(&self) -> Style {
        Style::default()
            .fg(self.progress_filled)
            .bg(self.progress_empty)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests;
