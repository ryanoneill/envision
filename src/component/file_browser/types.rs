//! Types for the file browser component.

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

/// Computes path segments from a path string.
pub(super) fn compute_segments(path: &str) -> Vec<String> {
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

/// Formats a file size in human-readable form.
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

/// A single file or directory entry.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: Option<u64>,
    modified: Option<u64>,
    extension: Option<String>,
    hidden: bool,
}

impl FileEntry {
    /// Creates a new file entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::FileEntry;
    ///
    /// let entry = FileEntry::file("main.rs", "/src/main.rs");
    /// assert_eq!(entry.name(), "main.rs");
    /// assert_eq!(entry.path(), "/src/main.rs");
    /// assert!(!entry.is_dir());
    /// ```
    pub fn file(name: impl Into<String>, path: impl Into<String>) -> Self {
        let name_str = name.into();
        let ext = name_str
            .rsplit('.')
            .next()
            .filter(|e| e.len() < name_str.len())
            .map(String::from);
        Self {
            hidden: name_str.starts_with('.'),
            name: name_str,
            path: path.into(),
            is_dir: false,
            size: None,
            modified: None,
            extension: ext,
        }
    }

    /// Creates a new directory entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::FileEntry;
    ///
    /// let entry = FileEntry::directory("src", "/src");
    /// assert_eq!(entry.name(), "src");
    /// assert!(entry.is_dir());
    /// ```
    pub fn directory(name: impl Into<String>, path: impl Into<String>) -> Self {
        let name_str = name.into();
        Self {
            hidden: name_str.starts_with('.'),
            name: name_str,
            path: path.into(),
            is_dir: true,
            size: None,
            modified: None,
            extension: None,
        }
    }

    /// Sets the file size (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::FileEntry;
    ///
    /// let entry = FileEntry::file("data.bin", "/data.bin").with_size(1024);
    /// assert_eq!(entry.size(), Some(1024));
    /// ```
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the modification timestamp (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::FileEntry;
    ///
    /// let entry = FileEntry::file("data.bin", "/data.bin").with_modified(1700000000);
    /// assert_eq!(entry.modified(), Some(1700000000));
    /// ```
    pub fn with_modified(mut self, modified: u64) -> Self {
        self.modified = Some(modified);
        self
    }

    /// Returns the entry name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the full path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns true if this is a directory.
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }

    /// Returns the file size in bytes, if known.
    pub fn size(&self) -> Option<u64> {
        self.size
    }

    /// Returns the modification timestamp, if known.
    pub fn modified(&self) -> Option<u64> {
        self.modified
    }

    /// Returns the file extension, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::FileEntry;
    ///
    /// let entry = FileEntry::file("main.rs", "/main.rs");
    /// assert_eq!(entry.extension(), Some("rs"));
    ///
    /// let dir = FileEntry::directory("src", "/src");
    /// assert_eq!(dir.extension(), None);
    /// ```
    pub fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }

    /// Returns true if this is a hidden entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::file_browser::FileEntry;
    ///
    /// let hidden = FileEntry::file(".gitignore", "/.gitignore");
    /// assert!(hidden.is_hidden());
    ///
    /// let visible = FileEntry::file("README.md", "/README.md");
    /// assert!(!visible.is_hidden());
    /// ```
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }
}

/// Sort field for file entries.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum FileSortField {
    /// Sort by name.
    Name,
    /// Sort by file size.
    Size,
    /// Sort by modification time.
    Modified,
    /// Sort by file extension.
    Extension,
}

/// Sort direction.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum FileSortDirection {
    /// Ascending order.
    Ascending,
    /// Descending order.
    Descending,
}

/// File selection mode.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum SelectionMode {
    /// Select a single file only.
    SingleFile,
    /// Select a single directory only.
    SingleDirectory,
    /// Select a single file or directory.
    Single,
    /// Select multiple files.
    MultipleFiles,
    /// Select multiple files or directories.
    Multiple,
}

/// Which part of the file browser has internal focus.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub(crate) enum FileBrowserFocus {
    PathBar,
    FileList,
    Filter,
}
