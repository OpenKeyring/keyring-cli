//! TUI Utilities
//!
//! Helper functions for TUI operations.

use ratatui::layout::Rect;

/// Calculate centered popup area
pub fn centered_popup(width: u16, height: u16, terminal_size: Rect) -> Rect {
    let x = (terminal_size.width.saturating_sub(width)) / 2;
    let y = (terminal_size.height.saturating_sub(height)) / 2;

    Rect::new(x, y, width, height)
}

/// Calculate popup area with percentage of terminal size
pub fn percentage_popup(width_percent: u16, height_percent: u16, terminal_size: Rect) -> Rect {
    let width = (terminal_size.width * width_percent) / 100;
    let height = (terminal_size.height * height_percent) / 100;
    centered_popup(width, height, terminal_size)
}

/// Truncate text to fit width with ellipsis
pub fn truncate_text(text: &str, width: usize) -> String {
    if text.len() <= width {
        return text.to_string();
    }

    if width <= 3 {
        "...".to_string()[..width].to_string()
    } else {
        format!("{}...", &text[..width - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_text_short() {
        assert_eq!(truncate_text("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_text_exact() {
        assert_eq!(truncate_text("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_text_long() {
        assert_eq!(truncate_text("hello world", 8), "hello...");
    }

    #[test]
    fn test_truncate_text_very_short() {
        assert_eq!(truncate_text("hello", 2), "..");
    }
}
