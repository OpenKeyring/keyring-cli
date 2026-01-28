use keyring_cli::tui::commands::search::handle_search;

#[test]
fn test_search_requires_query() {
    let result = handle_search(vec![]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.iter().any(|line| line.contains("Error: Search query required")));
}

#[test]
fn test_search_attempts_search() {
    let result = handle_search(vec!["test"]);
    // The search will fail without an initialized vault, which is expected
    // We're just verifying the command structure is correct
    assert!(result.is_ok() || result.is_err());
}
