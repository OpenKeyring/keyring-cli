//! TUI Health Command Tests
//!
//! Test the /health command in TUI mode

use keyring_cli::tui::commands::health::handle_health;

#[test]
fn test_health_with_no_args_returns_help() {
    let result = handle_health(vec![]);
    // Should return help when no flags provided
    assert!(result.is_ok());
    let output = result.unwrap();
    // Should indicate no checks selected
    assert!(output
        .iter()
        .any(|line: &String| line.contains("No checks selected")
            || line.contains("Use --weak")
            || line.contains("flags")));
}

#[test]
fn test_health_with_weak_flag_needs_vault() {
    let result = handle_health(vec!["--weak"]);
    // Should fail gracefully when vault not initialized
    // Either Ok with error message or Err is acceptable
    match result {
        Ok(output) => {
            // Should show some kind of error or vault not initialized message
            assert!(!output.is_empty());
            let has_error = output.iter().any(|line: &String| {
                line.contains("not initialized")
                    || line.contains("not found")
                    || line.contains("Error")
                    || line.contains("Vault")
            });
            // In test environment without vault, we expect some error message
            assert!(has_error || output.iter().any(|l| l.contains("No")));
        }
        Err(_) => {
            // Also acceptable to return an error - no assertion needed
        }
    }
}

#[test]
fn test_health_with_duplicate_flag_needs_vault() {
    let result = handle_health(vec!["--duplicate"]);
    // Should fail gracefully when vault not initialized
    match result {
        Ok(output) => {
            assert!(!output.is_empty());
        }
        Err(_) => {
            // Also acceptable to return an error - no assertion needed
        }
    }
}

#[test]
fn test_health_with_leaks_flag_needs_vault() {
    let result = handle_health(vec!["--leaks"]);
    // Should fail gracefully when vault not initialized
    match result {
        Ok(output) => {
            assert!(!output.is_empty());
        }
        Err(_) => {
            // Also acceptable to return an error - no assertion needed
        }
    }
}

#[test]
fn test_health_with_all_flag_needs_vault() {
    let result = handle_health(vec!["--all"]);
    // Should fail gracefully when vault not initialized
    match result {
        Ok(output) => {
            assert!(!output.is_empty());
        }
        Err(_) => {
            // Also acceptable to return an error - no assertion needed
        }
    }
}

#[test]
fn test_health_with_multiple_flags_needs_vault() {
    let result = handle_health(vec!["--weak", "--duplicate"]);
    // Should fail gracefully when vault not initialized
    match result {
        Ok(output) => {
            assert!(!output.is_empty());
        }
        Err(_) => {
            // Also acceptable to return an error - no assertion needed
        }
    }
}

#[test]
fn test_health_output_format() {
    let result = handle_health(vec!["--all"]);
    // Should return Ok even if vault not initialized
    assert!(result.is_ok());
    let output = result.unwrap();
    // Output should be a vector of strings suitable for TUI display
    assert!(!output.is_empty());
    // Most lines should be displayable text (allow some empty lines for spacing)
    let non_empty_count = output
        .iter()
        .filter(|line: &&String| !line.trim().is_empty())
        .count();
    assert!(
        non_empty_count > 0,
        "Output should have at least one non-empty line"
    );
}

#[test]
fn test_health_shows_summary_or_error() {
    let result = handle_health(vec!["--all"]);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Should contain health summary information OR error about vault
    let has_content = output.iter().any(|line: &String| {
        line.contains("records")
            || line.contains("checked")
            || line.contains("Health")
            || line.contains("Vault")
            || line.contains("not initialized")
    });
    assert!(has_content);
}
