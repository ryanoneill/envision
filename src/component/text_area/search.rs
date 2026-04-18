/// Search functionality for TextAreaState.
///
/// Provides text search with match highlighting and navigation.
/// Extracted to a submodule to keep the main module under the
/// 1000-line limit.
use super::TextAreaState;

impl TextAreaState {
    /// Returns the current search query, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TextArea, TextAreaMessage, TextAreaState, Component};
    ///
    /// let mut state = TextAreaState::new().with_value("hello world");
    /// assert_eq!(state.search_query(), None);
    ///
    /// TextArea::update(&mut state, TextAreaMessage::SetSearchQuery("hello".into()));
    /// assert_eq!(state.search_query(), Some("hello"));
    /// ```
    pub fn search_query(&self) -> Option<&str> {
        self.search_query.as_deref()
    }

    /// Returns the list of search matches as (line, byte_col) pairs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TextArea, TextAreaMessage, TextAreaState, Component};
    ///
    /// let mut state = TextAreaState::new().with_value("foo bar foo");
    /// TextArea::update(&mut state, TextAreaMessage::SetSearchQuery("foo".into()));
    /// assert_eq!(state.search_matches().len(), 2);
    /// ```
    pub fn search_matches(&self) -> &[(usize, usize)] {
        &self.search_matches
    }

    /// Returns the index of the current match within the match list.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TextArea, TextAreaMessage, TextAreaState, Component};
    ///
    /// let mut state = TextAreaState::new().with_value("aaa");
    /// TextArea::update(&mut state, TextAreaMessage::SetSearchQuery("a".into()));
    /// assert_eq!(state.current_match_index(), 0);
    /// ```
    pub fn current_match_index(&self) -> usize {
        self.current_match
    }

    /// Returns the current match as (line, byte_col), if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TextArea, TextAreaMessage, TextAreaState, Component};
    ///
    /// let mut state = TextAreaState::new().with_value("hello");
    /// TextArea::update(&mut state, TextAreaMessage::SetSearchQuery("hello".into()));
    /// assert_eq!(state.current_match_position(), Some((0, 0)));
    /// ```
    pub fn current_match_position(&self) -> Option<(usize, usize)> {
        self.search_matches.get(self.current_match).copied()
    }

    /// Returns true if search mode is active.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TextArea, TextAreaMessage, TextAreaState, Component};
    ///
    /// let mut state = TextAreaState::new();
    /// assert!(!state.is_searching());
    ///
    /// TextArea::update(&mut state, TextAreaMessage::StartSearch);
    /// assert!(state.is_searching());
    /// ```
    pub fn is_searching(&self) -> bool {
        self.search_query.is_some()
    }

    /// Starts search mode with an empty query.
    pub(super) fn start_search(&mut self) {
        if self.search_query.is_none() {
            self.search_query = Some(String::new());
            self.search_matches.clear();
            self.current_match = 0;
        }
    }

    /// Sets the search query and recomputes matches.
    pub(super) fn set_search_query(&mut self, query: String) {
        self.search_query = Some(query);
        self.recompute_matches();
        // Jump to first match at or after current cursor position
        self.jump_to_nearest_match_forward();
    }

    /// Advances to the next match, wrapping around.
    pub(super) fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = (self.current_match + 1) % self.search_matches.len();
        self.jump_cursor_to_current_match();
    }

    /// Goes to the previous match, wrapping around.
    pub(super) fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        if self.current_match == 0 {
            self.current_match = self.search_matches.len() - 1;
        } else {
            self.current_match -= 1;
        }
        self.jump_cursor_to_current_match();
    }

    /// Clears search state entirely.
    pub(super) fn clear_search(&mut self) {
        self.search_query = None;
        self.search_matches.clear();
        self.current_match = 0;
    }

    /// Recomputes matches for the current query against the lines.
    pub(super) fn recompute_matches(&mut self) {
        self.search_matches.clear();
        self.current_match = 0;

        let query = match &self.search_query {
            Some(q) if !q.is_empty() => q.clone(),
            _ => return,
        };

        for (line_idx, line) in self.lines.iter().enumerate() {
            let mut start = 0;
            while let Some(pos) = line[start..].find(&query) {
                let byte_col = start + pos;
                self.search_matches.push((line_idx, byte_col));
                // Advance past this match to find overlapping/subsequent matches
                start = byte_col + 1;
                if start >= line.len() {
                    break;
                }
            }
        }
    }

    /// Jumps the cursor to the current match position.
    fn jump_cursor_to_current_match(&mut self) {
        if let Some(&(row, col)) = self.search_matches.get(self.current_match) {
            self.cursor_row = row;
            self.cursor_col = col;
            self.clear_selection();
        }
    }

    /// Finds the nearest match at or after the cursor position and sets it as current.
    fn jump_to_nearest_match_forward(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        let cursor = (self.cursor_row, self.cursor_col);
        for (i, &match_pos) in self.search_matches.iter().enumerate() {
            if match_pos >= cursor {
                self.current_match = i;
                self.jump_cursor_to_current_match();
                return;
            }
        }

        // Wrap to first match if no match found after cursor
        self.current_match = 0;
        self.jump_cursor_to_current_match();
    }
}
