//! A large pixel text component for dashboard hero numbers and KPI values.
//!
//! [`BigText`] renders text using large block characters where each character
//! occupies 3 rows and a variable width. This is ideal for dashboard counters,
//! clocks, and key performance indicators that need to stand out visually.
//!
//! State is stored in [`BigTextState`] and updated via [`BigTextMessage`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Component, BigText, BigTextState, BigTextMessage};
//! use ratatui::prelude::*;
//!
//! let state = BigTextState::new("12:30")
//!     .with_color(Color::Cyan)
//!     .with_alignment(Alignment::Center);
//!
//! assert_eq!(state.text(), "12:30");
//! assert_eq!(state.color(), Some(Color::Cyan));
//! assert_eq!(state.alignment(), Alignment::Center);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{Component, Disableable};
use crate::theme::Theme;

/// Returns the 3-row block representation of a character.
///
/// Each supported character is rendered as an array of 3 string slices,
/// one per row. All rows for a given character have the same display width.
///
/// # Supported characters
///
/// - Digits: `0`-`9`
/// - Punctuation: `.`, `:`, `-`, `/`, `%`, ` `
/// - Uppercase letters: `A`-`Z`
///
/// Unsupported characters return a `?` placeholder glyph.
///
/// # Example
///
/// ```rust
/// use envision::component::big_char;
///
/// let rows = big_char('0');
/// assert_eq!(rows.len(), 3);
/// assert_eq!(rows[0], "█▀█");
/// assert_eq!(rows[1], "█ █");
/// assert_eq!(rows[2], "▀▀▀");
/// ```
pub fn big_char(ch: char) -> [&'static str; 3] {
    match ch {
        '0' => ["█▀█", "█ █", "▀▀▀"],
        '1' => ["▀█ ", " █ ", "▀▀▀"],
        '2' => ["▀▀█", "█▀▀", "▀▀▀"],
        '3' => ["▀▀█", " ▀█", "▀▀▀"],
        '4' => ["█ █", "▀▀█", "  ▀"],
        '5' => ["█▀▀", "▀▀█", "▀▀▀"],
        '6' => ["█▀▀", "█▀█", "▀▀▀"],
        '7' => ["▀▀█", "  █", "  ▀"],
        '8' => ["█▀█", "█▀█", "▀▀▀"],
        '9' => ["█▀█", "▀▀█", "▀▀▀"],
        '.' => [" ", "▄", " "],
        ':' => ["▄", " ", "▀"],
        '-' => ["   ", "▀▀▀", "   "],
        '/' => ["  █", " █ ", "█  "],
        '%' => ["█ █", " █ ", "█ █"],
        ' ' => ["   ", "   ", "   "],

        'A' => ["█▀█", "█▀█", "▀ ▀"],
        'B' => ["█▀▄", "█▀█", "▀▀▀"],
        'C' => ["█▀▀", "█  ", "▀▀▀"],
        'D' => ["█▀▄", "█ █", "▀▀▀"],
        'E' => ["█▀▀", "█▀▀", "▀▀▀"],
        'F' => ["█▀▀", "█▀ ", "▀  "],
        'G' => ["█▀▀", "█▀█", "▀▀▀"],
        'H' => ["█ █", "█▀█", "▀ ▀"],
        'I' => ["▀█▀", " █ ", "▀▀▀"],
        'J' => ["  █", "  █", "▀▀▀"],
        'K' => ["█ █", "█▀▄", "▀ ▀"],
        'L' => ["█  ", "█  ", "▀▀▀"],
        'M' => ["█▄█", "█ █", "▀ ▀"],
        'N' => ["█▀█", "█ █", "▀ ▀"],
        'O' => ["█▀█", "█ █", "▀▀▀"],
        'P' => ["█▀█", "█▀▀", "▀  "],
        'Q' => ["█▀█", "█▄█", "▀▀▀"],
        'R' => ["█▀█", "█▀▄", "▀ ▀"],
        'S' => ["█▀▀", "▀▀█", "▀▀▀"],
        'T' => ["▀█▀", " █ ", " ▀ "],
        'U' => ["█ █", "█ █", "▀▀▀"],
        'V' => ["█ █", "█ █", " ▀ "],
        'W' => ["█ █", "█ █", "▀▄▀"],
        'X' => ["█ █", " █ ", "█ █"],
        'Y' => ["█ █", " █ ", " ▀ "],
        'Z' => ["▀▀█", " █ ", "▀▀▀"],

        _ => ["▀▀▀", " ▀ ", " ▀ "],
    }
}

/// Returns the display width of a big character glyph.
///
/// This measures the Unicode display width of the first row of the
/// character's 3-row representation.
///
/// # Example
///
/// ```rust
/// use envision::component::big_char_width;
///
/// assert_eq!(big_char_width('0'), 3);
/// assert_eq!(big_char_width('.'), 1);
/// assert_eq!(big_char_width(':'), 1);
/// ```
pub fn big_char_width(ch: char) -> usize {
    unicode_width::UnicodeWidthStr::width(big_char(ch)[0])
}

/// Messages that can be sent to a BigText component.
#[derive(Clone, Debug, PartialEq)]
pub enum BigTextMessage {
    /// Replace the displayed text.
    SetText(String),
    /// Set the color override.
    SetColor(Option<Color>),
    /// Set the text alignment.
    SetAlignment(Alignment),
}

/// State for a BigText component.
///
/// Contains the text to display in large block characters, along with
/// optional color and alignment configuration.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct BigTextState {
    /// The text to render in large block characters.
    text: String,
    /// Optional color override for the text.
    color: Option<Color>,
    /// Text alignment within the render area.
    #[cfg_attr(feature = "serialization", serde(skip))]
    alignment: Alignment,
    /// Whether the component is disabled.
    disabled: bool,
}

impl Default for BigTextState {
    fn default() -> Self {
        Self {
            text: String::new(),
            color: None,
            alignment: Alignment::Center,
            disabled: false,
        }
    }
}

impl BigTextState {
    /// Creates a new BigText state with the given text.
    ///
    /// The default alignment is center and no color override is applied.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    ///
    /// let state = BigTextState::new("42");
    /// assert_eq!(state.text(), "42");
    /// assert_eq!(state.color(), None);
    /// assert!(!state.is_disabled());
    /// ```
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    // ---- Builders ----

    /// Sets the color override (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    /// use ratatui::style::Color;
    ///
    /// let state = BigTextState::new("99")
    ///     .with_color(Color::Green);
    /// assert_eq!(state.color(), Some(Color::Green));
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the text alignment (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    /// use ratatui::prelude::Alignment;
    ///
    /// let state = BigTextState::new("OK")
    ///     .with_alignment(Alignment::Left);
    /// assert_eq!(state.alignment(), Alignment::Left);
    /// ```
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    ///
    /// let state = BigTextState::new("OFF")
    ///     .with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // ---- Getters ----

    /// Returns the text being displayed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    ///
    /// let state = BigTextState::new("123");
    /// assert_eq!(state.text(), "123");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the optional color override.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    ///
    /// let state = BigTextState::new("0");
    /// assert_eq!(state.color(), None);
    /// ```
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Returns the text alignment.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    /// use ratatui::prelude::Alignment;
    ///
    /// let state = BigTextState::new("0");
    /// assert_eq!(state.alignment(), Alignment::Center);
    /// ```
    pub fn alignment(&self) -> Alignment {
        self.alignment
    }

    /// Returns whether the component is disabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    ///
    /// let state = BigTextState::new("0");
    /// assert!(!state.is_disabled());
    /// ```
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    // ---- Setters ----

    /// Sets the text to display.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    ///
    /// let mut state = BigTextState::new("old");
    /// state.set_text("new");
    /// assert_eq!(state.text(), "new");
    /// ```
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Sets the color override.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    /// use ratatui::style::Color;
    ///
    /// let mut state = BigTextState::new("0");
    /// state.set_color(Some(Color::Red));
    /// assert_eq!(state.color(), Some(Color::Red));
    /// ```
    pub fn set_color(&mut self, color: Option<Color>) {
        self.color = color;
    }

    /// Sets the text alignment.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    /// use ratatui::prelude::Alignment;
    ///
    /// let mut state = BigTextState::new("0");
    /// state.set_alignment(Alignment::Right);
    /// assert_eq!(state.alignment(), Alignment::Right);
    /// ```
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = alignment;
    }

    /// Sets the disabled state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    ///
    /// let mut state = BigTextState::new("0");
    /// state.set_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    // ---- Instance methods ----

    /// Maps an input event to a message (instance method).
    ///
    /// BigText is display-only, so this always returns `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    /// use envision::input::Event;
    /// use crossterm::event::KeyCode;
    ///
    /// let state = BigTextState::new("42");
    /// assert_eq!(state.handle_event(&Event::key(KeyCode::Enter)), None);
    /// ```
    pub fn handle_event(&self, event: &crate::input::Event) -> Option<BigTextMessage> {
        BigText::handle_event(self, event)
    }

    /// Updates the state with a message, returning any output (instance method).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    /// use envision::component::BigTextMessage;
    ///
    /// let mut state = BigTextState::new("old");
    /// state.update(BigTextMessage::SetText("new".to_string()));
    /// assert_eq!(state.text(), "new");
    /// ```
    pub fn update(&mut self, msg: BigTextMessage) -> Option<()> {
        BigText::update(self, msg)
    }

    /// Dispatches an event by mapping and updating (instance method).
    ///
    /// BigText is display-only, so this always returns `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BigTextState;
    /// use envision::input::Event;
    /// use crossterm::event::KeyCode;
    ///
    /// let mut state = BigTextState::new("42");
    /// assert_eq!(state.dispatch_event(&Event::key(KeyCode::Enter)), None);
    /// ```
    pub fn dispatch_event(&mut self, event: &crate::input::Event) -> Option<()> {
        BigText::dispatch_event(self, event)
    }
}

/// Builds the rendered content for one row of big text.
///
/// Given a text string and a row index (0-2), this concatenates the
/// appropriate row slice from each character's big font glyph, with a
/// single space separator between characters.
fn build_row(text: &str, row: usize) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }
        let glyph = big_char(ch.to_ascii_uppercase());
        result.push_str(glyph[row]);
    }
    result
}

/// A large pixel text component for dashboard hero numbers and KPI values.
///
/// Renders text using large 3-row block characters. This is a display-only
/// component and does not handle interactive events.
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, BigText, BigTextState};
/// use ratatui::prelude::*;
///
/// let state = BigTextState::new("99.9%")
///     .with_color(Color::Green)
///     .with_alignment(Alignment::Center);
///
/// assert_eq!(state.text(), "99.9%");
/// ```
pub struct BigText;

impl Component for BigText {
    type State = BigTextState;
    type Message = BigTextMessage;
    type Output = ();

    fn init() -> Self::State {
        BigTextState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            BigTextMessage::SetText(text) => state.text = text,
            BigTextMessage::SetColor(color) => state.color = color,
            BigTextMessage::SetAlignment(alignment) => state.alignment = alignment,
        }
        None
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::big_text("big_text")
                    .with_label(&state.text)
                    .with_disabled(state.disabled),
            );
        });

        if area.height == 0 || area.width == 0 {
            return;
        }

        let style = if state.disabled {
            theme.disabled_style()
        } else if let Some(color) = state.color {
            Style::default().fg(color)
        } else {
            theme.normal_style()
        };

        // Build the 3 rows of big text
        let lines: Vec<Line<'_>> = (0..3)
            .map(|row| {
                let row_text = build_row(&state.text, row);
                Line::from(Span::styled(row_text, style))
            })
            .collect();

        // Vertically center within the available area
        let content_height: u16 = 3;
        let vertical_offset = area.height.saturating_sub(content_height) / 2;

        let render_area = Rect::new(
            area.x,
            area.y + vertical_offset,
            area.width,
            content_height.min(area.height.saturating_sub(vertical_offset)),
        );

        if render_area.height > 0 {
            let paragraph = Paragraph::new(lines).alignment(state.alignment);
            frame.render_widget(paragraph, render_area);
        }
    }
}

impl Disableable for BigText {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod tests;
