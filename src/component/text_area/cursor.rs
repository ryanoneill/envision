/// Cursor movement and text editing helpers for TextAreaState.
///
/// These are private implementation details extracted to keep
/// the main module under the 1000-line limit.
use super::TextAreaState;

impl TextAreaState {
    /// Insert a character at the cursor position.
    pub(super) fn insert(&mut self, c: char) {
        self.lines[self.cursor_row].insert(self.cursor_col, c);
        self.cursor_col += c.len_utf8();
    }

    /// Insert a newline at the cursor position.
    pub(super) fn new_line(&mut self) {
        let remainder = self.lines[self.cursor_row].split_off(self.cursor_col);
        self.lines.insert(self.cursor_row + 1, remainder);
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    /// Delete the character before the cursor.
    pub(super) fn backspace(&mut self) -> bool {
        if self.cursor_col > 0 {
            // Find previous character boundary
            let prev_cursor = self.cursor_col;
            self.cursor_col = self.lines[self.cursor_row][..self.cursor_col]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.lines[self.cursor_row].drain(self.cursor_col..prev_cursor);
            true
        } else if self.cursor_row > 0 {
            // Join with previous line
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current_line);
            true
        } else {
            false
        }
    }

    /// Delete the character at the cursor.
    pub(super) fn delete(&mut self) -> bool {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            // Find next character boundary
            let next = self.lines[self.cursor_row][self.cursor_col..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_col + i)
                .unwrap_or(line_len);
            self.lines[self.cursor_row].drain(self.cursor_col..next);
            true
        } else if self.cursor_row < self.lines.len() - 1 {
            // Join with next line
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
            true
        } else {
            false
        }
    }

    /// Move cursor left by one character.
    pub(super) fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col = self.lines[self.cursor_row][..self.cursor_col]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        } else if self.cursor_row > 0 {
            // Wrap to end of previous line
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    /// Move cursor right by one character.
    pub(super) fn move_right(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            self.cursor_col = self.lines[self.cursor_row][self.cursor_col..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_col + i)
                .unwrap_or(line_len);
        } else if self.cursor_row < self.lines.len() - 1 {
            // Wrap to start of next line
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    /// Move cursor up by one line.
    pub(super) fn move_up(&mut self) {
        if self.cursor_row > 0 {
            // Remember char position, not byte position
            let char_pos = self.lines[self.cursor_row][..self.cursor_col]
                .chars()
                .count();
            self.cursor_row -= 1;
            // Restore to same char position (clamped to line length)
            let line = &self.lines[self.cursor_row];
            let char_count = line.chars().count();
            let target_pos = char_pos.min(char_count);
            self.cursor_col = line
                .char_indices()
                .nth(target_pos)
                .map(|(i, _)| i)
                .unwrap_or(line.len());
        }
    }

    /// Move cursor down by one line.
    pub(super) fn move_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            // Remember char position, not byte position
            let char_pos = self.lines[self.cursor_row][..self.cursor_col]
                .chars()
                .count();
            self.cursor_row += 1;
            // Restore to same char position (clamped to line length)
            let line = &self.lines[self.cursor_row];
            let char_count = line.chars().count();
            let target_pos = char_pos.min(char_count);
            self.cursor_col = line
                .char_indices()
                .nth(target_pos)
                .map(|(i, _)| i)
                .unwrap_or(line.len());
        }
    }

    /// Move cursor to the start of the previous word.
    pub(super) fn move_word_left(&mut self) {
        if self.cursor_col == 0 {
            // If at line start, move to previous line end
            if self.cursor_row > 0 {
                self.cursor_row -= 1;
                self.cursor_col = self.lines[self.cursor_row].len();
            }
            return;
        }

        let before = &self.lines[self.cursor_row][..self.cursor_col];
        let chars: Vec<(usize, char)> = before.char_indices().collect();

        // Skip trailing whitespace
        let mut idx = chars.len() - 1;
        while idx > 0 && chars[idx].1.is_whitespace() {
            idx -= 1;
        }

        // Skip word characters
        while idx > 0 && !chars[idx - 1].1.is_whitespace() {
            idx -= 1;
        }

        self.cursor_col = chars.get(idx).map(|(i, _)| *i).unwrap_or(0);
    }

    /// Move cursor to the end of the next word.
    pub(super) fn move_word_right(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col >= line_len {
            // If at line end, move to next line start
            if self.cursor_row < self.lines.len() - 1 {
                self.cursor_row += 1;
                self.cursor_col = 0;
            }
            return;
        }

        let after = &self.lines[self.cursor_row][self.cursor_col..];
        let chars: Vec<(usize, char)> = after.char_indices().collect();

        // Skip leading non-whitespace
        let mut idx = 0;
        while idx < chars.len() && !chars[idx].1.is_whitespace() {
            idx += 1;
        }

        // Skip whitespace
        while idx < chars.len() && chars[idx].1.is_whitespace() {
            idx += 1;
        }

        self.cursor_col = chars
            .get(idx)
            .map(|(i, _)| self.cursor_col + *i)
            .unwrap_or(line_len);
    }

    /// Delete the entire current line.
    pub(super) fn delete_line(&mut self) -> bool {
        if self.lines.len() > 1 {
            self.lines.remove(self.cursor_row);
            if self.cursor_row >= self.lines.len() {
                self.cursor_row = self.lines.len() - 1;
            }
            // Clamp cursor column to a valid char boundary.
            // Simply clamping to line_len is insufficient because that
            // could leave cursor_col in the middle of a multi-byte character.
            self.clamp_cursor_col();
            true
        } else {
            // Single line: just clear it
            if !self.lines[0].is_empty() {
                self.lines[0].clear();
                self.cursor_col = 0;
                true
            } else {
                false
            }
        }
    }

    /// Delete from cursor to end of line.
    pub(super) fn delete_to_end(&mut self) -> bool {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            self.lines[self.cursor_row].truncate(self.cursor_col);
            true
        } else {
            false
        }
    }

    /// Delete from cursor to beginning of line.
    pub(super) fn delete_to_start(&mut self) -> bool {
        if self.cursor_col > 0 {
            self.lines[self.cursor_row].drain(..self.cursor_col);
            self.cursor_col = 0;
            true
        } else {
            false
        }
    }

    /// Clamp cursor_col to a valid char boundary on the current line.
    ///
    /// After operations that change the current line (e.g., delete_line),
    /// cursor_col may no longer be on a char boundary. This method finds
    /// the nearest valid boundary at or before cursor_col.
    fn clamp_cursor_col(&mut self) {
        let line = &self.lines[self.cursor_row];
        let line_len = line.len();

        if self.cursor_col >= line_len {
            self.cursor_col = line_len;
            return;
        }

        // If already on a boundary, nothing to do
        if line.is_char_boundary(self.cursor_col) {
            return;
        }

        // Walk backwards to find the nearest char boundary
        let mut col = self.cursor_col;
        while col > 0 && !line.is_char_boundary(col) {
            col -= 1;
        }
        self.cursor_col = col;
    }
}
