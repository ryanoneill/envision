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
pub use catppuccin::*;

use ratatui::style::{Color, Modifier, Style};

// =============================================================================
// Nord Color Palette Constants
// =============================================================================

/// Nord Polar Night - darkest background
pub const NORD0: Color = Color::Rgb(46, 52, 64);
/// Nord Polar Night - dark background
pub const NORD1: Color = Color::Rgb(59, 66, 82);
/// Nord Polar Night - medium dark
pub const NORD2: Color = Color::Rgb(67, 76, 94);
/// Nord Polar Night - lighter dark (borders)
pub const NORD3: Color = Color::Rgb(76, 86, 106);

/// Nord Snow Storm - light text (dim)
pub const NORD4: Color = Color::Rgb(216, 222, 233);
/// Nord Snow Storm - light text (medium)
pub const NORD5: Color = Color::Rgb(229, 233, 240);
/// Nord Snow Storm - light text (bright)
pub const NORD6: Color = Color::Rgb(236, 239, 244);

/// Nord Frost - teal
pub const NORD7: Color = Color::Rgb(143, 188, 187);
/// Nord Frost - light blue (primary focus color)
pub const NORD8: Color = Color::Rgb(136, 192, 208);
/// Nord Frost - blue
pub const NORD9: Color = Color::Rgb(129, 161, 193);
/// Nord Frost - dark blue
pub const NORD10: Color = Color::Rgb(94, 129, 172);

/// Nord Aurora - red (error)
pub const NORD11: Color = Color::Rgb(191, 97, 106);
/// Nord Aurora - orange
pub const NORD12: Color = Color::Rgb(208, 135, 112);
/// Nord Aurora - yellow (warning)
pub const NORD13: Color = Color::Rgb(235, 203, 139);
/// Nord Aurora - green (success)
pub const NORD14: Color = Color::Rgb(163, 190, 140);
/// Nord Aurora - purple
pub const NORD15: Color = Color::Rgb(180, 142, 173);

// =============================================================================
// Dracula Color Palette Constants
// =============================================================================

/// Dracula - background (#282A36)
pub const DRACULA_BG: Color = Color::Rgb(40, 42, 54);
/// Dracula - current line (#44475A)
pub const DRACULA_CURRENT: Color = Color::Rgb(68, 71, 90);
/// Dracula - foreground (#F8F8F2)
pub const DRACULA_FG: Color = Color::Rgb(248, 248, 242);
/// Dracula - comment (#6272A4)
pub const DRACULA_COMMENT: Color = Color::Rgb(98, 114, 164);
/// Dracula - cyan (#8BE9FD)
pub const DRACULA_CYAN: Color = Color::Rgb(139, 233, 253);
/// Dracula - green (#50FA7B)
pub const DRACULA_GREEN: Color = Color::Rgb(80, 250, 123);
/// Dracula - orange (#FFB86C)
pub const DRACULA_ORANGE: Color = Color::Rgb(255, 184, 108);
/// Dracula - pink (#FF79C6)
pub const DRACULA_PINK: Color = Color::Rgb(255, 121, 198);
/// Dracula - purple (#BD93F9)
pub const DRACULA_PURPLE: Color = Color::Rgb(189, 147, 249);
/// Dracula - red (#FF5555)
pub const DRACULA_RED: Color = Color::Rgb(255, 85, 85);
/// Dracula - yellow (#F1FA8C)
pub const DRACULA_YELLOW: Color = Color::Rgb(241, 250, 140);

// =============================================================================
// Solarized Dark Color Palette Constants
// =============================================================================

/// Solarized Dark - base03 (darkest background, #002B36)
pub const SOLARIZED_BASE03: Color = Color::Rgb(0, 43, 54);
/// Solarized Dark - base02 (background highlights, #073642)
pub const SOLARIZED_BASE02: Color = Color::Rgb(7, 54, 66);
/// Solarized Dark - base01 (comments, #586E75)
pub const SOLARIZED_BASE01: Color = Color::Rgb(88, 110, 117);
/// Solarized Dark - base0 (primary text, #839496)
pub const SOLARIZED_BASE0: Color = Color::Rgb(131, 148, 150);
/// Solarized Dark - base1 (emphasized text, #93A1A1)
pub const SOLARIZED_BASE1: Color = Color::Rgb(147, 161, 161);
/// Solarized Dark - blue (#268BD2)
pub const SOLARIZED_BLUE: Color = Color::Rgb(38, 139, 210);
/// Solarized Dark - cyan (#2AA198)
pub const SOLARIZED_CYAN: Color = Color::Rgb(42, 161, 152);
/// Solarized Dark - green (#859900)
pub const SOLARIZED_GREEN: Color = Color::Rgb(133, 153, 0);
/// Solarized Dark - yellow (#B58900)
pub const SOLARIZED_YELLOW: Color = Color::Rgb(181, 137, 0);
/// Solarized Dark - orange (#CB4B16)
pub const SOLARIZED_ORANGE: Color = Color::Rgb(203, 75, 22);
/// Solarized Dark - red (#DC322F)
pub const SOLARIZED_RED: Color = Color::Rgb(220, 50, 47);
/// Solarized Dark - magenta (#D33682)
pub const SOLARIZED_MAGENTA: Color = Color::Rgb(211, 54, 130);

// =============================================================================
// Gruvbox Dark Color Palette Constants
// =============================================================================

/// Gruvbox Dark - bg (dark background, #282828)
pub const GRUVBOX_BG: Color = Color::Rgb(40, 40, 40);
/// Gruvbox Dark - bg1 (lighter background, #3C3836)
pub const GRUVBOX_BG1: Color = Color::Rgb(60, 56, 54);
/// Gruvbox Dark - fg (light foreground, #EBDBB2)
pub const GRUVBOX_FG: Color = Color::Rgb(235, 219, 178);
/// Gruvbox Dark - gray (#928374)
pub const GRUVBOX_GRAY: Color = Color::Rgb(146, 131, 116);
/// Gruvbox Dark - red (#FB4934)
pub const GRUVBOX_RED: Color = Color::Rgb(251, 73, 52);
/// Gruvbox Dark - green (#B8BB26)
pub const GRUVBOX_GREEN: Color = Color::Rgb(184, 187, 38);
/// Gruvbox Dark - yellow (#FABD2F)
pub const GRUVBOX_YELLOW: Color = Color::Rgb(250, 189, 47);
/// Gruvbox Dark - blue (#83A598)
pub const GRUVBOX_BLUE: Color = Color::Rgb(131, 165, 152);
/// Gruvbox Dark - purple (#D3869B)
pub const GRUVBOX_PURPLE: Color = Color::Rgb(211, 134, 155);
/// Gruvbox Dark - aqua (#8EC07C)
pub const GRUVBOX_AQUA: Color = Color::Rgb(142, 192, 124);
/// Gruvbox Dark - orange (#FE8019)
pub const GRUVBOX_ORANGE: Color = Color::Rgb(254, 128, 25);

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
/// assert_eq!(theme.focused, envision::theme::NORD8);
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
}

impl Default for Theme {
    /// Returns the default theme matching ratatui's standard colors.
    ///
    /// This theme uses:
    /// - Yellow for focused elements
    /// - DarkGray for disabled/placeholder elements
    /// - Cyan for primary/info
    /// - Standard Green/Yellow/Red for success/warning/error
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
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::Theme;
    ///
    /// let theme = Theme::nord();
    /// // Use with components:
    /// // let mut ctx = RenderContext::new(frame, area, &theme);
    /// // Button::view(&state, &mut ctx);
    /// ```
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
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::Theme;
    ///
    /// let theme = Theme::dracula();
    /// assert_eq!(theme.focused, envision::theme::DRACULA_PURPLE);
    /// ```
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
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::Theme;
    ///
    /// let theme = Theme::solarized_dark();
    /// assert_eq!(theme.focused, envision::theme::SOLARIZED_BLUE);
    /// ```
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
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::Theme;
    ///
    /// let theme = Theme::gruvbox_dark();
    /// assert_eq!(theme.focused, envision::theme::GRUVBOX_YELLOW);
    /// ```
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
    /// assert_eq!(theme.focused, envision::theme::CATPPUCCIN_LAVENDER);
    /// ```
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
