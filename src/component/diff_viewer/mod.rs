//! A diff viewer component for unified and side-by-side diff display.
//!
//! [`DiffViewer`] displays text diffs with hunk navigation, line-level
//! highlighting, and support for both unified and side-by-side display
//! modes. State is stored in [`DiffViewerState`], updated via
//! [`DiffViewerMessage`], and produces [`DiffViewerOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! # Constructing a Diff
//!
//! There are three ways to provide diff data:
//!
//! - **From unified diff text**: Parse standard `diff -u` output.
//! - **From two text strings**: Compute a line-based diff using LCS.
//! - **From pre-built hunks**: Provide `Vec<DiffHunk>` directly.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Focusable, DiffViewer, DiffViewerState,
//!     DiffViewerMessage, DiffMode,
//! };
//!
//! let old = "fn main() {\n    println!(\"hello\");\n}";
//! let new = "fn main() {\n    println!(\"world\");\n    return;\n}";
//!
//! let mut state = DiffViewerState::from_texts(old, new)
//!     .with_title("Changes")
//!     .with_mode(DiffMode::Unified);
//!
//! assert_eq!(state.hunk_count(), 1);
//! assert_eq!(state.added_count(), 2);
//! assert_eq!(state.removed_count(), 1);
//! ```

pub mod parser;
mod render;

use ratatui::prelude::*;

use super::{Component, Disableable, Focusable};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// Display mode for the diff.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum DiffMode {
    /// Show old and new lines interleaved with `+`/`-` markers.
    #[default]
    Unified,
    /// Show old and new files in two columns.
    SideBySide,
}

/// Type of a diff line.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum DiffLineType {
    /// Unchanged line present in both old and new.
    Context,
    /// Line added in the new version (green).
    Added,
    /// Line removed from the old version (red).
    Removed,
    /// Hunk header (`@@ ... @@`).
    Header,
}

/// A single line in the diff.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DiffLine {
    /// The type of this diff line.
    pub line_type: DiffLineType,
    /// The text content of this line (without the leading `+`/`-`/` ` prefix).
    pub content: String,
    /// Line number in the old file, if applicable.
    pub old_line_num: Option<usize>,
    /// Line number in the new file, if applicable.
    pub new_line_num: Option<usize>,
}

/// A hunk (section of changes) in the diff.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DiffHunk {
    /// The hunk header, e.g. `@@ -10,5 +10,7 @@`.
    pub header: String,
    /// Starting line number in the old file.
    pub old_start: usize,
    /// Starting line number in the new file.
    pub new_start: usize,
    /// The lines in this hunk (including the header as the first line).
    pub lines: Vec<DiffLine>,
}

/// Messages that can be sent to a DiffViewer.
#[derive(Clone, Debug, PartialEq)]
pub enum DiffViewerMessage {
    /// Set the diff from unified diff text.
    SetDiff(String),
    /// Compute a diff from two text strings.
    SetTexts {
        /// The old (original) text.
        old: String,
        /// The new (modified) text.
        new: String,
    },
    /// Set pre-built hunks directly.
    SetHunks(Vec<DiffHunk>),
    /// Clear the diff.
    Clear,
    /// Jump to the next hunk.
    NextHunk,
    /// Jump to the previous hunk.
    PrevHunk,
    /// Scroll up by one line.
    ScrollUp,
    /// Scroll down by one line.
    ScrollDown,
    /// Scroll up by the given number of lines.
    PageUp(usize),
    /// Scroll down by the given number of lines.
    PageDown(usize),
    /// Scroll to the top.
    Home,
    /// Scroll to the bottom.
    End,
    /// Toggle between unified and side-by-side mode.
    ToggleMode,
    /// Set the display mode directly.
    SetMode(DiffMode),
}

impl Eq for DiffViewerMessage {}

/// Output messages from a DiffViewer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiffViewerOutput {
    /// The current hunk index changed.
    HunkChanged(usize),
    /// The display mode changed.
    ModeChanged(DiffMode),
}

/// State for a DiffViewer component.
///
/// Contains the diff hunks, display mode, scroll position, and UI options.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DiffViewerState {
    /// The diff hunks.
    hunks: Vec<DiffHunk>,
    /// Display mode (unified or side-by-side).
    mode: DiffMode,
    /// Scroll state for vertical scrolling.
    pub(crate) scroll: ScrollState,
    /// Index of the currently-focused hunk.
    current_hunk: usize,
    /// Number of context lines for compute_diff.
    context_lines: usize,
    /// Whether to show line numbers.
    pub(crate) show_line_numbers: bool,
    /// Optional title for the component border.
    pub(crate) title: Option<String>,
    /// Label for the old file side.
    pub(crate) old_label: Option<String>,
    /// Label for the new file side.
    pub(crate) new_label: Option<String>,
    /// Whether the component is focused.
    pub(crate) focused: bool,
    /// Whether the component is disabled.
    pub(crate) disabled: bool,
}

impl Default for DiffViewerState {
    fn default() -> Self {
        Self {
            hunks: Vec::new(),
            mode: DiffMode::default(),
            scroll: ScrollState::default(),
            current_hunk: 0,
            context_lines: 3,
            show_line_numbers: true,
            title: None,
            old_label: None,
            new_label: None,
            focused: false,
            disabled: false,
        }
    }
}

impl PartialEq for DiffViewerState {
    fn eq(&self, other: &Self) -> bool {
        self.hunks == other.hunks
            && self.mode == other.mode
            && self.scroll == other.scroll
            && self.current_hunk == other.current_hunk
            && self.context_lines == other.context_lines
            && self.show_line_numbers == other.show_line_numbers
            && self.title == other.title
            && self.old_label == other.old_label
            && self.new_label == other.new_label
            && self.focused == other.focused
            && self.disabled == other.disabled
    }
}

impl DiffViewerState {
    /// Creates a new empty diff viewer state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::new();
    /// assert!(state.hunks().is_empty());
    /// assert_eq!(state.current_hunk(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a diff viewer from a unified diff string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let diff = "\
    /// --- a/file.rs
    /// +++ b/file.rs
    /// @@ -1,3 +1,3 @@
    ///  fn main() {
    /// -    old();
    /// +    new();
    ///  }
    /// ";
    /// let state = DiffViewerState::from_diff(diff);
    /// assert_eq!(state.hunk_count(), 1);
    /// ```
    pub fn from_diff(diff_text: &str) -> Self {
        let hunks = parser::parse_unified_diff(diff_text);
        let total_lines = count_total_lines(&hunks);
        Self {
            hunks,
            scroll: ScrollState::new(total_lines),
            ..Default::default()
        }
    }

    /// Creates a diff viewer by computing a diff from two text strings.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::from_texts("old line", "new line");
    /// assert_eq!(state.hunk_count(), 1);
    /// ```
    pub fn from_texts(old: &str, new: &str) -> Self {
        let context_lines = 3;
        let hunks = parser::compute_diff(old, new, context_lines);
        let total_lines = count_total_lines(&hunks);
        Self {
            hunks,
            context_lines,
            scroll: ScrollState::new(total_lines),
            ..Default::default()
        }
    }

    // ---- Builder methods ----

    /// Sets the display mode (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DiffViewerState, DiffMode};
    ///
    /// let state = DiffViewerState::new().with_mode(DiffMode::SideBySide);
    /// ```
    pub fn with_mode(mut self, mode: DiffMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the number of context lines for compute_diff (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::new().with_context_lines(5);
    /// ```
    pub fn with_context_lines(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
    }

    /// Sets whether to show line numbers (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::new().with_show_line_numbers(false);
    /// ```
    pub fn with_show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::new().with_title("My Diff");
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the label for the old file (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::new().with_old_label("original.rs");
    /// ```
    pub fn with_old_label(mut self, label: impl Into<String>) -> Self {
        self.old_label = Some(label.into());
        self
    }

    /// Sets the label for the new file (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::new().with_new_label("modified.rs");
    /// ```
    pub fn with_new_label(mut self, label: impl Into<String>) -> Self {
        self.new_label = Some(label.into());
        self
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::new().with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // ---- Public accessors ----

    /// Returns the title, if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::new().with_title("My Diff");
    /// assert_eq!(state.title(), Some("My Diff"));
    ///
    /// let state2 = DiffViewerState::new();
    /// assert_eq!(state2.title(), None);
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let mut state = DiffViewerState::new();
    /// state.set_title("Code Review");
    /// assert_eq!(state.title(), Some("Code Review"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Returns a reference to the diff hunks.
    pub fn hunks(&self) -> &[DiffHunk] {
        &self.hunks
    }

    /// Returns the number of hunks in the diff.
    pub fn hunk_count(&self) -> usize {
        self.hunks.len()
    }

    /// Returns the current hunk index.
    pub fn current_hunk(&self) -> usize {
        self.current_hunk
    }

    /// Returns the total number of displayable lines across all hunks.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::from_texts("old line", "new line");
    /// assert!(state.total_lines() > 0);
    /// ```
    pub fn total_lines(&self) -> usize {
        count_total_lines(&self.hunks)
    }

    /// Returns the count of added lines across all hunks.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::from_texts("old", "new\nextra");
    /// assert!(state.added_count() >= 1);
    /// ```
    pub fn added_count(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| l.line_type == DiffLineType::Added)
            .count()
    }

    /// Returns the count of removed lines across all hunks.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::from_texts("old\nextra", "new");
    /// assert!(state.removed_count() >= 1);
    /// ```
    pub fn removed_count(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| l.line_type == DiffLineType::Removed)
            .count()
    }

    /// Returns the count of changed lines (added + removed).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::from_texts("old line", "new line");
    /// assert_eq!(state.changed_count(), state.added_count() + state.removed_count());
    /// ```
    pub fn changed_count(&self) -> usize {
        self.added_count() + self.removed_count()
    }

    /// Returns whether line numbers are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::new();
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
    /// use envision::component::DiffViewerState;
    ///
    /// let mut state = DiffViewerState::new();
    /// state.set_show_line_numbers(false);
    /// assert!(!state.show_line_numbers());
    /// ```
    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    /// Returns the number of context lines.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::new();
    /// assert_eq!(state.context_lines(), 3);
    /// ```
    pub fn context_lines(&self) -> usize {
        self.context_lines
    }

    /// Returns the old label, if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::new().with_old_label("original.rs");
    /// assert_eq!(state.old_label(), Some("original.rs"));
    /// ```
    pub fn old_label(&self) -> Option<&str> {
        self.old_label.as_deref()
    }

    /// Sets the old label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let mut state = DiffViewerState::new();
    /// state.set_old_label("original.rs");
    /// assert_eq!(state.old_label(), Some("original.rs"));
    /// ```
    pub fn set_old_label(&mut self, label: impl Into<String>) {
        self.old_label = Some(label.into());
    }

    /// Returns the new label, if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let state = DiffViewerState::new().with_new_label("modified.rs");
    /// assert_eq!(state.new_label(), Some("modified.rs"));
    /// ```
    pub fn new_label(&self) -> Option<&str> {
        self.new_label.as_deref()
    }

    /// Sets the new label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DiffViewerState;
    ///
    /// let mut state = DiffViewerState::new();
    /// state.set_new_label("modified.rs");
    /// assert_eq!(state.new_label(), Some("modified.rs"));
    /// ```
    pub fn set_new_label(&mut self, label: impl Into<String>) {
        self.new_label = Some(label.into());
    }

    /// Returns the display mode.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DiffViewerState, DiffMode};
    ///
    /// let state = DiffViewerState::new();
    /// assert_eq!(state.mode(), &DiffMode::Unified);
    ///
    /// let state = DiffViewerState::new().with_mode(DiffMode::SideBySide);
    /// assert_eq!(state.mode(), &DiffMode::SideBySide);
    /// ```
    pub fn mode(&self) -> &DiffMode {
        &self.mode
    }

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

    /// Returns the current scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    // ---- Instance methods ----

    /// Maps an input event to a diff viewer message.
    pub fn handle_event(&self, event: &Event) -> Option<DiffViewerMessage> {
        DiffViewer::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<DiffViewerOutput> {
        DiffViewer::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DiffViewerState, DiffViewerMessage};
    ///
    /// let old = "line1\nline2\nline3";
    /// let new = "line1\nchanged\nline3";
    /// let mut state = DiffViewerState::from_texts(old, new);
    /// let _output = state.update(DiffViewerMessage::ScrollDown);
    /// ```
    pub fn update(&mut self, msg: DiffViewerMessage) -> Option<DiffViewerOutput> {
        DiffViewer::update(self, msg)
    }

    // ---- Internal helpers ----

    /// Collects all displayable lines for unified mode (flattened across hunks).
    pub(crate) fn collect_display_lines(&self) -> Vec<DiffLine> {
        self.hunks
            .iter()
            .flat_map(|h| h.lines.iter().cloned())
            .collect()
    }

    /// Collects paired lines for side-by-side mode.
    ///
    /// Returns a vec of (left, right) where each element is `Option<DiffLine>`.
    /// Context lines appear on both sides. Removed lines appear on the left,
    /// added lines on the right, and the opposite side gets `None`.
    pub(crate) fn collect_side_by_side_pairs(&self) -> Vec<(Option<DiffLine>, Option<DiffLine>)> {
        let mut pairs = Vec::new();

        for hunk in &self.hunks {
            // Add hunk header on both sides
            let header = hunk.lines.first().cloned();
            if let Some(ref h) = header {
                if h.line_type == DiffLineType::Header {
                    pairs.push((Some(h.clone()), Some(h.clone())));
                }
            }

            // Collect removed and added runs, pairing them together.
            let content_lines: Vec<_> = hunk
                .lines
                .iter()
                .filter(|l| l.line_type != DiffLineType::Header)
                .collect();

            let mut i = 0;
            while i < content_lines.len() {
                match content_lines[i].line_type {
                    DiffLineType::Context => {
                        pairs.push((
                            Some(content_lines[i].clone()),
                            Some(content_lines[i].clone()),
                        ));
                        i += 1;
                    }
                    DiffLineType::Removed => {
                        // Collect consecutive removed lines
                        let mut removed = Vec::new();
                        while i < content_lines.len()
                            && content_lines[i].line_type == DiffLineType::Removed
                        {
                            removed.push(content_lines[i].clone());
                            i += 1;
                        }
                        // Collect consecutive added lines
                        let mut added = Vec::new();
                        while i < content_lines.len()
                            && content_lines[i].line_type == DiffLineType::Added
                        {
                            added.push(content_lines[i].clone());
                            i += 1;
                        }
                        // Pair them up
                        let max_len = removed.len().max(added.len());
                        for j in 0..max_len {
                            let left = removed.get(j).cloned();
                            let right = added.get(j).cloned();
                            pairs.push((left, right));
                        }
                    }
                    DiffLineType::Added => {
                        pairs.push((None, Some(content_lines[i].clone())));
                        i += 1;
                    }
                    DiffLineType::Header => {
                        i += 1; // Skip (already handled)
                    }
                }
            }
        }

        pairs
    }

    /// Computes the scroll offset for the start of the given hunk index.
    fn scroll_offset_for_hunk(&self, hunk_idx: usize) -> usize {
        let mut offset = 0;
        for hunk in self.hunks.iter().take(hunk_idx) {
            offset += hunk.lines.len();
        }
        offset
    }
}

/// Counts total displayable lines across all hunks.
fn count_total_lines(hunks: &[DiffHunk]) -> usize {
    hunks.iter().map(|h| h.lines.len()).sum()
}

/// A diff viewer component for unified and side-by-side diff display.
///
/// Displays text diffs with hunk navigation and line-level highlighting.
/// Supports unified and side-by-side display modes.
///
/// # Key Bindings
///
/// - `Up` / `k` -- Scroll up one line
/// - `Down` / `j` -- Scroll down one line
/// - `n` -- Jump to next hunk
/// - `N` / `p` -- Jump to previous hunk
/// - `PageUp` / `Ctrl+u` -- Scroll up half a page
/// - `PageDown` / `Ctrl+d` -- Scroll down half a page
/// - `Home` / `g` -- Scroll to top
/// - `End` / `G` -- Scroll to bottom
/// - `m` -- Toggle between unified and side-by-side mode
pub struct DiffViewer;

impl Component for DiffViewer {
    type State = DiffViewerState;
    type Message = DiffViewerMessage;
    type Output = DiffViewerOutput;

    fn init() -> Self::State {
        DiffViewerState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        let key = event.as_key()?;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            KeyCode::Up | KeyCode::Char('k') if !ctrl => Some(DiffViewerMessage::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') if !ctrl => Some(DiffViewerMessage::ScrollDown),
            KeyCode::Char('n') if !shift && !ctrl => Some(DiffViewerMessage::NextHunk),
            KeyCode::Char('N') if shift => Some(DiffViewerMessage::PrevHunk),
            KeyCode::Char('p') if !ctrl => Some(DiffViewerMessage::PrevHunk),
            KeyCode::PageUp => Some(DiffViewerMessage::PageUp(10)),
            KeyCode::PageDown => Some(DiffViewerMessage::PageDown(10)),
            KeyCode::Char('u') if ctrl => Some(DiffViewerMessage::PageUp(10)),
            KeyCode::Char('d') if ctrl => Some(DiffViewerMessage::PageDown(10)),
            KeyCode::Home | KeyCode::Char('g') if !shift => Some(DiffViewerMessage::Home),
            KeyCode::End | KeyCode::Char('G') if shift || key.code == KeyCode::End => {
                Some(DiffViewerMessage::End)
            }
            KeyCode::Char('m') if !ctrl => Some(DiffViewerMessage::ToggleMode),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            DiffViewerMessage::SetDiff(text) => {
                state.hunks = parser::parse_unified_diff(&text);
                state.current_hunk = 0;
                state.scroll = ScrollState::new(count_total_lines(&state.hunks));
                None
            }
            DiffViewerMessage::SetTexts { old, new } => {
                state.hunks = parser::compute_diff(&old, &new, state.context_lines);
                state.current_hunk = 0;
                state.scroll = ScrollState::new(count_total_lines(&state.hunks));
                None
            }
            DiffViewerMessage::SetHunks(hunks) => {
                state.hunks = hunks;
                state.current_hunk = 0;
                state.scroll = ScrollState::new(count_total_lines(&state.hunks));
                None
            }
            DiffViewerMessage::Clear => {
                state.hunks.clear();
                state.current_hunk = 0;
                state.scroll = ScrollState::new(0);
                None
            }
            DiffViewerMessage::NextHunk => {
                if state.hunks.is_empty() {
                    return None;
                }
                let new_hunk = if state.current_hunk + 1 < state.hunks.len() {
                    state.current_hunk + 1
                } else {
                    0 // Wrap around
                };
                if new_hunk != state.current_hunk {
                    state.current_hunk = new_hunk;
                    let offset = state.scroll_offset_for_hunk(new_hunk);
                    state.scroll.set_offset(offset);
                    Some(DiffViewerOutput::HunkChanged(new_hunk))
                } else {
                    None
                }
            }
            DiffViewerMessage::PrevHunk => {
                if state.hunks.is_empty() {
                    return None;
                }
                let new_hunk = if state.current_hunk > 0 {
                    state.current_hunk - 1
                } else {
                    state.hunks.len() - 1 // Wrap around
                };
                if new_hunk != state.current_hunk {
                    state.current_hunk = new_hunk;
                    let offset = state.scroll_offset_for_hunk(new_hunk);
                    state.scroll.set_offset(offset);
                    Some(DiffViewerOutput::HunkChanged(new_hunk))
                } else {
                    None
                }
            }
            DiffViewerMessage::ScrollUp => {
                if state.scroll.scroll_up() {
                    update_current_hunk_from_scroll(state);
                    None
                } else {
                    None
                }
            }
            DiffViewerMessage::ScrollDown => {
                if state.scroll.scroll_down() {
                    update_current_hunk_from_scroll(state);
                    None
                } else {
                    None
                }
            }
            DiffViewerMessage::PageUp(n) => {
                if state.scroll.page_up(n) {
                    update_current_hunk_from_scroll(state);
                    None
                } else {
                    None
                }
            }
            DiffViewerMessage::PageDown(n) => {
                if state.scroll.page_down(n) {
                    update_current_hunk_from_scroll(state);
                    None
                } else {
                    None
                }
            }
            DiffViewerMessage::Home => {
                state.scroll.scroll_to_start();
                state.current_hunk = 0;
                None
            }
            DiffViewerMessage::End => {
                state.scroll.scroll_to_end();
                if !state.hunks.is_empty() {
                    state.current_hunk = state.hunks.len() - 1;
                }
                None
            }
            DiffViewerMessage::ToggleMode => {
                state.mode = match state.mode {
                    DiffMode::Unified => DiffMode::SideBySide,
                    DiffMode::SideBySide => DiffMode::Unified,
                };
                Some(DiffViewerOutput::ModeChanged(state.mode.clone()))
            }
            DiffViewerMessage::SetMode(mode) => {
                if state.mode != mode {
                    state.mode = mode.clone();
                    Some(DiffViewerOutput::ModeChanged(mode))
                } else {
                    None
                }
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        render::render(state, frame, area, theme);
    }
}

/// Updates current_hunk based on the current scroll offset.
fn update_current_hunk_from_scroll(state: &mut DiffViewerState) {
    let offset = state.scroll.offset();
    let mut cumulative = 0;
    for (i, hunk) in state.hunks.iter().enumerate() {
        cumulative += hunk.lines.len();
        if offset < cumulative {
            state.current_hunk = i;
            return;
        }
    }
    if !state.hunks.is_empty() {
        state.current_hunk = state.hunks.len() - 1;
    }
}

impl Focusable for DiffViewer {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for DiffViewer {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod tests;
