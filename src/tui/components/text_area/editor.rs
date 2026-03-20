//! Text editing operations for TextArea
//!
//! Contains methods for inserting and deleting text.

use super::TextArea;

impl TextArea {
    /// Insert a character at cursor position
    pub(super) fn insert_char(&mut self, ch: char) {
        // Check total length limit
        if let Some(max_length) = self.max_length {
            let total_len = self.text().len() + ch.len_utf8();
            if total_len > max_length {
                return;
            }
        }

        // Ensure cursor_row is valid
        self.ensure_valid_cursor_row();

        let line = &mut self.lines[self.cursor_row];
        if self.cursor_col <= line.len() {
            line.insert(self.cursor_col, ch);
            self.cursor_col += ch.len_utf8();
        }
    }

    /// Insert a newline at cursor position
    pub(super) fn insert_newline(&mut self) {
        if let Some(max_lines) = self.max_lines {
            if self.lines.len() >= max_lines {
                return; // Reached max lines limit
            }
        }

        // Ensure cursor_row is valid
        self.ensure_valid_cursor_row();

        let current_line = &mut self.lines[self.cursor_row];
        let new_part = current_line.split_off(self.cursor_col);
        self.lines.insert(self.cursor_row + 1, new_part);
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    /// Delete character before cursor (backspace)
    pub(super) fn backspace(&mut self) {
        if self.cursor_col > 0 {
            // Delete character in current line
            self.ensure_valid_cursor_row();

            let line = &mut self.lines[self.cursor_row];
            if self.cursor_col <= line.len() {
                // Find previous character position
                let prev_col = line[..self.cursor_col]
                    .char_indices()
                    .next_back()
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                line.remove(prev_col);
                self.cursor_col = prev_col;
            }
        } else if self.cursor_row > 0 {
            // Cursor at line start, merge with previous line
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current_line);
        }
    }

    /// Delete character after cursor (delete key)
    pub(super) fn delete(&mut self) {
        self.ensure_valid_cursor_row();

        let current_line = &self.lines[self.cursor_row];
        if self.cursor_col < current_line.len() {
            // Delete character in current line
            let line = &mut self.lines[self.cursor_row];
            if self.cursor_col < line.len() {
                line.remove(self.cursor_col);
            }
        } else if self.cursor_row < self.lines.len() - 1 {
            // Cursor at line end, merge with next line
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
        }
    }
}
