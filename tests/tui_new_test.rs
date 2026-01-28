use keyring_cli::tui::commands::new::handle_new;

#[test]
fn test_new_shows_instructions() {
    let result = handle_new();
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_empty());
}
