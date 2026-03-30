//! Implementation methods for the [`Tab`] type.
//!
//! Extracted from the main tab_bar module to keep file sizes manageable.

use unicode_width::UnicodeWidthStr;

use super::Tab;

impl Tab {
    /// Creates a new tab with the given id and label.
    ///
    /// By default the tab is not closable, not modified, and has no icon.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let tab = Tab::new("t1", "Overview");
    /// assert_eq!(tab.id(), "t1");
    /// assert_eq!(tab.label(), "Overview");
    /// assert!(!tab.closable());
    /// assert!(!tab.modified());
    /// assert_eq!(tab.icon(), None);
    /// ```
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            closable: false,
            modified: false,
            icon: None,
        }
    }

    /// Sets whether the tab is closable (builder).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let tab = Tab::new("t1", "File").with_closable(true);
    /// assert!(tab.closable());
    /// ```
    pub fn with_closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Sets whether the tab is modified (builder).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let tab = Tab::new("t1", "File").with_modified(true);
    /// assert!(tab.modified());
    /// ```
    pub fn with_modified(mut self, modified: bool) -> Self {
        self.modified = modified;
        self
    }

    /// Sets an icon for the tab (builder).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let tab = Tab::new("t1", "File").with_icon("R");
    /// assert_eq!(tab.icon(), Some("R"));
    /// ```
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Returns the tab's unique identifier.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let tab = Tab::new("editor-1", "main.rs");
    /// assert_eq!(tab.id(), "editor-1");
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the tab's display label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let tab = Tab::new("t1", "Overview");
    /// assert_eq!(tab.label(), "Overview");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns whether the tab is closable.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let tab = Tab::new("t1", "File");
    /// assert!(!tab.closable());
    ///
    /// let tab = Tab::new("t1", "File").with_closable(true);
    /// assert!(tab.closable());
    /// ```
    pub fn closable(&self) -> bool {
        self.closable
    }

    /// Returns whether the tab has unsaved modifications.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let tab = Tab::new("t1", "File").with_modified(true);
    /// assert!(tab.modified());
    /// ```
    pub fn modified(&self) -> bool {
        self.modified
    }

    /// Returns the tab's icon, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let tab = Tab::new("t1", "File");
    /// assert_eq!(tab.icon(), None);
    ///
    /// let tab = Tab::new("t1", "File").with_icon("R");
    /// assert_eq!(tab.icon(), Some("R"));
    /// ```
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    /// Sets the label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let mut tab = Tab::new("t1", "Old Name");
    /// tab.set_label("New Name");
    /// assert_eq!(tab.label(), "New Name");
    /// ```
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Sets the closable flag.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let mut tab = Tab::new("t1", "File");
    /// tab.set_closable(true);
    /// assert!(tab.closable());
    /// ```
    pub fn set_closable(&mut self, closable: bool) {
        self.closable = closable;
    }

    /// Sets the modified flag.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let mut tab = Tab::new("t1", "File");
    /// tab.set_modified(true);
    /// assert!(tab.modified());
    /// ```
    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    /// Sets the icon.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let mut tab = Tab::new("t1", "File");
    /// tab.set_icon(Some("R".to_string()));
    /// assert_eq!(tab.icon(), Some("R"));
    ///
    /// tab.set_icon(None);
    /// assert_eq!(tab.icon(), None);
    /// ```
    pub fn set_icon(&mut self, icon: Option<String>) {
        self.icon = icon;
    }

    /// Returns the rendered width of this tab, including decorations.
    ///
    /// Layout: ` [icon ]label[modified][close] `
    pub(super) fn rendered_width(&self, max_tab_width: Option<usize>) -> usize {
        let mut w: usize = 2; // leading and trailing space
        if let Some(icon) = &self.icon {
            w += icon.width() + 1; // icon + space
        }
        w += self.label.width();
        if self.modified {
            w += 1; // bullet
        }
        if self.closable {
            w += 2; // space + close char
        }
        if let Some(max) = max_tab_width {
            w.min(max)
        } else {
            w
        }
    }
}
