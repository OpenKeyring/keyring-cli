//! Cursor movement operations for TextArea
//!
//! Contains methods for moving the cursor within the text.

use super::TextArea;

impl TextArea {
    /// Move cursor up one line
    pub(super) fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            // Keep column position, but don't exceed new line length
            self.cursor_col = std::cmp::min(self.cursor_col, self.lines[self.cursor_row].len());
        }
    }

    /// Move cursor down one line
    pub(super) fn move_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            // Keep column position, but don't exceed new line length
            self.cursor_col = std::cmp::min(self.cursor_col, self.lines[self.cursor_row].len());
        }
    }

    /// Move cursor left one character
    pub(super) fn move_left(&mut self) {
        if self.cursor_col > 0 {
            // Find previous character position
            self.ensure_valid_cursor_row();

            let line = &self.lines[self.cursor_row];
            let prev_col = line[..self.cursor_col]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.cursor_col = prev_col;
        } else if self.cursor_row > 0 {
            // Move to end of previous line
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    /// Move cursor right one character
    pub(super) fn move_right(&mut self) {
        self.ensure_valid_cursor_row();

        let line = &self.lines[self.cursor_row];
        if self.cursor_col < line.len() {
            // Move to next character position
            let next_col = line[self.cursor_col..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_col + i)
                .unwrap_or(line.len());
            self.cursor_col = next_col;
        } else if self.cursor_row < self.lines.len() - 1 {
            // Move to start of next line
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    /// Move cursor to start of line
    pub(super) fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    /// Move cursor to end of line
    pub(super) fn move_end(&mut self) {
        self.ensure_valid_cursor_row();
        self.cursor_col = self.lines[self.cursor_row].len();
    }

    /// Move cursor to document start
    pub(super) fn move_document_start(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    /// Move cursor to document end
    pub(super) fn move_document_end(&mut self) {
        self.cursor_row = self.lines.len() - 1;
        self.cursor_col = self.lines[self.cursor_row].len();
        // Scroll position will be adjusted during render
    }

    /// Page up - move cursor up by half a page
    pub(super) fn page_up(&mut self) {
        let page_size = (self.lines.len() / 2).max(1);
        self.cursor_row = self.cursor_row.saturating_sub(page_size);
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
        self.cursor_col = 0;
    }

    /// Page down - move cursor down by half a page
    pub(super) fn page_down(&mut self) {
        let page_size = (self.lines.len() / 2).max(1);
        self.cursor_row = std::cmp::min(self.cursor_row + page_size, self.lines.len() - 1);
        self.cursor_col = 0;
    }

    /// Scroll up one line (Ctrl+Up)
    pub(super) fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scroll down one line (Ctrl+Down)
    pub(super) fn scroll_down(&mut self) {
        let max_scroll = self.lines.len().saturating_sub(10);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }
}
