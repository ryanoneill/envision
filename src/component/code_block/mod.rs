//! A code block display component with syntax highlighting.
//!
//! [`CodeBlock`] provides a read-only, scrollable code viewer with
//! keyword-based syntax highlighting for common programming languages.
//! It supports line numbers, line highlighting, and a title bar.
//! State is stored in [`CodeBlockState`], updated via
//! [`CodeBlockMessage`], and produces no output (unit type).
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! # Supported Languages
//!
//! Rust, Python, JavaScript, TypeScript, Go, Shell, JSON, YAML, TOML,
//! SQL, and Plain text. See [`Language`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Focusable, CodeBlock, CodeBlockState,
//!     CodeBlockMessage,
//! };
//! use envision::component::code_block::highlight::Language;
//!
//! let mut state = CodeBlockState::new()
//!     .with_code("fn main() {\n    println!(\"Hello, world!\");\n}")
//!     .with_language(Language::Rust)
//!     .with_title("main.rs")
//!     .with_line_numbers(true);
//!
//! assert_eq!(state.code(), "fn main() {\n    println!(\"Hello, world!\");\n}");
//! assert_eq!(state.language(), &Language::Rust);
//! assert_eq!(state.title(), Some("main.rs"));
//! assert!(state.show_line_numbers());
//! ```

pub mod highlight;
mod render;

use std::collections::HashSet;

use ratatui::prelude::*;

use self::highlight::Language;
use super::{Component, Disableable, Focusable, ViewContext};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// Messages that can be sent to a CodeBlock.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum CodeBlockMessage {
    /// Scroll up by one line.
    ScrollUp,
    /// Scroll down by one line.
    ScrollDown,
    /// Scroll up by a page (given number of lines).
    PageUp(usize),
    /// Scroll down by a page (given number of lines).
    PageDown(usize),
    /// Scroll to the top.
    Home,
    /// Scroll to the bottom.
    End,
    /// Replace the code content.
    SetCode(String),
    /// Set the language for syntax highlighting.
    SetLanguage(Language),
    /// Toggle line number display.
    ToggleLineNumbers,
    /// Add a highlighted line (1-based).
    HighlightLine(usize),
    /// Remove a highlighted line (1-based).
    UnhighlightLine(usize),
    /// Clear all highlighted lines.
    ClearHighlights,
}

/// State for a CodeBlock component.
///
/// Contains the source code, language selection, scroll position, and
/// display options.
#[derive(Clone, Debug, Default)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct CodeBlockState {
    /// The source code content.
    pub(crate) code: String,
    /// The language for syntax highlighting.
    pub(crate) language: Language,
    /// Scroll state tracking offset and providing scrollbar support.
    pub(crate) scroll: ScrollState,
    /// Whether to show line numbers.
    pub(crate) show_line_numbers: bool,
    /// Set of 1-based line numbers to highlight.
    pub(crate) highlight_lines: HashSet<usize>,
    /// Optional title for the border.
    pub(crate) title: Option<String>,
    /// Whether the component is focused.
    pub(crate) focused: bool,
    /// Whether the component is disabled.
    pub(crate) disabled: bool,
}

impl PartialEq for CodeBlockState {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
            && self.language == other.language
            && self.scroll == other.scroll
            && self.show_line_numbers == other.show_line_numbers
            && self.highlight_lines == other.highlight_lines
            && self.title == other.title
            && self.focused == other.focused
            && self.disabled == other.disabled
    }
}

impl CodeBlockState {
    /// Creates a new empty code block state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new();
    /// assert!(state.code().is_empty());
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the initial code content (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new()
    ///     .with_code("fn main() {}");
    /// assert_eq!(state.code(), "fn main() {}");
    /// ```
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self.scroll
            .set_content_length(self.code.lines().count().max(1));
        self
    }

    /// Sets the language (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    /// use envision::component::code_block::highlight::Language;
    ///
    /// let state = CodeBlockState::new()
    ///     .with_language(Language::Rust);
    /// assert_eq!(state.language(), &Language::Rust);
    /// ```
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new()
    ///     .with_title("main.rs");
    /// assert_eq!(state.title(), Some("main.rs"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether line numbers are shown (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new()
    ///     .with_line_numbers(true);
    /// assert!(state.show_line_numbers());
    /// ```
    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Sets the highlight lines (builder pattern).
    ///
    /// Lines are 1-based.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new()
    ///     .with_highlight_lines(vec![1, 3, 5]);
    /// assert!(state.is_line_highlighted(1));
    /// assert!(!state.is_line_highlighted(2));
    /// assert!(state.is_line_highlighted(3));
    /// ```
    pub fn with_highlight_lines(mut self, lines: Vec<usize>) -> Self {
        self.highlight_lines = lines.into_iter().collect();
        self
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new().with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // ---- Code accessors ----

    /// Returns the source code content.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Sets the code content.
    ///
    /// Resets the scroll offset to 0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let mut state = CodeBlockState::new();
    /// state.set_code("let x = 1;");
    /// assert_eq!(state.code(), "let x = 1;");
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn set_code(&mut self, code: impl Into<String>) {
        self.code = code.into();
        self.scroll = ScrollState::new(self.code.lines().count().max(1));
    }

    /// Returns the number of lines in the code.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new().with_code("a\nb\nc");
    /// assert_eq!(state.line_count(), 3);
    /// ```
    pub fn line_count(&self) -> usize {
        self.code.lines().count().max(1)
    }

    // ---- Language accessors ----

    /// Returns the current language.
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// Sets the language.
    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }

    // ---- Title accessors ----

    /// Returns the title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    // ---- Line number accessors ----

    /// Returns whether line numbers are shown.
    pub fn show_line_numbers(&self) -> bool {
        self.show_line_numbers
    }

    /// Sets whether line numbers are shown.
    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    // ---- Highlight line accessors ----

    /// Returns true if the given line (1-based) is highlighted.
    pub fn is_line_highlighted(&self, line: usize) -> bool {
        self.highlight_lines.contains(&line)
    }

    /// Adds a highlighted line (1-based).
    pub fn add_highlight_line(&mut self, line: usize) {
        self.highlight_lines.insert(line);
    }

    /// Removes a highlighted line (1-based).
    pub fn remove_highlight_line(&mut self, line: usize) {
        self.highlight_lines.remove(&line);
    }

    /// Clears all highlighted lines.
    pub fn clear_highlights(&mut self) {
        self.highlight_lines.clear();
    }

    /// Returns the set of highlighted line numbers.
    pub fn highlighted_lines(&self) -> &HashSet<usize> {
        &self.highlight_lines
    }

    // ---- Scroll accessors ----

    /// Returns the current scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    /// Sets the scroll offset.
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll.set_offset(offset);
    }

    // ---- State accessors ----

    /// Returns true if the component is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    // ---- Instance methods ----

    /// Maps an input event to a code block message.
    pub fn handle_event(&self, event: &Event) -> Option<CodeBlockMessage> {
        CodeBlock::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<()> {
        CodeBlock::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CodeBlockMessage, CodeBlockState};
    ///
    /// let mut state = CodeBlockState::new()
    ///     .with_code("Line 1\nLine 2\nLine 3");
    /// state.update(CodeBlockMessage::ScrollDown);
    /// assert_eq!(state.scroll_offset(), 1);
    /// ```
    pub fn update(&mut self, msg: CodeBlockMessage) -> Option<()> {
        CodeBlock::update(self, msg)
    }
}

/// A code block display component with syntax highlighting.
///
/// Displays source code with keyword-based syntax colouring, optional
/// line numbers, and scroll support. Content is read-only.
///
/// # Key Bindings
///
/// - `Up` / `k` -- Scroll up one line
/// - `Down` / `j` -- Scroll down one line
/// - `PageUp` / `Ctrl+u` -- Scroll up half a page
/// - `PageDown` / `Ctrl+d` -- Scroll down half a page
/// - `Home` / `g` -- Scroll to top
/// - `End` / `G` -- Scroll to bottom
/// - `l` -- Toggle line numbers
pub struct CodeBlock;

impl Component for CodeBlock {
    type State = CodeBlockState;
    type Message = CodeBlockMessage;
    type Output = ();

    fn init() -> Self::State {
        CodeBlockState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        let key = event.as_key()?;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            KeyCode::Up | KeyCode::Char('k') if !ctrl => Some(CodeBlockMessage::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') if !ctrl => Some(CodeBlockMessage::ScrollDown),
            KeyCode::PageUp => Some(CodeBlockMessage::PageUp(10)),
            KeyCode::PageDown => Some(CodeBlockMessage::PageDown(10)),
            KeyCode::Char('u') if ctrl => Some(CodeBlockMessage::PageUp(10)),
            KeyCode::Char('d') if ctrl => Some(CodeBlockMessage::PageDown(10)),
            KeyCode::Home | KeyCode::Char('g') if !shift => Some(CodeBlockMessage::Home),
            KeyCode::End | KeyCode::Char('G') if shift || key.code == KeyCode::End => {
                Some(CodeBlockMessage::End)
            }
            KeyCode::Char('l') if !ctrl => Some(CodeBlockMessage::ToggleLineNumbers),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            CodeBlockMessage::ScrollUp => {
                state.scroll.scroll_up();
            }
            CodeBlockMessage::ScrollDown => {
                state.scroll.scroll_down();
            }
            CodeBlockMessage::PageUp(n) => {
                state.scroll.page_up(n);
            }
            CodeBlockMessage::PageDown(n) => {
                state.scroll.page_down(n);
            }
            CodeBlockMessage::Home => {
                state.scroll.scroll_to_start();
            }
            CodeBlockMessage::End => {
                state.scroll.scroll_to_end();
            }
            CodeBlockMessage::SetCode(code) => {
                state.code = code;
                state.scroll = ScrollState::new(state.code.lines().count().max(1));
            }
            CodeBlockMessage::SetLanguage(lang) => {
                state.language = lang;
            }
            CodeBlockMessage::ToggleLineNumbers => {
                state.show_line_numbers = !state.show_line_numbers;
            }
            CodeBlockMessage::HighlightLine(line) => {
                state.highlight_lines.insert(line);
            }
            CodeBlockMessage::UnhighlightLine(line) => {
                state.highlight_lines.remove(&line);
            }
            CodeBlockMessage::ClearHighlights => {
                state.highlight_lines.clear();
            }
        }
        None
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        render::render(state, frame, area, theme, ctx.focused, ctx.disabled);
    }
}

impl Focusable for CodeBlock {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for CodeBlock {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod tests;
