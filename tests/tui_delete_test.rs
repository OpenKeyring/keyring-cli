use keyring_cli::tui::commands::delete::handle_delete;

#[test]
fn test_delete_requires_name() {
    let result = handle_delete(vec![]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.iter().any(|line: &String| line.contains("Error: Record name required")));
}

#[test]
fn test_delete_success_message() {
    let result = handle_delete(vec!["test-record"]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.iter().any(|line: &String| line.contains("Delete") || line.contains("Confirm")));
}
