//! Enhanced cell type that captures more information than ratatui's Cell.
//!
//! Uses [`CompactString`] for cell symbols to avoid heap allocation for typical
//! single-character cells. At 80×24, a terminal has 1,920 cells per frame;
//! `CompactString` stores up to 24 bytes inline, eliminating thousands of
//! small allocations per render cycle.

mod diff_option;
pub(crate) mod width;

pub use diff_option::SerializableDiffOption;

use compact_str::CompactString;
use ratatui::style::{Color, Modifier, Style};

/// An enhanced cell that captures all styling information plus metadata.
///
/// Unlike ratatui's `Cell`, this type:
/// - Is fully serializable for snapshots and JSON export
/// - Tracks when the cell was last modified (frame number)
/// - Can store optional semantic annotations
///
/// Style fields are private; read them through [`fg`](Self::fg),
/// [`bg`](Self::bg), [`modifiers`](Self::modifiers),
/// [`underline_color`](Self::underline_color) and
/// [`last_modified_frame`](Self::last_modified_frame).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serialization", serde(from = "EnhancedCellRepr"))]
pub struct EnhancedCell {
    /// The symbol (grapheme cluster) displayed in this cell
    symbol: CompactString,

    /// Foreground color
    fg: SerializableColor,

    /// Background color
    bg: SerializableColor,

    /// Text modifiers (bold, italic, etc.)
    modifiers: SerializableModifier,

    /// Underline color (if different from foreground)
    underline_color: Option<SerializableColor>,

    /// Frame number when this cell was last modified
    last_modified_frame: u64,

    /// How this cell participates in buffer diffing
    #[cfg_attr(
        feature = "serialization",
        serde(default, skip_serializing_if = "SerializableDiffOption::is_none")
    )]
    diff_option: SerializableDiffOption,
}

/// Deserialization form accepting both v0.17 (`skip`) and v0.18 (`diff_option`) JSON.
#[cfg(feature = "serialization")]
#[derive(serde::Deserialize)]
struct EnhancedCellRepr {
    symbol: CompactString,
    fg: SerializableColor,
    bg: SerializableColor,
    modifiers: SerializableModifier,
    underline_color: Option<SerializableColor>,
    last_modified_frame: u64,
    #[serde(default)]
    skip: bool,
    #[serde(default)]
    diff_option: SerializableDiffOption,
}

#[cfg(feature = "serialization")]
impl From<EnhancedCellRepr> for EnhancedCell {
    fn from(r: EnhancedCellRepr) -> Self {
        Self {
            symbol: r.symbol,
            fg: r.fg,
            bg: r.bg,
            modifiers: r.modifiers,
            underline_color: r.underline_color,
            last_modified_frame: r.last_modified_frame,
            diff_option: effective_diff_option(r.diff_option, r.skip),
        }
    }
}

/// ratatui's `is_skip` rule (`ratatui-core-0.1.2/src/buffer/diff.rs:203-205`):
/// the legacy skip flag is honoured only when no explicit option is set.
fn effective_diff_option(
    option: SerializableDiffOption,
    legacy_skip: bool,
) -> SerializableDiffOption {
    if legacy_skip && option.is_none() {
        SerializableDiffOption::Skip
    } else {
        option
    }
}

/// Reads ratatui's diff option, folding in its deprecated `skip` flag exactly
/// as ratatui's own diffing does (`is_skip`, `diff.rs:203-205`).
#[allow(deprecated)] // the one sanctioned read of ratatui's deprecated `Cell::skip`
fn captured_diff_option(cell: &ratatui::buffer::Cell) -> SerializableDiffOption {
    effective_diff_option(cell.diff_option.into(), cell.skip)
}

impl EnhancedCell {
    /// Creates a new empty cell
    pub fn new() -> Self {
        Self {
            symbol: CompactString::from(" "),
            fg: SerializableColor::Reset,
            bg: SerializableColor::Reset,
            modifiers: SerializableModifier::empty(),
            underline_color: None,
            last_modified_frame: 0,
            diff_option: SerializableDiffOption::None,
        }
    }

    /// Creates a cell with the given symbol
    pub fn with_symbol(symbol: impl Into<CompactString>) -> Self {
        Self {
            symbol: symbol.into(),
            ..Self::new()
        }
    }

    /// Creates an EnhancedCell from a ratatui Cell
    pub fn from_ratatui_cell(cell: &ratatui::buffer::Cell, frame: u64) -> Self {
        let style = cell.style();
        Self {
            symbol: CompactString::from(cell.symbol()),
            fg: style
                .fg
                .map(SerializableColor::from)
                .unwrap_or(SerializableColor::Reset),
            bg: style
                .bg
                .map(SerializableColor::from)
                .unwrap_or(SerializableColor::Reset),
            modifiers: SerializableModifier::from(style.add_modifier),
            underline_color: style.underline_color.map(SerializableColor::from),
            last_modified_frame: frame,
            diff_option: captured_diff_option(cell),
        }
    }

    /// Returns the symbol in this cell
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Sets the symbol in this cell
    pub fn set_symbol(&mut self, symbol: impl Into<CompactString>) {
        self.symbol = symbol.into();
    }

    /// Sets the symbol to a single character
    pub fn set_char(&mut self, c: char) {
        self.symbol.clear();
        self.symbol.push(c);
    }

    /// Display width of the symbol in terminal cells, as ratatui computes it.
    pub fn symbol_width(&self) -> usize {
        usize::from(width::symbol_cell_width(&self.symbol))
    }

    /// Foreground colour.
    pub fn fg(&self) -> SerializableColor {
        self.fg
    }

    /// Background colour.
    pub fn bg(&self) -> SerializableColor {
        self.bg
    }

    /// Text modifiers (bold, italic, ...).
    pub fn modifiers(&self) -> SerializableModifier {
        self.modifiers
    }

    /// Underline colour, if one was set separately from the foreground.
    pub fn underline_color(&self) -> Option<SerializableColor> {
        self.underline_color
    }

    /// Frame number at which this cell was last written.
    pub fn last_modified_frame(&self) -> u64 {
        self.last_modified_frame
    }

    /// How this cell participates in diffing.
    pub fn diff_option(&self) -> SerializableDiffOption {
        self.diff_option
    }

    /// Sets the diff option (mirrors ratatui's `Cell::set_diff_option`).
    pub fn set_diff_option(&mut self, option: SerializableDiffOption) {
        self.diff_option = option;
    }

    /// Stamps the frame number at which this cell was last written.
    pub(crate) fn set_last_modified_frame(&mut self, frame: u64) {
        self.last_modified_frame = frame;
    }

    /// Number of columns this cell owns on screen: the single owner
    /// definition for wide-glyph emulation (spec § 5, ruling R13).
    ///
    /// A blank cell (see [`is_blank_symbol`](Self::is_blank_symbol)) never owns
    /// continuation columns and returns 1, even when it keeps a `ForcedWidth`
    /// option. Blanking changes only the symbol, so a blanked owner can still
    /// carry one. Otherwise the result is `ForcedWidth(n)` when set, or
    /// ratatui's width of the symbol ("Width source").
    pub(crate) fn owner_width(&self) -> u16 {
        if self.is_blank_symbol() {
            return 1;
        }
        match self.diff_option {
            SerializableDiffOption::ForcedWidth(w) => w.get(),
            _ => width::symbol_cell_width(&self.symbol),
        }
    }

    /// True when the symbol is `" "` or empty, whatever the cell's style.
    pub(crate) fn is_blank_symbol(&self) -> bool {
        self.symbol.is_empty() || self.symbol == " "
    }

    /// Sets the style from a ratatui Style
    pub fn set_style(&mut self, style: Style) {
        if let Some(fg) = style.fg {
            self.fg = SerializableColor::from(fg);
        }
        if let Some(bg) = style.bg {
            self.bg = SerializableColor::from(bg);
        }
        self.modifiers = self
            .modifiers
            .union(SerializableModifier::from(style.add_modifier));
        self.modifiers = self
            .modifiers
            .difference(SerializableModifier::from(style.sub_modifier));
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
        self.symbol = CompactString::from(" ");
        self.fg = SerializableColor::Reset;
        self.bg = SerializableColor::Reset;
        self.modifiers = SerializableModifier::empty();
        self.underline_color = None;
        self.diff_option = SerializableDiffOption::None;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serialization", serde(rename_all = "snake_case"))]
pub enum SerializableColor {
    /// The terminal default colour.
    Reset,
    /// The `Black` ANSI colour.
    Black,
    /// The `Red` ANSI colour.
    Red,
    /// The `Green` ANSI colour.
    Green,
    /// The `Yellow` ANSI colour.
    Yellow,
    /// The `Blue` ANSI colour.
    Blue,
    /// The `Magenta` ANSI colour.
    Magenta,
    /// The `Cyan` ANSI colour.
    Cyan,
    /// The `Gray` ANSI colour.
    Gray,
    /// The `DarkGray` ANSI colour.
    DarkGray,
    /// The `LightRed` ANSI colour.
    LightRed,
    /// The `LightGreen` ANSI colour.
    LightGreen,
    /// The `LightYellow` ANSI colour.
    LightYellow,
    /// The `LightBlue` ANSI colour.
    LightBlue,
    /// The `LightMagenta` ANSI colour.
    LightMagenta,
    /// The `LightCyan` ANSI colour.
    LightCyan,
    /// The `White` ANSI colour.
    White,
    /// A 24-bit RGB colour.
    Rgb {
        /// Red channel.
        r: u8,
        /// Green channel.
        g: u8,
        /// Blue channel.
        b: u8,
    },
    /// An entry in the 256-colour palette.
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
    pub fn to_ansi_fg(self) -> String {
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
            SerializableColor::Rgb { r, g, b } => format!("\x1b[38;2;{r};{g};{b}m"),
            SerializableColor::Indexed(i) => format!("\x1b[38;5;{i}m"),
        }
    }

    /// Returns the ANSI escape code for this color as background
    pub fn to_ansi_bg(self) -> String {
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
            SerializableColor::Rgb { r, g, b } => format!("\x1b[48;2;{r};{g};{b}m"),
            SerializableColor::Indexed(i) => format!("\x1b[48;5;{i}m"),
        }
    }
}

/// A serializable version of ratatui's Modifier flags
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct SerializableModifier {
    /// Bold text.
    pub bold: bool,
    /// Dimmed text.
    pub dim: bool,
    /// Italic text.
    pub italic: bool,
    /// Underlined text.
    pub underlined: bool,
    /// Slowly blinking text.
    pub slow_blink: bool,
    /// Rapidly blinking text.
    pub rapid_blink: bool,
    /// Foreground and background swapped.
    pub reversed: bool,
    /// Concealed text.
    pub hidden: bool,
    /// Strikethrough text.
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
    pub fn to_ansi(self) -> String {
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
mod tests;
