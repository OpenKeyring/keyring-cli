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
//!
//! # Test Environment Setup for Snapshots
//!
//! For snapshot tests, use `TestSnapshotEnv` which creates cross-platform temp
//! directories and provides path normalization:
//!
//! ```rust
//! use crate::tui::testing::TestSnapshotEnv;
//!
//! #[test]
//! fn test_config_display() {
//!     let _env = TestSnapshotEnv::new();
//!     // Config paths will be normalized in snapshots
//!     let app = TuiApp::new();
//!     insta::assert_snapshot!(app.output_lines);
//! }
//! ```
//!
//! The `TestSnapshotEnv`:
//! - Creates temp directories using `tempfile::TempDir` (cross-platform)
//! - Normalizes paths in snapshot output to show `[CONFIG_PATH]` and `[DATA_PATH]`
//! - Cleans up automatically on drop

use ratatui::{backend::TestBackend, buffer::Buffer, layout::Rect, widgets::Widget, Terminal};
use tempfile::TempDir;

/// Test environment for snapshot tests.
///
/// Creates temporary directories and normalizes paths in snapshots.
///
/// # Example
///
/// ```rust
/// use crate::tui::testing::TestSnapshotEnv;
///
/// #[test]
/// fn test_snapshot() {
///     let _env = TestSnapshotEnv::new();
///     let app = TuiApp::new();
///     // Paths will be normalized when snapshotting
///     insta::assert_snapshot!(normalize_output_paths(&app.output_lines));
/// }
/// ```
pub struct TestSnapshotEnv {
    _temp_dir: TempDir,
    config_dir: std::path::PathBuf,
    data_dir: std::path::PathBuf,
}

impl TestSnapshotEnv {
    /// Create a new test environment with cross-platform temp directories.
    ///
    /// Sets `OK_CONFIG_DIR` and `OK_DATA_DIR` environment variables to
    /// temporary directories that will be automatically cleaned up when
    /// the `TestSnapshotEnv` is dropped.
    ///
    /// Uses a consistent prefix to ensure paths are reproducible across
    /// test runs for snapshot comparison.
    pub fn new() -> Self {
        // Clean up any existing environment variables first
        std::env::remove_var("OK_CONFIG_DIR");
        std::env::remove_var("OK_DATA_DIR");

        // Use a consistent prefix for reproducible paths in snapshot tests
        // The random suffix will still vary, but normalization handles this
        let temp_dir = TempDir::with_prefix("open-keyring-snapshot-").expect("Failed to create temp dir");
        let config_dir = temp_dir.path().join("config");
        let data_dir = temp_dir.path().join("data");

        std::fs::create_dir_all(&config_dir).expect("Failed to create config dir");
        std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

        std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
        std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());

        Self {
            _temp_dir: temp_dir,
            config_dir,
            data_dir,
        }
    }

    /// Get the config directory path (for reference in tests).
    pub fn config_dir(&self) -> &std::path::Path {
        &self.config_dir
    }

    /// Get the data directory path (for reference in tests).
    pub fn data_dir(&self) -> &std::path::Path {
        &self.data_dir
    }

    /// Normalize file paths in output for snapshot comparison.
    ///
    /// Replaces actual file paths with placeholders to ensure snapshots
    /// are consistent across different platforms and test runs.
    /// Returns a Vec<String> to maintain compatibility with existing snapshot format.
    ///
    /// # Example
    ///
    /// ```rust
    /// let env = TestSnapshotEnv::new();
    /// let app = TuiApp::new();
    /// let normalized = env.normalize_paths(&app.output_lines);
    /// insta::assert_debug_snapshot!(&normalized);
    /// ```
    ///
    /// This transforms paths like:
    /// - Unix: `/tmp/.tmp123456/data/passwords.db` → `[DATA_PATH]/passwords.db`
    /// - Windows: `C:\Users\...\AppData\Local\Temp\.tmp123456\data\...` → `[DATA_PATH]/...`
    pub fn normalize_paths(&self, lines: &[String]) -> Vec<String> {
        let mut result = Vec::new();

        for line in lines {
            let mut normalized = line.clone();

            // Replace the full database path with placeholder
            // The config stores the full path: /var/folders/.../T/.tmpXXX/data/passwords.db
            // We need to normalize: /var/folders/.../T/.tmpXXX → [TEMP_DIR]
            // Then normalize: /data/passwords.db → the actual data subpath
            let db_path = self.data_dir.join("passwords.db");
            let db_path_str = db_path.to_string_lossy().to_string();
            let db_path_normalized = db_path_str.replace('\\', "/");
            normalized = normalized.replace(&db_path_normalized, "[DATA_PATH]/passwords.db");
            normalized = normalized.replace(&db_path_str, "[DATA_PATH]/passwords.db");

            // Also try to normalize any parent temp directory paths
            // This handles cases where the path includes the temp dir name
            let temp_dir_str = self._temp_dir.path().to_string_lossy().to_string();
            let temp_dir_normalized = temp_dir_str.replace('\\', "/");
            normalized = normalized.replace(&temp_dir_normalized, "[TEMP_DIR]");
            normalized = normalized.replace(&temp_dir_str, "[TEMP_DIR]");

            // Replace config directory path with placeholder
            let config_path_str = self.config_dir.to_string_lossy().to_string();
            let config_path_normalized = config_path_str.replace('\\', "/");
            normalized = normalized.replace(&config_path_normalized, "[CONFIG_PATH]");
            normalized = normalized.replace(&config_path_str, "[CONFIG_PATH]");

            result.push(normalized);
        }

        result
    }
}

impl Drop for TestSnapshotEnv {
    fn drop(&mut self) {
        // Clean up environment variables
        std::env::remove_var("OK_CONFIG_DIR");
        std::env::remove_var("OK_DATA_DIR");
    }
}

/// Normalize file paths in output for snapshot comparison.
///
/// Replaces actual file paths with placeholders to ensure snapshots
/// are consistent across different platforms and test runs.
///
/// # Example
///
/// ```rust
/// let output = normalize_output_paths(&app.output_lines);
/// insta::assert_snapshot!(output);
/// ```
///
/// This transforms paths like:
/// - Unix: `/tmp/.tmp123456/data/passwords.db` → `[DATA_PATH]/passwords.db`
/// - Windows: `C:\Users\...\AppData\Local\Temp\.tmp123456\data\...` → `[DATA_PATH]/...`
pub fn normalize_output_paths(lines: &[String]) -> String {
    let mut result = Vec::new();

    for line in lines {
        let normalized = line.clone();

        // Replace config directory path with placeholder
        if let Ok(config_dir) = std::env::var("OK_CONFIG_DIR") {
            let normalized = normalized.replace(&config_dir, "[CONFIG_PATH]");
            // Also handle backslash on Windows
            let normalized = normalized.replace(&config_dir.replace('\\', "/"), "[CONFIG_PATH]");
            result.push(normalized);
        } else {
            result.push(normalized);
        }
    }

    // Join with newlines for snapshot comparison
    result.join("\n")
}

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
        normalizer.add_replace(r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}", "[TIMESTAMP]");

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
