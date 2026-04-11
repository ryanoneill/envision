//! A compound file browsing component with pluggable filesystem abstraction.
//!
//! [`FileBrowser`] provides a navigable directory listing with filtering, sorting,
//! and selection support. It uses a [`DirectoryProvider`] trait for filesystem
//! access, enabling both real filesystem and in-memory/test implementations.
//! State is stored in [`FileBrowserState`], updated via [`FileBrowserMessage`],
//! and produces [`FileBrowserOutput`]. Entries are represented as
//! [`FileEntry`].
//!
//! Focus and disabled state are managed via [`ViewContext`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     FileBrowser, FileBrowserState, FileBrowserMessage, Component,
//!     file_browser::{FileEntry, SelectionMode},
//! };
//!
//! let entries = vec![
//!     FileEntry::directory("src", "/src"),
//!     FileEntry::file("README.md", "/README.md"),
//! ];
//! let state = FileBrowserState::new("/", entries);
//!
//! assert_eq!(state.current_path(), "/");
//! assert_eq!(state.entries().len(), 2);
//! ```

mod types;
mod view;

pub use types::*;

use std::sync::Arc;

use ratatui::prelude::*;
use ratatui::widgets::ListState;

use super::{Component, ViewContext};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;
use types::FileBrowserFocus;

/// Trait for providing directory listings.
///
/// Implement this trait to connect the file browser to a filesystem,
/// virtual filesystem, or any other hierarchical data source.
pub trait DirectoryProvider: Send + 'static {
    /// Lists entries in the given directory path.
    fn list_entries(&self, path: &str) -> Vec<FileEntry>;

    /// Returns the parent path, or `None` if at the root.
    fn parent_path(&self, path: &str) -> Option<String>;

    /// Returns the path separator (default: "/").
    fn separator(&self) -> &str {
        "/"
    }
}

/// Messages that can be sent to a FileBrowser.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileBrowserMessage {
    /// Move selection up.
    Up,
    /// Move selection down.
    Down,
    /// Jump to first entry.
    First,
    /// Jump to last entry.
    Last,
    /// Page up.
    PageUp(usize),
    /// Page down.
    PageDown(usize),
    /// Enter selected directory or select file.
    Enter,
    /// Navigate to parent directory.
    Back,
    /// Toggle selection of current entry.
    ToggleSelect,
    /// Toggle visibility of hidden files.
    ToggleHidden,
    /// Cycle internal focus (PathBar -> FileList -> Filter).
    CycleFocus,
    /// Add a character to the filter.
    FilterChar(char),
    /// Remove last filter character.
    FilterBackspace,
    /// Clear the filter.
    FilterClear,
    /// Set the sort field.
    SetSort(FileSortField),
    /// Toggle sort direction.
    ToggleSortDirection,
    /// Navigate to a path segment in the breadcrumb.
    NavigateToSegment(usize),
    /// Refresh the file listing.
    Refresh,
}

/// Output messages from a FileBrowser.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileBrowserOutput {
    /// A file was selected (Enter on a file).
    FileSelected(FileEntry),
    /// A directory was entered.
    DirectoryEntered(String),
    /// Navigated back to parent.
    NavigatedBack(String),
    /// The selected index changed.
    SelectionChanged(usize),
    /// A path was toggled in multi-select mode.
    SelectionToggled(String),
    /// The filter text changed.
    FilterChanged(String),
    /// The sort field or direction changed.
    SortChanged(FileSortField, FileSortDirection),
    /// Hidden file visibility toggled.
    HiddenToggled(bool),
}

/// State for a FileBrowser component.
///
/// Manages the current directory, entries, filtering, sorting, and selection.
#[derive(Clone)]
pub struct FileBrowserState {
    current_path: String,
    path_segments: Vec<String>,
    entries: Vec<FileEntry>,
    filtered_indices: Vec<usize>,
    selected_index: Option<usize>,
    selected_paths: Vec<String>,
    filter_text: String,
    internal_focus: FileBrowserFocus,
    selection_mode: SelectionMode,
    sort_field: FileSortField,
    sort_direction: FileSortDirection,
    directories_first: bool,
    show_hidden: bool,
    pub(crate) list_state: ListState,
    #[allow(dead_code)]
    provider: Option<Arc<dyn DirectoryProvider>>,
}

impl Default for FileBrowserState {
    fn default() -> Self {
        Self {
            current_path: "/".to_string(),
            path_segments: vec!["/".to_string()],
            entries: Vec::new(),
            filtered_indices: Vec::new(),
            selected_index: None,
            selected_paths: Vec::new(),
            filter_text: String::new(),
            internal_focus: FileBrowserFocus::FileList,
            selection_mode: SelectionMode::Single,
            sort_field: FileSortField::Name,
            sort_direction: FileSortDirection::Ascending,
            directories_first: true,
            show_hidden: false,
            list_state: ListState::default(),
            provider: None,
        }
    }
}

impl std::fmt::Debug for FileBrowserState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileBrowserState")
            .field("current_path", &self.current_path)
            .field("path_segments", &self.path_segments)
            .field("entries", &self.entries)
            .field("filtered_indices", &self.filtered_indices)
            .field("selected_index", &self.selected_index)
            .field("selected_paths", &self.selected_paths)
            .field("filter_text", &self.filter_text)
            .field("internal_focus", &self.internal_focus)
            .field("selection_mode", &self.selection_mode)
            .field("sort_field", &self.sort_field)
            .field("sort_direction", &self.sort_direction)
            .field("directories_first", &self.directories_first)
            .field("show_hidden", &self.show_hidden)
            .field("list_state", &self.list_state)
            .field(
                "provider",
                &self.provider.as_ref().map(|_| "<DirectoryProvider>"),
            )
            .finish()
    }
}

impl PartialEq for FileBrowserState {
    fn eq(&self, other: &Self) -> bool {
        self.current_path == other.current_path
            && self.path_segments == other.path_segments
            && self.entries == other.entries
            && self.filtered_indices == other.filtered_indices
            && self.selected_index == other.selected_index
            && self.selected_paths == other.selected_paths
            && self.filter_text == other.filter_text
            && self.internal_focus == other.internal_focus
            && self.selection_mode == other.selection_mode
            && self.sort_field == other.sort_field
            && self.sort_direction == other.sort_direction
            && self.directories_first == other.directories_first
            && self.show_hidden == other.show_hidden
            && self.list_state.selected() == other.list_state.selected()
    }
}

impl FileBrowserState {
    /// Creates a new file browser with initial path and entries.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState};
    ///
    /// let entries = vec![
    ///     FileEntry::directory("src", "/src"),
    ///     FileEntry::file("main.rs", "/main.rs"),
    /// ];
    /// let state = FileBrowserState::new("/", entries);
    /// assert_eq!(state.current_path(), "/");
    /// assert_eq!(state.entries().len(), 2);
    /// ```
    pub fn new(path: impl Into<String>, entries: Vec<FileEntry>) -> Self {
        let path_str = path.into();
        let segments = compute_segments(&path_str);
        let mut state = Self {
            current_path: path_str,
            path_segments: segments,
            entries,
            ..Self::default()
        };
        state.sort_and_filter();
        if !state.filtered_indices.is_empty() {
            state.selected_index = Some(0);
            state.list_state.select(Some(0));
        }
        state
    }

    /// Creates a new file browser with a directory provider.
    pub fn with_provider(path: impl Into<String>, provider: Arc<dyn DirectoryProvider>) -> Self {
        let path_str = path.into();
        let entries = provider.list_entries(&path_str);
        let segments = compute_segments(&path_str);
        let mut state = Self {
            current_path: path_str,
            path_segments: segments,
            entries,
            provider: Some(provider),
            ..Self::default()
        };
        state.sort_and_filter();
        if !state.filtered_indices.is_empty() {
            state.selected_index = Some(0);
            state.list_state.select(Some(0));
        }
        state
    }

    /// Sets the selection mode (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState, SelectionMode};
    ///
    /// let state = FileBrowserState::new("/", vec![
    ///     FileEntry::file("a.txt", "/a.txt"),
    /// ]).with_selection_mode(SelectionMode::Multiple);
    /// assert_eq!(state.selection_mode(), &SelectionMode::Multiple);
    /// ```
    pub fn with_selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// Sets the sort field (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState, FileSortField};
    ///
    /// let state = FileBrowserState::new("/", vec![
    ///     FileEntry::file("a.txt", "/a.txt"),
    /// ]).with_sort_field(FileSortField::Size);
    /// assert_eq!(state.sort_field(), &FileSortField::Size);
    /// ```
    pub fn with_sort_field(mut self, field: FileSortField) -> Self {
        self.sort_field = field;
        self.sort_and_filter();
        self
    }

    /// Sets the sort direction (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState, FileSortDirection};
    ///
    /// let state = FileBrowserState::new("/", vec![
    ///     FileEntry::file("a.txt", "/a.txt"),
    /// ]).with_sort_direction(FileSortDirection::Descending);
    /// assert_eq!(state.sort_direction(), &FileSortDirection::Descending);
    /// ```
    pub fn with_sort_direction(mut self, direction: FileSortDirection) -> Self {
        self.sort_direction = direction;
        self.sort_and_filter();
        self
    }

    /// Sets whether directories are shown first (builder pattern).
    pub fn with_directories_first(mut self, directories_first: bool) -> Self {
        self.directories_first = directories_first;
        self.sort_and_filter();
        self
    }

    /// Sets whether hidden files are shown (builder pattern).
    pub fn with_show_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self.sort_and_filter();
        self
    }

    // ---- Accessors ----

    /// Returns the current directory path.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState};
    ///
    /// let state = FileBrowserState::new("/home/user", vec![]);
    /// assert_eq!(state.current_path(), "/home/user");
    /// ```
    pub fn current_path(&self) -> &str {
        &self.current_path
    }

    /// Returns the path segments for breadcrumb display.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState};
    ///
    /// let state = FileBrowserState::new("/home/user", vec![]);
    /// assert_eq!(state.path_segments(), &["/", "home", "user"]);
    /// ```
    pub fn path_segments(&self) -> &[String] {
        &self.path_segments
    }

    /// Returns all entries (unfiltered).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState};
    ///
    /// let state = FileBrowserState::new("/", vec![
    ///     FileEntry::file("a.txt", "/a.txt"),
    ///     FileEntry::directory("src", "/src"),
    /// ]);
    /// assert_eq!(state.entries().len(), 2);
    /// ```
    pub fn entries(&self) -> &[FileEntry] {
        &self.entries
    }

    /// Returns the indices of visible (filtered) entries.
    pub fn filtered_indices(&self) -> &[usize] {
        &self.filtered_indices
    }

    /// Returns the filtered entries.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState};
    ///
    /// let state = FileBrowserState::new("/", vec![
    ///     FileEntry::file("readme.md", "/readme.md"),
    ///     FileEntry::directory("src", "/src"),
    /// ]);
    /// assert_eq!(state.filtered_entries().len(), 2);
    /// ```
    pub fn filtered_entries(&self) -> Vec<&FileEntry> {
        self.filtered_indices
            .iter()
            .filter_map(|&i| self.entries.get(i))
            .collect()
    }

    /// Returns the currently selected entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState};
    ///
    /// let state = FileBrowserState::new("/", vec![
    ///     FileEntry::file("readme.md", "/readme.md"),
    /// ]);
    /// let entry = state.selected_entry().unwrap();
    /// assert_eq!(entry.name(), "readme.md");
    /// ```
    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.selected_index
            .and_then(|sel| self.filtered_indices.get(sel))
            .and_then(|&i| self.entries.get(i))
    }

    /// Returns the selected index within the filtered list.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState};
    ///
    /// let state = FileBrowserState::new("/", vec![FileEntry::file("a.txt", "/a.txt")]);
    /// assert_eq!(state.selected_index(), Some(0));
    ///
    /// let empty = FileBrowserState::new("/", vec![]);
    /// assert_eq!(empty.selected_index(), None);
    /// ```
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Returns the selected index within the filtered list.
    ///
    /// This is an alias for [`selected_index()`](Self::selected_index) that provides a
    /// consistent accessor name across all selection-based components.
    pub fn selected(&self) -> Option<usize> {
        self.selected_index()
    }

    /// Returns the currently selected file entry.
    ///
    /// This is an alias for [`selected_entry()`](Self::selected_entry) that provides a
    /// consistent accessor name across all selection-based components.
    pub fn selected_item(&self) -> Option<&FileEntry> {
        self.selected_entry()
    }

    /// Returns the selected paths (for multi-select mode).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState};
    ///
    /// let state = FileBrowserState::new("/", vec![]);
    /// assert!(state.selected_paths().is_empty());
    /// ```
    pub fn selected_paths(&self) -> &[String] {
        &self.selected_paths
    }

    /// Returns the current filter text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState};
    ///
    /// let state = FileBrowserState::new("/", vec![]);
    /// assert_eq!(state.filter_text(), "");
    /// ```
    pub fn filter_text(&self) -> &str {
        &self.filter_text
    }

    /// Returns the selection mode.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState, SelectionMode};
    ///
    /// let state = FileBrowserState::new("/", vec![])
    ///     .with_selection_mode(SelectionMode::Multiple);
    /// assert_eq!(state.selection_mode(), &SelectionMode::Multiple);
    /// ```
    pub fn selection_mode(&self) -> &SelectionMode {
        &self.selection_mode
    }

    /// Returns the sort field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState, FileSortField};
    ///
    /// let state = FileBrowserState::new("/", vec![]);
    /// assert_eq!(state.sort_field(), &FileSortField::Name);
    /// ```
    pub fn sort_field(&self) -> &FileSortField {
        &self.sort_field
    }

    /// Returns the sort direction.
    pub fn sort_direction(&self) -> &FileSortDirection {
        &self.sort_direction
    }

    /// Returns whether hidden files are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState};
    ///
    /// let state = FileBrowserState::new("/", vec![]);
    /// assert!(!state.show_hidden());
    ///
    /// let state = FileBrowserState::new("/", vec![]).with_show_hidden(true);
    /// assert!(state.show_hidden());
    /// ```
    pub fn show_hidden(&self) -> bool {
        self.show_hidden
    }

    /// Sets whether hidden files are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FileBrowserState;
    ///
    /// let mut state = FileBrowserState::new("/", vec![]);
    /// state.set_show_hidden(true);
    /// assert!(state.show_hidden());
    /// ```
    pub fn set_show_hidden(&mut self, show: bool) {
        self.show_hidden = show;
    }

    /// Returns which sub-area of the file browser currently has internal focus.
    pub(crate) fn internal_focus(&self) -> &FileBrowserFocus {
        &self.internal_focus
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState};
    /// use envision::component::{FileBrowserMessage, FileBrowserOutput};
    ///
    /// let mut state = FileBrowserState::new("/", vec![
    ///     FileEntry::file("a.txt", "/a.txt"),
    ///     FileEntry::file("b.txt", "/b.txt"),
    /// ]);
    /// let output = state.update(FileBrowserMessage::Down);
    /// assert_eq!(output, Some(FileBrowserOutput::SelectionChanged(1)));
    /// ```
    pub fn update(&mut self, msg: FileBrowserMessage) -> Option<FileBrowserOutput> {
        FileBrowser::update(self, msg)
    }

    // ---- Internal ----

    fn sort_and_filter(&mut self) {
        // Build indices of visible entries
        self.filtered_indices = (0..self.entries.len())
            .filter(|&i| {
                let entry = &self.entries[i];
                // Hidden filter
                if !self.show_hidden && entry.is_hidden() {
                    return false;
                }
                // Text filter
                if !self.filter_text.is_empty()
                    && !entry
                        .name()
                        .to_lowercase()
                        .contains(&self.filter_text.to_lowercase())
                {
                    return false;
                }
                true
            })
            .collect();

        // Sort the filtered indices
        let entries = &self.entries;
        let sort_field = &self.sort_field;
        let sort_direction = &self.sort_direction;
        let directories_first = self.directories_first;

        self.filtered_indices.sort_by(|&a, &b| {
            let ea = &entries[a];
            let eb = &entries[b];

            // Directories first
            if directories_first && ea.is_dir() != eb.is_dir() {
                return if ea.is_dir() {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }

            let ord = match sort_field {
                FileSortField::Name => ea.name().to_lowercase().cmp(&eb.name().to_lowercase()),
                FileSortField::Size => ea.size().cmp(&eb.size()),
                FileSortField::Modified => ea.modified().cmp(&eb.modified()),
                FileSortField::Extension => ea.extension().cmp(&eb.extension()),
            };

            match sort_direction {
                FileSortDirection::Ascending => ord,
                FileSortDirection::Descending => ord.reverse(),
            }
        });

        // Clamp selected index
        if self.filtered_indices.is_empty() {
            self.selected_index = None;
            self.list_state.select(None);
        } else if let Some(sel) = self.selected_index {
            if sel >= self.filtered_indices.len() {
                self.selected_index = Some(self.filtered_indices.len() - 1);
                self.list_state
                    .select(Some(self.filtered_indices.len() - 1));
            }
        }
    }

    fn navigate_to(&mut self, path: String, entries: Vec<FileEntry>) {
        self.current_path = path;
        self.path_segments = compute_segments(&self.current_path);
        self.entries = entries;
        self.filter_text.clear();
        self.sort_and_filter();
        self.selected_index = if self.filtered_indices.is_empty() {
            None
        } else {
            Some(0)
        };
        self.list_state.select(self.selected_index);
    }
}

fn compute_segments(path: &str) -> Vec<String> {
    let mut segments = Vec::new();
    if path.starts_with('/') {
        segments.push("/".to_string());
    }
    for part in path.split('/').filter(|s| !s.is_empty()) {
        segments.push(part.to_string());
    }
    if segments.is_empty() {
        segments.push("/".to_string());
    }
    segments
}

pub(crate) fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{}B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1}K", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1}M", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}G", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// A compound file browsing component.
///
/// Displays a navigable directory listing with filtering, sorting, and
/// selection support. Uses a [`DirectoryProvider`] trait for filesystem
/// access.
///
/// # Key Bindings (FileList focused)
///
/// - `Up` / `k` — Move selection up
/// - `Down` / `j` — Move selection down
/// - `Home` / `g` — Jump to first entry
/// - `End` / `G` — Jump to last entry
/// - `Enter` — Enter directory or select file
/// - `Backspace` — Navigate to parent (when filter is empty)
/// - `Space` — Toggle selection (multi-select mode)
/// - `Ctrl+h` — Toggle hidden files
/// - `Tab` / `BackTab` — Cycle internal focus
/// - Printable chars — Start/continue filtering
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     FileBrowser, FileBrowserState, FileBrowserMessage, Component,
///     file_browser::FileEntry,
/// };
///
/// let entries = vec![
///     FileEntry::directory("src", "/src"),
///     FileEntry::file("main.rs", "/main.rs"),
/// ];
/// let mut state = FileBrowserState::new("/", entries);
///
/// // Navigate down
/// FileBrowser::update(&mut state, FileBrowserMessage::Down);
/// ```
pub struct FileBrowser;

impl Component for FileBrowser {
    type State = FileBrowserState;
    type Message = FileBrowserMessage;
    type Output = FileBrowserOutput;

    fn init() -> Self::State {
        FileBrowserState::default()
    }

    fn handle_event(
        state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match state.internal_focus {
            FileBrowserFocus::FileList => match key.code {
                KeyCode::Up | KeyCode::Char('k') if !ctrl => Some(FileBrowserMessage::Up),
                KeyCode::Down | KeyCode::Char('j') if !ctrl => Some(FileBrowserMessage::Down),
                KeyCode::Home | KeyCode::Char('g') if !shift => Some(FileBrowserMessage::First),
                KeyCode::End | KeyCode::Char('G') if shift || key.code == KeyCode::End => {
                    Some(FileBrowserMessage::Last)
                }
                KeyCode::PageUp => Some(FileBrowserMessage::PageUp(10)),
                KeyCode::PageDown => Some(FileBrowserMessage::PageDown(10)),
                KeyCode::Enter => Some(FileBrowserMessage::Enter),
                KeyCode::Backspace if state.filter_text.is_empty() => {
                    Some(FileBrowserMessage::Back)
                }
                KeyCode::Backspace => Some(FileBrowserMessage::FilterBackspace),
                KeyCode::Char(' ') => Some(FileBrowserMessage::ToggleSelect),
                KeyCode::Char('h') if ctrl => Some(FileBrowserMessage::ToggleHidden),
                KeyCode::Tab => Some(FileBrowserMessage::CycleFocus),
                KeyCode::BackTab => Some(FileBrowserMessage::CycleFocus),
                KeyCode::Esc => Some(FileBrowserMessage::FilterClear),
                KeyCode::Char(c)
                    if !ctrl && c.is_alphanumeric() || c == '.' || c == '_' || c == '-' =>
                {
                    Some(FileBrowserMessage::FilterChar(c))
                }
                _ => None,
            },
            FileBrowserFocus::PathBar => match key.code {
                KeyCode::Tab | KeyCode::BackTab => Some(FileBrowserMessage::CycleFocus),
                _ => None,
            },
            FileBrowserFocus::Filter => match key.code {
                KeyCode::Tab | KeyCode::BackTab => Some(FileBrowserMessage::CycleFocus),
                KeyCode::Backspace => Some(FileBrowserMessage::FilterBackspace),
                KeyCode::Esc => Some(FileBrowserMessage::FilterClear),
                KeyCode::Char(c) if !ctrl => Some(FileBrowserMessage::FilterChar(c)),
                _ => None,
            },
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            FileBrowserMessage::Up => {
                if state.filtered_indices.is_empty() {
                    return None;
                }
                let new_index = match state.selected_index {
                    Some(0) | None => state.filtered_indices.len() - 1,
                    Some(i) => i - 1,
                };
                state.selected_index = Some(new_index);
                state.list_state.select(Some(new_index));
                Some(FileBrowserOutput::SelectionChanged(new_index))
            }
            FileBrowserMessage::Down => {
                if state.filtered_indices.is_empty() {
                    return None;
                }
                let new_index = match state.selected_index {
                    None => 0,
                    Some(i) => (i + 1) % state.filtered_indices.len(),
                };
                state.selected_index = Some(new_index);
                state.list_state.select(Some(new_index));
                Some(FileBrowserOutput::SelectionChanged(new_index))
            }
            FileBrowserMessage::First => {
                if state.filtered_indices.is_empty() {
                    return None;
                }
                state.selected_index = Some(0);
                state.list_state.select(Some(0));
                Some(FileBrowserOutput::SelectionChanged(0))
            }
            FileBrowserMessage::Last => {
                if state.filtered_indices.is_empty() {
                    return None;
                }
                let last = state.filtered_indices.len() - 1;
                state.selected_index = Some(last);
                state.list_state.select(Some(last));
                Some(FileBrowserOutput::SelectionChanged(last))
            }
            FileBrowserMessage::PageUp(n) => {
                if state.filtered_indices.is_empty() {
                    return None;
                }
                let new_index = state.selected_index.unwrap_or(0).saturating_sub(n);
                state.selected_index = Some(new_index);
                state.list_state.select(Some(new_index));
                Some(FileBrowserOutput::SelectionChanged(new_index))
            }
            FileBrowserMessage::PageDown(n) => {
                if state.filtered_indices.is_empty() {
                    return None;
                }
                let max = state.filtered_indices.len() - 1;
                let current = state.selected_index.unwrap_or(0);
                let new_index = (current + n).min(max);
                state.selected_index = Some(new_index);
                state.list_state.select(Some(new_index));
                Some(FileBrowserOutput::SelectionChanged(new_index))
            }
            FileBrowserMessage::Enter => {
                let entry = state.selected_entry()?.clone();
                if entry.is_dir() {
                    let path = entry.path().to_string();
                    let new_entries = state
                        .provider
                        .as_ref()
                        .map(|p| p.list_entries(&path))
                        .unwrap_or_default();
                    state.navigate_to(path.clone(), new_entries);
                    Some(FileBrowserOutput::DirectoryEntered(path))
                } else {
                    Some(FileBrowserOutput::FileSelected(entry))
                }
            }
            FileBrowserMessage::Back => {
                let parent = state
                    .provider
                    .as_ref()
                    .and_then(|p| p.parent_path(&state.current_path));
                if let Some(parent_path) = parent {
                    let new_entries = state
                        .provider
                        .as_ref()
                        .map(|p| p.list_entries(&parent_path))
                        .unwrap_or_default();
                    state.navigate_to(parent_path.clone(), new_entries);
                    Some(FileBrowserOutput::NavigatedBack(parent_path))
                } else {
                    None
                }
            }
            FileBrowserMessage::ToggleSelect => {
                let entry = state.selected_entry()?.clone();
                let path = entry.path().to_string();
                if let Some(pos) = state.selected_paths.iter().position(|p| p == &path) {
                    state.selected_paths.remove(pos);
                } else {
                    state.selected_paths.push(path.clone());
                }
                Some(FileBrowserOutput::SelectionToggled(path))
            }
            FileBrowserMessage::ToggleHidden => {
                state.show_hidden = !state.show_hidden;
                state.sort_and_filter();
                Some(FileBrowserOutput::HiddenToggled(state.show_hidden))
            }
            FileBrowserMessage::CycleFocus => {
                state.internal_focus = match state.internal_focus {
                    FileBrowserFocus::PathBar => FileBrowserFocus::FileList,
                    FileBrowserFocus::FileList => FileBrowserFocus::Filter,
                    FileBrowserFocus::Filter => FileBrowserFocus::PathBar,
                };
                None
            }
            FileBrowserMessage::FilterChar(c) => {
                state.filter_text.push(c);
                state.sort_and_filter();
                if !state.filtered_indices.is_empty() && state.selected_index.is_none() {
                    state.selected_index = Some(0);
                    state.list_state.select(Some(0));
                }
                Some(FileBrowserOutput::FilterChanged(state.filter_text.clone()))
            }
            FileBrowserMessage::FilterBackspace => {
                if state.filter_text.pop().is_some() {
                    state.sort_and_filter();
                    Some(FileBrowserOutput::FilterChanged(state.filter_text.clone()))
                } else {
                    None
                }
            }
            FileBrowserMessage::FilterClear => {
                if state.filter_text.is_empty() {
                    return None;
                }
                state.filter_text.clear();
                state.sort_and_filter();
                Some(FileBrowserOutput::FilterChanged(String::new()))
            }
            FileBrowserMessage::SetSort(field) => {
                state.sort_field = field.clone();
                state.sort_and_filter();
                Some(FileBrowserOutput::SortChanged(
                    field,
                    state.sort_direction.clone(),
                ))
            }
            FileBrowserMessage::ToggleSortDirection => {
                state.sort_direction = match state.sort_direction {
                    FileSortDirection::Ascending => FileSortDirection::Descending,
                    FileSortDirection::Descending => FileSortDirection::Ascending,
                };
                state.sort_and_filter();
                Some(FileBrowserOutput::SortChanged(
                    state.sort_field.clone(),
                    state.sort_direction.clone(),
                ))
            }
            FileBrowserMessage::NavigateToSegment(index) => {
                if index >= state.path_segments.len() {
                    return None;
                }
                // Reconstruct path from segments up to index
                let new_path = if index == 0 {
                    "/".to_string()
                } else {
                    let parts: Vec<&str> = state.path_segments[1..=index]
                        .iter()
                        .map(|s| s.as_str())
                        .collect();
                    format!("/{}", parts.join("/"))
                };
                let new_entries = state
                    .provider
                    .as_ref()
                    .map(|p| p.list_entries(&new_path))
                    .unwrap_or_default();
                state.navigate_to(new_path.clone(), new_entries);
                Some(FileBrowserOutput::DirectoryEntered(new_path))
            }
            FileBrowserMessage::Refresh => {
                let path = state.current_path.clone();
                let new_entries = state
                    .provider
                    .as_ref()
                    .map(|p| p.list_entries(&path))
                    .unwrap_or_default();
                state.entries = new_entries;
                state.sort_and_filter();
                None
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        view::render(state, frame, area, theme, ctx.focused, ctx.disabled);
    }
}

#[cfg(test)]
mod helper_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
