use keyring_cli::tui::commands::config::handle_config;

#[test]
fn test_config_requires_subcommand_or_shows_list() {
    let result = handle_config(vec![]);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Should show configuration list
    assert!(output.iter().any(|line| line.contains("Configuration")));
}

#[test]
fn test_config_list_shows_all_sections() {
    let result = handle_config(vec!["list"]);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Should show configuration sections
    assert!(output.iter().any(|line| line.contains("[Database]")));
    assert!(output.iter().any(|line| line.contains("[Sync]")));
    assert!(output.iter().any(|line| line.contains("[Clipboard]")));
}

#[test]
fn test_config_get_requires_key() {
    let result = handle_config(vec!["get"]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output
        .iter()
        .any(|line| line.contains("Error") && line.contains("required")));
}

#[test]
fn test_config_set_requires_key_and_value() {
    let result = handle_config(vec!["set"]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output
        .iter()
        .any(|line| line.contains("Error") && line.contains("Key and value required")));
}

#[test]
fn test_config_set_validates_key() {
    let result = handle_config(vec!["set", "invalid.key", "value"]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output
        .iter()
        .any(|line| line.contains("Invalid configuration key")));
}

#[test]
fn test_config_reset_shows_warning_without_force() {
    let result = handle_config(vec!["reset"]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.iter().any(|line| line.contains("This will reset")));
    assert!(output.iter().any(|line| line.contains("--force")));
}

#[test]
fn test_config_unknown_subcommand() {
    let result = handle_config(vec!["unknown"]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output
        .iter()
        .any(|line| line.contains("Unknown") || line.contains("Usage")));
}
