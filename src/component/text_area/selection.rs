/// Selection helpers for TextAreaState.
///
/// These are implementation details extracted to keep
/// the main module under the 1000-line limit.
use super::TextAreaState;

impl TextAreaState {
    /// Returns true if there is an active text selection.
    pub fn has_selection(&self) -> bool {
        match self.selection_anchor {
            Some((r, c)) => r != self.cursor_row || c != self.cursor_col,
            None => false,
        }
    }

    /// Returns the ordered selection positions as `((start_row, start_col), (end_row, end_col))`.
    pub fn selection_positions(&self) -> Option<((usize, usize), (usize, usize))> {
        let (ar, ac) = self.selection_anchor?;
        if ar == self.cursor_row && ac == self.cursor_col {
            return None;
        }
        let a = (ar, ac);
        let b = (self.cursor_row, self.cursor_col);
        if a < b { Some((a, b)) } else { Some((b, a)) }
    }

    /// Returns the selected text, or None if no selection.
    pub fn selected_text(&self) -> Option<String> {
        let ((sr, sc), (er, ec)) = self.selection_positions()?;
        if sr == er {
            Some(self.lines[sr][sc..ec].to_string())
        } else {
            let mut result = self.lines[sr][sc..].to_string();
            for row in (sr + 1)..er {
                result.push('\n');
                result.push_str(&self.lines[row]);
            }
            result.push('\n');
            result.push_str(&self.lines[er][..ec]);
            Some(result)
        }
    }

    /// Returns the internal clipboard contents.
    pub fn clipboard(&self) -> &str {
        &self.clipboard
    }

    pub(super) fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub(super) fn ensure_selection_anchor(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_row, self.cursor_col));
        }
    }

    /// Deletes selected text. Returns deleted text or None.
    pub(super) fn delete_selection(&mut self) -> Option<String> {
        let ((sr, sc), (er, ec)) = self.selection_positions()?;
        let deleted = self.selected_text()?;
        if sr == er {
            self.lines[sr].drain(sc..ec);
        } else {
            let after = self.lines[er][ec..].to_string();
            self.lines[sr].truncate(sc);
            self.lines[sr].push_str(&after);
            self.lines.drain((sr + 1)..=er);
        }
        self.cursor_row = sr;
        self.cursor_col = sc;
        self.selection_anchor = None;
        Some(deleted)
    }
}
