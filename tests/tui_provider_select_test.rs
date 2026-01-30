// tests/tui/provider_select_test.rs
use keyring_cli::cloud::CloudProvider;
use keyring_cli::tui::screens::provider_select::ProviderSelectScreen;

#[test]
fn test_provider_list() {
    let screen = ProviderSelectScreen::new();
    let providers = screen.get_providers();

    assert_eq!(providers.len(), 8);
    assert_eq!(providers[0].name, "iCloud Drive");
    assert_eq!(providers[0].shortcut, '1');
    assert_eq!(providers[4].name, "WebDAV");
}

#[test]
fn test_provider_selection() {
    let mut screen = ProviderSelectScreen::new();

    // Select provider with '5' (WebDAV)
    screen.handle_char('5');
    assert_eq!(screen.get_selected_provider(), Some(CloudProvider::WebDAV));
}

#[test]
fn test_provider_navigation() {
    let mut screen = ProviderSelectScreen::new();

    // Navigate down
    screen.handle_down();
    assert_eq!(screen.get_selected_index(), 1);

    // Navigate up
    screen.handle_up();
    assert_eq!(screen.get_selected_index(), 0);
}
