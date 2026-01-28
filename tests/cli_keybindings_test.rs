//! CLI keybindings command tests

#[test]
fn test_keybindings_args_list() {
    use keyring_cli::cli::commands::KeybindingsArgs;
    use clap::Parser;

    // Note: We're testing the library's KeybindingsArgs, not main.rs command
    let args = KeybindingsArgs::parse_from(&["ok", "keybindings", "--list"]);
    assert!(args.list);
    assert!(!args.validate);
    assert!(!args.reset);
    assert!(!args.edit);
}

#[test]
fn test_keybindings_args_validate() {
    use keyring_cli::cli::commands::KeybindingsArgs;
    use clap::Parser;

    let args = KeybindingsArgs::parse_from(&["ok", "keybindings", "--validate"]);
    assert!(args.validate);
    assert!(!args.list);
}

#[test]
fn test_keybindings_args_reset() {
    use keyring_cli::cli::commands::KeybindingsArgs;
    use clap::Parser;

    let args = KeybindingsArgs::parse_from(&["ok", "keybindings", "--reset"]);
    assert!(args.reset);
    assert!(!args.list);
}

#[test]
fn test_keybindings_args_edit() {
    use keyring_cli::cli::commands::KeybindingsArgs;
    use clap::Parser;

    let args = KeybindingsArgs::parse_from(&["ok", "keybindings", "--edit"]);
    assert!(args.edit);
    assert!(!args.list);
}

