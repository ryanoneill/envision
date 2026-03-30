//! Types for the file browser component.

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
