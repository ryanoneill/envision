//! Node search state machine for the Diagram component.
//!
//! When the user presses `/`, the diagram enters search mode. As they
//! type, matching nodes are highlighted. Enter jumps to the current
//! match, `n`/`N` cycle through matches, Escape cancels.

use super::types::DiagramNode;

/// Search state for the diagram.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SearchState {
    /// Whether search mode is active.
    pub(crate) active: bool,
    /// Current search query text.
    pub(crate) query: String,
    /// Indices of nodes matching the query.
    pub(crate) matches: Vec<usize>,
    /// Index into the `matches` vec for the current highlight.
    pub(crate) current_match: usize,
}

impl SearchState {
    /// Activates search mode.
    pub(crate) fn start(&mut self) {
        self.active = true;
        self.query.clear();
        self.matches.clear();
        self.current_match = 0;
    }

    /// Deactivates search mode and clears the query.
    pub(crate) fn cancel(&mut self) {
        self.active = false;
        self.query.clear();
        self.matches.clear();
        self.current_match = 0;
    }

    /// Appends a character to the query and recomputes matches.
    pub(crate) fn input(&mut self, ch: char, nodes: &[DiagramNode]) {
        self.query.push(ch);
        self.recompute_matches(nodes);
    }

    /// Removes the last character from the query and recomputes matches.
    pub(crate) fn backspace(&mut self, nodes: &[DiagramNode]) {
        self.query.pop();
        self.recompute_matches(nodes);
    }

    /// Advances to the next match.
    pub(crate) fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = (self.current_match + 1) % self.matches.len();
        }
    }

    /// Moves to the previous match.
    pub(crate) fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = if self.current_match == 0 {
                self.matches.len() - 1
            } else {
                self.current_match - 1
            };
        }
    }

    /// Returns the node index of the current match, if any.
    pub(crate) fn current_node_index(&self) -> Option<usize> {
        self.matches.get(self.current_match).copied()
    }

    /// Recomputes matches by searching node IDs and labels
    /// (case-insensitive substring match).
    fn recompute_matches(&mut self, nodes: &[DiagramNode]) {
        let query_lower = self.query.to_lowercase();
        self.matches = nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                if query_lower.is_empty() {
                    return false;
                }
                node.id().to_lowercase().contains(&query_lower)
                    || node.label().to_lowercase().contains(&query_lower)
            })
            .map(|(idx, _)| idx)
            .collect();

        // Clamp current_match to valid range
        if self.current_match >= self.matches.len() {
            self.current_match = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nodes() -> Vec<DiagramNode> {
        vec![
            DiagramNode::new("api", "API Gateway"),
            DiagramNode::new("auth", "Auth Service"),
            DiagramNode::new("db", "Database"),
            DiagramNode::new("cache", "Redis Cache"),
        ]
    }

    #[test]
    fn test_start_and_cancel() {
        let mut search = SearchState::default();
        assert!(!search.active);

        search.start();
        assert!(search.active);
        assert!(search.query.is_empty());

        search.cancel();
        assert!(!search.active);
    }

    #[test]
    fn test_search_by_label() {
        let nodes = make_nodes();
        let mut search = SearchState::default();
        search.start();

        search.input('a', &nodes);
        search.input('p', &nodes);
        search.input('i', &nodes);

        // "api" matches both id "api" and label "API Gateway"
        assert_eq!(search.matches.len(), 1);
        assert_eq!(search.matches[0], 0);
    }

    #[test]
    fn test_search_case_insensitive() {
        let nodes = make_nodes();
        let mut search = SearchState::default();
        search.start();

        search.input('D', &nodes);
        search.input('a', &nodes);
        search.input('t', &nodes);

        // "Dat" matches "Database" (case insensitive)
        assert_eq!(search.matches.len(), 1);
        assert_eq!(search.matches[0], 2);
    }

    #[test]
    fn test_search_multiple_matches() {
        let nodes = make_nodes();
        let mut search = SearchState::default();
        search.start();

        search.input('a', &nodes);

        // "a" matches: "api"/"API Gateway", "auth"/"Auth Service",
        // "db"/"Database", "cache"/"Redis Cache"
        assert!(search.matches.len() >= 3);
    }

    #[test]
    fn test_next_prev_match() {
        let nodes = make_nodes();
        let mut search = SearchState::default();
        search.start();
        search.input('a', &nodes);

        let first = search.current_node_index();
        search.next_match();
        let second = search.current_node_index();
        assert_ne!(first, second);

        search.prev_match();
        assert_eq!(search.current_node_index(), first);
    }

    #[test]
    fn test_backspace() {
        let nodes = make_nodes();
        let mut search = SearchState::default();
        search.start();

        search.input('a', &nodes);
        search.input('p', &nodes);
        search.input('i', &nodes);
        assert_eq!(search.matches.len(), 1);

        search.backspace(&nodes);
        // "ap" still matches "api"/"API Gateway"
        assert!(search.matches.len() >= 1);

        search.backspace(&nodes);
        search.backspace(&nodes);
        // Empty query matches nothing
        assert!(search.matches.is_empty());
    }

    #[test]
    fn test_match_contains() {
        let nodes = make_nodes();
        let mut search = SearchState::default();
        search.start();
        search.input('d', &nodes);
        search.input('b', &nodes);

        assert!(search.matches.contains(&2)); // "db" matches
        assert!(!search.matches.contains(&0)); // "api" doesn't
    }

    #[test]
    fn test_empty_query_no_matches() {
        let nodes = make_nodes();
        let mut search = SearchState::default();
        search.start();

        assert!(search.matches.is_empty());
        assert_eq!(search.current_node_index(), None);
    }

    #[test]
    fn test_next_wraps_around() {
        let nodes = make_nodes();
        let mut search = SearchState::default();
        search.start();
        search.input('a', &nodes);

        let count = search.matches.len();
        for _ in 0..count {
            search.next_match();
        }
        // Should have wrapped back to 0
        assert_eq!(search.current_match, 0);
    }
}
