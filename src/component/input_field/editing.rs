/// Cursor movement and text editing helpers for InputFieldState.
///
/// These are private implementation details extracted to keep
/// the main module under the 1000-line limit.
use super::InputFieldState;

impl InputFieldState {
    /// Move cursor left by one character.
    pub(super) fn move_left(&mut self) {
        if self.cursor > 0 {
            // Find the previous character boundary
            self.cursor = self.value[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right by one character.
    pub(super) fn move_right(&mut self) {
        if self.cursor < self.value.len() {
            // Find the next character boundary
            self.cursor = self.value[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.value.len());
        }
    }

    /// Move cursor to the start of the previous word.
    pub(super) fn move_word_left(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let before = &self.value[..self.cursor];
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

        self.cursor = chars.get(idx).map(|(i, _)| *i).unwrap_or(0);
    }

    /// Move cursor to the end of the next word.
    pub(super) fn move_word_right(&mut self) {
        if self.cursor >= self.value.len() {
            return;
        }

        let after = &self.value[self.cursor..];
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

        self.cursor = chars
            .get(idx)
            .map(|(i, _)| self.cursor + *i)
            .unwrap_or(self.value.len());
    }

    /// Insert a character at the cursor position.
    pub(super) fn insert(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete the character before the cursor.
    pub(super) fn backspace(&mut self) -> bool {
        if self.cursor > 0 {
            let prev_cursor = self.cursor;
            self.move_left();
            self.value.drain(self.cursor..prev_cursor);
            true
        } else {
            false
        }
    }

    /// Delete the character at the cursor.
    pub(super) fn delete(&mut self) -> bool {
        if self.cursor < self.value.len() {
            let next = self.value[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.value.len());
            self.value.drain(self.cursor..next);
            true
        } else {
            false
        }
    }

    /// Delete from cursor back to start of word.
    pub(super) fn delete_word_back(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }

        let end = self.cursor;
        self.move_word_left();
        self.value.drain(self.cursor..end);
        true
    }

    /// Delete from cursor forward to end of word.
    pub(super) fn delete_word_forward(&mut self) -> bool {
        if self.cursor >= self.value.len() {
            return false;
        }

        let start = self.cursor;
        self.move_word_right();
        let end = self.cursor;
        self.cursor = start;
        self.value.drain(start..end);
        true
    }
}
