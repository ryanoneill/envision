use super::StatusBarState;

impl StatusBarState {
    /// Sets the separator for the left section (builder pattern).
    ///
    /// When set, takes precedence over the global `separator` for
    /// left-section rendering. When `None` (default), the global
    /// `separator` applies.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let state = StatusBarState::with_separator(" · ")
    ///     .with_left_separator(" | ");
    /// assert_eq!(state.left_separator(), Some(" | "));
    /// assert_eq!(state.separator(), " · "); // global unchanged
    /// ```
    pub fn with_left_separator(mut self, separator: impl Into<String>) -> Self {
        self.left_separator = Some(separator.into());
        self
    }

    /// Sets the separator for the center section (builder pattern).
    ///
    /// When set, takes precedence over the global `separator` for
    /// center-section rendering. When `None` (default), the global
    /// `separator` applies.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let state = StatusBarState::new().with_center_separator(" :: ");
    /// assert_eq!(state.center_separator(), Some(" :: "));
    /// ```
    pub fn with_center_separator(mut self, separator: impl Into<String>) -> Self {
        self.center_separator = Some(separator.into());
        self
    }

    /// Sets the separator for the right section (builder pattern).
    ///
    /// When set, takes precedence over the global `separator` for
    /// right-section rendering. When `None` (default), the global
    /// `separator` applies.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// // Global " · " separator, but " " between right-section items.
    /// let state = StatusBarState::with_separator(" · ")
    ///     .with_right_separator(" ");
    /// assert_eq!(state.right_separator(), Some(" "));
    /// assert_eq!(state.separator(), " · "); // global unchanged
    /// ```
    pub fn with_right_separator(mut self, separator: impl Into<String>) -> Self {
        self.right_separator = Some(separator.into());
        self
    }

    /// Returns the left-section separator override, if set.
    ///
    /// `None` means the left section uses the global `separator()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let plain = StatusBarState::new();
    /// assert_eq!(plain.left_separator(), None);
    ///
    /// let overridden = plain.with_left_separator(" | ");
    /// assert_eq!(overridden.left_separator(), Some(" | "));
    /// ```
    pub fn left_separator(&self) -> Option<&str> {
        self.left_separator.as_deref()
    }

    /// Returns the center-section separator override, if set.
    ///
    /// `None` means the center section uses the global `separator()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let plain = StatusBarState::new();
    /// assert_eq!(plain.center_separator(), None);
    ///
    /// let overridden = plain.with_center_separator(" :: ");
    /// assert_eq!(overridden.center_separator(), Some(" :: "));
    /// ```
    pub fn center_separator(&self) -> Option<&str> {
        self.center_separator.as_deref()
    }

    /// Returns the right-section separator override, if set.
    ///
    /// `None` means the right section uses the global `separator()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let plain = StatusBarState::new();
    /// assert_eq!(plain.right_separator(), None);
    ///
    /// let overridden = plain.with_right_separator(" ");
    /// assert_eq!(overridden.right_separator(), Some(" "));
    /// ```
    pub fn right_separator(&self) -> Option<&str> {
        self.right_separator.as_deref()
    }
}
