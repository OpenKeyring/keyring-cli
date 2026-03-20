//! TextArea Component
//!
//! Multi-line text input component with scrolling and editing capabilities.

mod cursor;
mod editor;
mod handlers;
mod render;
#[cfg(test)]
mod tests;

use crate::tui::traits::{ComponentId, FieldValidation, ValidationResult};

/// TextArea component
///
/// Features:
/// - Multi-line text input and editing
/// - Vertical scrolling
/// - Cursor movement
/// - Placeholder display
/// - Focus state styling
/// - Validation support
pub struct TextArea {
    /// Component ID
    pub(super) id: ComponentId,
    /// Current lines (multi-line text)
    pub(super) lines: Vec<String>,
    /// Cursor position (row)
    pub(super) cursor_row: usize,
    /// Cursor column
    pub(super) cursor_col: usize,
    /// Vertical scroll offset
    pub(super) scroll_offset: usize,
    /// Maximum lines limit
    pub(super) max_lines: Option<usize>,
    /// Maximum length limit
    pub(super) max_length: Option<usize>,
    /// Placeholder text
    pub(super) placeholder: String,
    /// Whether focused
    pub(super) focused: bool,
    /// Validation configuration
    pub(super) validation: Option<FieldValidation>,
    /// Validation result
    pub(super) validation_result: Option<ValidationResult>,
    /// Border title
    pub(super) title: String,
}

impl TextArea {
    /// Create a new text area
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            id: ComponentId::new(0),
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            max_lines: None,
            max_length: None,
            placeholder: placeholder.into(),
            focused: false,
            validation: None,
            validation_result: Some(ValidationResult::valid()),
            title: String::new(),
        }
    }

    /// Set component ID
    #[must_use]
    pub fn with_id(mut self, id: ComponentId) -> Self {
        self.id = id;
        self
    }

    /// Set border title
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set maximum lines
    #[must_use]
    pub fn with_max_lines(mut self, max: usize) -> Self {
        self.max_lines = Some(max);
        self
    }

    /// Set maximum length
    #[must_use]
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Set validation configuration
    #[must_use]
    pub fn with_validation(mut self, validation: FieldValidation) -> Self {
        self.validation = Some(validation);
        self
    }

    /// Get all text content
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Set text content
    pub fn set_text(&mut self, text: String) {
        self.lines = text.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        // Move cursor to end
        self.cursor_row = self.lines.len() - 1;
        self.cursor_col = self.lines[self.cursor_row].len();

        // Ensure we don't exceed max lines
        if let Some(max_lines) = self.max_lines {
            if self.lines.len() > max_lines {
                self.lines.truncate(max_lines);
                if self.cursor_row >= max_lines {
                    self.cursor_row = max_lines - 1;
                    self.cursor_col = self.lines[self.cursor_row].len();
                }
            }
        }
    }

    /// Get placeholder
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Clear text
    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    /// Get current line text
    pub(super) fn current_line(&self) -> &str {
        if self.cursor_row < self.lines.len() {
            &self.lines[self.cursor_row]
        } else {
            ""
        }
    }

    /// Get mutable reference to current line
    pub(super) fn current_line_mut(&mut self) -> &mut String {
        if self.cursor_row >= self.lines.len() {
            self.lines.resize(self.cursor_row + 1, String::new());
        }
        &mut self.lines[self.cursor_row]
    }

    /// Ensure cursor_row is valid
    pub(super) fn ensure_valid_cursor_row(&mut self) {
        if self.cursor_row >= self.lines.len() {
            self.lines.resize(self.cursor_row + 1, String::new());
        }
    }

    /// Validate current text
    pub(super) fn validate(&self) -> ValidationResult {
        if let Some(ref validation) = self.validation {
            validation.validate(&self.text())
        } else {
            ValidationResult::valid()
        }
    }

    /// Update scroll offset to keep cursor visible
    pub(super) fn update_scroll_if_needed(&mut self, area: ratatui::layout::Rect) {
        let visible_start_row = self.scroll_offset;
        let visible_end_row = self.scroll_offset + area.height as usize - 2; // -2 for borders

        if self.cursor_row < visible_start_row {
            self.scroll_offset = self.cursor_row;
        } else if self.cursor_row >= visible_end_row {
            self.scroll_offset = self.cursor_row - (area.height as usize - 3);
        }
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new("")
    }
}
