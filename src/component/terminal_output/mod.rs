//! A terminal output display component with ANSI color support.
//!
//! [`TerminalOutput`] renders lines of text that may contain ANSI escape
//! sequences, displaying them with the corresponding colors and text
//! attributes. It is suitable for showing command output, build logs,
//! or any stream of terminal-formatted text.
//!
//! State is stored in [`TerminalOutputState`], updated via
//! [`TerminalOutputMessage`], and produces [`TerminalOutputOutput`].
//!
//!
//! # Features
//!
//! - ANSI SGR color rendering (standard, bright, 256-color palette)
//! - Auto-scroll to follow new output
//! - Optional line numbers
//! - Status bar showing running state, exit code, line count
//! - Scrollbar for long output
//! - Max-lines cap to bound memory usage
//!
//! # Example
//!
//! ```rust
//! # #[cfg(feature = "display-components")]
//! # {
//! use envision::component::{
//!     Component, TerminalOutput, TerminalOutputState,
//!     TerminalOutputMessage,
//! };
//!
//! let mut state = TerminalOutputState::new()
//!     .with_title("Build Output")
//!     .with_auto_scroll(true);
//!
//! state.push_line("Compiling envision v0.7.0");
//! state.push_line("\x1b[32m   Finished\x1b[0m in 2.5s");
//!
//! assert_eq!(state.line_count(), 2);
//! assert!(state.auto_scroll());
//! # }
//! ```

pub mod ansi;
mod render;

pub use ansi::{AnsiSegment, parse_ansi};

use ratatui::prelude::*;

use super::{Component, ViewContext};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// Messages that can be sent to a TerminalOutput component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalOutputMessage {
    /// Push a new line of output.
    PushLine(String),
    /// Push multiple lines of output at once.
    PushLines(Vec<String>),
    /// Clear all output lines.
    Clear,
    /// Scroll up by one line.
    ScrollUp,
    /// Scroll down by one line.
    ScrollDown,
    /// Scroll up by a page.
    PageUp(usize),
    /// Scroll down by a page.
    PageDown(usize),
    /// Scroll to the first line.
    Home,
    /// Scroll to the last line.
    End,
    /// Toggle auto-scroll mode.
    ToggleAutoScroll,
    /// Toggle line number display.
    ToggleLineNumbers,
    /// Set the running state.
    SetRunning(bool),
    /// Set the exit code (implies not running).
    SetExitCode(Option<i32>),
}

/// Output messages from a TerminalOutput component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalOutputOutput {
    /// The scroll position changed.
    ScrollChanged(usize),
    /// A line was added (total line count).
    LineAdded(usize),
    /// Lines were cleared.
    Cleared,
    /// Auto-scroll was toggled (new state).
    AutoScrollToggled(bool),
    /// Line numbers were toggled (new state).
    LineNumbersToggled(bool),
}

/// State for a TerminalOutput component.
///
/// Contains output lines, scroll state, and display configuration.
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "display-components")]
/// # {
/// use envision::component::TerminalOutputState;
///
/// let mut state = TerminalOutputState::new()
///     .with_max_lines(5000)
///     .with_title("Output");
///
/// state.push_line("line 1");
/// state.push_line("\x1b[31mred line\x1b[0m");
///
/// assert_eq!(state.line_count(), 2);
/// assert_eq!(state.lines()[0], "line 1");
/// # }
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct TerminalOutputState {
    /// The output lines (may contain ANSI escape sequences).
    lines: Vec<String>,
    /// Maximum number of lines to retain.
    max_lines: usize,
    /// Scroll state for viewport tracking.
    scroll: ScrollState,
    /// Whether to auto-scroll to follow new output.
    auto_scroll: bool,
    /// Whether to display line numbers.
    show_line_numbers: bool,
    /// Optional title for the component border.
    title: Option<String>,
    /// Optional exit code of the process.
    exit_code: Option<i32>,
    /// Whether the process is currently running.
    running: bool,
}

impl Default for TerminalOutputState {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            max_lines: 10_000,
            scroll: ScrollState::new(0),
            auto_scroll: true,
            show_line_numbers: false,
            title: None,
            exit_code: None,
            running: false,
        }
    }
}

impl TerminalOutputState {
    /// Creates a new empty terminal output state.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let state = TerminalOutputState::new();
    /// assert!(state.lines().is_empty());
    /// assert_eq!(state.line_count(), 0);
    /// assert!(state.auto_scroll());
    /// # }
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    // ---- Builder methods ----

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let state = TerminalOutputState::new().with_title("Build Log");
    /// assert_eq!(state.title(), Some("Build Log"));
    /// # }
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the maximum number of lines (builder pattern).
    ///
    /// When this limit is exceeded, the oldest lines are removed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let state = TerminalOutputState::new().with_max_lines(500);
    /// assert_eq!(state.max_lines(), 500);
    /// # }
    /// ```
    pub fn with_max_lines(mut self, max: usize) -> Self {
        self.max_lines = max;
        self
    }

    /// Sets auto-scroll (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let state = TerminalOutputState::new().with_auto_scroll(false);
    /// assert!(!state.auto_scroll());
    /// # }
    /// ```
    pub fn with_auto_scroll(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }

    /// Sets line number display (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let state = TerminalOutputState::new().with_line_numbers(true);
    /// assert!(state.show_line_numbers());
    /// # }
    /// ```
    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Sets the running state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let state = TerminalOutputState::new().with_running(true);
    /// assert!(state.running());
    /// # }
    /// ```
    pub fn with_running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    // ---- Line management ----

    /// Pushes a new line of output.
    ///
    /// If auto-scroll is enabled, the scroll position moves to the end.
    /// If the line count exceeds `max_lines`, the oldest lines are removed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let mut state = TerminalOutputState::new();
    /// state.push_line("hello");
    /// state.push_line("\x1b[31mred text\x1b[0m");
    /// assert_eq!(state.line_count(), 2);
    /// # }
    /// ```
    pub fn push_line(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
        self.enforce_max_lines();
        self.scroll.set_content_length(self.lines.len());
        if self.auto_scroll {
            self.scroll.scroll_to_end();
        }
    }

    /// Pushes multiple lines of output at once.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let mut state = TerminalOutputState::new();
    /// state.push_lines(vec!["line 1".to_string(), "line 2".to_string()]);
    /// assert_eq!(state.line_count(), 2);
    /// # }
    /// ```
    pub fn push_lines(&mut self, lines: Vec<String>) {
        self.lines.extend(lines);
        self.enforce_max_lines();
        self.scroll.set_content_length(self.lines.len());
        if self.auto_scroll {
            self.scroll.scroll_to_end();
        }
    }

    /// Clears all output lines and resets scroll.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let mut state = TerminalOutputState::new();
    /// state.push_line("hello");
    /// state.clear();
    /// assert!(state.lines().is_empty());
    /// # }
    /// ```
    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll = ScrollState::new(0);
    }

    /// Returns all output lines.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let mut state = TerminalOutputState::new();
    /// state.push_line("line 1");
    /// state.push_line("line 2");
    /// assert_eq!(state.lines(), &["line 1", "line 2"]);
    /// # }
    /// ```
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Returns the number of output lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
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

    // ---- Configuration accessors ----

    /// Returns the maximum number of lines.
    pub fn max_lines(&self) -> usize {
        self.max_lines
    }

    /// Sets the maximum number of lines.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let mut state = TerminalOutputState::new();
    /// state.set_max_lines(500);
    /// assert_eq!(state.max_lines(), 500);
    /// # }
    /// ```
    pub fn set_max_lines(&mut self, max: usize) {
        self.max_lines = max;
        self.enforce_max_lines();
    }

    /// Returns whether auto-scroll is enabled.
    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    /// Sets the auto-scroll state.
    pub fn set_auto_scroll(&mut self, auto_scroll: bool) {
        self.auto_scroll = auto_scroll;
    }

    /// Returns whether line numbers are shown.
    pub fn show_line_numbers(&self) -> bool {
        self.show_line_numbers
    }

    /// Sets whether to show line numbers.
    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    /// Returns the title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns the exit code, if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let mut state = TerminalOutputState::new();
    /// assert_eq!(state.exit_code(), None);
    /// state.set_exit_code(Some(0));
    /// assert_eq!(state.exit_code(), Some(0));
    /// # }
    /// ```
    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Sets the exit code.
    ///
    /// Setting an exit code also sets `running` to `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let mut state = TerminalOutputState::new().with_running(true);
    /// state.set_exit_code(Some(0));
    /// assert_eq!(state.exit_code(), Some(0));
    /// assert!(!state.running());
    /// # }
    /// ```
    pub fn set_exit_code(&mut self, code: Option<i32>) {
        self.exit_code = code;
        if code.is_some() {
            self.running = false;
        }
    }

    /// Returns whether the process is running.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::TerminalOutputState;
    ///
    /// let state = TerminalOutputState::new();
    /// assert!(!state.running());
    ///
    /// let state = TerminalOutputState::new().with_running(true);
    /// assert!(state.running());
    /// # }
    /// ```
    pub fn running(&self) -> bool {
        self.running
    }

    /// Sets the running state.
    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    // ---- State accessors ----

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "display-components")]
    /// # {
    /// use envision::component::{TerminalOutputMessage, TerminalOutputOutput, TerminalOutputState};
    ///
    /// let mut state = TerminalOutputState::new();
    /// let output = state.update(TerminalOutputMessage::PushLine("hello".to_string()));
    /// assert_eq!(output, Some(TerminalOutputOutput::LineAdded(1)));
    /// # }
    /// ```
    pub fn update(&mut self, msg: TerminalOutputMessage) -> Option<TerminalOutputOutput> {
        TerminalOutput::update(self, msg)
    }

    // ---- Internal ----

    /// Removes oldest lines when the count exceeds max_lines.
    fn enforce_max_lines(&mut self) {
        if self.lines.len() > self.max_lines {
            let excess = self.lines.len() - self.max_lines;
            self.lines.drain(..excess);
            self.scroll.set_content_length(self.lines.len());
        }
    }
}

/// A terminal output display component with ANSI color support.
///
/// Displays lines of terminal output with ANSI escape sequence rendering.
/// Supports scrolling, auto-scroll, line numbers, and a status bar.
///
/// # Key Bindings
///
/// - `Up` / `k` — Scroll up one line
/// - `Down` / `j` — Scroll down one line
/// - `PageUp` / `Ctrl+u` — Scroll up half a page
/// - `PageDown` / `Ctrl+d` — Scroll down half a page
/// - `Home` / `g` — Scroll to top
/// - `End` / `G` — Scroll to bottom
/// - `a` — Toggle auto-scroll
/// - `n` — Toggle line numbers
pub struct TerminalOutput;

impl Component for TerminalOutput {
    type State = TerminalOutputState;
    type Message = TerminalOutputMessage;
    type Output = TerminalOutputOutput;

    fn init() -> Self::State {
        TerminalOutputState::default()
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            KeyCode::Up | KeyCode::Char('k') if !ctrl => Some(TerminalOutputMessage::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') if !ctrl => Some(TerminalOutputMessage::ScrollDown),
            KeyCode::PageUp => Some(TerminalOutputMessage::PageUp(10)),
            KeyCode::PageDown => Some(TerminalOutputMessage::PageDown(10)),
            KeyCode::Char('u') if ctrl => Some(TerminalOutputMessage::PageUp(10)),
            KeyCode::Char('d') if ctrl => Some(TerminalOutputMessage::PageDown(10)),
            KeyCode::Home | KeyCode::Char('g') if !shift => Some(TerminalOutputMessage::Home),
            KeyCode::End | KeyCode::Char('G') if shift || key.code == KeyCode::End => {
                Some(TerminalOutputMessage::End)
            }
            KeyCode::Char('a') if !ctrl => Some(TerminalOutputMessage::ToggleAutoScroll),
            KeyCode::Char('n') if !ctrl => Some(TerminalOutputMessage::ToggleLineNumbers),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            TerminalOutputMessage::PushLine(line) => {
                state.push_line(line);
                Some(TerminalOutputOutput::LineAdded(state.lines.len()))
            }
            TerminalOutputMessage::PushLines(lines) => {
                let count = lines.len();
                state.push_lines(lines);
                if count > 0 {
                    Some(TerminalOutputOutput::LineAdded(state.lines.len()))
                } else {
                    None
                }
            }
            TerminalOutputMessage::Clear => {
                if state.lines.is_empty() {
                    None
                } else {
                    state.clear();
                    Some(TerminalOutputOutput::Cleared)
                }
            }
            TerminalOutputMessage::ScrollUp => {
                if state.auto_scroll {
                    state.auto_scroll = false;
                }
                if state.scroll.scroll_up() {
                    Some(TerminalOutputOutput::ScrollChanged(state.scroll.offset()))
                } else {
                    None
                }
            }
            TerminalOutputMessage::ScrollDown => {
                if state.scroll.scroll_down() {
                    Some(TerminalOutputOutput::ScrollChanged(state.scroll.offset()))
                } else {
                    None
                }
            }
            TerminalOutputMessage::PageUp(n) => {
                if state.auto_scroll {
                    state.auto_scroll = false;
                }
                if state.scroll.page_up(n) {
                    Some(TerminalOutputOutput::ScrollChanged(state.scroll.offset()))
                } else {
                    None
                }
            }
            TerminalOutputMessage::PageDown(n) => {
                if state.scroll.page_down(n) {
                    Some(TerminalOutputOutput::ScrollChanged(state.scroll.offset()))
                } else {
                    None
                }
            }
            TerminalOutputMessage::Home => {
                if state.auto_scroll {
                    state.auto_scroll = false;
                }
                if state.scroll.scroll_to_start() {
                    Some(TerminalOutputOutput::ScrollChanged(0))
                } else {
                    None
                }
            }
            TerminalOutputMessage::End => {
                if state.scroll.scroll_to_end() {
                    Some(TerminalOutputOutput::ScrollChanged(state.scroll.offset()))
                } else {
                    None
                }
            }
            TerminalOutputMessage::ToggleAutoScroll => {
                state.auto_scroll = !state.auto_scroll;
                if state.auto_scroll {
                    state.scroll.set_content_length(state.lines.len());
                    state.scroll.scroll_to_end();
                }
                Some(TerminalOutputOutput::AutoScrollToggled(state.auto_scroll))
            }
            TerminalOutputMessage::ToggleLineNumbers => {
                state.show_line_numbers = !state.show_line_numbers;
                Some(TerminalOutputOutput::LineNumbersToggled(
                    state.show_line_numbers,
                ))
            }
            TerminalOutputMessage::SetRunning(running) => {
                state.running = running;
                None
            }
            TerminalOutputMessage::SetExitCode(code) => {
                state.exit_code = code;
                if code.is_some() {
                    state.running = false;
                }
                None
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        render::render(state, frame, area, theme, ctx.focused, ctx.disabled);
    }
}

#[cfg(test)]
mod tests;
