//! Palette item type and fuzzy matching.

/// An item in the command palette.
///
/// Each item has a unique id and display label, with optional description,
/// shortcut hint, and category.
///
/// # Example
///
/// ```rust
/// use envision::component::PaletteItem;
///
/// let item = PaletteItem::new("open-file", "Open File")
///     .with_description("Open a file from disk")
///     .with_shortcut("Ctrl+O")
///     .with_category("File");
///
/// assert_eq!(item.id, "open-file");
/// assert_eq!(item.label, "Open File");
/// assert_eq!(item.description.as_deref(), Some("Open a file from disk"));
/// assert_eq!(item.shortcut.as_deref(), Some("Ctrl+O"));
/// assert_eq!(item.category.as_deref(), Some("File"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct PaletteItem {
    /// Unique identifier for the item.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Optional description/subtitle.
    pub description: Option<String>,
    /// Optional keyboard shortcut hint.
    pub shortcut: Option<String>,
    /// Optional category/group.
    pub category: Option<String>,
}

impl PaletteItem {
    /// Creates a new palette item with the given id and label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaletteItem;
    ///
    /// let item = PaletteItem::new("save", "Save File");
    /// assert_eq!(item.id, "save");
    /// assert_eq!(item.label, "Save File");
    /// assert!(item.description.is_none());
    /// assert!(item.shortcut.is_none());
    /// assert!(item.category.is_none());
    /// ```
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            shortcut: None,
            category: None,
        }
    }

    /// Sets the description (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaletteItem;
    ///
    /// let item = PaletteItem::new("open", "Open File")
    ///     .with_description("Open a file from the filesystem");
    /// assert_eq!(item.description.as_deref(), Some("Open a file from the filesystem"));
    /// ```
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets the keyboard shortcut hint (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaletteItem;
    ///
    /// let item = PaletteItem::new("save", "Save").with_shortcut("Ctrl+S");
    /// assert_eq!(item.shortcut.as_deref(), Some("Ctrl+S"));
    /// ```
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Sets the category/group (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaletteItem;
    ///
    /// let item = PaletteItem::new("open", "Open").with_category("File");
    /// assert_eq!(item.category.as_deref(), Some("File"));
    /// ```
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}

/// Computes a fuzzy match score for a query against text.
///
/// Returns `None` if the query does not match, or `Some(score)` where
/// higher scores indicate better matches.
///
/// Match priority:
/// 1. Exact prefix match (highest score)
/// 2. Substring match (medium score)
/// 3. Fuzzy character-by-character match (lowest score)
///
/// # Example
///
/// ```rust
/// # use envision::component::command_palette::fuzzy_score;
/// // Exact prefix match (highest)
/// assert!(fuzzy_score("open", "Open File").unwrap() > fuzzy_score("file", "Open File").unwrap());
///
/// // Substring match (medium)
/// assert!(fuzzy_score("file", "Open File").unwrap() > fuzzy_score("ofl", "Open File").unwrap());
///
/// // No match returns None
/// assert!(fuzzy_score("xyz", "Open File").is_none());
/// ```
pub fn fuzzy_score(query: &str, text: &str) -> Option<usize> {
    if query.is_empty() {
        return Some(0);
    }
    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    // Exact prefix match (highest priority).
    // Shorter text = more specific match, so we reward shorter text.
    if text_lower.starts_with(&query_lower) {
        return Some(1000 + text.len().saturating_sub(query.len()));
    }

    // Substring match (medium priority).
    if text_lower.contains(&query_lower) {
        return Some(500);
    }

    // Fuzzy: each query char must appear in order within the text.
    let mut text_chars = text_lower.chars();
    let mut matched = 0usize;
    for qc in query_lower.chars() {
        loop {
            match text_chars.next() {
                Some(tc) if tc == qc => {
                    matched += 1;
                    break;
                }
                Some(_) => {}
                None => return None,
            }
        }
    }
    Some(matched)
}
