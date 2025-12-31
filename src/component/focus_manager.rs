//! Focus management for coordinating keyboard focus between components.
//!
//! `FocusManager` provides a way to track and navigate keyboard focus
//! across multiple focusable components in a TUI application.
//!
//! # Example
//!
//! ```rust
//! use envision::component::FocusManager;
//!
//! #[derive(Clone, PartialEq, Debug)]
//! enum Field { Username, Password, Submit }
//!
//! // Create a focus manager with a focus order
//! let mut focus = FocusManager::with_initial_focus(vec![
//!     Field::Username,
//!     Field::Password,
//!     Field::Submit,
//! ]);
//!
//! // Initially focused on first item
//! assert_eq!(focus.focused(), Some(&Field::Username));
//!
//! // Navigate forward (Tab)
//! focus.focus_next();
//! assert_eq!(focus.focused(), Some(&Field::Password));
//!
//! // Navigate backward (Shift+Tab)
//! focus.focus_prev();
//! assert_eq!(focus.focused(), Some(&Field::Username));
//!
//! // Check if a specific field is focused
//! assert!(focus.is_focused(&Field::Username));
//! assert!(!focus.is_focused(&Field::Password));
//! ```

/// Manages keyboard focus across multiple components.
///
/// `FocusManager` is generic over an ID type, which is typically a user-defined
/// enum representing the focusable elements in your application. It tracks
/// which element is currently focused and provides methods for navigation.
///
/// # Type Parameters
///
/// - `Id`: The type used to identify focusable elements. Must implement
///   `Clone` and `PartialEq`.
///
/// # Navigation Behavior
///
/// - `focus_next()` moves focus forward, wrapping from last to first
/// - `focus_prev()` moves focus backward, wrapping from first to last
/// - When unfocused, `focus_next()` focuses the first item, `focus_prev()` focuses the last
#[derive(Clone, Debug)]
pub struct FocusManager<Id> {
    order: Vec<Id>,
    focused: Option<usize>,
}

impl<Id> Default for FocusManager<Id> {
    fn default() -> Self {
        Self {
            order: Vec::new(),
            focused: None,
        }
    }
}

impl<Id: Clone + PartialEq> FocusManager<Id> {
    /// Creates a new focus manager with the given focus order.
    ///
    /// The manager starts with no element focused. Use `with_initial_focus`
    /// to start with the first element focused.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FocusManager;
    ///
    /// let focus: FocusManager<&str> = FocusManager::new(vec!["a", "b", "c"]);
    /// assert_eq!(focus.focused(), None);
    /// ```
    pub fn new(order: Vec<Id>) -> Self {
        Self {
            order,
            focused: None,
        }
    }

    /// Creates a new focus manager with the first element focused.
    ///
    /// If the order is empty, no element will be focused.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FocusManager;
    ///
    /// let focus = FocusManager::with_initial_focus(vec!["a", "b", "c"]);
    /// assert_eq!(focus.focused(), Some(&"a"));
    /// ```
    pub fn with_initial_focus(order: Vec<Id>) -> Self {
        let focused = if order.is_empty() { None } else { Some(0) };
        Self { order, focused }
    }

    /// Returns the currently focused ID, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FocusManager;
    ///
    /// let mut focus = FocusManager::new(vec!["a", "b"]);
    /// assert_eq!(focus.focused(), None);
    ///
    /// focus.focus(&"a");
    /// assert_eq!(focus.focused(), Some(&"a"));
    /// ```
    pub fn focused(&self) -> Option<&Id> {
        self.focused.and_then(|idx| self.order.get(idx))
    }

    /// Returns true if the given ID is currently focused.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FocusManager;
    ///
    /// let focus = FocusManager::with_initial_focus(vec!["a", "b"]);
    /// assert!(focus.is_focused(&"a"));
    /// assert!(!focus.is_focused(&"b"));
    /// ```
    pub fn is_focused(&self, id: &Id) -> bool {
        self.focused() == Some(id)
    }

    /// Focuses a specific ID.
    ///
    /// Returns `true` if the ID was found and focused, `false` if the ID
    /// is not in the focus order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FocusManager;
    ///
    /// let mut focus = FocusManager::new(vec!["a", "b", "c"]);
    ///
    /// assert!(focus.focus(&"b"));
    /// assert_eq!(focus.focused(), Some(&"b"));
    ///
    /// assert!(!focus.focus(&"unknown"));
    /// assert_eq!(focus.focused(), Some(&"b")); // Unchanged
    /// ```
    pub fn focus(&mut self, id: &Id) -> bool {
        if let Some(idx) = self.order.iter().position(|item| item == id) {
            self.focused = Some(idx);
            true
        } else {
            false
        }
    }

    /// Moves focus to the next item in the order.
    ///
    /// If no item is currently focused, focuses the first item.
    /// Wraps from the last item to the first.
    ///
    /// Returns the newly focused ID, or `None` if the order is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FocusManager;
    ///
    /// let mut focus = FocusManager::with_initial_focus(vec!["a", "b", "c"]);
    ///
    /// assert_eq!(focus.focus_next(), Some(&"b"));
    /// assert_eq!(focus.focus_next(), Some(&"c"));
    /// assert_eq!(focus.focus_next(), Some(&"a")); // Wraps
    /// ```
    pub fn focus_next(&mut self) -> Option<&Id> {
        if self.order.is_empty() {
            return None;
        }

        let next_idx = match self.focused {
            Some(idx) => (idx + 1) % self.order.len(),
            None => 0,
        };

        self.focused = Some(next_idx);
        self.order.get(next_idx)
    }

    /// Moves focus to the previous item in the order.
    ///
    /// If no item is currently focused, focuses the last item.
    /// Wraps from the first item to the last.
    ///
    /// Returns the newly focused ID, or `None` if the order is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FocusManager;
    ///
    /// let mut focus = FocusManager::with_initial_focus(vec!["a", "b", "c"]);
    ///
    /// assert_eq!(focus.focus_prev(), Some(&"c")); // Wraps from first to last
    /// assert_eq!(focus.focus_prev(), Some(&"b"));
    /// assert_eq!(focus.focus_prev(), Some(&"a"));
    /// ```
    pub fn focus_prev(&mut self) -> Option<&Id> {
        if self.order.is_empty() {
            return None;
        }

        let prev_idx = match self.focused {
            Some(idx) => {
                if idx == 0 {
                    self.order.len() - 1
                } else {
                    idx - 1
                }
            }
            None => self.order.len() - 1,
        };

        self.focused = Some(prev_idx);
        self.order.get(prev_idx)
    }

    /// Removes focus entirely.
    ///
    /// After calling this, `focused()` will return `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FocusManager;
    ///
    /// let mut focus = FocusManager::with_initial_focus(vec!["a", "b"]);
    /// assert!(focus.focused().is_some());
    ///
    /// focus.blur();
    /// assert!(focus.focused().is_none());
    /// ```
    pub fn blur(&mut self) {
        self.focused = None;
    }

    /// Focuses the first item in the order.
    ///
    /// Returns the focused ID, or `None` if the order is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FocusManager;
    ///
    /// let mut focus = FocusManager::new(vec!["a", "b", "c"]);
    /// focus.focus(&"c");
    ///
    /// assert_eq!(focus.focus_first(), Some(&"a"));
    /// assert_eq!(focus.focused(), Some(&"a"));
    /// ```
    pub fn focus_first(&mut self) -> Option<&Id> {
        if self.order.is_empty() {
            self.focused = None;
            None
        } else {
            self.focused = Some(0);
            self.order.first()
        }
    }

    /// Focuses the last item in the order.
    ///
    /// Returns the focused ID, or `None` if the order is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FocusManager;
    ///
    /// let mut focus = FocusManager::new(vec!["a", "b", "c"]);
    ///
    /// assert_eq!(focus.focus_last(), Some(&"c"));
    /// assert_eq!(focus.focused(), Some(&"c"));
    /// ```
    pub fn focus_last(&mut self) -> Option<&Id> {
        if self.order.is_empty() {
            self.focused = None;
            None
        } else {
            self.focused = Some(self.order.len() - 1);
            self.order.last()
        }
    }

    /// Returns the focus order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FocusManager;
    ///
    /// let focus = FocusManager::new(vec!["a", "b", "c"]);
    /// assert_eq!(focus.order(), &["a", "b", "c"]);
    /// ```
    pub fn order(&self) -> &[Id] {
        &self.order
    }

    /// Returns `true` if the focus order is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FocusManager;
    ///
    /// let empty: FocusManager<&str> = FocusManager::default();
    /// assert!(empty.is_empty());
    ///
    /// let non_empty = FocusManager::new(vec!["a"]);
    /// assert!(!non_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Returns the number of items in the focus order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FocusManager;
    ///
    /// let focus = FocusManager::new(vec!["a", "b", "c"]);
    /// assert_eq!(focus.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.order.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestField {
        A,
        B,
        C,
    }

    #[test]
    fn test_new_unfocused() {
        let focus = FocusManager::new(vec![TestField::A, TestField::B]);
        assert_eq!(focus.focused(), None);
        assert!(!focus.is_focused(&TestField::A));
    }

    #[test]
    fn test_with_initial_focus() {
        let focus = FocusManager::with_initial_focus(vec![TestField::A, TestField::B]);
        assert_eq!(focus.focused(), Some(&TestField::A));
        assert!(focus.is_focused(&TestField::A));
    }

    #[test]
    fn test_with_initial_focus_empty() {
        let focus: FocusManager<TestField> = FocusManager::with_initial_focus(vec![]);
        assert_eq!(focus.focused(), None);
    }

    #[test]
    fn test_focus_specific() {
        let mut focus = FocusManager::new(vec![TestField::A, TestField::B, TestField::C]);

        assert!(focus.focus(&TestField::B));
        assert_eq!(focus.focused(), Some(&TestField::B));
        assert!(focus.is_focused(&TestField::B));
        assert!(!focus.is_focused(&TestField::A));
    }

    #[test]
    fn test_focus_not_found() {
        let mut focus = FocusManager::new(vec![TestField::A, TestField::B]);

        // Try to focus something not in the order
        assert!(!focus.focus(&TestField::C));
        assert_eq!(focus.focused(), None);
    }

    #[test]
    fn test_focus_not_found_preserves_current() {
        let mut focus = FocusManager::with_initial_focus(vec![TestField::A, TestField::B]);

        // Try to focus something not in the order
        assert!(!focus.focus(&TestField::C));
        // Should preserve current focus
        assert_eq!(focus.focused(), Some(&TestField::A));
    }

    #[test]
    fn test_focus_next_basic() {
        let mut focus =
            FocusManager::with_initial_focus(vec![TestField::A, TestField::B, TestField::C]);

        assert_eq!(focus.focus_next(), Some(&TestField::B));
        assert_eq!(focus.focused(), Some(&TestField::B));

        assert_eq!(focus.focus_next(), Some(&TestField::C));
        assert_eq!(focus.focused(), Some(&TestField::C));
    }

    #[test]
    fn test_focus_next_wraps() {
        let mut focus =
            FocusManager::with_initial_focus(vec![TestField::A, TestField::B, TestField::C]);

        focus.focus(&TestField::C);
        assert_eq!(focus.focus_next(), Some(&TestField::A)); // Wraps to first
    }

    #[test]
    fn test_focus_next_from_unfocused() {
        let mut focus = FocusManager::new(vec![TestField::A, TestField::B]);

        assert_eq!(focus.focus_next(), Some(&TestField::A)); // Focuses first
    }

    #[test]
    fn test_focus_prev_basic() {
        let mut focus =
            FocusManager::with_initial_focus(vec![TestField::A, TestField::B, TestField::C]);

        focus.focus(&TestField::C);

        assert_eq!(focus.focus_prev(), Some(&TestField::B));
        assert_eq!(focus.focused(), Some(&TestField::B));

        assert_eq!(focus.focus_prev(), Some(&TestField::A));
        assert_eq!(focus.focused(), Some(&TestField::A));
    }

    #[test]
    fn test_focus_prev_wraps() {
        let mut focus =
            FocusManager::with_initial_focus(vec![TestField::A, TestField::B, TestField::C]);

        assert_eq!(focus.focus_prev(), Some(&TestField::C)); // Wraps to last
    }

    #[test]
    fn test_focus_prev_from_unfocused() {
        let mut focus = FocusManager::new(vec![TestField::A, TestField::B, TestField::C]);

        assert_eq!(focus.focus_prev(), Some(&TestField::C)); // Focuses last
    }

    #[test]
    fn test_blur() {
        let mut focus = FocusManager::with_initial_focus(vec![TestField::A, TestField::B]);

        assert!(focus.focused().is_some());
        focus.blur();
        assert!(focus.focused().is_none());
    }

    #[test]
    fn test_focus_first() {
        let mut focus = FocusManager::new(vec![TestField::A, TestField::B, TestField::C]);
        focus.focus(&TestField::C);

        assert_eq!(focus.focus_first(), Some(&TestField::A));
        assert_eq!(focus.focused(), Some(&TestField::A));
    }

    #[test]
    fn test_focus_last() {
        let mut focus = FocusManager::new(vec![TestField::A, TestField::B, TestField::C]);

        assert_eq!(focus.focus_last(), Some(&TestField::C));
        assert_eq!(focus.focused(), Some(&TestField::C));
    }

    #[test]
    fn test_empty_manager() {
        let mut focus: FocusManager<TestField> = FocusManager::default();

        assert!(focus.is_empty());
        assert_eq!(focus.len(), 0);
        assert_eq!(focus.focused(), None);
        assert_eq!(focus.focus_next(), None);
        assert_eq!(focus.focus_prev(), None);
        assert_eq!(focus.focus_first(), None);
        assert_eq!(focus.focus_last(), None);
        assert!(!focus.focus(&TestField::A));
    }

    #[test]
    fn test_single_item() {
        let mut focus = FocusManager::with_initial_focus(vec![TestField::A]);

        assert_eq!(focus.focused(), Some(&TestField::A));

        // Next wraps to same item
        assert_eq!(focus.focus_next(), Some(&TestField::A));

        // Prev wraps to same item
        assert_eq!(focus.focus_prev(), Some(&TestField::A));
    }

    #[test]
    fn test_order() {
        let focus = FocusManager::new(vec![TestField::A, TestField::B, TestField::C]);
        assert_eq!(focus.order(), &[TestField::A, TestField::B, TestField::C]);
    }

    #[test]
    fn test_len() {
        let focus = FocusManager::new(vec![TestField::A, TestField::B, TestField::C]);
        assert_eq!(focus.len(), 3);
    }

    #[test]
    fn test_is_empty() {
        let empty: FocusManager<TestField> = FocusManager::default();
        assert!(empty.is_empty());

        let non_empty = FocusManager::new(vec![TestField::A]);
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_default() {
        let focus: FocusManager<TestField> = FocusManager::default();
        assert!(focus.is_empty());
        assert_eq!(focus.focused(), None);
    }

    #[test]
    fn test_clone() {
        let focus = FocusManager::with_initial_focus(vec![TestField::A, TestField::B]);
        let cloned = focus.clone();

        assert_eq!(cloned.focused(), Some(&TestField::A));
        assert_eq!(cloned.len(), 2);
    }

    #[test]
    fn test_with_string_ids() {
        let mut focus = FocusManager::with_initial_focus(vec![
            "username".to_string(),
            "password".to_string(),
            "submit".to_string(),
        ]);

        assert_eq!(focus.focused(), Some(&"username".to_string()));
        assert_eq!(focus.focus_next(), Some(&"password".to_string()));
        assert!(focus.focus(&"submit".to_string()));
        assert_eq!(focus.focused(), Some(&"submit".to_string()));
    }
}
