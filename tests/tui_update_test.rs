use keyring_cli::tui::commands::update::handle_update;

#[test]
fn test_update_requires_name() {
    let result = handle_update(vec![]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output
        .iter()
        .any(|line: &String| line.contains("Error: Record name required")));
}

#[test]
fn test_update_wizard_starts() {
    let result = handle_update(vec!["test-record"]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output
        .iter()
        .any(|line: &String| line.contains("Update") || line.contains("Interactive")));
}
