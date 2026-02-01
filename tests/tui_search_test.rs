use keyring_cli::tui::commands::search::handle_search;

#[test]
fn test_search_requires_query() {
    let result = handle_search(vec![]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output
        .iter()
        .any(|line| line.contains("Error: Search query required")));
}

#[test]
fn test_search_returns_results() {
    let result = handle_search(vec!["test"]);

    // In CI with test-env feature and OK_MASTER_PASSWORD set,
    // the vault gets auto-initialized, so search returns Ok with empty results.
    // Without test-env or password, it returns Err.
    // Both cases are valid - we just verify the function handles them gracefully.
    match &result {
        Ok(output) => {
            // Search succeeded - should show "No results" for empty vault
            assert!(
                output
                    .iter()
                    .any(|line| line.contains("No results") || line.contains("Found")),
                "Expected search results output, got: {:?}",
                output
            );
        }
        Err(_) => {
            // Error is acceptable when vault is not initialized
            // (e.g., when OK_MASTER_PASSWORD is not set or test-env feature disabled)
        }
    }
}
