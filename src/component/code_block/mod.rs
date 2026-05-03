//! A code block display component with syntax highlighting.
//!
//! [`CodeBlock`] provides a read-only, scrollable code viewer with
//! keyword-based syntax highlighting for common programming languages.
//! It supports line numbers, line highlighting, and a title bar.
//! State is stored in [`CodeBlockState`], updated via
//! [`CodeBlockMessage`], and produces no output (unit type).
//!
//!
//! # Supported Languages
//!
//! Rust, Python, JavaScript, TypeScript, Go, Shell, JSON, YAML, TOML,
//! SQL, HCL, and Plain text. See [`Language`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, CodeBlock, CodeBlockState,
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

pub use self::highlight::Language;
use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};
use crate::scroll::ScrollState;

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
    /// Scroll left by one column.
    ScrollLeft,
    /// Scroll right by one column.
    ScrollRight,
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
    /// Horizontal scroll offset in characters.
    pub(crate) horizontal_offset: usize,
    /// Whether to show line numbers.
    pub(crate) show_line_numbers: bool,
    /// Set of 1-based line numbers to highlight.
    pub(crate) highlight_lines: HashSet<usize>,
    /// Optional title for the border.
    pub(crate) title: Option<String>,
}

impl PartialEq for CodeBlockState {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
            && self.language == other.language
            && self.scroll == other.scroll
            && self.horizontal_offset == other.horizontal_offset
            && self.show_line_numbers == other.show_line_numbers
            && self.highlight_lines == other.highlight_lines
            && self.title == other.title
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

    // ---- Code accessors ----

    /// Returns the source code content.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new().with_code("fn main() {}");
    /// assert_eq!(state.code(), "fn main() {}");
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    /// use envision::component::code_block::highlight::Language;
    ///
    /// let state = CodeBlockState::new().with_language(Language::Python);
    /// assert_eq!(state.language(), &Language::Python);
    /// ```
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// Sets the language.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    /// use envision::component::code_block::highlight::Language;
    ///
    /// let mut state = CodeBlockState::new();
    /// state.set_language(Language::Go);
    /// assert_eq!(state.language(), &Language::Go);
    /// ```
    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }

    // ---- Title accessors ----

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new().with_title("server.rs");
    /// assert_eq!(state.title(), Some("server.rs"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let mut state = CodeBlockState::new();
    /// state.set_title(Some("app.rs".to_string()));
    /// assert_eq!(state.title(), Some("app.rs"));
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    // ---- Line number accessors ----

    /// Returns whether line numbers are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new().with_line_numbers(true);
    /// assert!(state.show_line_numbers());
    /// ```
    pub fn show_line_numbers(&self) -> bool {
        self.show_line_numbers
    }

    /// Sets whether line numbers are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let mut state = CodeBlockState::new();
    /// state.set_show_line_numbers(true);
    /// assert!(state.show_line_numbers());
    /// ```
    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    // ---- Highlight line accessors ----

    /// Returns true if the given line (1-based) is highlighted.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new().with_highlight_lines(vec![2, 4]);
    /// assert!(state.is_line_highlighted(2));
    /// assert!(!state.is_line_highlighted(3));
    /// ```
    pub fn is_line_highlighted(&self, line: usize) -> bool {
        self.highlight_lines.contains(&line)
    }

    /// Adds a highlighted line (1-based).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let mut state = CodeBlockState::new();
    /// state.add_highlight_line(3);
    /// assert!(state.is_line_highlighted(3));
    /// ```
    pub fn add_highlight_line(&mut self, line: usize) {
        self.highlight_lines.insert(line);
    }

    /// Removes a highlighted line (1-based).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let mut state = CodeBlockState::new().with_highlight_lines(vec![1, 2, 3]);
    /// state.remove_highlight_line(2);
    /// assert!(!state.is_line_highlighted(2));
    /// assert!(state.is_line_highlighted(1));
    /// ```
    pub fn remove_highlight_line(&mut self, line: usize) {
        self.highlight_lines.remove(&line);
    }

    /// Clears all highlighted lines.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let mut state = CodeBlockState::new().with_highlight_lines(vec![1, 2, 3]);
    /// state.clear_highlights();
    /// assert!(state.highlighted_lines().is_empty());
    /// ```
    pub fn clear_highlights(&mut self) {
        self.highlight_lines.clear();
    }

    /// Returns the set of highlighted line numbers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new().with_highlight_lines(vec![1, 3]);
    /// assert_eq!(state.highlighted_lines().len(), 2);
    /// assert!(state.highlighted_lines().contains(&1));
    /// ```
    pub fn highlighted_lines(&self) -> &HashSet<usize> {
        &self.highlight_lines
    }

    // ---- Scroll accessors ----

    /// Returns the current scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new();
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    /// Sets the scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let mut state = CodeBlockState::new()
    ///     .with_code("line1\nline2\nline3\nline4\nline5");
    /// state.set_scroll_offset(2);
    /// assert_eq!(state.scroll_offset(), 2);
    /// ```
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll.set_offset(offset);
    }

    /// Returns the horizontal scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let state = CodeBlockState::new();
    /// assert_eq!(state.horizontal_offset(), 0);
    /// ```
    pub fn horizontal_offset(&self) -> usize {
        self.horizontal_offset
    }

    /// Sets the horizontal scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CodeBlockState;
    ///
    /// let mut state = CodeBlockState::new();
    /// state.set_horizontal_offset(10);
    /// assert_eq!(state.horizontal_offset(), 10);
    /// ```
    pub fn set_horizontal_offset(&mut self, offset: usize) {
        self.horizontal_offset = offset;
    }

    // ---- State accessors ----

    // ---- Instance methods ----

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
/// - `Left` / `h` -- Scroll left one column
/// - `Right` / `l` -- Scroll right one column
/// - `PageUp` / `Ctrl+u` -- Scroll up half a page
/// - `PageDown` / `Ctrl+d` -- Scroll down half a page
/// - `Home` / `g` -- Scroll to top
/// - `End` / `G` -- Scroll to bottom
/// - `n` -- Toggle line numbers
pub struct CodeBlock;

impl Component for CodeBlock {
    type State = CodeBlockState;
    type Message = CodeBlockMessage;
    type Output = ();

    fn init() -> Self::State {
        CodeBlockState::default()
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;
        let ctrl = key.modifiers.ctrl();

        match key.code {
            Key::Up | Key::Char('k') if !ctrl => Some(CodeBlockMessage::ScrollUp),
            Key::Down | Key::Char('j') if !ctrl => Some(CodeBlockMessage::ScrollDown),
            Key::Left | Key::Char('h') if !ctrl => Some(CodeBlockMessage::ScrollLeft),
            Key::Right | Key::Char('l') if !ctrl => Some(CodeBlockMessage::ScrollRight),
            Key::PageUp => Some(CodeBlockMessage::PageUp(10)),
            Key::PageDown => Some(CodeBlockMessage::PageDown(10)),
            Key::Char('u') if ctrl => Some(CodeBlockMessage::PageUp(10)),
            Key::Char('d') if ctrl => Some(CodeBlockMessage::PageDown(10)),
            Key::Char('g') if key.modifiers.shift() => Some(CodeBlockMessage::End),
            Key::Home | Key::Char('g') => Some(CodeBlockMessage::Home),
            Key::End => Some(CodeBlockMessage::End),
            Key::Char('n') if !ctrl => Some(CodeBlockMessage::ToggleLineNumbers),
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
            CodeBlockMessage::ScrollLeft => {
                state.horizontal_offset = state.horizontal_offset.saturating_sub(1);
            }
            CodeBlockMessage::ScrollRight => {
                // Clamp to max line width (computed lazily)
                let max_width = state.code.lines().map(|l| l.len()).max().unwrap_or(0);
                if state.horizontal_offset < max_width {
                    state.horizontal_offset += 1;
                }
            }
            CodeBlockMessage::Home => {
                state.scroll.scroll_to_start();
                state.horizontal_offset = 0;
            }
            CodeBlockMessage::End => {
                state.scroll.scroll_to_end();
            }
            CodeBlockMessage::SetCode(code) => {
                state.code = code;
                state.scroll = ScrollState::new(state.code.lines().count().max(1));
                state.horizontal_offset = 0;
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

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        render::render(
            state,
            ctx.frame,
            ctx.area,
            ctx.theme,
            ctx.focused,
            ctx.disabled,
            ctx.chrome_owned,
        );
    }
}

#[cfg(test)]
mod tests;
