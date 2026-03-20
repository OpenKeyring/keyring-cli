//! CLI keybindings command tests

#[test]
fn test_keybindings_args_list() {
    use clap::Parser;
    use keyring_cli::cli::commands::KeybindingsArgs;

    // KeybindingsArgs is an Args struct, not a Subcommand
    // So we parse flags directly without the "keybindings" subcommand
    let args = KeybindingsArgs::parse_from(["ok", "--list"]);
    assert!(args.list);
    assert!(!args.validate);
    assert!(!args.reset);
    assert!(!args.edit);
}

#[test]
fn test_keybindings_args_validate() {
    use clap::Parser;
    use keyring_cli::cli::commands::KeybindingsArgs;

    let args = KeybindingsArgs::parse_from(["ok", "--validate"]);
    assert!(args.validate);
    assert!(!args.list);
}

#[test]
fn test_keybindings_args_reset() {
    use clap::Parser;
    use keyring_cli::cli::commands::KeybindingsArgs;

    let args = KeybindingsArgs::parse_from(["ok", "--reset"]);
    assert!(args.reset);
    assert!(!args.list);
}

#[test]
fn test_keybindings_args_edit() {
    use clap::Parser;
    use keyring_cli::cli::commands::KeybindingsArgs;

    let args = KeybindingsArgs::parse_from(["ok", "--edit"]);
    assert!(args.edit);
    assert!(!args.list);
}
