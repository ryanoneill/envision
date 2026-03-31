//! Shell-style command history with stash/restore semantics.
//!
//! When browsing history, the current buffer is stashed. Editing while browsing
//! freezes the history entry as the live buffer and exits browse mode.

/// A ring buffer for input history.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct History {
    /// Previous entries (oldest first).
    entries: Vec<String>,
    /// Maximum number of entries.
    max_entries: usize,
    /// Current browse index (None = not browsing).
    browse_index: Option<usize>,
    /// Stashed buffer (saved when entering browse mode).
    stash: Option<String>,
}

impl Default for History {
    fn default() -> Self {
        Self::new(100)
    }
}

// PartialEq: history is not part of logical equality
impl PartialEq for History {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl History {
    /// Creates a new history with the given maximum entry count.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
            browse_index: None,
            stash: None,
        }
    }

    /// Pushes an entry to history.
    ///
    /// Ignores empty strings and consecutive duplicates.
    pub fn push(&mut self, entry: String) {
        if entry.is_empty() {
            return;
        }
        // Skip consecutive duplicate
        if self.entries.last() == Some(&entry) {
            return;
        }
        self.entries.push(entry);
        while self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
        self.exit_browse();
    }

    /// Moves to the previous (older) history entry.
    ///
    /// If not currently browsing, stashes `current_buffer` and starts
    /// from the most recent entry. Returns the history entry, or `None`
    /// if at the oldest entry or history is empty.
    pub fn prev(&mut self, current_buffer: &str) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }

        let new_index = match self.browse_index {
            None => {
                // Start browsing: stash current buffer
                self.stash = Some(current_buffer.to_string());
                self.entries.len() - 1
            }
            Some(0) => return None, // Already at oldest
            Some(i) => i - 1,
        };

        self.browse_index = Some(new_index);
        Some(&self.entries[new_index])
    }

    /// Moves to the next (newer) history entry.
    ///
    /// If at the most recent entry, restores the stashed buffer and
    /// exits browse mode. Returns the entry/stash, or `None` if not
    /// browsing.
    pub fn next(&mut self) -> Option<String> {
        let i = self.browse_index?;

        if i + 1 >= self.entries.len() {
            // Restore stashed buffer
            let stash = self.stash.take().unwrap_or_default();
            self.browse_index = None;
            Some(stash)
        } else {
            self.browse_index = Some(i + 1);
            Some(self.entries[i + 1].clone())
        }
    }

    /// Exits browse mode without restoring the stash.
    ///
    /// Called when the user edits while browsing (the browsed entry
    /// becomes the new live buffer).
    pub fn exit_browse(&mut self) {
        self.browse_index = None;
        self.stash = None;
    }

    /// Returns true if currently browsing history.
    pub fn is_browsing(&self) -> bool {
        self.browse_index.is_some()
    }

    /// Returns the number of history entries.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the maximum number of entries.
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Sets the maximum number of entries.
    ///
    /// If the current count exceeds the new maximum, oldest entries are removed.
    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
        while self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Returns the history entries.
    pub fn entries(&self) -> &[String] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let h = History::new(10);
        assert_eq!(h.count(), 0);
        assert!(!h.is_browsing());
    }

    #[test]
    fn test_push() {
        let mut h = History::new(10);
        h.push("hello".to_string());
        assert_eq!(h.count(), 1);
        assert_eq!(h.entries()[0], "hello");
    }

    #[test]
    fn test_push_ignores_empty() {
        let mut h = History::new(10);
        h.push(String::new());
        assert_eq!(h.count(), 0);
    }

    #[test]
    fn test_push_ignores_consecutive_duplicates() {
        let mut h = History::new(10);
        h.push("hello".to_string());
        h.push("hello".to_string());
        assert_eq!(h.count(), 1);
    }

    #[test]
    fn test_push_allows_non_consecutive_duplicates() {
        let mut h = History::new(10);
        h.push("hello".to_string());
        h.push("world".to_string());
        h.push("hello".to_string());
        assert_eq!(h.count(), 3);
    }

    #[test]
    fn test_max_entries() {
        let mut h = History::new(3);
        h.push("a".to_string());
        h.push("b".to_string());
        h.push("c".to_string());
        h.push("d".to_string());
        assert_eq!(h.count(), 3);
        assert_eq!(h.entries()[0], "b");
    }

    #[test]
    fn test_prev_empty_history() {
        let mut h = History::new(10);
        assert_eq!(h.prev("buffer"), None);
    }

    #[test]
    fn test_prev_starts_browsing() {
        let mut h = History::new(10);
        h.push("first".to_string());
        h.push("second".to_string());
        let entry = h.prev("current");
        assert_eq!(entry, Some("second"));
        assert!(h.is_browsing());
    }

    #[test]
    fn test_prev_traverses_backwards() {
        let mut h = History::new(10);
        h.push("first".to_string());
        h.push("second".to_string());
        assert_eq!(h.prev("current"), Some("second"));
        assert_eq!(h.prev("current"), Some("first"));
        assert_eq!(h.prev("current"), None); // At oldest
    }

    #[test]
    fn test_next_not_browsing() {
        let mut h = History::new(10);
        assert_eq!(h.next(), None);
    }

    #[test]
    fn test_next_restores_stash() {
        let mut h = History::new(10);
        h.push("first".to_string());
        h.prev("my buffer");
        let restored = h.next();
        assert_eq!(restored, Some("my buffer".to_string()));
        assert!(!h.is_browsing());
    }

    #[test]
    fn test_prev_next_cycle() {
        let mut h = History::new(10);
        h.push("a".to_string());
        h.push("b".to_string());
        h.push("c".to_string());

        assert_eq!(h.prev("current"), Some("c"));
        assert_eq!(h.prev("current"), Some("b"));
        assert_eq!(h.next(), Some("c".to_string()));
        assert_eq!(h.next(), Some("current".to_string()));
    }

    #[test]
    fn test_exit_browse() {
        let mut h = History::new(10);
        h.push("first".to_string());
        h.prev("current");
        assert!(h.is_browsing());
        h.exit_browse();
        assert!(!h.is_browsing());
    }
}
