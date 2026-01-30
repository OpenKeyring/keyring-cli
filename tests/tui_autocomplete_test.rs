// tests/tui/autocomplete_test.rs
use keyring_cli::tui::TuiApp;

#[test]
fn test_command_autocomplete() {
    let mut app = TuiApp::new();
    app.input_buffer = "/ne".to_string();

    app.handle_autocomplete();

    // Should complete to "/new " (with space for args)
    assert_eq!(app.input_buffer, "/new ");
}

#[test]
fn test_command_autocomplete_full_match() {
    let mut app = TuiApp::new();
    app.input_buffer = "/new".to_string();

    app.handle_autocomplete();

    // Should complete to "/new " (with space)
    assert_eq!(app.input_buffer, "/new ");
}

#[test]
fn test_command_autocomplete_no_match() {
    let mut app = TuiApp::new();
    app.input_buffer = "/xyz".to_string();
    let original = app.input_buffer.clone();

    app.handle_autocomplete();

    // Should not change buffer when no match
    assert_eq!(app.input_buffer, original);
}

#[test]
fn test_command_autocomplete_multiple_matches() {
    let mut app = TuiApp::new();
    app.input_buffer = "/s".to_string();

    app.handle_autocomplete();

    // Should complete to one of the matches (either "/show " or "/search ")
    let is_valid = app.input_buffer == "/show " || app.input_buffer == "/search " || app.input_buffer == "/set";
    assert!(is_valid, "Expected valid autocomplete, got: {}", app.input_buffer);
}

#[test]
fn test_command_autocomplete_empty_buffer() {
    let mut app = TuiApp::new();
    app.input_buffer = String::new();

    app.handle_autocomplete();

    // Should not crash, buffer should remain empty or show "/"
    assert!(app.input_buffer.is_empty() || app.input_buffer == "/");
}

#[test]
fn test_command_autocomplete_with_partial_space() {
    let mut app = TuiApp::new();
    app.input_buffer = "/show g".to_string();

    // For command autocomplete, use handle_autocomplete()
    // For record name autocomplete, use handle_autocomplete_with_db() with vault
    app.handle_autocomplete();

    // Should at least contain the original prefix
    assert!(app.input_buffer.starts_with("/show"));
}

#[tokio::test]
async fn test_record_autocomplete() {
    let mut app = TuiApp::new();

    // For now, test that the method exists and doesn't crash
    // Real record autocomplete would require a vault with records
    app.input_buffer = "git".to_string();
    let result = app.handle_autocomplete_with_db(None).await;

    // Should succeed (no vault = no crash)
    assert!(result.is_ok());
}

#[test]
fn test_autocomplete_shows_matches() {
    let mut app = TuiApp::new();

    app.input_buffer = "/s".to_string();
    app.handle_autocomplete();

    // Should have output line showing matches
    assert!(app.output_lines.iter().any(|line| line.contains("Matching commands")));

    // The output should show the matching commands (/search, /show, /sync)
    let matches_line = app.output_lines.iter()
        .find(|line| line.contains("Matching commands"))
        .unwrap();
    assert!(matches_line.contains("/search") || matches_line.contains("/show") || matches_line.contains("/sync"));
}
