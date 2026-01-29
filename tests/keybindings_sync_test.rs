//! Keybindings sync actions tests
//!
//! Test-Driven Development tests for sync-related keyboard shortcuts.

use keyring_cli::tui::keybindings::{Action, KeyBindingManager};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
fn test_sync_actions_exist() {
    // Test new sync-related actions exist
    // These will fail to compile until we add the variants
    let _ = Action::OpenSettings;
    let _ = Action::SyncNow;
    let _ = Action::ShowHelp;
    let _ = Action::RefreshView;
    let _ = Action::SaveConfig;
    let _ = Action::DisableSync;
}

#[test]
fn test_sync_shortcut_parsing() {
    let manager = KeyBindingManager::new();

    // Debug: print all bindings
    println!("\n=== All bindings ===");
    for (action, key) in manager.all_bindings() {
        println!("  {:?} -> {:?}", action, key);
    }
    println!("====================\n");

    // Test F2 -> OpenSettings
    let f2 = KeyEvent::new(KeyCode::F(2), KeyModifiers::empty());
    let action = manager.get_action(&f2);
    println!("F2 action: {:?}", action);
    assert_eq!(action, Some(Action::OpenSettings));

    // Test F5 -> SyncNow
    let f5 = KeyEvent::new(KeyCode::F(5), KeyModifiers::empty());
    assert_eq!(manager.get_action(&f5), Some(Action::SyncNow));

    // Test ? -> ShowHelp (YAML has "?" with no modifier)
    let question = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty());
    assert_eq!(manager.get_action(&question), Some(Action::ShowHelp));

    // Test Ctrl+R -> RefreshView
    let ctrl_r = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
    assert_eq!(manager.get_action(&ctrl_r), Some(Action::RefreshView));

    // Test Ctrl+S -> SaveConfig
    let ctrl_s = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert_eq!(manager.get_action(&ctrl_s), Some(Action::SaveConfig));

    // Test Ctrl+D -> DisableSync
    let ctrl_d = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
    assert_eq!(manager.get_action(&ctrl_d), Some(Action::DisableSync));
}

#[test]
fn test_action_display_for_sync_actions() {
    // Test that sync actions can be displayed for help
    assert_eq!(format!("{}", Action::OpenSettings), "OpenSettings");
    assert_eq!(format!("{}", Action::SyncNow), "SyncNow");
    assert_eq!(format!("{}", Action::ShowHelp), "ShowHelp");
    assert_eq!(format!("{}", Action::RefreshView), "RefreshView");
    assert_eq!(format!("{}", Action::SaveConfig), "SaveConfig");
    assert_eq!(format!("{}", Action::DisableSync), "DisableSync");
}

#[test]
fn test_action_command_names_for_sync_actions() {
    assert_eq!(Action::OpenSettings.command_name(), "/settings");
    assert_eq!(Action::SyncNow.command_name(), "/sync");
    assert_eq!(Action::ShowHelp.command_name(), "/help");
    assert_eq!(Action::RefreshView.command_name(), "/refresh");
    assert_eq!(Action::SaveConfig.command_name(), "/save");
    assert_eq!(Action::DisableSync.command_name(), "/disable_sync");
}

#[test]
fn test_action_descriptions_for_sync_actions() {
    assert!(!Action::OpenSettings.description().is_empty());
    assert!(!Action::SyncNow.description().is_empty());
    assert!(!Action::ShowHelp.description().is_empty());
    assert!(!Action::RefreshView.description().is_empty());
    assert!(!Action::SaveConfig.description().is_empty());
    assert!(!Action::DisableSync.description().is_empty());
}
