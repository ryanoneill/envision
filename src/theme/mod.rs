//! Theming support for Envision components.
//!
//! The theme module provides customizable color schemes for all UI components.
//! Two themes are included by default: a `Default` theme matching ratatui's
//! standard colors, and a `Nord` theme based on the popular Nord color palette.
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
//! // Components use theme in their view() method:
//! // Component::view(&state, frame, area, &nord_theme);
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
    /// // Button::view(&state, frame, area, &theme);
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

    // =========================================================================
    // Style Helper Methods
    // =========================================================================

    /// Returns a style for focused elements.
    ///
    /// Uses the theme's focused color for foreground.
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
    pub fn focused_border_style(&self) -> Style {
        Style::default().fg(self.focused)
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

    /// Returns a style for disabled elements.
    pub fn disabled_style(&self) -> Style {
        Style::default().fg(self.disabled)
    }

    /// Returns a style for placeholder text.
    pub fn placeholder_style(&self) -> Style {
        Style::default().fg(self.placeholder)
    }

    /// Returns a style for default/normal elements.
    pub fn normal_style(&self) -> Style {
        Style::default()
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
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.focused, Color::Yellow);
        assert_eq!(theme.disabled, Color::DarkGray);
        assert_eq!(theme.success, Color::Green);
        assert_eq!(theme.warning, Color::Yellow);
        assert_eq!(theme.error, Color::Red);
        assert_eq!(theme.info, Color::Cyan);
    }

    #[test]
    fn test_nord_theme() {
        let theme = Theme::nord();
        assert_eq!(theme.focused, NORD8);
        assert_eq!(theme.selected, NORD9);
        assert_eq!(theme.disabled, NORD3);
        assert_eq!(theme.success, NORD14);
        assert_eq!(theme.warning, NORD13);
        assert_eq!(theme.error, NORD11);
        assert_eq!(theme.info, NORD8);
        assert_eq!(theme.background, NORD0);
        assert_eq!(theme.foreground, NORD6);
    }

    #[test]
    fn test_nord_colors() {
        assert_eq!(NORD0, Color::Rgb(46, 52, 64));
        assert_eq!(NORD8, Color::Rgb(136, 192, 208));
        assert_eq!(NORD14, Color::Rgb(163, 190, 140));
    }

    #[test]
    fn test_focused_style() {
        let theme = Theme::default();
        let style = theme.focused_style();
        assert_eq!(style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_focused_bold_style() {
        let theme = Theme::default();
        let style = theme.focused_bold_style();
        assert_eq!(style.fg, Some(Color::Yellow));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_selected_style_focused() {
        let theme = Theme::default();
        let style = theme.selected_style(true);
        assert_eq!(style.fg, Some(Color::Yellow));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_selected_style_unfocused() {
        let theme = Theme::default();
        let style = theme.selected_style(false);
        assert_eq!(style.fg, None);
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_disabled_style() {
        let theme = Theme::default();
        let style = theme.disabled_style();
        assert_eq!(style.fg, Some(Color::DarkGray));
    }

    #[test]
    fn test_placeholder_style() {
        let theme = Theme::default();
        let style = theme.placeholder_style();
        assert_eq!(style.fg, Some(Color::DarkGray));
    }

    #[test]
    fn test_semantic_styles() {
        let theme = Theme::default();

        let success = theme.success_style();
        assert_eq!(success.fg, Some(Color::Green));

        let warning = theme.warning_style();
        assert_eq!(warning.fg, Some(Color::Yellow));

        let error = theme.error_style();
        assert_eq!(error.fg, Some(Color::Red));

        let info = theme.info_style();
        assert_eq!(info.fg, Some(Color::Cyan));
    }

    #[test]
    fn test_progress_filled_style() {
        let theme = Theme::default();
        let style = theme.progress_filled_style();
        assert_eq!(style.fg, Some(Color::Cyan));
        assert_eq!(style.bg, Some(Color::Black));
    }

    #[test]
    fn test_theme_clone() {
        let theme = Theme::nord();
        let cloned = theme.clone();
        assert_eq!(theme, cloned);
    }

    #[test]
    fn test_theme_debug() {
        let theme = Theme::default();
        let debug = format!("{:?}", theme);
        assert!(debug.contains("Theme"));
    }

    #[test]
    fn test_custom_theme() {
        let custom = Theme {
            focused: Color::Magenta,
            selected: Color::LightBlue,
            ..Theme::default()
        };
        assert_eq!(custom.focused, Color::Magenta);
        assert_eq!(custom.selected, Color::LightBlue);
        assert_eq!(custom.disabled, Color::DarkGray); // From default
    }

    #[test]
    fn test_normal_style() {
        let theme = Theme::default();
        let style = theme.normal_style();
        assert_eq!(style.fg, None);
        assert_eq!(style.bg, None);
    }

    #[test]
    fn test_border_style() {
        let theme = Theme::nord();
        let style = theme.border_style();
        assert_eq!(style.fg, Some(NORD3));
    }

    #[test]
    fn test_selected_highlight_style_focused() {
        let theme = Theme::nord();
        let style = theme.selected_highlight_style(true);
        assert_eq!(style.bg, Some(NORD9));
        assert_eq!(style.fg, Some(NORD6));
    }

    #[test]
    fn test_selected_highlight_style_unfocused() {
        let theme = Theme::nord();
        let style = theme.selected_highlight_style(false);
        assert_eq!(style.bg, Some(NORD3));
        assert_eq!(style.fg, Some(NORD6));
    }
}
