//! TUI Testing Utilities
//!
//! This module provides helper functions and structures for testing TUI components
//! using snapshot testing with `insta` and ratatui's `TestBackend`.
//!
//! # Usage
//!
//! ```rust
//! use crate::tui::testing::{render_snapshot, SnapshotSequence};
//!
//! // Single snapshot
//! let output = render_snapshot(80, 24, |frame| {
//!     screen.render(frame, frame.area());
//! });
//! insta::assert_snapshot!(output);
//!
//! // Multi-step sequence
//! let mut seq = SnapshotSequence::new("my_test");
//! seq.step("initial", render_snapshot(80, 24, |f| app.render(f)));
//! app.handle_input('\n');
//! seq.step("after_enter", render_snapshot(80, 24, |f| app.render(f)));
//! ```

use ratatui::{
    backend::TestBackend,
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
    Terminal,
};

/// Snapshot normalizer for handling dynamic content in TUI output.
///
/// Use this to normalize timestamps, UUIDs, and other dynamic values
/// before snapshotting to reduce noise in test failures.
#[derive(Debug, Default)]
pub struct SnapshotNormalizer {
    replacements: Vec<(regex::Regex, String)>,
}

impl SnapshotNormalizer {
    /// Create a new normalizer with default replacements.
    pub fn new() -> Self {
        let mut normalizer = Self::default();

        // Timestamps
        normalizer.add_replace(
            r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}",
            "[TIMESTAMP]",
        );

        // UUIDs
        normalizer.add_replace(
            r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}",
            "[UUID]",
        );

        // Duration strings
        normalizer.add_replace(r"\d+[mhd]\s+ago", "[DURATION]");

        // File sizes
        normalizer.add_replace(r"\d+\.?\d*\s*(KB|MB|GB)", "[FILESIZE]");

        normalizer
    }

    /// Add a custom regex replacement pattern.
    pub fn add_replace(&mut self, pattern: &str, replacement: &str) -> &mut Self {
        self.replacements.push((
            regex::Regex::new(pattern).expect("Invalid regex pattern"),
            replacement.to_string(),
        ));
        self
    }

    /// Normalize the input string by applying all replacement patterns.
    pub fn normalize(&self, input: &str) -> String {
        let mut result = input.to_string();
        for (regex, replacement) in &self.replacements {
            result = regex.replace_all(&result, replacement.as_str()).to_string();
        }
        result
    }
}

/// Manages a sequence of snapshots for multi-step interaction tests.
///
/// This is useful for testing user flows like command execution, form input,
/// or screen navigation where you want to capture state at each step.
#[derive(Debug)]
pub struct SnapshotSequence {
    name: String,
    steps: Vec<(String, String)>,
    normalizer: SnapshotNormalizer,
}

impl SnapshotSequence {
    /// Create a new snapshot sequence.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
            normalizer: SnapshotNormalizer::new(),
        }
    }

    /// Set a custom normalizer for this sequence.
    pub fn with_normalizer(mut self, normalizer: SnapshotNormalizer) -> Self {
        self.normalizer = normalizer;
        self
    }

    /// Add a snapshot step.
    pub fn step(&mut self, label: impl Into<String>, snapshot: String) -> &mut Self {
        let normalized = self.normalizer.normalize(&snapshot);
        self.steps.push((label.into(), normalized));
        self
    }

    /// Convert the entire sequence to a string for snapshotting.
    ///
    /// Example output:
    /// ```text
    /// Sequence: my_test
    ///
    /// Step 1: initial
    /// ──────────────
    /// <snapshot content>
    ///
    /// Step 2: after_input
    /// ───────────────────
    /// <snapshot content>
    /// ```
    pub fn to_string(&self) -> String {
        let mut output = format!("Sequence: {}\n\n", self.name);

        for (i, (label, snapshot)) in self.steps.iter().enumerate() {
            output.push_str(&format!("Step {}: {}\n", i + 1, label));
            output.push_str(&"─".repeat(label.len() + 7));
            output.push('\n');
            output.push_str(snapshot);
            output.push_str("\n\n");
        }

        output
    }
}

/// Render a widget or closure to a string for snapshot testing.
///
/// # Arguments
///
/// * `width` - Terminal width in columns
/// * `height` - Terminal height in rows
/// * `render_fn` - Closure that renders content to the frame
///
/// # Example
///
/// ```rust
/// let screen = WelcomeScreen::new();
/// let output = render_snapshot(80, 24, |frame| {
///     screen.render(frame, frame.area());
/// });
/// insta::assert_snapshot!(output);
/// ```
pub fn render_snapshot<F>(width: u16, height: u16, render_fn: F) -> String
where
    F: FnOnce(&mut ratatui::Frame),
{
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

    terminal
        .draw(|f| render_fn(f))
        .expect("Failed to draw to terminal");

    buffer_to_string(terminal.backend().buffer())
}

/// Convert a ratatui Buffer to a string representation.
///
/// This extracts the visible content from the buffer, handling colors
/// and modifiers to produce a readable text representation.
pub fn buffer_to_string(buffer: &Buffer) -> String {
    let area = buffer.area();
    let mut output = String::new();

    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = buffer.cell((x, y)).unwrap();
            output.push_str(cell.symbol());
        }
        if y < area.bottom() - 1 {
            output.push('\n');
        }
    }

    // Trim trailing whitespace from each line
    output
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Create a test frame with the given dimensions.
///
/// This is useful for unit tests that need a frame but don't need
/// the full terminal abstraction.
///
/// Note: In ratatui 0.28+, Frame cannot be constructed directly.
/// Use `render_snapshot` for most testing scenarios instead.
pub fn test_frame(width: u16, height: u16) -> (Buffer, Rect) {
    let backend = TestBackend::new(width, height);
    let buffer = backend.buffer().clone();
    let area = Rect::new(0, 0, width, height);
    (buffer, area)
}

/// Assert that two buffers are equal.
///
/// This is useful for testing that a render produces the exact expected output.
pub fn assert_buffer_eq(actual: &Buffer, expected: &Buffer) {
    let actual_str = buffer_to_string(actual);
    let expected_str = buffer_to_string(expected);

    assert_eq!(
        actual_str, expected_str,
        "Buffers differ:\nExpected:\n{}\n\nActual:\n{}",
        expected_str, actual_str
    );
}

/// Extract plain text content from a buffer, stripping all styling.
///
/// This returns only the text characters without any color or modifier information.
pub fn buffer_text(buffer: &Buffer) -> String {
    buffer_to_string(buffer)
}

/// Find the first line containing the specified text in a buffer.
///
/// Returns the line content if found, None otherwise.
///
/// # Example
///
/// ```rust
/// let buffer = render_to_buffer(...);
/// if let Some(line) = find_line_with_text(&buffer, "Password:") {
///     assert!(line.contains("Password:"));
/// }
/// ```
pub fn find_line_with_text(buffer: &Buffer, text: &str) -> Option<String> {
    let content = buffer_text(buffer);
    content
        .lines()
        .find(|line| line.contains(text))
        .map(|line| line.to_string())
}

/// Find all lines containing the specified text in a buffer.
///
/// Returns a vector of matching lines.
pub fn find_lines_with_text(buffer: &Buffer, text: &str) -> Vec<String> {
    let content = buffer_text(buffer);
    content
        .lines()
        .filter(|line| line.contains(text))
        .map(|line| line.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_normalizer_timestamps() {
        let normalizer = SnapshotNormalizer::new();
        let input = "Created at 2026-02-01 14:30:45";
        let output = normalizer.normalize(input);
        assert_eq!(output, "Created at [TIMESTAMP]");
    }

    #[test]
    fn test_snapshot_normalizer_uuids() {
        let normalizer = SnapshotNormalizer::new();
        let input = "ID: 550e8400-e29b-41d4-a716-446655440000";
        let output = normalizer.normalize(input);
        assert_eq!(output, "ID: [UUID]");
    }

    #[test]
    fn test_snapshot_normalizer_duration() {
        let normalizer = SnapshotNormalizer::new();
        let input = "Last updated 5m ago";
        let output = normalizer.normalize(input);
        assert_eq!(output, "Last updated [DURATION]");
    }

    #[test]
    fn test_snapshot_sequence() {
        let mut seq = SnapshotSequence::new("test_sequence");
        seq.step("first", "Initial state".to_string());
        seq.step("second", "After action".to_string());

        let output = seq.to_string();
        assert!(output.contains("Sequence: test_sequence"));
        assert!(output.contains("Step 1: first"));
        assert!(output.contains("Step 2: second"));
        assert!(output.contains("Initial state"));
        assert!(output.contains("After action"));
    }

    #[test]
    fn test_render_snapshot_basic() {
        let output = render_snapshot(20, 3, |frame| {
            use ratatui::widgets::Paragraph;
            Paragraph::new("Hello, World!").render(frame.area(), frame.buffer_mut());
        });

        assert!(output.contains("Hello, World!"));
    }

    #[test]
    fn test_buffer_to_string() {
        let backend = TestBackend::new(10, 2);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                use ratatui::widgets::Paragraph;
                Paragraph::new("Line 1").render(f.area(), f.buffer_mut());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let output = buffer_to_string(buffer);
        assert!(output.contains("Line 1"));
    }

    #[test]
    fn test_find_line_with_text() {
        let backend = TestBackend::new(20, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                use ratatui::widgets::Paragraph;
                let text = "First line\nSecond line\nThird line";
                Paragraph::new(text).render(f.area(), f.buffer_mut());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let line = find_line_with_text(buffer, "Second");
        assert!(line.is_some());
        assert!(line.unwrap().contains("Second"));
    }

    #[test]
    fn test_find_lines_with_text() {
        let backend = TestBackend::new(20, 4);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                use ratatui::widgets::Paragraph;
                let text = "First line\nSecond line\nThird line";
                Paragraph::new(text).render(f.area(), f.buffer_mut());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let lines = find_lines_with_text(buffer, "line");
        assert_eq!(lines.len(), 3);
    }
}
