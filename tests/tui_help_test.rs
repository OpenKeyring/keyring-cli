//! Help Screen Tests
//!
//! TDD tests for the help screen implementation.

use keyring_cli::tui::screens::help::HelpScreen;

#[test]
fn test_help_screen_new() {
    let screen = HelpScreen::new();

    // Should have 5 sections
    assert_eq!(screen.get_sections().len(), 5);
}

#[test]
fn test_global_section_content() {
    let screen = HelpScreen::new();
    let sections = screen.get_sections();

    let global = &sections[0];
    assert_eq!(global.title, "Global");

    // Should have at least 3 shortcuts
    assert!(global.shortcuts.len() >= 3);

    // Check for common global shortcuts
    let has_quit = global.shortcuts.iter().any(|s| s.action.contains("Quit"));
    assert!(has_quit, "Global section should have Quit shortcut");
}

#[test]
fn test_navigation_section_content() {
    let screen = HelpScreen::new();
    let sections = screen.get_sections();

    let nav = &sections[1];
    assert_eq!(nav.title, "Navigation");

    // Should have navigation shortcuts
    assert!(nav.shortcuts.len() >= 2);

    // Check for arrow keys
    let has_arrows = nav.shortcuts.iter().any(|s| {
        s.keys.contains("↑")
            || s.keys.contains("↓")
            || s.keys.contains("Up")
            || s.keys.contains("Down")
    });
    assert!(
        has_arrows,
        "Navigation section should have arrow key shortcuts"
    );
}

#[test]
fn test_operations_section_content() {
    let screen = HelpScreen::new();
    let sections = screen.get_sections();

    let ops = &sections[2];
    assert_eq!(ops.title, "Operations");

    // Should have operation shortcuts
    assert!(ops.shortcuts.len() >= 2);
}

#[test]
fn test_sync_section_content() {
    let screen = HelpScreen::new();
    let sections = screen.get_sections();

    let sync = &sections[3];
    assert_eq!(sync.title, "Sync");

    // Should have sync-related shortcuts
    assert!(!sync.shortcuts.is_empty());
}

#[test]
fn test_password_management_section_content() {
    let screen = HelpScreen::new();
    let sections = screen.get_sections();

    let pwd = &sections[4];
    assert_eq!(pwd.title, "Password Management");

    // Should have password management shortcuts
    assert!(pwd.shortcuts.len() >= 2);
}

#[test]
fn test_scroll_down() {
    let mut screen = HelpScreen::new();

    // Initially at scroll position 0
    assert_eq!(screen.get_scroll_position(), 0);

    // Scroll down
    screen.handle_scroll_down();
    assert_eq!(screen.get_scroll_position(), 1);

    // Scroll down multiple times
    screen.handle_scroll_down();
    screen.handle_scroll_down();
    assert_eq!(screen.get_scroll_position(), 3);
}

#[test]
fn test_scroll_up() {
    let mut screen = HelpScreen::new();

    // Scroll down first
    screen.handle_scroll_down();
    screen.handle_scroll_down();
    assert_eq!(screen.get_scroll_position(), 2);

    // Scroll up
    screen.handle_scroll_up();
    assert_eq!(screen.get_scroll_position(), 1);

    // Scroll up more
    screen.handle_scroll_up();
    assert_eq!(screen.get_scroll_position(), 0);
}

#[test]
fn test_scroll_boundary() {
    let mut screen = HelpScreen::new();

    // Can't scroll up from position 0
    screen.handle_scroll_up();
    assert_eq!(screen.get_scroll_position(), 0);

    // Scroll down multiple times to test max boundary
    for _ in 0..100 {
        screen.handle_scroll_down();
    }

    // Should not exceed total line count
    let max_scroll = screen.get_max_scroll_position();
    assert!(screen.get_scroll_position() <= max_scroll);
}

#[test]
fn test_shortcut_format() {
    let screen = HelpScreen::new();
    let sections = screen.get_sections();

    // All shortcuts should have non-empty keys and actions
    for section in &sections {
        for shortcut in &section.shortcuts {
            assert!(
                !shortcut.keys.is_empty(),
                "Shortcut keys should not be empty"
            );
            assert!(
                !shortcut.action.is_empty(),
                "Shortcut action should not be empty"
            );
        }
    }
}

#[test]
fn test_all_sections_have_content() {
    let screen = HelpScreen::new();
    let sections = screen.get_sections();

    // Every section should have at least one shortcut
    for section in &sections {
        assert!(
            !section.shortcuts.is_empty(),
            "Section '{}' should have shortcuts",
            section.title
        );
    }
}
