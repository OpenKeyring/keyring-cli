//! Settings Screen Tests
//!
//! TDD tests for the settings screen implementation.

use keyring_cli::tui::screens::settings::{SettingsItem, SettingsSection, SettingsScreen};

#[test]
fn test_settings_screen_new() {
    let screen = SettingsScreen::new();

    // Should have 3 sections: Security, Sync, SyncOptions
    assert_eq!(screen.get_sections().len(), 3);
}

#[test]
fn test_security_section_items() {
    let screen = SettingsScreen::new();
    let sections = screen.get_sections();

    let security = &sections[0];
    assert_eq!(security.title, "Security");

    // Security section should have 2 items
    assert_eq!(security.items.len(), 2);
    assert_eq!(security.items[0].label, "Change Password");
    assert_eq!(security.items[1].label, "Biometric Unlock");
}

#[test]
fn test_sync_section_items() {
    let screen = SettingsScreen::new();
    let sections = screen.get_sections();

    let sync = &sections[1];
    assert_eq!(sync.title, "Sync");

    // Sync section should have 4 items
    assert_eq!(sync.items.len(), 4);
    assert_eq!(sync.items[0].label, "Status");
    assert_eq!(sync.items[1].label, "Provider");
    assert_eq!(sync.items[2].label, "Devices");
    assert_eq!(sync.items[3].label, "Configure");
}

#[test]
fn test_sync_options_section_items() {
    let screen = SettingsScreen::new();
    let sections = screen.get_sections();

    let options = &sections[2];
    assert_eq!(options.title, "Sync Options");

    // Sync Options section should have 3 items
    assert_eq!(options.items.len(), 3);
    assert_eq!(options.items[0].label, "Auto-sync");
    assert_eq!(options.items[1].label, "File Monitoring");
    assert_eq!(options.items[2].label, "Debounce");
}

#[test]
fn test_navigation_down() {
    let mut screen = SettingsScreen::new();

    // Start at first item
    assert_eq!(screen.get_selected_section_index(), 0);
    assert_eq!(screen.get_selected_item_index(), 0);

    // Navigate down
    screen.handle_down();
    assert_eq!(screen.get_selected_section_index(), 0);
    assert_eq!(screen.get_selected_item_index(), 1);

    // Navigate to next section
    screen.handle_down();
    assert_eq!(screen.get_selected_section_index(), 1);
    assert_eq!(screen.get_selected_item_index(), 0);
}

#[test]
fn test_navigation_up() {
    let mut screen = SettingsScreen::new();

    // Move to second item
    screen.handle_down();

    // Navigate up
    screen.handle_up();
    assert_eq!(screen.get_selected_section_index(), 0);
    assert_eq!(screen.get_selected_item_index(), 0);
}

#[test]
fn test_navigation_wrapping() {
    let mut screen = SettingsScreen::new();

    // Get total item count
    let total_items = screen.get_total_item_count();

    // Navigate to last item
    for _ in 0..(total_items - 1) {
        screen.handle_down();
    }

    // Navigate down from last item should wrap to first
    screen.handle_down();
    assert_eq!(screen.get_selected_section_index(), 0);
    assert_eq!(screen.get_selected_item_index(), 0);

    // Navigate up from first item should wrap to last
    screen.handle_up();
    assert_eq!(screen.get_selected_section_index(), 2);
    assert_eq!(screen.get_selected_item_index(), 2);
}

#[test]
fn test_selected_item_tracking() {
    let mut screen = SettingsScreen::new();

    // Navigate to "Provider" item (section 1, item 1)
    screen.handle_down();
    screen.handle_down();
    screen.handle_down();

    assert_eq!(screen.get_selected_section_index(), 1);
    assert_eq!(screen.get_selected_item_index(), 1);

    let selected = screen.get_selected_item();
    assert!(selected.is_some());
    let item = selected.unwrap();
    assert_eq!(item.label, "Provider");
}

#[test]
fn test_item_values() {
    let screen = SettingsScreen::new();
    let sections = screen.get_sections();

    // Check sync section values
    let sync = &sections[1];
    assert_eq!(sync.items[0].value, "Unsynced"); // Status
    assert_eq!(sync.items[1].value, "None"); // Provider
    assert_eq!(sync.items[2].value, "1 device"); // Devices

    // Check sync options values
    let options = &sections[2];
    assert_eq!(options.items[0].value, "Off"); // Auto-sync
    assert_eq!(options.items[1].value, "Off"); // File monitoring
    assert_eq!(options.items[2].value, "5s"); // Debounce
}

#[test]
fn test_toggle_boolean_option() {
    let mut screen = SettingsScreen::new();

    // Navigate to Auto-sync (section 2, item 0)
    for _ in 0..6 {
        screen.handle_down();
    }

    assert_eq!(screen.get_selected_section_index(), 2);
    assert_eq!(screen.get_selected_item_index(), 0);

    // Toggle should change value
    screen.handle_toggle();
    let sections = screen.get_sections();
    assert_eq!(sections[2].items[0].value, "On");

    // Toggle again
    screen.handle_toggle();
    let sections = screen.get_sections();
    assert_eq!(sections[2].items[0].value, "Off");
}

#[test]
fn test_action_returns() {
    let mut screen = SettingsScreen::new();

    // Navigate to "Configure" item (section 1, item 3)
    // Section 0 has 2 items, so we need 2 + 3 = 5 down presses
    for _ in 0..5 {
        screen.handle_down();
    }

    assert_eq!(screen.get_selected_section_index(), 1);
    assert_eq!(screen.get_selected_item_index(), 3);

    // Handle Enter should return Configure action
    let action = screen.handle_enter();
    assert!(action.is_some());
}
