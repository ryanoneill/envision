//! Enhanced cell type that captures more information than ratatui's Cell.

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr;

/// An enhanced cell that captures all styling information plus metadata.
///
/// Unlike ratatui's `Cell`, this type:
/// - Is fully serializable for snapshots and JSON export
/// - Tracks when the cell was last modified (frame number)
/// - Can store optional semantic annotations
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnhancedCell {
    /// The symbol (grapheme cluster) displayed in this cell
    symbol: String,

    /// Foreground color
    pub fg: SerializableColor,

    /// Background color
    pub bg: SerializableColor,

    /// Text modifiers (bold, italic, etc.)
    pub modifiers: SerializableModifier,

    /// Underline color (if different from foreground)
    pub underline_color: Option<SerializableColor>,

    /// Frame number when this cell was last modified
    pub last_modified_frame: u64,

    /// Whether this cell should be skipped during rendering
    pub skip: bool,
}

impl EnhancedCell {
    /// Creates a new empty cell
    pub fn new() -> Self {
        Self {
            symbol: " ".to_string(),
            fg: SerializableColor::Reset,
            bg: SerializableColor::Reset,
            modifiers: SerializableModifier::empty(),
            underline_color: None,
            last_modified_frame: 0,
            skip: false,
        }
    }

    /// Creates a cell with the given symbol
    pub fn with_symbol(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            ..Self::new()
        }
    }

    /// Creates an EnhancedCell from a ratatui Cell
    pub fn from_ratatui_cell(cell: &ratatui::buffer::Cell, frame: u64) -> Self {
        let style = cell.style();
        Self {
            symbol: cell.symbol().to_string(),
            fg: style.fg.map(SerializableColor::from).unwrap_or(SerializableColor::Reset),
            bg: style.bg.map(SerializableColor::from).unwrap_or(SerializableColor::Reset),
            modifiers: SerializableModifier::from(style.add_modifier),
            underline_color: style.underline_color.map(SerializableColor::from),
            last_modified_frame: frame,
            skip: cell.skip,
        }
    }

    /// Returns the symbol in this cell
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Sets the symbol in this cell
    pub fn set_symbol(&mut self, symbol: impl Into<String>) {
        self.symbol = symbol.into();
    }

    /// Sets the symbol to a single character
    pub fn set_char(&mut self, c: char) {
        self.symbol.clear();
        self.symbol.push(c);
    }

    /// Returns the display width of the symbol
    pub fn symbol_width(&self) -> usize {
        self.symbol.width()
    }

    /// Sets the style from a ratatui Style
    pub fn set_style(&mut self, style: Style) {
        if let Some(fg) = style.fg {
            self.fg = SerializableColor::from(fg);
        }
        if let Some(bg) = style.bg {
            self.bg = SerializableColor::from(bg);
        }
        self.modifiers = self.modifiers.union(SerializableModifier::from(style.add_modifier));
        self.modifiers = self.modifiers.difference(SerializableModifier::from(style.sub_modifier));
        if let Some(underline) = style.underline_color {
            self.underline_color = Some(SerializableColor::from(underline));
        }
    }

    /// Returns the style as a ratatui Style
    pub fn style(&self) -> Style {
        Style::new()
            .fg(self.fg.into())
            .bg(self.bg.into())
            .add_modifier(self.modifiers.into())
    }

    /// Resets the cell to empty state
    pub fn reset(&mut self) {
        self.symbol = " ".to_string();
        self.fg = SerializableColor::Reset;
        self.bg = SerializableColor::Reset;
        self.modifiers = SerializableModifier::empty();
        self.underline_color = None;
        self.skip = false;
    }

    /// Returns true if this cell is empty (space with default styling)
    pub fn is_empty(&self) -> bool {
        self.symbol == " "
            && self.fg == SerializableColor::Reset
            && self.bg == SerializableColor::Reset
            && self.modifiers.is_empty()
    }
}

impl Default for EnhancedCell {
    fn default() -> Self {
        Self::new()
    }
}

/// A serializable version of ratatui's Color enum
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SerializableColor {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb { r: u8, g: u8, b: u8 },
    Indexed(u8),
}

impl From<Color> for SerializableColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Reset => SerializableColor::Reset,
            Color::Black => SerializableColor::Black,
            Color::Red => SerializableColor::Red,
            Color::Green => SerializableColor::Green,
            Color::Yellow => SerializableColor::Yellow,
            Color::Blue => SerializableColor::Blue,
            Color::Magenta => SerializableColor::Magenta,
            Color::Cyan => SerializableColor::Cyan,
            Color::Gray => SerializableColor::Gray,
            Color::DarkGray => SerializableColor::DarkGray,
            Color::LightRed => SerializableColor::LightRed,
            Color::LightGreen => SerializableColor::LightGreen,
            Color::LightYellow => SerializableColor::LightYellow,
            Color::LightBlue => SerializableColor::LightBlue,
            Color::LightMagenta => SerializableColor::LightMagenta,
            Color::LightCyan => SerializableColor::LightCyan,
            Color::White => SerializableColor::White,
            Color::Rgb(r, g, b) => SerializableColor::Rgb { r, g, b },
            Color::Indexed(i) => SerializableColor::Indexed(i),
        }
    }
}

impl From<SerializableColor> for Color {
    fn from(color: SerializableColor) -> Self {
        match color {
            SerializableColor::Reset => Color::Reset,
            SerializableColor::Black => Color::Black,
            SerializableColor::Red => Color::Red,
            SerializableColor::Green => Color::Green,
            SerializableColor::Yellow => Color::Yellow,
            SerializableColor::Blue => Color::Blue,
            SerializableColor::Magenta => Color::Magenta,
            SerializableColor::Cyan => Color::Cyan,
            SerializableColor::Gray => Color::Gray,
            SerializableColor::DarkGray => Color::DarkGray,
            SerializableColor::LightRed => Color::LightRed,
            SerializableColor::LightGreen => Color::LightGreen,
            SerializableColor::LightYellow => Color::LightYellow,
            SerializableColor::LightBlue => Color::LightBlue,
            SerializableColor::LightMagenta => Color::LightMagenta,
            SerializableColor::LightCyan => Color::LightCyan,
            SerializableColor::White => Color::White,
            SerializableColor::Rgb { r, g, b } => Color::Rgb(r, g, b),
            SerializableColor::Indexed(i) => Color::Indexed(i),
        }
    }
}

impl SerializableColor {
    /// Returns the ANSI escape code for this color as foreground
    pub fn to_ansi_fg(&self) -> String {
        match self {
            SerializableColor::Reset => "\x1b[39m".to_string(),
            SerializableColor::Black => "\x1b[30m".to_string(),
            SerializableColor::Red => "\x1b[31m".to_string(),
            SerializableColor::Green => "\x1b[32m".to_string(),
            SerializableColor::Yellow => "\x1b[33m".to_string(),
            SerializableColor::Blue => "\x1b[34m".to_string(),
            SerializableColor::Magenta => "\x1b[35m".to_string(),
            SerializableColor::Cyan => "\x1b[36m".to_string(),
            SerializableColor::Gray => "\x1b[37m".to_string(),
            SerializableColor::DarkGray => "\x1b[90m".to_string(),
            SerializableColor::LightRed => "\x1b[91m".to_string(),
            SerializableColor::LightGreen => "\x1b[92m".to_string(),
            SerializableColor::LightYellow => "\x1b[93m".to_string(),
            SerializableColor::LightBlue => "\x1b[94m".to_string(),
            SerializableColor::LightMagenta => "\x1b[95m".to_string(),
            SerializableColor::LightCyan => "\x1b[96m".to_string(),
            SerializableColor::White => "\x1b[97m".to_string(),
            SerializableColor::Rgb { r, g, b } => format!("\x1b[38;2;{};{};{}m", r, g, b),
            SerializableColor::Indexed(i) => format!("\x1b[38;5;{}m", i),
        }
    }

    /// Returns the ANSI escape code for this color as background
    pub fn to_ansi_bg(&self) -> String {
        match self {
            SerializableColor::Reset => "\x1b[49m".to_string(),
            SerializableColor::Black => "\x1b[40m".to_string(),
            SerializableColor::Red => "\x1b[41m".to_string(),
            SerializableColor::Green => "\x1b[42m".to_string(),
            SerializableColor::Yellow => "\x1b[43m".to_string(),
            SerializableColor::Blue => "\x1b[44m".to_string(),
            SerializableColor::Magenta => "\x1b[45m".to_string(),
            SerializableColor::Cyan => "\x1b[46m".to_string(),
            SerializableColor::Gray => "\x1b[47m".to_string(),
            SerializableColor::DarkGray => "\x1b[100m".to_string(),
            SerializableColor::LightRed => "\x1b[101m".to_string(),
            SerializableColor::LightGreen => "\x1b[102m".to_string(),
            SerializableColor::LightYellow => "\x1b[103m".to_string(),
            SerializableColor::LightBlue => "\x1b[104m".to_string(),
            SerializableColor::LightMagenta => "\x1b[105m".to_string(),
            SerializableColor::LightCyan => "\x1b[106m".to_string(),
            SerializableColor::White => "\x1b[107m".to_string(),
            SerializableColor::Rgb { r, g, b } => format!("\x1b[48;2;{};{};{}m", r, g, b),
            SerializableColor::Indexed(i) => format!("\x1b[48;5;{}m", i),
        }
    }
}

/// A serializable version of ratatui's Modifier flags
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SerializableModifier {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underlined: bool,
    pub slow_blink: bool,
    pub rapid_blink: bool,
    pub reversed: bool,
    pub hidden: bool,
    pub crossed_out: bool,
}

impl SerializableModifier {
    /// Creates an empty modifier set
    pub const fn empty() -> Self {
        Self {
            bold: false,
            dim: false,
            italic: false,
            underlined: false,
            slow_blink: false,
            rapid_blink: false,
            reversed: false,
            hidden: false,
            crossed_out: false,
        }
    }

    /// Returns true if no modifiers are set
    pub fn is_empty(&self) -> bool {
        !self.bold
            && !self.dim
            && !self.italic
            && !self.underlined
            && !self.slow_blink
            && !self.rapid_blink
            && !self.reversed
            && !self.hidden
            && !self.crossed_out
    }

    /// Returns the union of two modifier sets
    pub fn union(self, other: Self) -> Self {
        Self {
            bold: self.bold || other.bold,
            dim: self.dim || other.dim,
            italic: self.italic || other.italic,
            underlined: self.underlined || other.underlined,
            slow_blink: self.slow_blink || other.slow_blink,
            rapid_blink: self.rapid_blink || other.rapid_blink,
            reversed: self.reversed || other.reversed,
            hidden: self.hidden || other.hidden,
            crossed_out: self.crossed_out || other.crossed_out,
        }
    }

    /// Returns self with modifiers from other removed
    pub fn difference(self, other: Self) -> Self {
        Self {
            bold: self.bold && !other.bold,
            dim: self.dim && !other.dim,
            italic: self.italic && !other.italic,
            underlined: self.underlined && !other.underlined,
            slow_blink: self.slow_blink && !other.slow_blink,
            rapid_blink: self.rapid_blink && !other.rapid_blink,
            reversed: self.reversed && !other.reversed,
            hidden: self.hidden && !other.hidden,
            crossed_out: self.crossed_out && !other.crossed_out,
        }
    }

    /// Returns the ANSI escape codes for these modifiers
    pub fn to_ansi(&self) -> String {
        let mut codes = Vec::new();
        if self.bold {
            codes.push("1");
        }
        if self.dim {
            codes.push("2");
        }
        if self.italic {
            codes.push("3");
        }
        if self.underlined {
            codes.push("4");
        }
        if self.slow_blink {
            codes.push("5");
        }
        if self.rapid_blink {
            codes.push("6");
        }
        if self.reversed {
            codes.push("7");
        }
        if self.hidden {
            codes.push("8");
        }
        if self.crossed_out {
            codes.push("9");
        }
        if codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", codes.join(";"))
        }
    }
}

impl From<Modifier> for SerializableModifier {
    fn from(modifier: Modifier) -> Self {
        Self {
            bold: modifier.contains(Modifier::BOLD),
            dim: modifier.contains(Modifier::DIM),
            italic: modifier.contains(Modifier::ITALIC),
            underlined: modifier.contains(Modifier::UNDERLINED),
            slow_blink: modifier.contains(Modifier::SLOW_BLINK),
            rapid_blink: modifier.contains(Modifier::RAPID_BLINK),
            reversed: modifier.contains(Modifier::REVERSED),
            hidden: modifier.contains(Modifier::HIDDEN),
            crossed_out: modifier.contains(Modifier::CROSSED_OUT),
        }
    }
}

impl From<SerializableModifier> for Modifier {
    fn from(modifier: SerializableModifier) -> Self {
        let mut m = Modifier::empty();
        if modifier.bold {
            m |= Modifier::BOLD;
        }
        if modifier.dim {
            m |= Modifier::DIM;
        }
        if modifier.italic {
            m |= Modifier::ITALIC;
        }
        if modifier.underlined {
            m |= Modifier::UNDERLINED;
        }
        if modifier.slow_blink {
            m |= Modifier::SLOW_BLINK;
        }
        if modifier.rapid_blink {
            m |= Modifier::RAPID_BLINK;
        }
        if modifier.reversed {
            m |= Modifier::REVERSED;
        }
        if modifier.hidden {
            m |= Modifier::HIDDEN;
        }
        if modifier.crossed_out {
            m |= Modifier::CROSSED_OUT;
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_cell_default() {
        let cell = EnhancedCell::new();
        assert_eq!(cell.symbol(), " ");
        assert_eq!(cell.fg, SerializableColor::Reset);
        assert_eq!(cell.bg, SerializableColor::Reset);
        assert!(cell.modifiers.is_empty());
        assert!(cell.is_empty());
    }

    #[test]
    fn test_enhanced_cell_with_symbol() {
        let cell = EnhancedCell::with_symbol("A");
        assert_eq!(cell.symbol(), "A");
        assert!(!cell.is_empty());
    }

    #[test]
    fn test_serializable_color_roundtrip() {
        let colors = vec![
            Color::Red,
            Color::Rgb(100, 150, 200),
            Color::Indexed(42),
            Color::Reset,
        ];

        for color in colors {
            let serializable = SerializableColor::from(color);
            let back: Color = serializable.into();
            assert_eq!(color, back);
        }
    }

    #[test]
    fn test_serializable_modifier_roundtrip() {
        let modifier = Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED;
        let serializable = SerializableModifier::from(modifier);
        assert!(serializable.bold);
        assert!(serializable.italic);
        assert!(serializable.underlined);
        assert!(!serializable.dim);

        let back: Modifier = serializable.into();
        assert_eq!(modifier, back);
    }

    #[test]
    fn test_ansi_fg_codes() {
        assert_eq!(SerializableColor::Red.to_ansi_fg(), "\x1b[31m");
        assert_eq!(
            SerializableColor::Rgb { r: 255, g: 128, b: 0 }.to_ansi_fg(),
            "\x1b[38;2;255;128;0m"
        );
        assert_eq!(SerializableColor::Indexed(42).to_ansi_fg(), "\x1b[38;5;42m");
    }

    #[test]
    fn test_modifier_ansi_codes() {
        let mut modifier = SerializableModifier::empty();
        modifier.bold = true;
        modifier.italic = true;
        assert_eq!(modifier.to_ansi(), "\x1b[1;3m");
    }

    #[test]
    fn test_cell_serialization() {
        let cell = EnhancedCell {
            symbol: "X".to_string(),
            fg: SerializableColor::Red,
            bg: SerializableColor::Blue,
            modifiers: SerializableModifier {
                bold: true,
                ..Default::default()
            },
            underline_color: None,
            last_modified_frame: 5,
            skip: false,
        };

        let json = serde_json::to_string(&cell).unwrap();
        let deserialized: EnhancedCell = serde_json::from_str(&json).unwrap();
        assert_eq!(cell, deserialized);
    }
}
