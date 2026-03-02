//! Undo/redo history stack for text editing components.
//!
//! Provides a generic [`UndoStack`] that tracks snapshots of component state
//! before text modifications. Consecutive edits of the same kind (e.g.,
//! character insertions) are grouped into a single undo step.

/// Identifies the kind of edit for grouping consecutive same-type operations.
///
/// Consecutive edits with the same `EditKind` are merged into a single undo
/// step. For example, typing "hello" produces one undo entry (not five).
/// [`EditKind::Other`] is never grouped — each such edit is its own step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditKind {
    /// Character insertion (non-whitespace).
    Insert,
    /// Character deletion (backspace, delete).
    Delete,
    /// Non-groupable operation (clear, set_value, paste, etc.).
    Other,
}

/// Generic undo/redo history stack.
///
/// Stores snapshots of state before modifications. Supports grouping of
/// consecutive same-kind edits and configurable maximum history depth.
#[derive(Debug, Clone)]
pub(crate) struct UndoStack<T> {
    undo: Vec<T>,
    redo: Vec<T>,
    max_size: usize,
    last_kind: Option<EditKind>,
}

impl<T> Default for UndoStack<T> {
    fn default() -> Self {
        Self::new(100)
    }
}

impl<T> UndoStack<T> {
    /// Creates a new undo stack with the given maximum history depth.
    pub fn new(max_size: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            max_size,
            last_kind: None,
        }
    }

    /// Returns true if there are entries to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Returns true if there are entries to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Saves a snapshot before a text modification.
    ///
    /// If `kind` matches the previous edit kind and is groupable (Insert or
    /// Delete), the snapshot is skipped (grouped with the previous entry).
    /// The redo stack is always cleared on new edits.
    pub fn save(&mut self, snapshot: T, kind: EditKind) {
        self.redo.clear();

        let should_push = match kind {
            EditKind::Other => true,
            _ => self.last_kind != Some(kind),
        };

        if should_push {
            self.undo.push(snapshot);
            self.enforce_limit();
        }

        self.last_kind = Some(kind);
    }

    /// Breaks the current edit group.
    ///
    /// The next call to [`save`](Self::save) will always create a new undo
    /// entry, even if the edit kind matches the previous one. Use this to
    /// create word boundaries (e.g., on whitespace insertion).
    pub fn break_group(&mut self) {
        self.last_kind = None;
    }

    /// Undoes the last edit group, returning the saved snapshot.
    ///
    /// The `current` state is pushed onto the redo stack before restoring.
    pub fn undo(&mut self, current: T) -> Option<T> {
        let snapshot = self.undo.pop()?;
        self.redo.push(current);
        self.last_kind = None;
        Some(snapshot)
    }

    /// Redoes the last undone edit, returning the saved snapshot.
    ///
    /// The `current` state is pushed onto the undo stack before restoring.
    pub fn redo(&mut self, current: T) -> Option<T> {
        let snapshot = self.redo.pop()?;
        self.undo.push(current);
        self.last_kind = None;
        Some(snapshot)
    }

    /// Clears all undo and redo history.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.last_kind = None;
    }

    fn enforce_limit(&mut self) {
        while self.undo.len() > self.max_size {
            self.undo.remove(0);
        }
    }
}

// PartialEq: undo history is not part of logical equality.
// Two states with different undo histories but same content are equal.
impl<T> PartialEq for UndoStack<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack_empty() {
        let stack: UndoStack<String> = UndoStack::new(100);
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_default_stack() {
        let stack: UndoStack<String> = UndoStack::default();
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_save_and_undo() {
        let mut stack = UndoStack::new(100);
        stack.save("before".to_string(), EditKind::Other);
        assert!(stack.can_undo());

        let restored = stack.undo("after".to_string());
        assert_eq!(restored, Some("before".to_string()));
        assert!(!stack.can_undo());
        assert!(stack.can_redo());
    }

    #[test]
    fn test_undo_then_redo() {
        let mut stack = UndoStack::new(100);
        stack.save("initial".to_string(), EditKind::Other);

        let restored = stack.undo("modified".to_string()).unwrap();
        assert_eq!(restored, "initial");

        let redone = stack.redo("initial".to_string()).unwrap();
        assert_eq!(redone, "modified");
    }

    #[test]
    fn test_undo_empty_returns_none() {
        let mut stack: UndoStack<String> = UndoStack::new(100);
        assert_eq!(stack.undo("current".to_string()), None);
    }

    #[test]
    fn test_redo_empty_returns_none() {
        let mut stack: UndoStack<String> = UndoStack::new(100);
        assert_eq!(stack.redo("current".to_string()), None);
    }

    #[test]
    fn test_grouping_same_insert_kind() {
        let mut stack = UndoStack::new(100);
        // Multiple Insert saves with same kind → only first is saved
        stack.save("state0".to_string(), EditKind::Insert);
        stack.save("state1".to_string(), EditKind::Insert);
        stack.save("state2".to_string(), EditKind::Insert);

        // Only one undo entry (the first save)
        let restored = stack.undo("state3".to_string());
        assert_eq!(restored, Some("state0".to_string()));
        assert!(!stack.can_undo());
    }

    #[test]
    fn test_grouping_same_delete_kind() {
        let mut stack = UndoStack::new(100);
        stack.save("state0".to_string(), EditKind::Delete);
        stack.save("state1".to_string(), EditKind::Delete);

        let restored = stack.undo("state2".to_string());
        assert_eq!(restored, Some("state0".to_string()));
        assert!(!stack.can_undo());
    }

    #[test]
    fn test_other_never_grouped() {
        let mut stack = UndoStack::new(100);
        stack.save("state0".to_string(), EditKind::Other);
        stack.save("state1".to_string(), EditKind::Other);
        stack.save("state2".to_string(), EditKind::Other);

        // Three separate undo entries
        assert_eq!(stack.undo("state3".to_string()), Some("state2".to_string()));
        assert_eq!(stack.undo("state2".to_string()), Some("state1".to_string()));
        assert_eq!(stack.undo("state1".to_string()), Some("state0".to_string()));
        assert!(!stack.can_undo());
    }

    #[test]
    fn test_kind_change_breaks_group() {
        let mut stack = UndoStack::new(100);
        stack.save("empty".to_string(), EditKind::Insert);
        stack.save("h".to_string(), EditKind::Insert); // grouped
        stack.save("he".to_string(), EditKind::Delete); // new group (kind changed)

        // Two undo entries
        assert_eq!(stack.undo("h".to_string()), Some("he".to_string()));
        assert_eq!(stack.undo("he".to_string()), Some("empty".to_string()));
        assert!(!stack.can_undo());
    }

    #[test]
    fn test_break_group() {
        let mut stack = UndoStack::new(100);
        stack.save("state0".to_string(), EditKind::Insert);
        stack.break_group();
        stack.save("state1".to_string(), EditKind::Insert); // new group despite same kind

        // Two separate undo entries
        assert_eq!(stack.undo("state2".to_string()), Some("state1".to_string()));
        assert_eq!(stack.undo("state1".to_string()), Some("state0".to_string()));
    }

    #[test]
    fn test_new_edit_clears_redo() {
        let mut stack = UndoStack::new(100);
        stack.save("state0".to_string(), EditKind::Other);
        stack.undo("state1".to_string());
        assert!(stack.can_redo());

        // New edit clears redo stack
        stack.save("state0_again".to_string(), EditKind::Other);
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_max_size_enforced() {
        let mut stack = UndoStack::new(3);
        stack.save("a".to_string(), EditKind::Other);
        stack.save("b".to_string(), EditKind::Other);
        stack.save("c".to_string(), EditKind::Other);
        stack.save("d".to_string(), EditKind::Other);

        // Oldest entry ("a") dropped, 3 remain
        assert_eq!(stack.undo("e".to_string()), Some("d".to_string()));
        assert_eq!(stack.undo("d".to_string()), Some("c".to_string()));
        assert_eq!(stack.undo("c".to_string()), Some("b".to_string()));
        assert!(!stack.can_undo()); // "a" was dropped
    }

    #[test]
    fn test_clear() {
        let mut stack = UndoStack::new(100);
        stack.save("state0".to_string(), EditKind::Other);
        stack.undo("state1".to_string());
        stack.clear();
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_multiple_undo_redo_cycles() {
        let mut stack = UndoStack::new(100);
        stack.save("v0".to_string(), EditKind::Other);
        stack.save("v1".to_string(), EditKind::Other);

        // Undo twice
        let r1 = stack.undo("v2".to_string()).unwrap();
        assert_eq!(r1, "v1");
        let r2 = stack.undo("v1".to_string()).unwrap();
        assert_eq!(r2, "v0");

        // Redo twice
        let f1 = stack.redo("v0".to_string()).unwrap();
        assert_eq!(f1, "v1");
        let f2 = stack.redo("v1".to_string()).unwrap();
        assert_eq!(f2, "v2");
    }

    #[test]
    fn test_undo_resets_last_kind() {
        let mut stack = UndoStack::new(100);
        stack.save("before_insert".to_string(), EditKind::Insert);

        // Undo resets kind tracking
        stack.undo("after_insert".to_string());

        // Next Insert should create a new group (not be skipped)
        stack.save("new_insert".to_string(), EditKind::Insert);
        assert!(stack.can_undo());
    }

    #[test]
    fn test_partial_eq_always_equal() {
        let mut stack1: UndoStack<String> = UndoStack::new(100);
        stack1.save("a".to_string(), EditKind::Other);

        let stack2: UndoStack<String> = UndoStack::new(50);
        assert_eq!(stack1, stack2);
    }
}
